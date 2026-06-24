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
    portability::{
        BuiltinSchemaProvider, NoopPolicyProvider, NoopStorageProvider, PolicyProvider,
        SchemaProvider, StorageProvider,
    },
    workflows,
};

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
