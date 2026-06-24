//! Execution observability — the `AuditProvider` trait and closed event taxonomy.
//!
//! Every workflow run can be observed by plugging in an [`AuditProvider`]
//! implementation. The built-in [`NoopAuditProvider`] discards all events and
//! satisfies the interface without any I/O, keeping the no-audit path at the
//! cost of a single indirect call per event.
//!
//! # Observer neutrality
//!
//! The presence or absence of an `AuditProvider` must never alter a run's
//! `@out` values, execution branch outcomes, or error codes. All `record_event`
//! calls are write-only; no return value flows back into the executor.

// ─── Extension point ──────────────────────────────────────────────────────────

/// Stateful observer registered once per workflow run.
///
/// The executor calls [`AuditProvider::record_event`] synchronously for each
/// observable event (before returning control to the next step) and calls
/// [`AuditProvider::run_complete`] exactly once when the run terminates.
///
/// # Contract
///
/// - `record_event` is on the hot path; implementations should avoid blocking I/O.
/// - `run_complete` is always called — even when the run errors or halts.
/// - The same instance must not be shared across concurrent runs.
pub trait AuditProvider {
    /// Record a single execution event. Called synchronously in emission order.
    fn record_event(&self, event: ExecutionEvent);

    /// Called once when the run terminates; carries the run-level summary.
    fn run_complete(&self, manifest: RunManifest);
}

/// Built-in no-op implementation — discards all events with zero allocation.
pub struct NoopAuditProvider;

impl AuditProvider for NoopAuditProvider {
    #[inline]
    fn record_event(&self, _event: ExecutionEvent) {}

    #[inline]
    fn run_complete(&self, _manifest: RunManifest) {}
}

// ─── Event taxonomy ───────────────────────────────────────────────────────────

/// Discriminant for loop types (used in [`ExecutionEvent::LoopIteration`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoopType {
    /// A `~FOR` bounded-collection loop.
    For,
    /// A `~UNTIL MAX:n` conditional loop.
    Until,
}

/// Structural descriptor for model I/O fields.
///
/// Never contains raw user text — only shape metadata. This enforces the
/// data-safety boundary: model call traces can be shared without leaking
/// workflow content.
#[derive(Debug, Clone)]
pub struct FieldDescriptor {
    /// Number of top-level fields.
    pub field_count: u32,
    /// Type hints for each field (e.g. `["Text"]`, `["Map"]`).
    pub type_hints: Vec<String>,
    /// Approximate total byte count (content length, not content).
    pub total_bytes: u32,
}

impl FieldDescriptor {
    /// Single text field of `byte_len` bytes.
    pub fn text(byte_len: u32) -> Self {
        FieldDescriptor {
            field_count: 1,
            type_hints: vec!["Text".to_string()],
            total_bytes: byte_len,
        }
    }

    /// Map-like multi-field descriptor.
    pub fn map_like(field_count: u32, byte_len: u32) -> Self {
        FieldDescriptor {
            field_count,
            type_hints: vec!["Map".to_string()],
            total_bytes: byte_len,
        }
    }
}

/// Closed set of observable execution events. Adding a new variant is a
/// spec-level minor-version amendment (HO-6).
#[derive(Debug, Clone)]
pub enum ExecutionEvent {
    /// Before a step's command is dispatched.
    StepStart {
        step_index: u32,
        step_command: String,
        /// Snapshot of variable names bound at this point (no values).
        input_vars: Vec<String>,
    },
    /// After a step's command returns successfully.
    StepEnd {
        step_index: u32,
        step_command: String,
        /// Pipeline target names written by this step (empty if no `→`).
        output_vars: Vec<String>,
        elapsed_ms: u64,
    },
    /// When a step produces a NODUS error code.
    StepError {
        step_index: u32,
        step_command: String,
        /// NODUS:* error code from the vocabulary taxonomy.
        error_code: String,
        error_detail: String,
    },
    /// When a hard (`!!NEVER`/`!!ALWAYS`) constraint fires.
    ConstraintHit {
        rule_name: String,
        triggering_step_index: u32,
        /// `true` when the executor halted execution as a result.
        halt: bool,
    },
    /// When a conditional (`?IF`/`?ELIF`/`?ELSE`) resolves.
    BranchTaken {
        step_index: u32,
        /// `"if"`, `"elif"`, or `"else"`.
        branch_label: String,
        condition_result: bool,
    },
    /// At the start of each `~FOR`/`~UNTIL` loop body.
    LoopIteration {
        step_index: u32,
        loop_type: LoopType,
        iteration_number: u32,
        /// Variables bound for this iteration (e.g. the `~FOR` loop variable).
        bound_vars: Vec<String>,
    },
    /// When `RUN(@macro_name)` begins execution.
    MacroEnter {
        macro_name: String,
        call_step_index: u32,
    },
    /// When a macro body completes.
    MacroExit {
        macro_name: String,
        call_step_index: u32,
        elapsed_ms: u64,
    },
    /// Immediately before dispatching to the `ModelProvider` (GEN, ANALYZE, …).
    ModelCall {
        step_index: u32,
        /// Command that triggered the model call.
        command: String,
        /// Structural descriptor — no raw user text (data-safety boundary).
        input_summary: FieldDescriptor,
    },
    /// Immediately after the `ModelProvider` returns.
    ModelResponse {
        step_index: u32,
        command: String,
        /// Structural descriptor of the response — no raw content.
        output_summary: FieldDescriptor,
        elapsed_ms: u64,
    },
}

// ─── Run manifest ─────────────────────────────────────────────────────────────

/// Overall status reported in the run manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunStatus {
    /// All steps completed without errors.
    Ok,
    /// Steps completed but non-fatal errors occurred.
    Error,
    /// A hard-constraint rule halted execution.
    ConstraintHalt,
    /// Validation errors prevented execution from starting.
    ValidationError,
}

/// Run-level summary delivered to [`AuditProvider::run_complete`] once per run.
#[derive(Debug, Clone)]
pub struct RunManifest {
    /// `wf:` identifier (without the `wf:` prefix).
    pub workflow_name: String,
    /// `BUILTIN_SCHEMA_VERSION` at run time.
    pub schema_version: String,
    /// Caller-supplied unique identifier (UUID or equivalent).
    pub run_id: String,
    /// ISO-8601 timestamp supplied by the caller at run construction.
    pub started_at: String,
    /// Wall-clock duration of the run.
    pub elapsed_ms: u64,
    pub status: RunStatus,
    /// First NODUS:* error code encountered, if any.
    pub error_code: Option<String>,
    /// Number of steps that reached execution (excludes unentered branches).
    pub total_steps: u32,
    /// Total events emitted to [`AuditProvider::record_event`] during this run.
    pub event_count: u32,
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    // ── RecordingProvider test helper ────────────────────────────────────────

    pub(crate) struct RecordingProvider {
        pub events: Arc<Mutex<Vec<ExecutionEvent>>>,
        pub manifests: Arc<Mutex<Vec<RunManifest>>>,
    }

    impl RecordingProvider {
        pub fn new() -> Self {
            RecordingProvider {
                events: Arc::new(Mutex::new(Vec::new())),
                manifests: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    impl AuditProvider for RecordingProvider {
        fn record_event(&self, event: ExecutionEvent) {
            self.events.lock().unwrap().push(event);
        }
        fn run_complete(&self, manifest: RunManifest) {
            self.manifests.lock().unwrap().push(manifest);
        }
    }

    // ── NoopAuditProvider ────────────────────────────────────────────────────

    #[test]
    fn noop_provider_discards_all() {
        let noop = NoopAuditProvider;
        noop.record_event(ExecutionEvent::StepStart {
            step_index: 1,
            step_command: "GEN".to_string(),
            input_vars: vec!["query".to_string()],
        });
        noop.record_event(ExecutionEvent::StepEnd {
            step_index: 1,
            step_command: "GEN".to_string(),
            output_vars: vec!["$out".to_string()],
            elapsed_ms: 5,
        });
        noop.record_event(ExecutionEvent::StepError {
            step_index: 1,
            step_command: "PUBLISH".to_string(),
            error_code: "NODUS:RULE_VIOLATION".to_string(),
            error_detail: "!!NEVER: publish".to_string(),
        });
        noop.record_event(ExecutionEvent::ConstraintHit {
            rule_name: "!!NEVER: publish".to_string(),
            triggering_step_index: 1,
            halt: true,
        });
        noop.record_event(ExecutionEvent::BranchTaken {
            step_index: 2,
            branch_label: "if".to_string(),
            condition_result: true,
        });
        noop.record_event(ExecutionEvent::LoopIteration {
            step_index: 3,
            loop_type: LoopType::For,
            iteration_number: 0,
            bound_vars: vec!["$item".to_string()],
        });
        noop.record_event(ExecutionEvent::LoopIteration {
            step_index: 4,
            loop_type: LoopType::Until,
            iteration_number: 1,
            bound_vars: vec![],
        });
        noop.record_event(ExecutionEvent::MacroEnter {
            macro_name: "refine".to_string(),
            call_step_index: 5,
        });
        noop.record_event(ExecutionEvent::MacroExit {
            macro_name: "refine".to_string(),
            call_step_index: 5,
            elapsed_ms: 2,
        });
        noop.record_event(ExecutionEvent::ModelCall {
            step_index: 1,
            command: "GEN".to_string(),
            input_summary: FieldDescriptor::text(42),
        });
        noop.record_event(ExecutionEvent::ModelResponse {
            step_index: 1,
            command: "GEN".to_string(),
            output_summary: FieldDescriptor::text(100),
            elapsed_ms: 10,
        });
        noop.run_complete(RunManifest {
            workflow_name: "test".to_string(),
            schema_version: "0.4.6".to_string(),
            run_id: "run-0".to_string(),
            started_at: "2026-06-24T00:00:00Z".to_string(),
            elapsed_ms: 20,
            status: RunStatus::Ok,
            error_code: None,
            total_steps: 5,
            event_count: 10,
        });
        // All 10 event variants + run_complete accepted without panic.
    }

    // ── RecordingProvider captures events ────────────────────────────────────

    #[test]
    fn step_start_end_emitted_in_order() {
        let recording = RecordingProvider::new();
        recording.record_event(ExecutionEvent::StepStart {
            step_index: 1,
            step_command: "GEN".to_string(),
            input_vars: vec![],
        });
        recording.record_event(ExecutionEvent::StepEnd {
            step_index: 1,
            step_command: "GEN".to_string(),
            output_vars: vec!["$out".to_string()],
            elapsed_ms: 3,
        });

        let events = recording.events.lock().unwrap();
        assert_eq!(events.len(), 2);
        assert!(matches!(
            &events[0],
            ExecutionEvent::StepStart { step_index: 1, .. }
        ));
        assert!(matches!(
            &events[1],
            ExecutionEvent::StepEnd { step_index: 1, .. }
        ));
    }

    #[test]
    fn constraint_hit_halt_true() {
        let recording = RecordingProvider::new();
        recording.record_event(ExecutionEvent::ConstraintHit {
            rule_name: "!!NEVER: DELETE".to_string(),
            triggering_step_index: 2,
            halt: true,
        });
        let events = recording.events.lock().unwrap();
        match &events[0] {
            ExecutionEvent::ConstraintHit { halt, .. } => assert!(*halt),
            other => panic!("expected ConstraintHit, got {other:?}"),
        }
    }

    #[test]
    fn branch_taken_if_and_else() {
        let recording = RecordingProvider::new();
        recording.record_event(ExecutionEvent::BranchTaken {
            step_index: 1,
            branch_label: "if".to_string(),
            condition_result: true,
        });
        recording.record_event(ExecutionEvent::BranchTaken {
            step_index: 2,
            branch_label: "else".to_string(),
            condition_result: false,
        });
        let events = recording.events.lock().unwrap();
        assert!(matches!(
            &events[0],
            ExecutionEvent::BranchTaken {
                condition_result: true,
                ..
            }
        ));
        assert!(matches!(
            &events[1],
            ExecutionEvent::BranchTaken {
                condition_result: false,
                ..
            }
        ));
    }

    #[test]
    fn loop_iteration_for() {
        let recording = RecordingProvider::new();
        for i in 0u32..3 {
            recording.record_event(ExecutionEvent::LoopIteration {
                step_index: 1,
                loop_type: LoopType::For,
                iteration_number: i,
                bound_vars: vec!["$item".to_string()],
            });
        }
        let events = recording.events.lock().unwrap();
        assert_eq!(events.len(), 3);
        for (i, ev) in events.iter().enumerate() {
            match ev {
                ExecutionEvent::LoopIteration {
                    loop_type: LoopType::For,
                    iteration_number,
                    ..
                } => {
                    assert_eq!(*iteration_number, i as u32);
                }
                other => panic!("expected LoopIteration(For), got {other:?}"),
            }
        }
    }

    #[test]
    fn loop_iteration_until() {
        let recording = RecordingProvider::new();
        recording.record_event(ExecutionEvent::LoopIteration {
            step_index: 2,
            loop_type: LoopType::Until,
            iteration_number: 0,
            bound_vars: vec![],
        });
        let events = recording.events.lock().unwrap();
        assert!(matches!(
            &events[0],
            ExecutionEvent::LoopIteration {
                loop_type: LoopType::Until,
                ..
            }
        ));
    }

    #[test]
    fn macro_enter_exit_pair() {
        let recording = RecordingProvider::new();
        recording.record_event(ExecutionEvent::MacroEnter {
            macro_name: "summarize".to_string(),
            call_step_index: 3,
        });
        recording.record_event(ExecutionEvent::MacroExit {
            macro_name: "summarize".to_string(),
            call_step_index: 3,
            elapsed_ms: 7,
        });
        let events = recording.events.lock().unwrap();
        assert!(
            matches!(&events[0], ExecutionEvent::MacroEnter { macro_name, .. } if macro_name == "summarize")
        );
        assert!(
            matches!(&events[1], ExecutionEvent::MacroExit { macro_name, .. } if macro_name == "summarize")
        );
    }

    #[test]
    fn model_call_no_raw_content() {
        let desc = FieldDescriptor::text(256);
        // Confirm FieldDescriptor holds only structural metadata, never the actual text.
        assert_eq!(desc.field_count, 1);
        assert_eq!(desc.type_hints, vec!["Text"]);
        assert_eq!(desc.total_bytes, 256);

        let event = ExecutionEvent::ModelCall {
            step_index: 1,
            command: "GEN".to_string(),
            input_summary: desc,
        };
        // Verify debug output contains no prose that could be mistaken for content.
        let debug = format!("{event:?}");
        assert!(debug.contains("total_bytes: 256"));
        assert!(!debug.contains("prompt"));
    }

    #[test]
    fn run_manifest_fields_populated() {
        let manifest = RunManifest {
            workflow_name: "hello".to_string(),
            schema_version: "0.4.6".to_string(),
            run_id: "r-001".to_string(),
            started_at: "2026-06-24T12:00:00Z".to_string(),
            elapsed_ms: 42,
            status: RunStatus::Ok,
            error_code: None,
            total_steps: 3,
            event_count: 6,
        };
        let recording = RecordingProvider::new();
        recording.run_complete(manifest);
        let manifests = recording.manifests.lock().unwrap();
        assert_eq!(manifests[0].workflow_name, "hello");
        assert_eq!(manifests[0].run_id, "r-001");
        assert_eq!(manifests[0].event_count, 6);
        assert_eq!(manifests[0].status, RunStatus::Ok);
    }
}
