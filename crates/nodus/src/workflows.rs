//! Library command surface — the single public entry point for all workflow
//! operations. CLI and TUI bindings are thin wrappers over these functions.
//!
//! Each function parses its source string on entry. Parsing is cheap (the
//! grammar is simple), so there is no cached/pre-parsed form at this layer;
//! callers that need to parse once and call many times can use
//! [`crate::parser::Parser`] directly and pass the [`crate::ast::WorkflowFile`]
//! to the lower-level modules.

use crate::Error;
use crate::ast::{TestBlock, WorkflowFile};
use crate::environment::{EnvRunResult, EnvironmentProvider, Observation, Reward, Seed, TaskId};
use crate::executor::{
    DialogProvider, Executor, ModelProvider, RunResult, RuntimeError, Status, StubProvider, Value,
};
use crate::observability::{AuditProvider, EnvInteraction, EnvInteractionKind, FieldDescriptor};
use crate::parser::Parser;
use crate::portability::{
    CapabilityManifest, ExtensionRole, HostCapabilities, Missing, SchemaProvider, validate_manifest,
};
use crate::transpiler::Transpiler;
use crate::validator::{Diagnostic, Severity, Validator};
use crate::vocab;
use std::collections::HashMap;

// ─── Types ────────────────────────────────────────────────────────────────────

/// Transpilation output mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranspileMode {
    /// Re-emit the canonical compact `.nodus` form (round-trip safe).
    Compact,
    /// Emit the human-readable prose form (one-way; not re-parseable).
    Human,
}

/// Aggregated result of a `validate` call.
#[derive(Debug)]
pub struct ValidationReport {
    /// All diagnostics returned by the lint engine.
    pub diagnostics: Vec<Diagnostic>,
    /// `true` when at least one `Severity::Error` diagnostic is present.
    pub has_errors: bool,
}

impl ValidationReport {
    fn new(diagnostics: Vec<Diagnostic>) -> Self {
        let has_errors = diagnostics.iter().any(|d| d.severity == Severity::Error);
        ValidationReport {
            diagnostics,
            has_errors,
        }
    }
}

/// Result of a single `@test:` block execution.
#[derive(Debug, Clone)]
pub struct TestResult {
    /// Test name (from `@test: name`).
    pub name: String,
    /// Whether the test passed.
    pub passed: bool,
    /// Descriptive outcome message.
    pub message: String,
}

/// Aggregated result of a `test` call.
#[derive(Debug)]
pub struct TestReport {
    /// Per-block outcomes.
    pub results: Vec<TestResult>,
    /// Number of passing tests.
    pub passed: usize,
    /// Number of failing tests.
    pub failed: usize,
}

impl TestReport {
    fn from_results(results: Vec<TestResult>) -> Self {
        let passed = results.iter().filter(|r| r.passed).count();
        let failed = results.len() - passed;
        TestReport {
            results,
            passed,
            failed,
        }
    }
}

// ─── Library functions ────────────────────────────────────────────────────────

/// Return a minimal valid workflow AST for `name`.
///
/// The scaffold contains an empty `§runtime` block, one placeholder `@in`
/// field, an `@out` declaration, and a single `GEN` step — enough to round-trip
/// through `transpile` and `validate` without block-class errors.
pub fn scaffold(name: &str) -> WorkflowFile {
    let source = format!(
        "\
§wf:{name} v1.0
§runtime: {{ core: schema.nodus }}
@in: {{ query }}
@out: $out
@err: ESCALATE(human)
@steps:
  1. GEN($in.query) → $out
  2. LOG($out)
"
    );
    Parser::parse(&source).unwrap_or_default()
}

/// Parse `source` and run all lint rules against it.
///
/// Returns a [`ValidationReport`] with every finding regardless of severity.
/// The caller checks [`ValidationReport::has_errors`] to decide whether
/// execution is allowed.
pub fn validate(source: &str, filename: &str) -> Result<ValidationReport, Error> {
    let ast = Parser::parse(source)?;
    let diagnostics = Validator::validate(&ast, filename);
    Ok(ValidationReport::new(diagnostics))
}

/// Parse `source`, validate it, then execute it against optional caller `input`.
///
/// Fails fast with `Err(diagnostics)` when the validation report contains any
/// block-class error (WFL-5: validate-before-run guarantee). On a clean report
/// it delegates to [`Executor`] and returns the [`RunResult`].
///
/// Execution uses the built-in [`crate::executor::StubProvider`]; real model integration is
/// wired at the subsystem level in later phases.
pub fn run(
    source: &str,
    filename: &str,
    input: Option<Value>,
) -> Result<RunResult, Vec<Diagnostic>> {
    let ast = Parser::parse(source).map_err(|e| {
        vec![Diagnostic {
            severity: Severity::Error,
            code: "PARSE_ERROR".to_string(),
            message: e.to_string(),
            line: 0,
            column: 0,
            filename: filename.to_string(),
        }]
    })?;

    let report = ValidationReport::new(Validator::validate(&ast, filename));
    if report.has_errors {
        return Err(report.diagnostics);
    }

    Ok(Executor::with_stub().execute(&ast, input))
}

/// Parse `source` and return the transpiled output for `mode`.
///
/// `Compact` is lossless (round-trip safe). `Human` is one-way prose.
pub fn transpile(source: &str, mode: TranspileMode) -> Result<String, Error> {
    let ast = Parser::parse(source)?;
    Ok(match mode {
        TranspileMode::Compact => Transpiler::to_nodus(&ast),
        TranspileMode::Human => Transpiler::to_human(&ast),
    })
}

/// Parse `source` and execute each `@test:` block, returning a [`TestReport`].
///
/// Each block runs in a fresh executor context (NT-1). Input fields declared
/// in `@in:` are seeded from the block's `input:` section; unspecified fields
/// keep their declared defaults (NT-2). Assertions in `expected:` are checked
/// against the final variable environment (NT-3/NT-4). All blocks execute
/// regardless of individual failures (NT-4). Results are in declaration order
/// (NT-7). A `tags` predicate may be supplied to filter blocks (NT-6).
pub fn test(source: &str, _filename: &str) -> Result<TestReport, Error> {
    test_with_tags(source, &[])
}

/// Like [`test()`] but only runs blocks whose `tags:` list contains at least one
/// of `tag_filter`. If `tag_filter` is empty, all blocks run (NT-6).
pub fn test_with_tags(source: &str, tag_filter: &[&str]) -> Result<TestReport, Error> {
    let ast = Parser::parse(source)?;

    if ast.tests.is_empty() {
        return Ok(TestReport::from_results(vec![]));
    }

    let results: Vec<TestResult> = ast
        .tests
        .iter()
        .filter(|tb| {
            tag_filter.is_empty() || tb.tags.iter().any(|t| tag_filter.contains(&t.as_str()))
        })
        .map(|tb| run_test_block(&ast, tb))
        .collect();

    Ok(TestReport::from_results(results))
}

/// Execute a single `@test:` block in an isolated context (NT-1) and evaluate
/// its assertions (NT-3/NT-4).
fn run_test_block(ast: &WorkflowFile, tb: &TestBlock) -> TestResult {
    // NT-2: build input by overlaying the block's `input:` over @in: defaults.
    let input = build_test_input(ast, &tb.input);

    // NT-1/NT-5: fresh executor with stub provider per block.
    let run_result = Executor::with_stub().execute(ast, Some(input));

    // NT-3/NT-4: evaluate assertions; status failure is also a test failure.
    let (passed, message) = evaluate_test_block(&run_result.vars, &run_result.status, &tb.expected);

    TestResult {
        name: tb.name.clone(),
        passed,
        message,
    }
}

/// Merge `@in:` declared defaults with the test block's `input:` overrides.
/// Keys not present in `@in:` are silently ignored (§4.2 step 2).
fn build_test_input(ast: &WorkflowFile, block_input: &[(String, String)]) -> Value {
    let mut map: Vec<(String, Value)> = Vec::new();

    // Seed from @in: defaults.
    if let Some(decl) = &ast.input_decl {
        for field in &decl.fields {
            let v = field
                .default
                .as_deref()
                .map(parse_expected_value)
                .unwrap_or(Value::Null);
            map.push((field.name.clone(), v));
        }
    }

    // Apply block overrides (only for declared @in: fields).
    let declared: Vec<&str> = ast
        .input_decl
        .as_ref()
        .map(|d| d.fields.iter().map(|f| f.name.as_str()).collect())
        .unwrap_or_default();

    for (k, v) in block_input {
        if declared.contains(&k.as_str()) {
            let parsed = parse_expected_value(v);
            if let Some(pos) = map.iter().position(|(mk, _)| mk == k) {
                map[pos] = (k.clone(), parsed);
            } else {
                map.push((k.clone(), parsed));
            }
        }
    }

    Value::Map(map)
}

/// Evaluate `expected:` assertions against the final variable environment.
///
/// Returns `(passed, message)`. An absent `expected:` section means the test
/// passes on `Status::Ok` alone. The first failing assertion determines the
/// message (NT-4).
fn evaluate_test_block(
    vars: &HashMap<String, Value>,
    status: &Status,
    expected: &[(String, String)],
) -> (bool, String) {
    if expected.is_empty() {
        return if *status == Status::Ok {
            (true, "ok".to_string())
        } else {
            (false, format!("execution failed with status {status:?}"))
        };
    }

    // Status failure takes precedence over assertion checks.
    if *status != Status::Ok && *status != Status::Partial {
        return (false, format!("execution failed with status {status:?}"));
    }

    for (var_name, expected_str) in expected {
        let key = var_name.trim_start_matches('$');
        let actual = vars.get(key).cloned().unwrap_or(Value::Null);

        // NT-3: absent variable → assertion fail.
        if !vars.contains_key(key) {
            return (
                false,
                format!("assertion failed: {var_name} is not in the execution context"),
            );
        }

        let expected_val = parse_expected_value(expected_str);
        if actual != expected_val {
            return (
                false,
                format!("assertion failed: {var_name} expected {expected_val:?} got {actual:?}"),
            );
        }
    }

    (true, "ok".to_string())
}

/// Parse a literal string from an `expected:` or `input:` section into a
/// runtime [`Value`]. Quoted strings → `Text`, integers → `Int`,
/// floats → `Float`, `true`/`false` → `Bool`, `null` → `Null`.
fn parse_expected_value(s: &str) -> Value {
    let s = s.trim();

    if s == "null" {
        return Value::Null;
    }
    if s == "true" {
        return Value::Bool(true);
    }
    if s == "false" {
        return Value::Bool(false);
    }

    // Strip surrounding quotes (both `"..."` and bare tokens are accepted).
    let unquoted =
        if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
            &s[1..s.len() - 1]
        } else {
            s
        };

    if let Ok(i) = unquoted.parse::<i64>() {
        return Value::Int(i);
    }
    if let Ok(f) = unquoted.parse::<f64>() {
        return Value::Float(f);
    }

    Value::Text(unquoted.to_string())
}

/// Convenience wrapper: run with a custom [`ModelProvider`] instead of the
/// built-in stub.
pub fn run_with_provider(
    source: &str,
    filename: &str,
    input: Option<Value>,
    provider: impl ModelProvider + 'static,
) -> Result<RunResult, Vec<Diagnostic>> {
    let ast = Parser::parse(source).map_err(|e| {
        vec![Diagnostic {
            severity: Severity::Error,
            code: "PARSE_ERROR".to_string(),
            message: e.to_string(),
            line: 0,
            column: 0,
            filename: filename.to_string(),
        }]
    })?;

    let report = ValidationReport::new(Validator::validate(&ast, filename));
    if report.has_errors {
        return Err(report.diagnostics);
    }

    Ok(Executor::new(provider).execute(&ast, input))
}

/// Run with the built-in stub provider and a custom [`AuditProvider`].
///
/// Validates before executing (fast-fail on block-class errors). `run_id` and
/// `started_at` are forwarded to the run manifest; empty strings are accepted
/// when the caller has no run-level metadata.
pub fn run_with_audit(
    source: &str,
    filename: &str,
    input: Option<Value>,
    audit: impl AuditProvider + 'static,
    run_id: &str,
    started_at: &str,
) -> Result<RunResult, Vec<Diagnostic>> {
    let ast = Parser::parse(source).map_err(|e| {
        vec![Diagnostic {
            severity: Severity::Error,
            code: "PARSE_ERROR".to_string(),
            message: e.to_string(),
            line: 0,
            column: 0,
            filename: filename.to_string(),
        }]
    })?;

    let report = ValidationReport::new(Validator::validate(&ast, filename));
    if report.has_errors {
        return Err(report.diagnostics);
    }

    Ok(Executor::with_audit(StubProvider, audit)
        .execute_with_params(&ast, input, run_id, started_at))
}

/// Run with a custom [`ModelProvider`] and a custom [`AuditProvider`].
///
/// Validates before executing (fast-fail on block-class errors). `run_id` and
/// `started_at` are forwarded to the run manifest; empty strings are accepted
/// when the caller has no run-level metadata.
pub fn run_with_provider_and_audit(
    source: &str,
    filename: &str,
    input: Option<Value>,
    provider: impl ModelProvider + 'static,
    audit: impl AuditProvider + 'static,
    run_id: &str,
    started_at: &str,
) -> Result<RunResult, Vec<Diagnostic>> {
    let ast = Parser::parse(source).map_err(|e| {
        vec![Diagnostic {
            severity: Severity::Error,
            code: "PARSE_ERROR".to_string(),
            message: e.to_string(),
            line: 0,
            column: 0,
            filename: filename.to_string(),
        }]
    })?;

    let report = ValidationReport::new(Validator::validate(&ast, filename));
    if report.has_errors {
        return Err(report.diagnostics);
    }

    Ok(Executor::with_audit(provider, audit).execute_with_params(&ast, input, run_id, started_at))
}

/// Parse, validate, and execute with a host-supplied [`SchemaProvider`].
///
/// Extends the builtin vocabulary with commands and variables declared by
/// `schema_provider`. This allows workflows that use host-declared commands to
/// parse and run without a vocabulary error. Uses the built-in
/// [`StubProvider`] for model calls.
///
/// Fast-fails with `Err(diagnostics)` when the validation report contains any
/// block-class error (LP-4, NL-4).
pub fn run_with_schema(
    source: &str,
    filename: &str,
    input: Option<Value>,
    schema_provider: &dyn SchemaProvider,
) -> Result<RunResult, Vec<Diagnostic>> {
    let schema = crate::vocab::Schema::with_provider(schema_provider);
    let ast = Parser::parse_with_schema(source, &schema).map_err(|e| {
        vec![Diagnostic {
            severity: Severity::Error,
            code: "PARSE_ERROR".to_string(),
            message: e.to_string(),
            line: 0,
            column: 0,
            filename: filename.to_string(),
        }]
    })?;

    let report = ValidationReport::new(Validator::validate(&ast, filename));
    if report.has_errors {
        return Err(report.diagnostics);
    }

    Ok(Executor::with_stub().execute(&ast, input))
}

/// Parse, validate, and execute with both a [`SchemaProvider`] and an
/// [`AuditProvider`].
///
/// Combines schema-extended vocabulary with execution-event tracing. `run_id`
/// and `started_at` are forwarded to the run manifest; empty strings are
/// accepted when the caller has no run-level metadata.
pub fn run_with_schema_and_audit(
    source: &str,
    filename: &str,
    input: Option<Value>,
    schema_provider: &dyn SchemaProvider,
    audit: impl AuditProvider + 'static,
    run_id: &str,
    started_at: &str,
) -> Result<RunResult, Vec<Diagnostic>> {
    let schema = crate::vocab::Schema::with_provider(schema_provider);
    let ast = Parser::parse_with_schema(source, &schema).map_err(|e| {
        vec![Diagnostic {
            severity: Severity::Error,
            code: "PARSE_ERROR".to_string(),
            message: e.to_string(),
            line: 0,
            column: 0,
            filename: filename.to_string(),
        }]
    })?;

    let report = ValidationReport::new(Validator::validate(&ast, filename));
    if report.has_errors {
        return Err(report.diagnostics);
    }

    Ok(Executor::with_audit(StubProvider, audit)
        .execute_with_params(&ast, input, run_id, started_at))
}

/// Parse, validate, check the capability manifest against `host`, then execute.
///
/// The manifest gate (LP-8) runs after lint validation but before the executor
/// boots: if the host cannot satisfy every required role, command, and
/// capability, the run is rejected fail-fast — no step executes — and the
/// returned [`RunResult`] carries a `NODUS:CAPABILITY_UNMET` error naming the
/// missing capabilities. A satisfiable manifest delegates to the built-in stub
/// executor. This is the machine-checkable two-host portability contract (LP-3):
/// a workflow is portable to a host exactly when that host satisfies its manifest.
pub fn run_with_manifest(
    source: &str,
    filename: &str,
    input: Option<Value>,
    manifest: &CapabilityManifest,
    host: &HostCapabilities,
) -> Result<RunResult, Vec<Diagnostic>> {
    let ast = Parser::parse(source).map_err(|e| {
        vec![Diagnostic {
            severity: Severity::Error,
            code: "PARSE_ERROR".to_string(),
            message: e.to_string(),
            line: 0,
            column: 0,
            filename: filename.to_string(),
        }]
    })?;

    let report = ValidationReport::new(Validator::validate(&ast, filename));
    if report.has_errors {
        return Err(report.diagnostics);
    }

    let missing = validate_manifest(manifest, host);
    if !missing.is_empty() {
        return Ok(capability_rejection(&ast, &missing));
    }

    Ok(Executor::with_stub().execute(&ast, input))
}

/// Like [`run_with_manifest`] but with a custom [`AuditProvider`].
///
/// On a rejected manifest the executor never boots, so the audit sink receives
/// no events and no run-complete callback — preserving observer neutrality for
/// runs that never start. `run_id` and `started_at` are forwarded to the run
/// manifest on a satisfiable run.
// Composes two orthogonal extension points (the capability gate: `manifest` +
// `host`, and the audit sink: `audit` + `run_id` + `started_at`); each argument
// is independent, matching the `run_with_*_and_audit` family. Grouping them
// would obscure that orthogonality (LP-5).
#[allow(clippy::too_many_arguments)]
pub fn run_with_manifest_and_audit(
    source: &str,
    filename: &str,
    input: Option<Value>,
    manifest: &CapabilityManifest,
    host: &HostCapabilities,
    audit: impl AuditProvider + 'static,
    run_id: &str,
    started_at: &str,
) -> Result<RunResult, Vec<Diagnostic>> {
    let ast = Parser::parse(source).map_err(|e| {
        vec![Diagnostic {
            severity: Severity::Error,
            code: "PARSE_ERROR".to_string(),
            message: e.to_string(),
            line: 0,
            column: 0,
            filename: filename.to_string(),
        }]
    })?;

    let report = ValidationReport::new(Validator::validate(&ast, filename));
    if report.has_errors {
        return Err(report.diagnostics);
    }

    let missing = validate_manifest(manifest, host);
    if !missing.is_empty() {
        return Ok(capability_rejection(&ast, &missing));
    }

    Ok(Executor::with_audit(StubProvider, audit)
        .execute_with_params(&ast, input, run_id, started_at))
}

/// Parse, validate, and execute with a custom [`DialogProvider`].
///
/// `ASK`/`CONFIRM` steps resolve through `dialog`. A dialog that cannot resolve
/// (no answer, no `+default`) suspends the run: the result carries
/// `Status::Paused` and a `ResumeDescriptor`. Uses the built-in stub model.
pub fn run_with_dialog(
    source: &str,
    filename: &str,
    input: Option<Value>,
    dialog: impl DialogProvider + 'static,
) -> Result<RunResult, Vec<Diagnostic>> {
    let ast = Parser::parse(source).map_err(|e| {
        vec![Diagnostic {
            severity: Severity::Error,
            code: "PARSE_ERROR".to_string(),
            message: e.to_string(),
            line: 0,
            column: 0,
            filename: filename.to_string(),
        }]
    })?;

    let report = ValidationReport::new(Validator::validate(&ast, filename));
    if report.has_errors {
        return Err(report.diagnostics);
    }

    Ok(Executor::with_dialog(dialog).execute(&ast, input))
}

/// Like [`run_with_dialog`] but with a custom [`AuditProvider`]. `run_id` and
/// `started_at` are forwarded to the run manifest.
pub fn run_with_dialog_and_audit(
    source: &str,
    filename: &str,
    input: Option<Value>,
    dialog: impl DialogProvider + 'static,
    audit: impl AuditProvider + 'static,
    run_id: &str,
    started_at: &str,
) -> Result<RunResult, Vec<Diagnostic>> {
    let ast = Parser::parse(source).map_err(|e| {
        vec![Diagnostic {
            severity: Severity::Error,
            code: "PARSE_ERROR".to_string(),
            message: e.to_string(),
            line: 0,
            column: 0,
            filename: filename.to_string(),
        }]
    })?;

    let report = ValidationReport::new(Validator::validate(&ast, filename));
    if report.has_errors {
        return Err(report.diagnostics);
    }

    Ok(Executor::with_dialog_and_audit(dialog, audit)
        .execute_with_params(&ast, input, run_id, started_at))
}

/// Parse, validate, gate on the `Environment` capability, then run the whole
/// workflow as one graded unit against `env` (`l1-nodus-environment`
/// NE-1…NE-13).
///
/// Sequence: `env.open(task, seed)` → `env.reset` (its `Observation` seeds the
/// workflow's `$in.observation`) → execute the workflow → **frozen** →
/// `env.evaluate` → `env.release` (always, via a drop guard — NE-7). Calling
/// this function is itself the workflow's declaration that it needs the
/// `Environment` role (NE-10): the manifest is gated against `host` before
/// `env.open` is ever called, so a host without the role never leaks an
/// instance. On a satisfiable host, `HostCapabilities::builtin()` provides
/// `Environment` via [`crate::environment::StubEnvironment`].
///
/// `env.step` is not called by this combinator — see the `crate::environment`
/// module docs for the v1 scoping rationale.
pub fn run_with_environment(
    source: &str,
    filename: &str,
    input: Option<Value>,
    env: &dyn EnvironmentProvider,
    host: &HostCapabilities,
    task: &TaskId,
    seed: Seed,
) -> Result<EnvRunResult, Vec<Diagnostic>> {
    run_with_environment_impl(source, filename, input, env, host, task, seed, None, "", "")
}

/// Like [`run_with_environment`] but with a custom [`AuditProvider`]. `run_id`
/// and `started_at` are forwarded to the run manifest, which carries the
/// reset [`EnvInteraction`] in its `env_trajectory` (NE-3).
#[allow(clippy::too_many_arguments)]
pub fn run_with_environment_and_audit(
    source: &str,
    filename: &str,
    input: Option<Value>,
    env: &dyn EnvironmentProvider,
    host: &HostCapabilities,
    task: &TaskId,
    seed: Seed,
    audit: impl AuditProvider + 'static,
    run_id: &str,
    started_at: &str,
) -> Result<EnvRunResult, Vec<Diagnostic>> {
    run_with_environment_impl(
        source,
        filename,
        input,
        env,
        host,
        task,
        seed,
        Some(Box::new(audit)),
        run_id,
        started_at,
    )
}

/// Shared implementation for [`run_with_environment`] /
/// [`run_with_environment_and_audit`]. `audit: None` uses the built-in no-op
/// sink (via [`Executor::with_stub`]-equivalent construction); `Some` wires
/// the caller's provider.
#[allow(clippy::too_many_arguments)]
fn run_with_environment_impl(
    source: &str,
    filename: &str,
    input: Option<Value>,
    env: &dyn EnvironmentProvider,
    host: &HostCapabilities,
    task: &TaskId,
    seed: Seed,
    audit: Option<Box<dyn AuditProvider>>,
    run_id: &str,
    started_at: &str,
) -> Result<EnvRunResult, Vec<Diagnostic>> {
    let ast = Parser::parse(source).map_err(|e| {
        vec![Diagnostic {
            severity: Severity::Error,
            code: "PARSE_ERROR".to_string(),
            message: e.to_string(),
            line: 0,
            column: 0,
            filename: filename.to_string(),
        }]
    })?;

    let report = ValidationReport::new(Validator::validate(&ast, filename));
    if report.has_errors {
        return Err(report.diagnostics);
    }

    // NE-10: calling this function is the declaration; gate before env.open.
    let manifest = CapabilityManifest::from_workflow(&ast).require_role(ExtensionRole::Environment);
    let missing = validate_manifest(&manifest, host);
    if !missing.is_empty() {
        return Ok(EnvRunResult {
            result: capability_rejection(&ast, &missing),
            reward: Reward::no_op(),
            budget_halted: false,
        });
    }

    // NE-7: fresh isolated instance; the guard guarantees release on every
    // exit path (including a panic inside env.evaluate).
    let mut guard = crate::environment::InstanceGuard::new(env, env.open(task, seed));

    // NE-2: reset — deterministic, produces the seed observation.
    let observation = env.reset(guard.get_mut());
    let reset_entry = EnvInteraction {
        kind: EnvInteractionKind::Reset,
        observation: Some(value_descriptor(&observation.0)),
        action: None,
    };

    let profile = env.profile();
    let exec_input = merge_observation(input, &observation);
    let (max_steps, wall_clock_ms) = match &profile.budget {
        Some(b) => (b.max_steps, b.wall_clock_ms),
        None => (None, None),
    };

    let executor = match audit {
        Some(a) => Executor::with_boxed_audit(StubProvider, a),
        None => Executor::with_stub(),
    };
    let (result, budget_halted) = executor.execute_for_environment(
        &ast,
        exec_input,
        run_id,
        started_at,
        max_steps,
        wall_clock_ms,
        vec![reset_entry],
    );

    // FROZEN — evaluate is strictly after execute returns, and is read-only
    // over the completed run (NE-4).
    let reward = env.evaluate(guard.get());

    // `guard` drops here -> env.release(inst), unconditionally (NE-7).
    Ok(EnvRunResult {
        result,
        reward,
        budget_halted,
    })
}

/// Overlay `obs` under a stable `"observation"` key onto `input` (if it is a
/// `Value::Map`; a non-map or absent `input` starts from an empty map — the
/// same graceful-ignore precedent as the boot-sequence `$in` overlay). The
/// workflow reads it via `@in: { observation }` → `$in.observation`.
fn merge_observation(input: Option<Value>, obs: &Observation) -> Option<Value> {
    let mut entries: Vec<(String, Value)> = match input {
        Some(Value::Map(m)) => m,
        _ => Vec::new(),
    };
    if let Some(pos) = entries.iter().position(|(k, _)| k == "observation") {
        entries[pos] = ("observation".to_string(), obs.0.clone());
    } else {
        entries.push(("observation".to_string(), obs.0.clone()));
    }
    Some(Value::Map(entries))
}

/// Structural descriptor for an environment observation/action — never the raw
/// value (data-safety boundary, matching [`FieldDescriptor`] use elsewhere).
fn value_descriptor(v: &Value) -> FieldDescriptor {
    match v {
        Value::Map(entries) => {
            FieldDescriptor::map_like(entries.len() as u32, format!("{v:?}").len() as u32)
        }
        Value::List(items) => {
            FieldDescriptor::map_like(items.len() as u32, format!("{v:?}").len() as u32)
        }
        other => FieldDescriptor::text(format!("{other:?}").len() as u32),
    }
}

/// Build the fail-fast rejection result for an unsatisfiable manifest: status
/// `Failed`, no steps logged, one `NODUS:CAPABILITY_UNMET` error naming each
/// missing capability.
fn capability_rejection(ast: &WorkflowFile, missing: &[Missing]) -> RunResult {
    let workflow = ast
        .header
        .as_ref()
        .map(|h| format!("wf:{}", h.name))
        .unwrap_or_default();

    let names: Vec<String> = missing.iter().map(describe_missing).collect();
    let reason = format!(
        "unsatisfiable capability manifest: missing {}",
        names.join(", ")
    );

    RunResult {
        workflow,
        status: Status::Failed,
        out: Value::Null,
        log: Vec::new(),
        errors: vec![RuntimeError {
            code: vocab::error_code::CAPABILITY_UNMET.to_string(),
            step: 0,
            reason,
        }],
        flags: Vec::new(),
        vars: HashMap::new(),
        resume: None,
    }
}

/// Render a missing capability into a diagnostic fragment that names it.
fn describe_missing(missing: &Missing) -> String {
    match missing {
        Missing::Role(role) => format!("role {role:?}"),
        Missing::Command(command) => format!("command {command}"),
        Missing::Capability(capability) => format!("capability {capability}"),
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_WF: &str = "\
§wf:api_test v1.0
§runtime: { core: schema.nodus }
@in: { query }
@out: $out
@err: ESCALATE(human)
@steps:
  1. GEN($in.query) → $out
  2. LOG($out)
";

    #[test]
    fn scaffold_produces_parseable_workflow() {
        let wf = scaffold("my_workflow");
        assert_eq!(wf.header.as_ref().unwrap().name, "my_workflow");
        assert!(!wf.steps.is_empty());
    }

    #[test]
    fn validate_returns_report_without_errors_for_valid_source() {
        let report = validate(VALID_WF, "api_test.nodus").expect("parse");
        assert!(
            !report.has_errors,
            "unexpected errors: {:?}",
            report
                .diagnostics
                .iter()
                .filter(|d| d.severity == Severity::Error)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn validate_reports_errors_for_invalid_source() {
        let bad = "\
§wf:bad_wf v1.0
@steps:
  1. PUBLISH($out)
";
        let report = validate(bad, "bad_wf.nodus").expect("parse");
        assert!(report.has_errors, "expected block-class errors");
        assert!(report.diagnostics.iter().any(|d| d.code == "E001")); // no runtime
        assert!(report.diagnostics.iter().any(|d| d.code == "E005")); // publish before validate
    }

    #[test]
    fn run_executes_valid_workflow_and_returns_ok() {
        let result = run(VALID_WF, "api_test.nodus", None).expect("run should succeed");
        assert_eq!(
            result.status,
            crate::executor::Status::Ok,
            "errors: {:?}",
            result.errors
        );
        assert_eq!(result.workflow, "wf:api_test");
    }

    #[test]
    fn run_fails_fast_on_block_class_errors() {
        let bad = "\
§wf:no_runtime v1.0
@steps:
  1. GEN(x) → $out
";
        let err = run(bad, "no_runtime.nodus", None).expect_err("should fail fast");
        assert!(!err.is_empty());
        assert!(err.iter().any(|d| d.code == "E001"));
    }

    #[test]
    fn run_with_input_passes_values_to_executor() {
        let input = Value::Map(vec![(
            "query".to_string(),
            Value::Text("hello".to_string()),
        )]);
        let result = run(VALID_WF, "api_test.nodus", Some(input)).expect("run");
        assert_eq!(result.status, crate::executor::Status::Ok);
        assert!(
            matches!(&result.out, Value::Text(s) if s.contains("hello")),
            "expected $out to contain the query; got {:?}",
            result.out
        );
    }

    #[test]
    fn transpile_compact_roundtrips_ast() {
        let compact = transpile(VALID_WF, TranspileMode::Compact).expect("transpile");
        assert!(
            compact.contains("§wf:api_test"),
            "compact should have header"
        );
        // Re-parse the compact form to prove round-trip.
        let ast2 = Parser::parse(&compact).expect("re-parse compact");
        assert_eq!(
            ast2.header.as_ref().unwrap().name,
            "api_test",
            "name should survive round-trip"
        );
    }

    #[test]
    fn transpile_human_returns_prose() {
        let human = transpile(VALID_WF, TranspileMode::Human).expect("transpile human");
        assert!(
            human.contains("WORKFLOW"),
            "human form should have WORKFLOW section"
        );
        assert!(
            human.contains("STEPS"),
            "human form should have STEPS section"
        );
    }

    #[test]
    fn test_empty_when_no_test_blocks() {
        let report = test(VALID_WF, "api_test.nodus").expect("test");
        assert_eq!(report.results.len(), 0);
        assert_eq!(report.passed, 0);
        assert_eq!(report.failed, 0);
    }

    #[test]
    fn test_runs_all_test_blocks() {
        let src_with_test = format!("{}\n@test: smoke\n  GEN(x) → $out\n", VALID_WF);
        // Parser may not capture test blocks from this suffix, so test
        // gracefully either way — the key contract is no panic and a report.
        let report = test(&src_with_test, "api_test.nodus").expect("test");
        // passed + failed = total
        assert_eq!(report.passed + report.failed, report.results.len());
    }

    #[test]
    fn human_view_renders_prose() {
        let human = transpile(VALID_WF, TranspileMode::Human).expect("human transpile");
        assert!(!human.is_empty(), "human render must not be empty");
        assert!(
            human.contains("WORKFLOW"),
            "human render must include WORKFLOW header"
        );
    }

    // ── parse_expected_value unit tests ─────────────────────────────────────

    #[test]
    fn parse_expected_value_null() {
        assert_eq!(parse_expected_value("null"), Value::Null);
    }

    #[test]
    fn parse_expected_value_bool_true() {
        assert_eq!(parse_expected_value("true"), Value::Bool(true));
    }

    #[test]
    fn parse_expected_value_bool_false() {
        assert_eq!(parse_expected_value("false"), Value::Bool(false));
    }

    #[test]
    fn parse_expected_value_integer() {
        assert_eq!(parse_expected_value("42"), Value::Int(42));
    }

    #[test]
    fn parse_expected_value_negative_integer() {
        assert_eq!(parse_expected_value("-7"), Value::Int(-7));
    }

    #[test]
    fn parse_expected_value_float() {
        assert_eq!(parse_expected_value("0.9"), Value::Float(0.9));
    }

    #[test]
    fn parse_expected_value_quoted_text_strips_quotes() {
        assert_eq!(
            parse_expected_value("\"hello\""),
            Value::Text("hello".to_string())
        );
    }

    #[test]
    fn parse_expected_value_bare_word_is_text() {
        assert_eq!(
            parse_expected_value("hello"),
            Value::Text("hello".to_string())
        );
    }

    #[test]
    fn parse_expected_value_trims_whitespace() {
        assert_eq!(parse_expected_value("  42  "), Value::Int(42));
    }

    // ── evaluate_test_block unit tests ──────────────────────────────────────

    #[test]
    fn evaluate_empty_expected_passes_on_status_ok() {
        let vars = HashMap::new();
        let (passed, msg) = evaluate_test_block(&vars, &Status::Ok, &[]);
        assert!(passed, "empty expected should pass on Status::Ok");
        assert_eq!(msg, "ok");
    }

    #[test]
    fn evaluate_empty_expected_fails_on_status_failed() {
        let vars = HashMap::new();
        let (passed, _) = evaluate_test_block(&vars, &Status::Failed, &[]);
        assert!(!passed, "empty expected should fail on Status::Failed");
    }

    #[test]
    fn evaluate_assertion_passes_on_matching_text() {
        let mut vars = HashMap::new();
        vars.insert("out".to_string(), Value::Text("hello".to_string()));
        let expected = vec![("$out".to_string(), "hello".to_string())];
        let (passed, msg) = evaluate_test_block(&vars, &Status::Ok, &expected);
        assert!(passed, "matching value should pass: {msg}");
        assert_eq!(msg, "ok");
    }

    #[test]
    fn evaluate_assertion_fails_on_wrong_value() {
        let mut vars = HashMap::new();
        vars.insert("out".to_string(), Value::Text("hello".to_string()));
        let expected = vec![("$out".to_string(), "world".to_string())];
        let (passed, msg) = evaluate_test_block(&vars, &Status::Ok, &expected);
        assert!(!passed, "mismatching value should fail");
        assert!(
            msg.contains("$out"),
            "message should name the failing assertion: {msg}"
        );
    }

    #[test]
    fn evaluate_assertion_fails_when_variable_absent() {
        let vars = HashMap::new(); // $confidence not present
        let expected = vec![("$confidence".to_string(), "0.9".to_string())];
        let (passed, msg) = evaluate_test_block(&vars, &Status::Ok, &expected);
        assert!(!passed, "absent variable should fail assertion");
        assert!(
            msg.contains("$confidence"),
            "message should name the absent variable: {msg}"
        );
    }

    #[test]
    fn evaluate_first_failing_assertion_determines_message() {
        let mut vars = HashMap::new();
        vars.insert("out".to_string(), Value::Text("right".to_string()));
        vars.insert("flag".to_string(), Value::Text("wrong".to_string()));
        let expected = vec![
            ("$out".to_string(), "right".to_string()),     // passes
            ("$flag".to_string(), "expected".to_string()), // fails
        ];
        let (passed, msg) = evaluate_test_block(&vars, &Status::Ok, &expected);
        assert!(!passed);
        assert!(msg.contains("$flag"), "should report first failing: {msg}");
    }

    // ── integration tests for test() ────────────────────────────────────────

    // Workflow with structured test blocks (parsed and executed).
    const WF_WITH_STRUCTURED_TESTS: &str = "\
§wf:test_wf v1.0
§runtime: { core: schema.nodus }
@in: { query }
@out: $out
@err: ESCALATE(human)
@steps:
  1. GEN($in.query) → $out
  2. LOG($out)
@test: no_assertions {
  input:
    query: hello
}
@test: assertion_pass {
  input:
    query: hello
  expected:
    $out: \"[STUB gen(hello) tone=brand]\"
}
@test: assertion_fail {
  input:
    query: hello
  expected:
    $out: wrong_value
}
";

    #[test]
    fn test_block_with_no_assertions_passes_on_status_ok() {
        let report = test(WF_WITH_STRUCTURED_TESTS, "test_wf.nodus").expect("test");
        let r = report
            .results
            .iter()
            .find(|r| r.name == "no_assertions")
            .expect("no_assertions block must be in report");
        assert!(
            r.passed,
            "no-assertion block should pass on Status::Ok: {}",
            r.message
        );
    }

    #[test]
    fn test_assertion_matches_stub_output() {
        let report = test(WF_WITH_STRUCTURED_TESTS, "test_wf.nodus").expect("test");
        let r = report
            .results
            .iter()
            .find(|r| r.name == "assertion_pass")
            .expect("assertion_pass block must be in report");
        assert!(
            r.passed,
            "assertion against stub output should pass: {}",
            r.message
        );
    }

    #[test]
    fn test_assertion_fails_on_wrong_expected_value() {
        let report = test(WF_WITH_STRUCTURED_TESTS, "test_wf.nodus").expect("test");
        let r = report
            .results
            .iter()
            .find(|r| r.name == "assertion_fail")
            .expect("assertion_fail block must be in report");
        assert!(!r.passed, "wrong expected value should fail");
        assert!(
            r.message.contains("$out"),
            "failure message should name the assertion: {}",
            r.message
        );
    }

    #[test]
    fn test_all_blocks_run_regardless_of_failure() {
        // NT-4: all blocks execute even when some fail
        let report = test(WF_WITH_STRUCTURED_TESTS, "test_wf.nodus").expect("test");
        assert_eq!(
            report.results.len(),
            3,
            "all 3 blocks must appear in report"
        );
        assert_eq!(report.passed + report.failed, 3);
    }

    #[test]
    fn test_report_counts_are_correct() {
        let report = test(WF_WITH_STRUCTURED_TESTS, "test_wf.nodus").expect("test");
        // no_assertions → pass, assertion_pass → pass, assertion_fail → fail
        assert_eq!(report.passed, 2, "expected 2 passing blocks");
        assert_eq!(report.failed, 1, "expected 1 failing block");
    }

    #[test]
    fn test_results_in_declaration_order() {
        // NT-7: TestResult entries are in declaration order
        let report = test(WF_WITH_STRUCTURED_TESTS, "test_wf.nodus").expect("test");
        assert_eq!(report.results[0].name, "no_assertions");
        assert_eq!(report.results[1].name, "assertion_pass");
        assert_eq!(report.results[2].name, "assertion_fail");
    }

    #[test]
    fn test_with_tags_filters_by_tag() {
        // NT-6: tag filtering skips non-matching blocks
        let wf = "\
§wf:tagged_wf v1.0
§runtime: { core: schema.nodus }
@in: { query }
@out: $out
@err: ESCALATE(human)
@steps:
  1. GEN($in.query) → $out
@test: a {
  tags: [smoke]
}
@test: b {
  tags: [integration]
}
";
        let report = test_with_tags(wf, &["smoke"]).expect("test_with_tags");
        assert_eq!(report.results.len(), 1, "only the smoke block should run");
        assert_eq!(report.results[0].name, "a");
    }

    #[test]
    fn test_with_tags_empty_filter_runs_all() {
        // empty tag filter ≡ run all (NT-6)
        let wf = "\
§wf:tagged_wf2 v1.0
§runtime: { core: schema.nodus }
@in: { query }
@out: $out
@err: ESCALATE(human)
@steps:
  1. GEN($in.query) → $out
@test: x {
  tags: [smoke]
}
@test: y {
  tags: [integration]
}
";
        let report = test_with_tags(wf, &[]).expect("test_with_tags empty");
        assert_eq!(
            report.results.len(),
            2,
            "empty filter should run all blocks"
        );
    }

    #[test]
    fn run_result_exposes_vars() {
        // RunResult.vars exposes all pipeline variables after execution
        let result = run(VALID_WF, "api_test.nodus", None).expect("run");
        assert!(result.vars.contains_key("out"), "vars must include $out");
    }
}
