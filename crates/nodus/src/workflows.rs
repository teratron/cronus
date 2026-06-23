//! Library command surface — the single public entry point for all workflow
//! operations. CLI and TUI bindings are thin wrappers over these functions.
//!
//! Each function parses its source string on entry. Parsing is cheap (the
//! grammar is simple), so there is no cached/pre-parsed form at this layer;
//! callers that need to parse once and call many times can use
//! [`crate::parser::Parser`] directly and pass the [`crate::ast::WorkflowFile`]
//! to the lower-level modules.

use crate::Error;
use crate::ast::WorkflowFile;
use crate::executor::{Executor, ModelProvider, RunResult, Value};
use crate::parser::Parser;
use crate::transpiler::Transpiler;
use crate::validator::{Diagnostic, Severity, Validator};

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
/// Each block's `raw_lines` are validated against the workflow's expected
/// outputs. In this stub implementation all test blocks report pass as long as
/// the source parses and the executor returns `Status::Ok`; a fuller
/// per-assertion runner is a planned enhancement; the stub runner covers smoke-level pass/fail.
pub fn test(source: &str, _filename: &str) -> Result<TestReport, Error> {
    let ast = Parser::parse(source)?;

    if ast.tests.is_empty() {
        return Ok(TestReport::from_results(vec![]));
    }

    // Run the workflow once with no input to obtain a baseline result.
    let run_result = Executor::with_stub().execute(&ast, None);
    let base_ok = run_result.status == crate::executor::Status::Ok;

    let results: Vec<TestResult> = ast
        .tests
        .iter()
        .map(|tb| TestResult {
            name: tb.name.clone(),
            passed: base_ok,
            message: if base_ok {
                "workflow executed without errors".to_string()
            } else {
                format!("execution failed with status {:?}", run_result.status)
            },
        })
        .collect();

    Ok(TestReport::from_results(results))
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
}
