//! Integration tests for the observability layer.
//!
//! T-4T01: Observer neutrality — audit providers must not alter RunResult.
//! T-4T02: Public API — run_with_audit / run_with_provider_and_audit contracts.

use nodus::{
    AuditProvider, Determinism, ExecutionEvent, ExecutionMode, Executor, Measurement, RunManifest,
    RunResult, SimFidelity, Status,
    workflows::{run_with_audit, run_with_provider_and_audit},
};
use std::sync::{Arc, Mutex};

// ─── Test helper ─────────────────────────────────────────────────────────────

#[derive(Clone)]
struct RecordingProvider {
    events: Arc<Mutex<Vec<ExecutionEvent>>>,
    manifests: Arc<Mutex<Vec<RunManifest>>>,
}

impl RecordingProvider {
    fn new() -> Self {
        RecordingProvider {
            events: Arc::new(Mutex::new(Vec::new())),
            manifests: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn event_count(&self) -> usize {
        self.events.lock().unwrap().len()
    }

    fn manifest_count(&self) -> usize {
        self.manifests.lock().unwrap().len()
    }

    fn has_event<F: Fn(&ExecutionEvent) -> bool>(&self, pred: F) -> bool {
        self.events.lock().unwrap().iter().any(pred)
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

// ─── Shared workflow source ───────────────────────────────────────────────────

const DETERMINISTIC_WF: &str = "\
§wf:obs_test v1.0
§runtime: { core: schema.nodus }
@in: { query }
@out: $out
@err: ESCALATE(human)
@steps:
  1. GEN($in.query) → $out
  2. LOG($out)
";

fn run_plain(wf: &str) -> RunResult {
    nodus::workflows::run(wf, "obs_test.nodus", None).expect("plain run")
}

// ─── T-4T01: Observer neutrality (HO-5) ──────────────────────────────────────

#[test]
fn observer_neutrality() {
    let plain = run_plain(DETERMINISTIC_WF);

    let recorder = RecordingProvider::new();
    let with_audit = run_with_audit(
        DETERMINISTIC_WF,
        "obs_test.nodus",
        None,
        recorder.clone(),
        "",
        "",
    )
    .expect("audit run");

    assert_eq!(
        plain.status, with_audit.status,
        "status must be identical with and without audit provider"
    );
    assert_eq!(
        plain.out, with_audit.out,
        "out must be identical with and without audit provider"
    );
    assert_eq!(
        plain.errors.len(),
        with_audit.errors.len(),
        "error count must be identical"
    );
}

// ─── T-4T02: Public API integration tests ────────────────────────────────────

#[test]
fn run_with_audit_api() {
    let recorder = RecordingProvider::new();
    let result = run_with_audit(
        DETERMINISTIC_WF,
        "obs_test.nodus",
        None,
        recorder.clone(),
        "run-001",
        "2026-06-24T00:00:00Z",
    )
    .expect("run_with_audit");

    assert_eq!(result.status, Status::Ok, "errors: {:?}", result.errors);

    // Audit provider must have received events.
    assert!(recorder.event_count() > 0, "expected at least one event");

    // run_complete must have been called exactly once.
    assert_eq!(
        recorder.manifest_count(),
        1,
        "expected exactly one manifest"
    );

    // GEN step must have emitted StepStart.
    assert!(
        recorder.has_event(
            |e| matches!(e, ExecutionEvent::StepStart { step_command, .. }
            if step_command == "GEN")
        ),
        "expected StepStart for GEN"
    );

    // Manifest run_id must match what was passed.
    let manifests = recorder.manifests.lock().unwrap();
    assert_eq!(manifests[0].run_id, "run-001");
}

#[test]
fn run_with_provider_and_audit_api() {
    use nodus::executor::StubProvider;

    let recorder = RecordingProvider::new();
    let result = run_with_provider_and_audit(
        DETERMINISTIC_WF,
        "obs_test.nodus",
        None,
        StubProvider,
        recorder.clone(),
        "",
        "",
    )
    .expect("run_with_provider_and_audit");

    assert_eq!(result.status, Status::Ok, "errors: {:?}", result.errors);

    // ModelCall and ModelResponse must appear for the GEN step.
    assert!(
        recorder.has_event(|e| matches!(e, ExecutionEvent::ModelCall { command, .. }
            if command == "GEN")),
        "expected ModelCall for GEN"
    );
    assert!(
        recorder.has_event(
            |e| matches!(e, ExecutionEvent::ModelResponse { command, .. }
            if command == "GEN")
        ),
        "expected ModelResponse for GEN"
    );
}

#[test]
fn run_with_audit_fast_fails_on_invalid_source() {
    let recorder = RecordingProvider::new();
    let bad = "\
§wf:bad v1.0
@steps:
  1. PUBLISH($out)
";
    let err = run_with_audit(bad, "bad.nodus", None, recorder, "", "");
    assert!(err.is_err(), "should fast-fail on validation errors");
}

// ─── T-14T01: Run-Manifest Identity & Reproducibility (HO-12/15/18/19/20) ────

const NEVER_FETCH_UPPER: &str = "\
§wf:never_fetch v1.0
§runtime: { core: schema.nodus }
!!NEVER: FETCH
@in: { url: str }
@out: $out
@err: ESCALATE(human)
@steps:
  1. FETCH($in.url) → $out
  2. LOG($out)
";

// Same rule, different literal casing — a genuinely different error_detail
// string triggered by the identical step (HO-19 message-independence probe).
const NEVER_FETCH_MIXED_CASE: &str = "\
§wf:never_fetch v1.0
§runtime: { core: schema.nodus }
!!NEVER: Fetch
@in: { url: str }
@out: $out
@err: ESCALATE(human)
@steps:
  1. FETCH($in.url) → $out
  2. LOG($out)
";

fn ast_of(src: &str) -> nodus::ast::WorkflowFile {
    nodus::parser::Parser::parse(src).expect("parse")
}

// HO-15: the same workflow run twice produces identical step_identity on
// every StepStart/StepEnd for the same step.
#[test]
fn step_identity_is_stable_across_runs() {
    let recorder_a = RecordingProvider::new();
    let recorder_b = RecordingProvider::new();

    run_with_audit(
        DETERMINISTIC_WF,
        "obs_test.nodus",
        None,
        recorder_a.clone(),
        "",
        "",
    )
    .expect("run a");
    run_with_audit(
        DETERMINISTIC_WF,
        "obs_test.nodus",
        None,
        recorder_b.clone(),
        "",
        "",
    )
    .expect("run b");

    let ids_a: Vec<String> = recorder_a
        .events
        .lock()
        .unwrap()
        .iter()
        .filter_map(|e| match e {
            ExecutionEvent::StepStart { step_identity, .. }
            | ExecutionEvent::StepEnd { step_identity, .. } => Some(step_identity.clone()),
            _ => None,
        })
        .collect();
    let ids_b: Vec<String> = recorder_b
        .events
        .lock()
        .unwrap()
        .iter()
        .filter_map(|e| match e {
            ExecutionEvent::StepStart { step_identity, .. }
            | ExecutionEvent::StepEnd { step_identity, .. } => Some(step_identity.clone()),
            _ => None,
        })
        .collect();

    assert!(!ids_a.is_empty(), "expected at least one step_identity");
    assert_eq!(
        ids_a, ids_b,
        "step_identity must be identical across two runs of the same workflow"
    );
}

// HO-15: a step's identity is derived from its own definition (number +
// command), not per-run allocated — a different command at the same position
// yields a different identity.
#[test]
fn step_identity_differs_for_a_different_command() {
    let other_wf = "\
§wf:other v1.0
§runtime: { core: schema.nodus }
@out: $out
@steps:
  1. LOG($out)
";
    let recorder_gen = RecordingProvider::new();
    let recorder_log = RecordingProvider::new();
    run_with_audit(
        DETERMINISTIC_WF,
        "obs_test.nodus",
        None,
        recorder_gen.clone(),
        "",
        "",
    )
    .expect("gen run");
    run_with_audit(other_wf, "other.nodus", None, recorder_log.clone(), "", "").expect("log run");

    let first_id = |r: &RecordingProvider| {
        r.events
            .lock()
            .unwrap()
            .iter()
            .find_map(|e| match e {
                ExecutionEvent::StepStart { step_identity, .. } => Some(step_identity.clone()),
                _ => None,
            })
            .expect("a StepStart event")
    };
    assert_ne!(
        first_id(&recorder_gen),
        first_id(&recorder_log),
        "different step definitions must have different identities"
    );
}

// HO-19: fault_identity is stable across two runs whose error_detail differs
// (rule-text casing differs), because it is built only from step_identity +
// code — never from the rendered error_detail text.
#[test]
fn fault_identity_is_message_independent() {
    let recorder_upper = RecordingProvider::new();
    let recorder_mixed = RecordingProvider::new();

    let result_upper = run_with_audit(
        NEVER_FETCH_UPPER,
        "never_fetch.nodus",
        Some(nodus::executor::Value::Map(vec![(
            "url".to_string(),
            nodus::executor::Value::Text("http://x".to_string()),
        )])),
        recorder_upper.clone(),
        "",
        "",
    )
    .expect("upper run");
    let result_mixed = run_with_audit(
        NEVER_FETCH_MIXED_CASE,
        "never_fetch.nodus",
        Some(nodus::executor::Value::Map(vec![(
            "url".to_string(),
            nodus::executor::Value::Text("http://x".to_string()),
        )])),
        recorder_mixed.clone(),
        "",
        "",
    )
    .expect("mixed run");

    assert_eq!(result_upper.status, Status::Failed);
    assert_eq!(result_mixed.status, Status::Failed);

    let fault_of = |r: &RecordingProvider| {
        r.events
            .lock()
            .unwrap()
            .iter()
            .find_map(|e| match e {
                ExecutionEvent::StepError {
                    fault_identity,
                    error_detail,
                    ..
                } => Some((fault_identity.clone(), error_detail.clone())),
                _ => None,
            })
            .expect("a StepError event")
    };
    let (fault_upper, detail_upper) = fault_of(&recorder_upper);
    let (fault_mixed, detail_mixed) = fault_of(&recorder_mixed);

    assert_ne!(
        detail_upper, detail_mixed,
        "the two rule texts must actually differ, or this test proves nothing"
    );
    assert_eq!(
        fault_upper, fault_mixed,
        "fault_identity must be identical despite differing error_detail text"
    );
}

// HO-12 / HO-18: a caller declaring Simulated + exposure switches sees both
// reflected in the manifest (and mirrored into repro); declaring nothing
// (the plain execute() path) yields the Real / empty default.
#[test]
fn execution_mode_and_exposure_switches_round_trip() {
    let recorder = RecordingProvider::new();
    let executor = Executor::with_audit(nodus::executor::StubProvider, recorder.clone());
    let ast = ast_of(DETERMINISTIC_WF);

    let _ = executor.execute_with_manifest_context(
        &ast,
        None,
        "run-sim",
        "2026-07-24T00:00:00Z",
        ExecutionMode::Simulated {
            fidelity: SimFidelity::Structural,
        },
        vec![("new_ui".to_string(), "treatment".to_string())],
    );

    let manifests = recorder.manifests.lock().unwrap();
    assert_eq!(manifests.len(), 1);
    assert_eq!(
        manifests[0].execution_mode,
        ExecutionMode::Simulated {
            fidelity: SimFidelity::Structural
        }
    );
    assert_eq!(
        manifests[0].exposure_switches,
        vec![("new_ui".to_string(), "treatment".to_string())]
    );
    // Mirrored into the reproduction recipe (HO-20).
    assert_eq!(
        manifests[0].repro.execution_mode,
        manifests[0].execution_mode
    );
    assert_eq!(
        manifests[0].repro.exposure_switches,
        manifests[0].exposure_switches
    );
}

#[test]
fn default_execution_context_is_real_with_no_switches() {
    let recorder = RecordingProvider::new();
    run_with_audit(
        DETERMINISTIC_WF,
        "obs_test.nodus",
        None,
        recorder.clone(),
        "",
        "",
    )
    .expect("plain audited run");
    let manifests = recorder.manifests.lock().unwrap();
    assert_eq!(manifests[0].execution_mode, ExecutionMode::Real);
    assert!(manifests[0].exposure_switches.is_empty());
}

// HO-20: determinism is stated from whether a model call occurred — never
// inferred from the recipe merely being present.
#[test]
fn repro_determinism_reflects_model_calls() {
    let recorder_gen = RecordingProvider::new();
    run_with_audit(
        DETERMINISTIC_WF,
        "obs_test.nodus",
        None,
        recorder_gen.clone(),
        "",
        "",
    )
    .expect("gen run");
    let manifests_gen = recorder_gen.manifests.lock().unwrap();
    assert_eq!(
        manifests_gen[0].repro.determinism,
        Determinism::ContainsModelCalls,
        "a workflow that ran GEN must be marked ContainsModelCalls"
    );

    let no_model_wf = "\
§wf:no_model v1.0
§runtime: { core: schema.nodus }
@out: $out
@steps:
  1. LOG($out)
";
    let recorder_plain = RecordingProvider::new();
    run_with_audit(
        no_model_wf,
        "no_model.nodus",
        None,
        recorder_plain.clone(),
        "",
        "",
    )
    .expect("no-model run");
    let manifests_plain = recorder_plain.manifests.lock().unwrap();
    assert_eq!(
        manifests_plain[0].repro.determinism,
        Determinism::Deterministic,
        "a workflow with no model call must be marked Deterministic"
    );
}

// HO-20: an uncapturable field is None, never silently omitted or defaulted
// to an empty-but-indistinguishable value; nodus_version matches the crate.
#[test]
fn repro_needs_vocabulary_is_none_and_version_matches_crate() {
    let recorder = RecordingProvider::new();
    run_with_audit(
        DETERMINISTIC_WF,
        "obs_test.nodus",
        None,
        recorder.clone(),
        "",
        "",
    )
    .expect("run");
    let manifests = recorder.manifests.lock().unwrap();
    assert!(manifests[0].repro.needs_vocabulary.is_none());
    assert_eq!(manifests[0].repro.nodus_version, env!("CARGO_PKG_VERSION"));
}

// HO-20: workflow_digest is deterministic for identical sources and differs
// for a materially different workflow.
#[test]
fn repro_workflow_digest_is_deterministic_and_distinguishing() {
    let recorder_a = RecordingProvider::new();
    let recorder_b = RecordingProvider::new();
    run_with_audit(
        DETERMINISTIC_WF,
        "obs_test.nodus",
        None,
        recorder_a.clone(),
        "",
        "",
    )
    .expect("run a");
    run_with_audit(
        DETERMINISTIC_WF,
        "obs_test.nodus",
        None,
        recorder_b.clone(),
        "",
        "",
    )
    .expect("run b");
    let digest_a = recorder_a.manifests.lock().unwrap()[0]
        .repro
        .workflow_digest
        .clone();
    let digest_b = recorder_b.manifests.lock().unwrap()[0]
        .repro
        .workflow_digest
        .clone();
    assert_eq!(
        digest_a, digest_b,
        "identical source must yield the same digest"
    );
    assert!(!digest_a.is_empty());

    let other_wf = "\
§wf:different v1.0
§runtime: { core: schema.nodus }
@out: $out
@steps:
  1. LOG($out)
";
    let recorder_other = RecordingProvider::new();
    run_with_audit(
        other_wf,
        "different.nodus",
        None,
        recorder_other.clone(),
        "",
        "",
    )
    .expect("other run");
    let digest_other = recorder_other.manifests.lock().unwrap()[0]
        .repro
        .workflow_digest
        .clone();
    assert_ne!(
        digest_a, digest_other,
        "a materially different workflow must yield a different digest"
    );
}

// ─── T-15T01: Aggregation-Safe Event Stream (HO-7 + HO-14) ───────────────────

const MULTI_EVENT_WF: &str = "\
§wf:multi_event v1.0
§runtime: { core: schema.nodus }
@in: { items: list }
@out: $out
@err: ESCALATE(human)
@steps:
  1. GEN(\"summarize\") → $out
  2. ~FOR $item IN $in.items
       LOG($item)
     ~END
  3. LOG($out)
";

const ASK_DEFAULT_WF: &str = "\
§wf:ask_default v1.0
§runtime: { core: schema.nodus }
@out: $answer
@err: ESCALATE(human)
@steps:
  1. ASK(question) +default=yes → $answer
  2. LOG($answer)
";

/// Extract `(seq, correlation_id)` from any `ExecutionEvent` variant — every
/// one of the 10 carries both (HO-7).
fn event_seq_and_correlation(e: &ExecutionEvent) -> (u64, &str) {
    match e {
        ExecutionEvent::StepStart {
            seq,
            correlation_id,
            ..
        }
        | ExecutionEvent::StepEnd {
            seq,
            correlation_id,
            ..
        }
        | ExecutionEvent::StepError {
            seq,
            correlation_id,
            ..
        }
        | ExecutionEvent::ConstraintHit {
            seq,
            correlation_id,
            ..
        }
        | ExecutionEvent::BranchTaken {
            seq,
            correlation_id,
            ..
        }
        | ExecutionEvent::LoopIteration {
            seq,
            correlation_id,
            ..
        }
        | ExecutionEvent::MacroEnter {
            seq,
            correlation_id,
            ..
        }
        | ExecutionEvent::MacroExit {
            seq,
            correlation_id,
            ..
        }
        | ExecutionEvent::ModelCall {
            seq,
            correlation_id,
            ..
        }
        | ExecutionEvent::ModelResponse {
            seq,
            correlation_id,
            ..
        } => (*seq, correlation_id.as_str()),
    }
}

#[test]
fn seq_is_dense_and_gap_free_across_a_multi_event_run() {
    let recorder = RecordingProvider::new();
    let input = nodus::executor::Value::Map(vec![(
        "items".to_string(),
        nodus::executor::Value::List(vec![
            nodus::executor::Value::Text("a".to_string()),
            nodus::executor::Value::Text("b".to_string()),
        ]),
    )]);
    let result = run_with_audit(
        MULTI_EVENT_WF,
        "multi_event.nodus",
        Some(input),
        recorder.clone(),
        "run-dense",
        "2026-07-24T00:00:00Z",
    )
    .expect("run_with_audit");
    assert_eq!(result.status, Status::Ok, "errors: {:?}", result.errors);

    let events = recorder.events.lock().unwrap();
    assert!(
        events.len() >= 10,
        "expected a rich multi-event trace (loop x2 + GEN + LOG); got {} events",
        events.len()
    );

    let mut seqs: Vec<u64> = events
        .iter()
        .map(|e| event_seq_and_correlation(e).0)
        .collect();
    seqs.sort_unstable();
    let expected: Vec<u64> = (0..seqs.len() as u64).collect();
    assert_eq!(
        seqs, expected,
        "seq must be dense and gap-free (0..N-1, no duplicates, no holes)"
    );
}

#[test]
fn correlation_id_is_one_per_run_and_equals_manifest_run_id() {
    let recorder = RecordingProvider::new();
    let input = nodus::executor::Value::Map(vec![(
        "items".to_string(),
        nodus::executor::Value::List(vec![nodus::executor::Value::Text("a".to_string())]),
    )]);
    run_with_audit(
        MULTI_EVENT_WF,
        "multi_event.nodus",
        Some(input),
        recorder.clone(),
        "run-corr",
        "2026-07-24T00:00:00Z",
    )
    .expect("run_with_audit");

    let events = recorder.events.lock().unwrap();
    let manifests = recorder.manifests.lock().unwrap();
    assert_eq!(manifests[0].run_id, "run-corr");
    assert!(
        events
            .iter()
            .all(|e| event_seq_and_correlation(e).1 == "run-corr"),
        "every event's correlation_id must equal RunManifest.run_id"
    );
}

#[test]
fn manifest_event_count_is_highest_seq_plus_one() {
    let recorder = RecordingProvider::new();
    let input = nodus::executor::Value::Map(vec![(
        "items".to_string(),
        nodus::executor::Value::List(vec![
            nodus::executor::Value::Text("a".to_string()),
            nodus::executor::Value::Text("b".to_string()),
        ]),
    )]);
    run_with_audit(
        MULTI_EVENT_WF,
        "multi_event.nodus",
        Some(input),
        recorder.clone(),
        "run-gapcheck",
        "2026-07-24T00:00:00Z",
    )
    .expect("run_with_audit");

    let events = recorder.events.lock().unwrap();
    let manifests = recorder.manifests.lock().unwrap();
    let highest_seq = events
        .iter()
        .map(|e| event_seq_and_correlation(e).0)
        .max()
        .expect("at least one event");
    assert_eq!(
        manifests[0].event_count as u64,
        highest_seq + 1,
        "event_count must equal the highest emitted seq + 1 (the HO-7 gap check)"
    );
}

#[test]
fn correlation_id_generated_when_run_id_empty() {
    // The plain execute() path (run_id == "") must not emit an uncorrelated
    // event — a fallback correlation_id is generated, and RunManifest.run_id
    // is set to the same value (the HO-7 identity, even on this path).
    let recorder = RecordingProvider::new();
    run_with_audit(
        DETERMINISTIC_WF,
        "obs_test.nodus",
        None,
        recorder.clone(),
        "",
        "",
    )
    .expect("run with empty run_id");
    let events = recorder.events.lock().unwrap();
    let manifests = recorder.manifests.lock().unwrap();
    assert!(
        !manifests[0].run_id.is_empty(),
        "a run_id must be generated"
    );
    assert!(
        events
            .iter()
            .all(|e| event_seq_and_correlation(e).1 == manifests[0].run_id),
        "every event's correlation_id must equal the generated run_id"
    );
}

#[test]
fn dialog_step_elapsed_is_unavailable_timed_step_is_taken() {
    // T-15C01: the dialog path never times its own step (it may suspend
    // awaiting a human) — Unavailable, not a fabricated 0.
    let recorder_dialog = RecordingProvider::new();
    run_with_audit(
        ASK_DEFAULT_WF,
        "ask_default.nodus",
        None,
        recorder_dialog.clone(),
        "",
        "",
    )
    .expect("ask_default run");
    let dialog_events = recorder_dialog.events.lock().unwrap();
    let dialog_step_end_unavailable = dialog_events.iter().any(|e| {
        matches!(
            e,
            ExecutionEvent::StepEnd {
                elapsed_ms: Measurement::Unavailable,
                ..
            }
        )
    });
    assert!(
        dialog_step_end_unavailable,
        "dialog StepEnd must carry Measurement::Unavailable, not a fabricated 0"
    );

    // Control: an ordinary timed step (GEN) is distinguishably Taken(_).
    let recorder_timed = RecordingProvider::new();
    run_with_audit(
        DETERMINISTIC_WF,
        "obs_test.nodus",
        None,
        recorder_timed.clone(),
        "",
        "",
    )
    .expect("timed run");
    let timed_events = recorder_timed.events.lock().unwrap();
    let gen_step_end_taken = timed_events.iter().any(|e| {
        matches!(
            e,
            ExecutionEvent::StepEnd {
                step_command,
                elapsed_ms: Measurement::Taken(_),
                ..
            } if step_command == "GEN"
        )
    });
    assert!(
        gen_step_end_taken,
        "an ordinary timed StepEnd must carry Measurement::Taken(_), distinguishable from Unavailable"
    );
}

#[test]
fn measurement_unavailable_is_never_equal_to_taken_zero() {
    // The whole point of HO-14: Unavailable must never compare equal to a
    // fabricated Taken(0) — they are fully distinct states.
    assert_ne!(Measurement::Unavailable, Measurement::Taken(0));
}

#[test]
fn emit_choke_point_is_the_only_record_event_call_site() {
    // Structural guard for T-15B01: executor.rs must route every emission
    // through the single choke point, not call self.audit.record_event
    // directly at N scattered sites. This test pins the source-level
    // discipline so a future edit that bypasses it is caught in review.
    let src = include_str!("../src/executor.rs");
    let direct_calls = src.matches("self.audit.record_event(").count();
    assert_eq!(
        direct_calls, 1,
        "expected exactly one self.audit.record_event( call site (inside emit() itself); found {direct_calls}"
    );
}
