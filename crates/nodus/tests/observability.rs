//! Integration tests for the observability layer.
//!
//! T-4T01: Observer neutrality — audit providers must not alter RunResult.
//! T-4T02: Public API — run_with_audit / run_with_provider_and_audit contracts.

use nodus::{
    AuditProvider, ExecutionEvent, RunManifest, RunResult, Status,
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
