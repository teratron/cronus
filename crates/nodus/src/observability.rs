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

// ─── Aggregation-safe measurement (HO-14) ─────────────────────────────────────

/// A numeric that was either genuinely measured, or explicitly could not be.
///
/// `Unavailable` is **never** rendered as `0`, omitted, or carried forward from
/// a previous observation — each of those corrupts a downstream aggregate
/// (mean, minimum, coverage) while looking exactly like data. *Not
/// applicable* is a different thing entirely: a field the taxonomy does not
/// define for a given event type is simply absent from that variant, and
/// never carries this marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Measurement {
    Taken(u64),
    Unavailable,
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
///
/// Every variant carries `seq` (run-monotonic, dense, gap-free — HO-7) and
/// `correlation_id` (shared by every event of one run, equal to
/// `RunManifest.run_id` — HO-7). Both are assigned by
/// [`crate::executor::Executor`]'s single emission choke point, never by a
/// caller — there is no public constructor that lets a caller pick either.
#[derive(Debug, Clone)]
pub enum ExecutionEvent {
    /// Before a step's command is dispatched.
    StepStart {
        step_index: u32,
        step_command: String,
        /// Snapshot of variable names bound at this point (no values).
        input_vars: Vec<String>,
        /// Definition-derived, stable cross-run identity (HO-15).
        step_identity: String,
        seq: u64,
        correlation_id: String,
    },
    /// After a step's command returns successfully.
    StepEnd {
        step_index: u32,
        step_command: String,
        /// Pipeline target names written by this step (empty if no `→`).
        output_vars: Vec<String>,
        elapsed_ms: Measurement,
        /// Definition-derived, stable cross-run identity (HO-15).
        step_identity: String,
        seq: u64,
        correlation_id: String,
    },
    /// When a step produces a NODUS error code.
    StepError {
        step_index: u32,
        step_command: String,
        /// NODUS:* error code from the vocabulary taxonomy.
        error_code: String,
        error_detail: String,
        /// Definition-derived, stable cross-run identity (HO-15).
        step_identity: String,
        /// Stable, message-independent grouping input (HO-19).
        fault_identity: FaultIdentity,
        seq: u64,
        correlation_id: String,
    },
    /// When a hard (`!!NEVER`/`!!ALWAYS`) constraint fires.
    ConstraintHit {
        rule_name: String,
        triggering_step_index: u32,
        /// `true` when the executor halted execution as a result.
        halt: bool,
        seq: u64,
        correlation_id: String,
    },
    /// When a conditional (`?IF`/`?ELIF`/`?ELSE`) resolves.
    BranchTaken {
        step_index: u32,
        /// `"if"`, `"elif"`, or `"else"`.
        branch_label: String,
        condition_result: bool,
        seq: u64,
        correlation_id: String,
    },
    /// At the start of each `~FOR`/`~UNTIL` loop body.
    LoopIteration {
        step_index: u32,
        loop_type: LoopType,
        iteration_number: Measurement,
        /// Variables bound for this iteration (e.g. the `~FOR` loop variable).
        bound_vars: Vec<String>,
        seq: u64,
        correlation_id: String,
    },
    /// When `RUN(@macro_name)` begins execution.
    MacroEnter {
        macro_name: String,
        call_step_index: u32,
        seq: u64,
        correlation_id: String,
    },
    /// When a macro body completes.
    MacroExit {
        macro_name: String,
        call_step_index: u32,
        elapsed_ms: Measurement,
        seq: u64,
        correlation_id: String,
    },
    /// Immediately before dispatching to the `ModelProvider` (GEN, ANALYZE, …).
    ModelCall {
        step_index: u32,
        /// Command that triggered the model call.
        command: String,
        /// Structural descriptor — no raw user text (data-safety boundary).
        input_summary: FieldDescriptor,
        seq: u64,
        correlation_id: String,
    },
    /// Immediately after the `ModelProvider` returns.
    ModelResponse {
        step_index: u32,
        command: String,
        /// Structural descriptor of the response — no raw content.
        output_summary: FieldDescriptor,
        elapsed_ms: Measurement,
        seq: u64,
        correlation_id: String,
    },
}

// ─── Environment trajectory side-band (NE-3) ──────────────────────────────────

/// Discriminant for an environment-lifecycle interaction recorded in the
/// trajectory (NE-3). `Step` is reserved for a host that drives
/// [`crate::environment::EnvironmentProvider::step`] directly; the built-in
/// `run_with_environment` combinator emits `Reset` only (v1 scope — see
/// `crate::environment` module docs). `evaluate`'s outcome is not represented
/// here: it occurs after the run is frozen — after `run_complete` has already
/// fired — and is delivered directly as the environment run's `Reward` instead
/// (NE-4/NE-5), not duplicated into the trajectory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvInteractionKind {
    Reset,
    Step,
}

/// One environment-lifecycle interaction, carried through [`RunManifest`]
/// rather than a parallel store (NE-3) — no new [`ExecutionEvent`] variant is
/// added, so HO-6's closed taxonomy is preserved (the HO-8/HO-13 additive-field
/// discipline). `observation`/`action` are structural descriptors only — never
/// raw content, matching the [`FieldDescriptor`] data-safety boundary used
/// elsewhere.
#[derive(Debug, Clone)]
pub struct EnvInteraction {
    pub kind: EnvInteractionKind,
    pub observation: Option<FieldDescriptor>,
    pub action: Option<FieldDescriptor>,
}

// ─── Run-manifest identity & reproducibility (HO-12/15/18/19/20) ─────────────

/// Definition-derived, stable step identity (HO-15) — NOT per-run allocated.
/// The same value across repeated runs, retries/resumes (NL-12), and recursive
/// children (NL-18); it changes only when the step's own definition changes
/// (its number or its command name). Deterministic (NL-6).
///
/// Distinct from HO-7's within-run `(correlation_id, seq)` ordering: this is
/// the cross-run comparison key, not a within-run position.
///
/// Takes `(step_number, command_name)` rather than a `&Step` reference — the
/// executor's emission sites carry these two pieces of the definition, not a
/// `Step` value, so deriving identity from them avoids threading an AST
/// reference through the dispatch path for no semantic gain.
pub fn step_identity(step_number: u32, command_name: &str) -> String {
    format!("{step_number}:{command_name}")
}

/// A stable, message-independent grouping input for a failing step (HO-19).
///
/// Composed only from stable inputs — [`step_identity`], the emitted
/// `NODUS:*` code, and an optional workflow-declared discriminator that
/// outranks the code when present. **Never** derived from `error_detail`
/// rendered text, which routinely carries per-occurrence interpolated values
/// (an id, a path, a count) that would shatter one recurring failure into
/// thousands of singletons. nodus performs no grouping itself — it guarantees
/// only that the inputs a host groups on are stable and content-independent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FaultIdentity {
    pub step_identity: String,
    /// The canonical `NODUS:*` code (see [`crate::vocab::error_code`]).
    pub code: String,
    /// A workflow-declared discriminator, if any. No `.nodus` grammar
    /// declares one yet, so this is always `None` today — the carrier exists
    /// ahead of the declaring syntax (an honest, not a silent, gap).
    pub discriminator: Option<String>,
}

/// The execution mode a run declared (HO-12): real, or simulated at a stated
/// fidelity. nodus substitutes no providers itself — a host that wires
/// modeled providers for a simulation declares the mode; nodus only records
/// what was declared. Absent/default is `Real` (today's behaviour), so a
/// caller declaring nothing is unaffected (HO-5).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ExecutionMode {
    #[default]
    Real,
    Simulated {
        fidelity: SimFidelity,
    },
}

/// Declared fidelity of a simulated run (HO-12).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimFidelity {
    Structural,
    Modeled,
    Shadow,
}

/// Whether re-running the recorded workflow reproduces the same outcome
/// (HO-20). **Stated, never inferred** from the mere presence of a
/// [`ReproRecipe`] — nodus's own evaluation is deterministic (NL-6), but a run
/// that made any model call is not, and the recipe must say which of the two
/// it is rather than let a reader assume exactness.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Determinism {
    #[default]
    Deterministic,
    ContainsModelCalls,
}

/// What re-executing this run requires, recorded from the manifest alone
/// (HO-20) — the host reconstructs nothing. nodus embeds nothing into
/// produced artifacts, names no file format, and performs no replay; it only
/// records what it itself resolved (LP-1/LP-2).
///
/// An uncapturable field is `None` — **never silently omitted** — since a
/// short recipe must not read as a complete one (the HO-14 honesty rule
/// applied at the manifest grain). `needs_vocabulary` is `None` today because
/// `@needs` selective vocabulary loading is not yet implemented; this is a
/// declared omission, not a defect.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReproRecipe {
    /// Content identity of the workflow definition (a `std`-only digest —
    /// the [`crate::environment::CandidateResult`] precedent; zero-dep, LP-1).
    pub workflow_digest: String,
    /// The LP-8 capability-manifest roles/commands that satisfied this run.
    pub capability_set: Vec<String>,
    /// Mirrors [`RunManifest::exposure_switches`] (HO-18) into the recipe.
    pub exposure_switches: Vec<(String, String)>,
    /// Mirrors [`RunManifest::execution_mode`] (HO-12) into the recipe.
    pub execution_mode: ExecutionMode,
    /// The producing crate version (`env!("CARGO_PKG_VERSION")`).
    pub nodus_version: String,
    /// Resolved `@needs` vocabulary units. `None` until `@needs` selective
    /// loading is implemented — an honest declared omission.
    pub needs_vocabulary: Option<Vec<String>>,
    /// Stated, never inferred (see the type's own documentation).
    pub determinism: Determinism,
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
    pub elapsed_ms: Measurement,
    pub status: RunStatus,
    /// First NODUS:* error code encountered, if any.
    pub error_code: Option<String>,
    /// Number of steps that reached execution (excludes unentered branches).
    pub total_steps: u32,
    /// Total events emitted to [`AuditProvider::record_event`] during this run.
    /// Doubles as the HO-7 gap-check: for an undamaged trace this equals the
    /// highest emitted `seq` + 1 (both derive from the same counter through
    /// the executor's single emission choke point — T-15B01/T-15B02).
    pub event_count: u32,
    /// Environment-lifecycle interactions for this run (NE-3). Empty for every
    /// run that did not go through [`crate::environment`]'s combinators —
    /// additive field, HO-5 observer neutrality preserved for the common case.
    pub env_trajectory: Vec<EnvInteraction>,
    /// Real vs. simulated, and at what fidelity (HO-12). Default `Real`.
    pub execution_mode: ExecutionMode,
    /// Resolved `(switch_name, value)` pairs the host froze once at run start
    /// (HO-18, LP-19 non-straddling). Empty = prevailing defaults.
    pub exposure_switches: Vec<(String, String)>,
    /// What re-executing this run requires, from the manifest alone (HO-20).
    pub repro: ReproRecipe,
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

    /// Test-only helper: a fixed seq/correlation pair, since these tests
    /// exercise event *shape*, not the executor's real assignment discipline
    /// (that lives in `executor.rs`'s integration tests).
    fn sc(seq: u64) -> (u64, String) {
        (seq, "corr-test".to_string())
    }

    #[test]
    fn noop_provider_discards_all() {
        let noop = NoopAuditProvider;
        let (seq, cid) = sc(0);
        noop.record_event(ExecutionEvent::StepStart {
            step_index: 1,
            step_command: "GEN".to_string(),
            input_vars: vec!["query".to_string()],
            step_identity: step_identity(1, "GEN"),
            seq,
            correlation_id: cid,
        });
        let (seq, cid) = sc(1);
        noop.record_event(ExecutionEvent::StepEnd {
            step_index: 1,
            step_command: "GEN".to_string(),
            output_vars: vec!["$out".to_string()],
            elapsed_ms: Measurement::Taken(5),
            step_identity: step_identity(1, "GEN"),
            seq,
            correlation_id: cid,
        });
        let (seq, cid) = sc(2);
        noop.record_event(ExecutionEvent::StepError {
            step_index: 1,
            step_command: "PUBLISH".to_string(),
            error_code: "NODUS:RULE_VIOLATION".to_string(),
            error_detail: "!!NEVER: publish".to_string(),
            step_identity: step_identity(1, "PUBLISH"),
            fault_identity: FaultIdentity {
                step_identity: step_identity(1, "PUBLISH"),
                code: "NODUS:RULE_VIOLATION".to_string(),
                discriminator: None,
            },
            seq,
            correlation_id: cid,
        });
        let (seq, cid) = sc(3);
        noop.record_event(ExecutionEvent::ConstraintHit {
            rule_name: "!!NEVER: publish".to_string(),
            triggering_step_index: 1,
            halt: true,
            seq,
            correlation_id: cid,
        });
        let (seq, cid) = sc(4);
        noop.record_event(ExecutionEvent::BranchTaken {
            step_index: 2,
            branch_label: "if".to_string(),
            condition_result: true,
            seq,
            correlation_id: cid,
        });
        let (seq, cid) = sc(5);
        noop.record_event(ExecutionEvent::LoopIteration {
            step_index: 3,
            loop_type: LoopType::For,
            iteration_number: Measurement::Taken(0),
            bound_vars: vec!["$item".to_string()],
            seq,
            correlation_id: cid,
        });
        let (seq, cid) = sc(6);
        noop.record_event(ExecutionEvent::LoopIteration {
            step_index: 4,
            loop_type: LoopType::Until,
            iteration_number: Measurement::Taken(1),
            bound_vars: vec![],
            seq,
            correlation_id: cid,
        });
        let (seq, cid) = sc(7);
        noop.record_event(ExecutionEvent::MacroEnter {
            macro_name: "refine".to_string(),
            call_step_index: 5,
            seq,
            correlation_id: cid,
        });
        let (seq, cid) = sc(8);
        noop.record_event(ExecutionEvent::MacroExit {
            macro_name: "refine".to_string(),
            call_step_index: 5,
            elapsed_ms: Measurement::Taken(2),
            seq,
            correlation_id: cid,
        });
        let (seq, cid) = sc(9);
        noop.record_event(ExecutionEvent::ModelCall {
            step_index: 1,
            command: "GEN".to_string(),
            input_summary: FieldDescriptor::text(42),
            seq,
            correlation_id: cid,
        });
        let (seq, cid) = sc(10);
        noop.record_event(ExecutionEvent::ModelResponse {
            step_index: 1,
            command: "GEN".to_string(),
            output_summary: FieldDescriptor::text(100),
            elapsed_ms: Measurement::Taken(10),
            seq,
            correlation_id: cid,
        });
        noop.run_complete(RunManifest {
            workflow_name: "test".to_string(),
            schema_version: "0.4.6".to_string(),
            run_id: "run-0".to_string(),
            started_at: "2026-06-24T00:00:00Z".to_string(),
            elapsed_ms: Measurement::Taken(20),
            status: RunStatus::Ok,
            error_code: None,
            total_steps: 5,
            event_count: 10,
            env_trajectory: Vec::new(),
            execution_mode: ExecutionMode::default(),
            exposure_switches: Vec::new(),
            repro: ReproRecipe::default(),
        });
        // All 10 event variants + run_complete accepted without panic.
    }

    // ── RecordingProvider captures events ────────────────────────────────────

    #[test]
    fn step_start_end_emitted_in_order() {
        let recording = RecordingProvider::new();
        let (seq, cid) = sc(0);
        recording.record_event(ExecutionEvent::StepStart {
            step_index: 1,
            step_command: "GEN".to_string(),
            input_vars: vec![],
            step_identity: step_identity(1, "GEN"),
            seq,
            correlation_id: cid,
        });
        let (seq, cid) = sc(1);
        recording.record_event(ExecutionEvent::StepEnd {
            step_index: 1,
            step_command: "GEN".to_string(),
            output_vars: vec!["$out".to_string()],
            elapsed_ms: Measurement::Taken(3),
            step_identity: step_identity(1, "GEN"),
            seq,
            correlation_id: cid,
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
        let (seq, cid) = sc(0);
        recording.record_event(ExecutionEvent::ConstraintHit {
            rule_name: "!!NEVER: DELETE".to_string(),
            triggering_step_index: 2,
            halt: true,
            seq,
            correlation_id: cid,
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
        let (seq, cid) = sc(0);
        recording.record_event(ExecutionEvent::BranchTaken {
            step_index: 1,
            branch_label: "if".to_string(),
            condition_result: true,
            seq,
            correlation_id: cid,
        });
        let (seq, cid) = sc(1);
        recording.record_event(ExecutionEvent::BranchTaken {
            step_index: 2,
            branch_label: "else".to_string(),
            condition_result: false,
            seq,
            correlation_id: cid,
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
        for i in 0u64..3 {
            let (seq, cid) = sc(i);
            recording.record_event(ExecutionEvent::LoopIteration {
                step_index: 1,
                loop_type: LoopType::For,
                iteration_number: Measurement::Taken(i),
                bound_vars: vec!["$item".to_string()],
                seq,
                correlation_id: cid,
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
                    assert_eq!(*iteration_number, Measurement::Taken(i as u64));
                }
                other => panic!("expected LoopIteration(For), got {other:?}"),
            }
        }
    }

    #[test]
    fn loop_iteration_until() {
        let recording = RecordingProvider::new();
        let (seq, cid) = sc(0);
        recording.record_event(ExecutionEvent::LoopIteration {
            step_index: 2,
            loop_type: LoopType::Until,
            iteration_number: Measurement::Taken(0),
            bound_vars: vec![],
            seq,
            correlation_id: cid,
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
        let (seq, cid) = sc(0);
        recording.record_event(ExecutionEvent::MacroEnter {
            macro_name: "summarize".to_string(),
            call_step_index: 3,
            seq,
            correlation_id: cid,
        });
        let (seq, cid) = sc(1);
        recording.record_event(ExecutionEvent::MacroExit {
            macro_name: "summarize".to_string(),
            call_step_index: 3,
            elapsed_ms: Measurement::Taken(7),
            seq,
            correlation_id: cid,
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

        let (seq, cid) = sc(0);
        let event = ExecutionEvent::ModelCall {
            step_index: 1,
            command: "GEN".to_string(),
            input_summary: desc,
            seq,
            correlation_id: cid,
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
            elapsed_ms: Measurement::Taken(42),
            status: RunStatus::Ok,
            error_code: None,
            total_steps: 3,
            event_count: 6,
            env_trajectory: Vec::new(),
            execution_mode: ExecutionMode::default(),
            exposure_switches: Vec::new(),
            repro: ReproRecipe::default(),
        };
        let recording = RecordingProvider::new();
        recording.run_complete(manifest);
        let manifests = recording.manifests.lock().unwrap();
        assert_eq!(manifests[0].workflow_name, "hello");
        assert_eq!(manifests[0].run_id, "r-001");
        assert_eq!(manifests[0].event_count, 6);
        assert_eq!(manifests[0].status, RunStatus::Ok);
        assert_eq!(manifests[0].execution_mode, ExecutionMode::Real);
    }

    // ── Measurement (HO-14) ──────────────────────────────────────────────────

    #[test]
    fn measurement_unavailable_is_not_taken_zero() {
        assert_ne!(Measurement::Unavailable, Measurement::Taken(0));
    }

    #[test]
    fn measurement_round_trips_through_recording_provider() {
        let recording = RecordingProvider::new();
        let (seq, cid) = sc(0);
        recording.record_event(ExecutionEvent::StepEnd {
            step_index: 1,
            step_command: "ASK".to_string(),
            output_vars: vec![],
            elapsed_ms: Measurement::Unavailable,
            step_identity: step_identity(1, "ASK"),
            seq,
            correlation_id: cid,
        });
        let events = recording.events.lock().unwrap();
        assert!(matches!(
            &events[0],
            ExecutionEvent::StepEnd {
                elapsed_ms: Measurement::Unavailable,
                ..
            }
        ));
    }
}
