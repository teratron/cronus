//! Integration tests for the portability extension seam.
//!
//! Verifies the LP-4 vocabulary-isolation contract: builtin constants are
//! never mutated; host commands extend the Schema value only. Tests cover:
//! - `run_with_schema` recognizes host-declared commands (LP-4)
//! - Without schema extension, host commands are silently parsed as text and
//!   never appear in the execution log (LP-4 isolation gate)
//! - `NoopStorageProvider` and `NoopPolicyProvider` compile and satisfy their
//!   traits without executor wiring (LP-2 no-op contract)

use nodus::{
    executor::{Status, Value},
    observability::{AuditProvider, ExecutionEvent, RunManifest},
    portability::{
        BuiltinSchemaProvider, CapabilityManifest, ExtensionRole, HostCapabilities,
        NoopPolicyProvider, NoopStorageProvider, PolicyProvider, SchemaProvider, StorageProvider,
    },
    workflows,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

// ─── Workflow fixture ─────────────────────────────────────────────────────────

const HOST_CMD_WF: &str = r#"§wf:host_cmd_test v1.0
§runtime: { core: schema.nodus }
@in: { query }
@out: $out
@err: ESCALATE(human)
@steps:
  1. CUSTOM_CMD($in.query) → $out
  2. LOG($out)
"#;

// ─── SchemaProvider implementation used in tests ─────────────────────────────

struct TestSchemaProvider;

impl SchemaProvider for TestSchemaProvider {
    fn host_commands(&self) -> &[&str] {
        &["CUSTOM_CMD"]
    }

    fn host_reserved_variables(&self) -> &[&str] {
        &[]
    }
}

// ─── T-5T01: LP-4 vocabulary extension gate ──────────────────────────────────

#[test]
fn host_schema_extends_builtin() {
    // run_with_schema recognizes CUSTOM_CMD as a valid command call.
    // The command executes (hits the UNKNOWN_COMMAND fallthrough in the executor
    // which returns Value::Null without setting an error), so status is Ok.
    let result = workflows::run_with_schema(
        HOST_CMD_WF,
        "host_cmd_test.nodus",
        None,
        &TestSchemaProvider,
    )
    .expect("run_with_schema must succeed when CUSTOM_CMD is registered");

    assert_eq!(
        result.status,
        Status::Ok,
        "registered host command must not cause a non-Ok status; errors: {:?}",
        result.errors
    );

    // CUSTOM_CMD must appear in the execution log, proving it was dispatched.
    assert!(
        result.log.iter().any(|e| e.command == "CUSTOM_CMD"),
        "CUSTOM_CMD must appear in the execution log; log: {:?}",
        result.log
    );
}

#[test]
fn host_schema_unknown_command_not_dispatched() {
    // Without schema extension, CUSTOM_CMD is an unknown ALL_CAPS identifier.
    // The parser's step-body fallthrough treats it as raw text (a comment node),
    // so it is never dispatched to the executor. The workflow still parses and
    // validates without block-class errors (the validator never emits E002 —
    // vocabulary enforcement is the lexer/parser's gate). Status is Ok because
    // no runtime errors are added, but CUSTOM_CMD is absent from the log.
    let result = workflows::run(HOST_CMD_WF, "host_cmd_test.nodus", None)
        .expect("run must succeed — vocabulary errors are a parse-layer gate, not a block error");

    assert_eq!(
        result.status,
        Status::Ok,
        "unregistered command silently skipped must not cause non-Ok status"
    );

    // CUSTOM_CMD must NOT appear in the execution log.
    assert!(
        !result.log.iter().any(|e| e.command == "CUSTOM_CMD"),
        "unregistered CUSTOM_CMD must be absent from the log (parsed as text); log: {:?}",
        result.log
    );
}

// ─── T-5T02: Noop-provider compilation test ──────────────────────────────────

#[test]
fn noop_storage_and_policy_compile() {
    // Verify NoopStorageProvider and NoopPolicyProvider satisfy their traits
    // without any executor wiring. Exercises LP-2: every extension point ships
    // with a built-in no-op implementation that satisfies the interface.
    let storage = NoopStorageProvider;
    let policy = NoopPolicyProvider;

    // store is a no-op (does not panic, returns ())
    storage.store("key", &Value::Null);
    storage.store("key2", &Value::Text("hello".to_string()));

    // load always returns None
    assert!(storage.load("key").is_none());
    assert!(storage.load("missing").is_none());

    // evaluate always returns true
    assert!(policy.evaluate("spend_cap", &Value::Null));
    assert!(policy.evaluate("tool_access", &Value::Map(Vec::new())));

    // BuiltinSchemaProvider returns empty slices
    let bs = BuiltinSchemaProvider;
    assert!(bs.host_commands().is_empty());
    assert!(bs.host_reserved_variables().is_empty());
}

// ─── LP-8 capability manifest fixtures ───────────────────────────────────────

const MANIFEST_WF: &str = r#"§wf:manifest_test v1.0
§runtime: { core: schema.nodus }
@in: { query }
@out: $out
@err: ESCALATE(human)
@steps:
  1. GEN($in.query) → $out
  2. LOG($out)
"#;

/// Audit sink that counts every event and run-complete callback it receives.
/// Used to prove a rejected run emits nothing (observer neutrality).
struct CountingAudit {
    events: Arc<AtomicUsize>,
}

impl AuditProvider for CountingAudit {
    fn record_event(&self, _event: ExecutionEvent) {
        self.events.fetch_add(1, Ordering::SeqCst);
    }

    fn run_complete(&self, _manifest: RunManifest) {
        self.events.fetch_add(1, Ordering::SeqCst);
    }
}

// ─── Pre-run fail-fast capability gate ───────────────────────────────────────

#[test]
fn run_with_manifest_rejects_unmet_capability() {
    // A workflow needing Storage, run against the builtin host (no Storage), is
    // rejected before any step executes.
    let manifest = CapabilityManifest::new().require_role(ExtensionRole::Storage);
    let host = HostCapabilities::builtin();

    let result =
        workflows::run_with_manifest(MANIFEST_WF, "manifest_test.nodus", None, &manifest, &host)
            .expect("the manifest gate returns a RunResult, not a parse error");

    assert_eq!(
        result.status,
        Status::Failed,
        "an unsatisfiable manifest must fail the run"
    );
    assert!(
        result.log.is_empty(),
        "no step may execute on a rejected run; log: {:?}",
        result.log
    );
    assert!(
        result
            .errors
            .iter()
            .any(|e| { e.code == "NODUS:CAPABILITY_UNMET" && e.reason.contains("Storage") }),
        "rejection must name the missing Storage capability; errors: {:?}",
        result.errors
    );
}

#[test]
fn run_with_manifest_runs_when_satisfiable() {
    // A model-only workflow's derived manifest is satisfied by the builtin host.
    let manifest = CapabilityManifest::from_workflow(
        &nodus::parser::Parser::parse(MANIFEST_WF).expect("parse"),
    );
    let host = HostCapabilities::builtin();

    let result =
        workflows::run_with_manifest(MANIFEST_WF, "manifest_test.nodus", None, &manifest, &host)
            .expect("run");

    assert_eq!(
        result.status,
        Status::Ok,
        "the builtin host satisfies a model-only manifest; errors: {:?}",
        result.errors
    );
    assert!(
        !result.log.is_empty(),
        "steps must execute when satisfiable"
    );
}

// ─── LP-3 two-host substitution ──────────────────────────────────────────────

#[test]
fn manifest_lp3_two_host_substitution() {
    // The LP-3 reduction: portability ⇔ "does host B satisfy the same manifest
    // host A satisfied?"
    let manifest = CapabilityManifest::new().require_role(ExtensionRole::Storage);

    // Host A provides Storage → satisfiable → runs to completion.
    let host_a = HostCapabilities::builtin().with_role(ExtensionRole::Storage);
    let result_a =
        workflows::run_with_manifest(MANIFEST_WF, "manifest_test.nodus", None, &manifest, &host_a)
            .expect("host A run");
    assert_eq!(
        result_a.status,
        Status::Ok,
        "host A satisfies the manifest; errors: {:?}",
        result_a.errors
    );

    // Host B lacks Storage → the same manifest is unsatisfiable → rejected.
    let host_b = HostCapabilities::builtin();
    let result_b =
        workflows::run_with_manifest(MANIFEST_WF, "manifest_test.nodus", None, &manifest, &host_b)
            .expect("host B run");
    assert_eq!(
        result_b.status,
        Status::Failed,
        "host B does not satisfy the manifest"
    );
    assert!(
        result_b.errors.iter().any(|e| e.reason.contains("Storage")),
        "host B rejection must name Storage; errors: {:?}",
        result_b.errors
    );
}

// ─── Rejection precedes side effects (observer neutrality) ───────────────────

#[test]
fn manifest_rejects_before_side_effects() {
    let manifest = CapabilityManifest::new().require_role(ExtensionRole::Storage);

    // Rejected run: the audit sink must record nothing.
    let host = HostCapabilities::builtin();
    let counter = Arc::new(AtomicUsize::new(0));
    let result = workflows::run_with_manifest_and_audit(
        MANIFEST_WF,
        "manifest_test.nodus",
        None,
        &manifest,
        &host,
        CountingAudit {
            events: counter.clone(),
        },
        "run-rejected",
        "2026-06-26T00:00:00Z",
    )
    .expect("audited manifest run");

    assert_eq!(result.status, Status::Failed);
    assert_eq!(
        counter.load(Ordering::SeqCst),
        0,
        "a rejected run must emit no audit events (observer neutrality)"
    );

    // Control: a satisfiable audited run DOES emit events — proves the sink counts.
    let host_ok = HostCapabilities::builtin().with_role(ExtensionRole::Storage);
    let counter_ok = Arc::new(AtomicUsize::new(0));
    let _ = workflows::run_with_manifest_and_audit(
        MANIFEST_WF,
        "manifest_test.nodus",
        None,
        &manifest,
        &host_ok,
        CountingAudit {
            events: counter_ok.clone(),
        },
        "run-ok",
        "2026-06-26T00:00:00Z",
    )
    .expect("audited ok run");
    assert!(
        counter_ok.load(Ordering::SeqCst) > 0,
        "a real run must emit audit events"
    );
}
