//! Validator — structural lint and schema-vocabulary checks.
//!
//! Runs 33 rules against a parsed [`WorkflowFile`] AST and returns a flat list
//! of [`Diagnostic`]s. Rules are grouped by severity:
//!
//! - **Error** (E001–E017): block execution when found.
//! - **Warning** (W001–W014): workflow runs but has unsafe or incomplete patterns.
//! - **Info** (I001–I006): style suggestions.
//!
//! AST nodes carry no source positions in this iteration; diagnostics therefore
//! report `line = 0, column = 0`. When the parser is extended to attach spans,
//! the diagnostic helpers below will naturally carry real positions.

use crate::ast::{
    Conditional, ConfigDecl, FieldConstraint, ForLoop, ParallelBlock, Stmt, UntilLoop, WorkflowFile,
};
use crate::executor::Value;
use crate::vocab;

// ─── Diagnostic types ─────────────────────────────────────────────────────────

/// Severity of a lint finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Blocks execution.
    Error,
    /// Unsafe or incomplete but runnable.
    Warning,
    /// Style suggestion only.
    Info,
}

/// A single lint finding from the validator.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Rule severity.
    pub severity: Severity,
    /// Unique rule code (e.g. `"E005"`).
    pub code: String,
    /// Human-readable message.
    pub message: String,
    /// Source line (`0` when AST carries no positions).
    pub line: u32,
    /// Source column (`0` when AST carries no positions).
    pub column: u32,
    /// The filename that was validated.
    pub filename: String,
}

impl Diagnostic {
    fn new(severity: Severity, code: &str, message: impl Into<String>, filename: &str) -> Self {
        Diagnostic {
            severity,
            code: code.to_string(),
            message: message.into(),
            line: 0,
            column: 0,
            filename: filename.to_string(),
        }
    }
}

// ─── Validator ────────────────────────────────────────────────────────────────

/// Stateless lint engine for NODUS workflow ASTs.
pub struct Validator;

impl Validator {
    /// Run all 33 lint rules against `ast` and return accumulated diagnostics.
    pub fn validate(ast: &WorkflowFile, filename: &str) -> Vec<Diagnostic> {
        let mut d = Vec::new();

        // Errors
        d.extend(Self::e001_runtime_present(ast, filename));
        // E002/E003: ordering guaranteed by parser — always pass
        d.extend(Self::e004_variables_declared(ast, filename));
        d.extend(Self::e005_publish_after_validate(ast, filename));
        // E006: ROUTE target filesystem check — deferred (no project_root)
        // E007/E008: loop/parallel balance enforced by parser — always pass
        d.extend(Self::e009_required_no_default(ast, filename));
        d.extend(Self::e010_until_has_max(ast, filename));
        // E011: core schema path filesystem check — deferred
        d.extend(Self::e012_name_matches_file(ast, filename));
        d.extend(Self::e013_no_reserved_pipeline_target(ast, filename));
        d.extend(Self::e014_no_forward_references(ast, filename));
        d.extend(Self::e015_no_duplicate_test_names(ast, filename));
        d.extend(Self::e016_halt_requires_escalate(ast, filename));
        d.extend(Self::e017_retry_bounded(ast, filename));
        d.extend(Self::e018_restart_max_bounded(ast, filename));
        d.extend(Self::e019_restart_scope(ast, filename));

        // Warnings
        d.extend(Self::w001_err_handler(ast, filename));
        d.extend(Self::w002_has_tests(ast, filename));
        d.extend(Self::w003_human_mode(ast, filename));
        d.extend(Self::w004_step_count(ast, filename));
        d.extend(Self::w005_nesting_depth(ast, filename));
        d.extend(Self::w006_route_test_coverage(ast, filename));
        d.extend(Self::w007_out_assigned(ast, filename));
        d.extend(Self::w008_log_last(ast, filename));
        d.extend(Self::w009_test_no_expected(ast, filename));
        // W010: extends resolve filesystem check — deferred
        d.extend(Self::w015_test_pair_separator(ast, filename));
        d.extend(Self::w011_known_vocabulary(ast, filename));
        d.extend(Self::w014_switch_has_arms(ast, filename));

        // Info
        d.extend(Self::i001_step_comments(ast, filename));
        d.extend(Self::i003_smoke_tag(ast, filename));
        d.extend(Self::i004_pref_tones(ast, filename));
        d.extend(Self::i006_header_fields(ast, filename));

        d
    }

    // ─── Errors ───────────────────────────────────────────────────────────────

    fn e001_runtime_present(wf: &WorkflowFile, filename: &str) -> Vec<Diagnostic> {
        if wf.runtime.is_none() {
            return vec![Diagnostic::new(
                Severity::Error,
                "E001",
                "Missing §runtime block. Agent cannot resolve schema.",
                filename,
            )];
        }
        vec![]
    }

    fn e004_variables_declared(wf: &WorkflowFile, filename: &str) -> Vec<Diagnostic> {
        let mut declared: std::collections::HashSet<String> = vocab::RESERVED_VARIABLES
            .iter()
            .map(|s| s.to_string())
            .collect();

        if let Some(input) = &wf.input_decl {
            for f in &input.fields {
                declared.insert(format!("$in.{}", f.name));
                declared.insert(format!("${}", f.name));
            }
        }

        let mut used: std::collections::HashSet<String> = std::collections::HashSet::new();
        for step in &wf.steps {
            collect_vars_step(step, &mut declared, &mut used);
        }

        used.difference(&declared)
            .filter(|var| {
                // allow dotted sub-access when root is declared
                let root = var.split('.').next().unwrap_or("");
                !declared.contains(root)
            })
            .map(|var| {
                Diagnostic::new(
                    Severity::Error,
                    "E004",
                    format!("{var} used but never assigned."),
                    filename,
                )
            })
            .collect()
    }

    fn e005_publish_after_validate(wf: &WorkflowFile, filename: &str) -> Vec<Diagnostic> {
        let mut has_validate = false;
        for step in &wf.steps {
            for cmd in extract_commands_step(step) {
                if cmd.name == "VALIDATE" {
                    has_validate = true;
                }
                if cmd.name == "PUBLISH" && !has_validate {
                    return vec![Diagnostic::new(
                        Severity::Error,
                        "E005",
                        "PUBLISH() called without prior VALIDATE().",
                        filename,
                    )];
                }
            }
        }
        vec![]
    }

    fn e009_required_no_default(wf: &WorkflowFile, filename: &str) -> Vec<Diagnostic> {
        let Some(input) = &wf.input_decl else {
            return vec![];
        };
        input
            .fields
            .iter()
            .filter(|f| !f.optional && f.default.is_some())
            .map(|f| {
                Diagnostic::new(
                    Severity::Error,
                    "E009",
                    format!(
                        "Required field '{}' has a default — use ? to mark optional.",
                        f.name
                    ),
                    filename,
                )
            })
            .collect()
    }

    fn e010_until_has_max(wf: &WorkflowFile, filename: &str) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        for step in &wf.steps {
            find_until_loops_step(step, &mut |ul: &UntilLoop| {
                if ul.max_iterations.is_none() {
                    diags.push(Diagnostic::new(
                        Severity::Error,
                        "E010",
                        "~UNTIL loop missing MAX:n. Risk of unbounded loop.",
                        filename,
                    ));
                }
            });
        }
        diags
    }

    fn e012_name_matches_file(wf: &WorkflowFile, filename: &str) -> Vec<Diagnostic> {
        if filename.is_empty() {
            return vec![];
        }
        let Some(header) = &wf.header else {
            return vec![];
        };
        // Strip directory and extension from filename.
        let basename = std::path::Path::new(filename)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        if !header.name.is_empty() && header.name != basename {
            return vec![Diagnostic::new(
                Severity::Error,
                "E012",
                format!(
                    "Workflow name '{}' does not match filename '{basename}.nodus'.",
                    header.name
                ),
                filename,
            )];
        }
        vec![]
    }

    fn e013_no_reserved_pipeline_target(wf: &WorkflowFile, filename: &str) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        for step in &wf.steps {
            for cmd in extract_commands_step(step) {
                if let Some(target) = &cmd.pipeline_target {
                    let root = target.split('.').next().unwrap_or(target);
                    if vocab::RUNTIME_OWNED_VARIABLES.contains(&root) {
                        diags.push(Diagnostic::new(
                            Severity::Error,
                            "E013",
                            format!("Pipeline target '{root}' is runtime-owned and must not be reassigned."),
                            filename,
                        ));
                    }
                }
            }
        }
        diags
    }

    fn e014_no_forward_references(wf: &WorkflowFile, filename: &str) -> Vec<Diagnostic> {
        let mut pre_declared: std::collections::HashSet<String> = vocab::RESERVED_VARIABLES
            .iter()
            .map(|s| s.split('.').next().unwrap_or(s).to_string())
            .collect();
        if let Some(input) = &wf.input_decl {
            for f in &input.fields {
                pre_declared.insert(format!("${}", f.name));
            }
        }

        // Collect pipeline targets declared anywhere within each top-level step.
        let step_targets: Vec<std::collections::HashSet<String>> = wf
            .steps
            .iter()
            .map(|step| {
                let mut targets = std::collections::HashSet::new();
                let mut dummy_used = std::collections::HashSet::new();
                collect_vars_step(step, &mut targets, &mut dummy_used);
                targets
            })
            .collect();

        let mut diags = Vec::new();
        let mut available = pre_declared.clone();

        for (i, step) in wf.steps.iter().enumerate() {
            // Within-step self-reference is permitted: include the current step's
            // own targets in the availability set before checking usages.
            let extended: std::collections::HashSet<String> = available
                .iter()
                .chain(step_targets[i].iter())
                .cloned()
                .collect();

            let mut dummy_targets = std::collections::HashSet::new();
            let mut step_used = std::collections::HashSet::new();
            collect_vars_step(step, &mut dummy_targets, &mut step_used);

            for usage in &step_used {
                let root = usage.split('.').next().unwrap_or(usage);
                if !extended.contains(root) && !extended.contains(usage.as_str()) {
                    let forward = step_targets[i + 1..]
                        .iter()
                        .any(|ts| ts.contains(root) || ts.contains(usage.as_str()));
                    if forward {
                        diags.push(Diagnostic::new(
                            Severity::Error,
                            "E014",
                            format!("{usage} referenced before assignment."),
                            filename,
                        ));
                    }
                }
            }

            available.extend(step_targets[i].iter().cloned());
        }

        diags
    }

    fn e015_no_duplicate_test_names(wf: &WorkflowFile, filename: &str) -> Vec<Diagnostic> {
        let mut seen = std::collections::HashSet::new();
        let mut diags = Vec::new();
        for tb in &wf.tests {
            if !seen.insert(tb.name.clone()) {
                diags.push(Diagnostic::new(
                    Severity::Error,
                    "E015",
                    format!("Duplicate @test: name '{}'.", tb.name),
                    filename,
                ));
            }
        }
        diags
    }

    fn e016_halt_requires_escalate(wf: &WorkflowFile, filename: &str) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        for step in &wf.steps {
            if let Some(body) = &step.body {
                find_halt_branches_stmt(body, &mut diags, filename);
            }
            for sub in &step.sub_steps {
                find_halt_branches_stmt(sub, &mut diags, filename);
            }
        }
        diags
    }

    fn e017_retry_bounded(wf: &WorkflowFile, filename: &str) -> Vec<Diagnostic> {
        wf.steps
            .iter()
            .filter_map(|step| match step.retry {
                Some(n) if n == 0 || n > 10 => Some(Diagnostic::new(
                    Severity::Error,
                    "E017",
                    "~RETRY:n requires a bound n with 1 ≤ n ≤ 10.",
                    filename,
                )),
                _ => None,
            })
            .collect()
    }

    /// `restart_max` (NL-23, the run-grain analog of `~UNTIL MAX:n`) must be a
    /// declared bound with 1 ≤ n ≤ 10, mirroring `e017_retry_bounded`'s shape.
    fn e018_restart_max_bounded(wf: &WorkflowFile, filename: &str) -> Vec<Diagnostic> {
        match wf.runtime.as_ref().and_then(|rt| rt.restart_max) {
            Some(n) if n == 0 || n > 10 => vec![Diagnostic::new(
                Severity::Error,
                "E018",
                "restart_max requires a bound n with 1 ≤ n ≤ 10.",
                filename,
            )],
            _ => vec![],
        }
    }

    /// NL-23(b): a `$restart` request is legal only from a run-boundary step,
    /// never from inside a `~FOR`/`~PARALLEL` body or a `?SWITCH` arm — which
    /// context resumes and what of the in-flight siblings survives is
    /// undefined otherwise. Statically detectable: nesting is a fixed AST
    /// shape, not a runtime value. `~UNTIL` and `~MAP` are deliberately not
    /// walked here — `~UNTIL` is outside this rule's scope (an open question
    /// flagged for a future spec pass), and `~MAP` is structurally immune:
    /// `execute_map` always rewrites its inner command's pipeline target to a
    /// scratch variable before running it, so `$restart` cannot be its target.
    fn e019_restart_scope(wf: &WorkflowFile, filename: &str) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        for step in &wf.steps {
            if let Some(body) = &step.body {
                restart_scope_stmt(body, &mut diags, filename);
            }
            for sub in &step.sub_steps {
                restart_scope_stmt(sub, &mut diags, filename);
            }
        }
        diags
    }

    // ─── Warnings ─────────────────────────────────────────────────────────────

    fn w014_switch_has_arms(wf: &WorkflowFile, filename: &str) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        for step in &wf.steps {
            if let Some(body) = &step.body {
                find_empty_switches_stmt(body, &mut diags, filename);
            }
            for sub in &step.sub_steps {
                find_empty_switches_stmt(sub, &mut diags, filename);
            }
        }
        diags
    }

    fn w001_err_handler(wf: &WorkflowFile, filename: &str) -> Vec<Diagnostic> {
        if wf.error_decl.is_none() {
            return vec![Diagnostic::new(
                Severity::Warning,
                "W001",
                "No @err handler. Errors will trigger NODUS:UNHANDLED_ERROR.",
                filename,
            )];
        }
        vec![]
    }

    fn w002_has_tests(wf: &WorkflowFile, filename: &str) -> Vec<Diagnostic> {
        if wf.tests.is_empty() {
            return vec![Diagnostic::new(
                Severity::Warning,
                "W002",
                "No @test blocks found.",
                filename,
            )];
        }
        vec![]
    }

    fn w003_human_mode(wf: &WorkflowFile, filename: &str) -> Vec<Diagnostic> {
        if wf.human_mode.is_none() {
            return vec![Diagnostic::new(
                Severity::Warning,
                "W003",
                "No HUMAN MODE section. Workflow is opaque to human reviewers.",
                filename,
            )];
        }
        vec![]
    }

    fn w004_step_count(wf: &WorkflowFile, filename: &str) -> Vec<Diagnostic> {
        let n = wf.steps.len();
        if n > 20 {
            return vec![Diagnostic::new(
                Severity::Warning,
                "W004",
                format!("Workflow has {n} steps (recommended max: 20). Consider splitting."),
                filename,
            )];
        }
        vec![]
    }

    fn w005_nesting_depth(wf: &WorkflowFile, filename: &str) -> Vec<Diagnostic> {
        for step in &wf.steps {
            if let Some(body) = &step.body
                && max_conditional_depth(body, 0) > 3
            {
                return vec![Diagnostic::new(
                    Severity::Warning,
                    "W005",
                    "Conditional nesting depth exceeds 3.",
                    filename,
                )];
            }
        }
        vec![]
    }

    fn w006_route_test_coverage(wf: &WorkflowFile, filename: &str) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        for step in &wf.steps {
            for cmd in extract_commands_step(step) {
                if cmd.name == "ROUTE" {
                    for target in &cmd.args {
                        let covered = wf
                            .tests
                            .iter()
                            .any(|t| t.raw_lines.iter().any(|l| l.contains(target.as_str())));
                        if !covered {
                            diags.push(Diagnostic::new(
                                Severity::Warning,
                                "W006",
                                format!("ROUTE({target}) has no @test coverage."),
                                filename,
                            ));
                        }
                    }
                }
            }
        }
        diags
    }

    fn w007_out_assigned(wf: &WorkflowFile, filename: &str) -> Vec<Diagnostic> {
        let Some(out_decl) = &wf.output_decl else {
            return vec![];
        };
        if out_decl.variable.is_empty() {
            return vec![];
        }
        let target = &out_decl.variable;
        for step in &wf.steps {
            for cmd in extract_commands_step(step) {
                if cmd.pipeline_target.as_deref() == Some(target.as_str()) {
                    return vec![];
                }
            }
        }
        vec![Diagnostic::new(
            Severity::Warning,
            "W007",
            format!("{target} declared in @out but never assigned in @steps."),
            filename,
        )]
    }

    fn w008_log_last(wf: &WorkflowFile, filename: &str) -> Vec<Diagnostic> {
        if wf.steps.is_empty() {
            return vec![];
        }
        // Check whether LOG appears in the last two steps.
        let check_range = if wf.steps.len() >= 2 {
            &wf.steps[wf.steps.len() - 2..]
        } else {
            &wf.steps[..]
        };
        for step in check_range.iter().rev() {
            for cmd in extract_commands_step(step) {
                if cmd.name == "LOG" {
                    return vec![];
                }
            }
        }
        // LOG exists but is not last.
        let has_log = wf
            .steps
            .iter()
            .any(|s| extract_commands_step(s).iter().any(|c| c.name == "LOG"));
        if has_log {
            return vec![Diagnostic::new(
                Severity::Warning,
                "W008",
                "LOG() is not the last step.",
                filename,
            )];
        }
        vec![]
    }

    fn w009_test_no_expected(wf: &WorkflowFile, filename: &str) -> Vec<Diagnostic> {
        wf.tests
            .iter()
            .filter(|tb| tb.expected.is_empty())
            .map(|tb| {
                Diagnostic::new(
                    Severity::Warning,
                    "W009",
                    format!(
                        "@test: '{}' has no expected: section — passes trivially on Status::Ok.",
                        tb.name
                    ),
                    filename,
                )
            })
            .collect()
    }

    /// W015 (l2-nodus-testing.md §10.3, realizing NT-9's "not a silent
    /// assertion-miss" clause) — a token inside a `@test:` block's `input:`
    /// or `expected:` section that looks like a key-value pair but uses a
    /// separator other than `:` (the corpus case: `expected: { status =
    /// SUCCESS }`). `parse_test_body` only recognizes `key : value` triples,
    /// so a non-conforming pair's tokens are silently skipped and the
    /// assertion never reaches the evaluator — this makes that drop visible.
    /// Mirrors `parse_test_body`'s own section-tracking control flow so a
    /// legitimately-consumed `key : value` triple is never re-examined.
    fn w015_test_pair_separator(wf: &WorkflowFile, filename: &str) -> Vec<Diagnostic> {
        #[derive(PartialEq, Clone, Copy)]
        enum Section {
            None,
            Input,
            Expected,
            Tags,
        }

        let mut diags = Vec::new();
        for test in &wf.tests {
            let raw = &test.raw_lines;
            let mut section = Section::None;
            let mut i = 0usize;
            while i < raw.len() {
                let tok = &raw[i];

                if matches!(tok.as_str(), "input" | "expected" | "tags")
                    && raw.get(i + 1).map(|s| s == ":").unwrap_or(false)
                {
                    section = match tok.as_str() {
                        "input" => Section::Input,
                        "expected" => Section::Expected,
                        _ => Section::Tags,
                    };
                    i += 2;
                    continue;
                }

                if matches!(section, Section::Input | Section::Expected) {
                    if matches!(tok.as_str(), "{" | "}" | ",") {
                        i += 1;
                        continue;
                    }
                    if raw.get(i + 1).map(|s| s == ":").unwrap_or(false) {
                        // A conforming key : value triple — consumed exactly
                        // as parse_test_body consumes it, so its value token
                        // is never re-examined as a would-be key below.
                        i += 3;
                        continue;
                    }
                    if let Some(sep) = raw.get(i + 1)
                        && !matches!(sep.as_str(), "{" | "}" | ",")
                        && raw.get(i + 2).is_some()
                    {
                        diags.push(Diagnostic::new(
                            Severity::Warning,
                            "W015",
                            format!(
                                "@test: '{}': '{tok}' looks like an assertion but uses '{sep}' instead of ':' — this pair is silently dropped, not evaluated.",
                                test.name
                            ),
                            filename,
                        ));
                        i += 3;
                        continue;
                    }
                }
                i += 1;
            }
        }
        diags
    }

    /// W011/W012/W013 — advisory checks against the closed vocabulary registries.
    /// `~flag` extractors, `^validator` names, and `@in` field types outside the
    /// builtin registries are warned (NL-1 strengthening); warnings never block a
    /// run, so workflows using host-specific vocabulary degrade gracefully.
    fn w011_known_vocabulary(wf: &WorkflowFile, filename: &str) -> Vec<Diagnostic> {
        let schema = vocab::Schema::builtin();
        let mut diags = Vec::new();

        // W013 — @in field types against the primitive registry.
        if let Some(input) = &wf.input_decl {
            for f in &input.fields {
                if !f.type_name.is_empty() && !schema.is_known_type(&f.type_name) {
                    diags.push(Diagnostic::new(
                        Severity::Warning,
                        "W013",
                        format!(
                            "Unknown field type '{}' on @in field '{}'.",
                            f.type_name, f.name
                        ),
                        filename,
                    ));
                }
            }
        }

        // W011 — ~flag extractors; W012 — ^validator names.
        for step in &wf.steps {
            for cmd in extract_commands_step(step) {
                for flag in &cmd.flags {
                    let name = flag.trim_start_matches('~');
                    if !schema.is_known_flag(name) {
                        diags.push(Diagnostic::new(
                            Severity::Warning,
                            "W011",
                            format!("Unknown analysis flag '~{name}'."),
                            filename,
                        ));
                    }
                }
                for validator in &cmd.validators {
                    let name = validator.trim_start_matches('^');
                    if !schema.is_known_validator(name) {
                        diags.push(Diagnostic::new(
                            Severity::Warning,
                            "W012",
                            format!("Unknown validator '^{name}'."),
                            filename,
                        ));
                    }
                }
            }
        }

        diags
    }

    // ─── Info ─────────────────────────────────────────────────────────────────

    fn i001_step_comments(wf: &WorkflowFile, filename: &str) -> Vec<Diagnostic> {
        wf.steps
            .iter()
            .filter(|s| s.comment.is_empty())
            .map(|s| {
                Diagnostic::new(
                    Severity::Info,
                    "I001",
                    format!("Step {} has no comment.", s.number),
                    filename,
                )
            })
            .collect()
    }

    fn i003_smoke_tag(wf: &WorkflowFile, filename: &str) -> Vec<Diagnostic> {
        if wf.tests.is_empty() {
            return vec![];
        }
        let has_smoke = wf
            .tests
            .iter()
            .any(|t| t.name.contains("smoke") || t.raw_lines.iter().any(|l| l.contains("smoke")));
        if !has_smoke {
            return vec![Diagnostic::new(
                Severity::Info,
                "I003",
                "No smoke test defined.",
                filename,
            )];
        }
        vec![]
    }

    fn i004_pref_tones(wf: &WorkflowFile, filename: &str) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        for pref in &wf.preferences {
            if let Some(pos) = pref.preferred.find('=') {
                let val = pref.preferred[pos + 1..].trim();
                if !val.is_empty() && !val.starts_with('$') && !vocab::VALID_TONES.contains(&val) {
                    diags.push(Diagnostic::new(
                        Severity::Info,
                        "I004",
                        format!("Unknown tone '{val}' in !PREF rule."),
                        filename,
                    ));
                }
            }
        }
        diags
    }

    fn i006_header_fields(wf: &WorkflowFile, filename: &str) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        if let Some(h) = &wf.header {
            if h.name.is_empty() {
                diags.push(Diagnostic::new(
                    Severity::Info,
                    "I006",
                    "Header name is empty.",
                    filename,
                ));
            }
            if h.version.is_empty() {
                diags.push(Diagnostic::new(
                    Severity::Info,
                    "I006",
                    "Header version is empty.",
                    filename,
                ));
            }
        }
        diags
    }
}

// ─── AST helpers ──────────────────────────────────────────────────────────────

use crate::ast::{CommandCall, Step};

fn collect_vars_step(
    step: &Step,
    declared: &mut std::collections::HashSet<String>,
    used: &mut std::collections::HashSet<String>,
) {
    if let Some(body) = &step.body {
        collect_vars_stmt(body, declared, used);
    }
    for sub in &step.sub_steps {
        collect_vars_stmt(sub, declared, used);
    }
    // NL-22: a ~COMPENSATE clause's args/target are ordinary variable
    // references too — E004 should see them the same as the step's own body.
    if let Some(comp) = &step.compensation {
        collect_vars_cmd(comp, declared, used);
    }
}

fn collect_vars_stmt(
    node: &Stmt,
    declared: &mut std::collections::HashSet<String>,
    used: &mut std::collections::HashSet<String>,
) {
    match node {
        Stmt::Command(cmd) => {
            for arg in &cmd.args {
                if arg.starts_with('$') {
                    used.insert(arg.split('.').next().unwrap_or(arg).to_string());
                }
            }
            for (_, val) in &cmd.modifiers {
                if val.starts_with('$') {
                    used.insert(val.split('.').next().unwrap_or(val).to_string());
                }
            }
            if let Some(target) = &cmd.pipeline_target {
                declared.insert(target.split('.').next().unwrap_or(target).to_string());
            }
        }
        Stmt::Conditional(cond) => collect_vars_conditional(cond, declared, used),
        Stmt::ForLoop(fl) => collect_vars_for(fl, declared, used),
        Stmt::UntilLoop(ul) => collect_vars_until(ul, declared, used),
        Stmt::Parallel(pb) => collect_vars_parallel(pb, declared, used),
        Stmt::Switch(sw) => {
            if sw.scrutinee.starts_with('$') {
                used.insert(
                    sw.scrutinee
                        .split('.')
                        .next()
                        .unwrap_or(&sw.scrutinee)
                        .to_string(),
                );
            }
            for (_, action) in &sw.arms {
                collect_vars_cmd(action, declared, used);
            }
            if let Some(default) = &sw.default {
                collect_vars_cmd(default, declared, used);
            }
        }
        Stmt::Map(mb) => {
            if mb.collection.starts_with('$') {
                used.insert(
                    mb.collection
                        .split('.')
                        .next()
                        .unwrap_or(&mb.collection)
                        .to_string(),
                );
            }
            // `~MAP` binds `$it` implicitly per element (l2-nodus-control-flow.md §4.3);
            // declare it before walking the body so E004 does not flag it as undeclared.
            declared.insert("$it".to_string());
            collect_vars_cmd(&mb.command, declared, used);
            if let Some(target) = &mb.target {
                declared.insert(target.split('.').next().unwrap_or(target).to_string());
            }
        }
        Stmt::VarRef(v) => {
            used.insert(v.name.split('.').next().unwrap_or(&v.name).to_string());
        }
        Stmt::Comment(_) => {}
    }
}

fn collect_vars_conditional(
    cond: &Conditional,
    declared: &mut std::collections::HashSet<String>,
    used: &mut std::collections::HashSet<String>,
) {
    for cap in cond.condition.split_whitespace() {
        if cap.starts_with('$') {
            used.insert(cap.split('.').next().unwrap_or(cap).to_string());
        }
    }
    if let Some(action) = &cond.action {
        collect_vars_cmd(action, declared, used);
    }
    for child in &cond.body {
        collect_vars_stmt(child, declared, used);
    }
    for br in &cond.elif_branches {
        collect_vars_conditional(br, declared, used);
    }
    if let Some(else_br) = &cond.else_branch {
        collect_vars_conditional(else_br, declared, used);
    }
}

fn collect_vars_for(
    fl: &ForLoop,
    declared: &mut std::collections::HashSet<String>,
    used: &mut std::collections::HashSet<String>,
) {
    if fl.variable.starts_with('$') {
        declared.insert(
            fl.variable
                .split('.')
                .next()
                .unwrap_or(&fl.variable)
                .to_string(),
        );
    }
    if fl.collection.starts_with('$') {
        used.insert(
            fl.collection
                .split('.')
                .next()
                .unwrap_or(&fl.collection)
                .to_string(),
        );
    }
    for child in &fl.body {
        collect_vars_stmt(child, declared, used);
    }
}

fn collect_vars_until(
    ul: &UntilLoop,
    declared: &mut std::collections::HashSet<String>,
    used: &mut std::collections::HashSet<String>,
) {
    for cap in ul.condition.split_whitespace() {
        if cap.starts_with('$') {
            used.insert(cap.split('.').next().unwrap_or(cap).to_string());
        }
    }
    for child in &ul.body {
        collect_vars_stmt(child, declared, used);
    }
}

fn collect_vars_parallel(
    pb: &ParallelBlock,
    declared: &mut std::collections::HashSet<String>,
    used: &mut std::collections::HashSet<String>,
) {
    for child in &pb.branches {
        collect_vars_stmt(child, declared, used);
    }
    if let Some(target) = &pb.join_target {
        declared.insert(target.split('.').next().unwrap_or(target).to_string());
    }
}

fn collect_vars_cmd(
    cmd: &CommandCall,
    declared: &mut std::collections::HashSet<String>,
    used: &mut std::collections::HashSet<String>,
) {
    for arg in &cmd.args {
        if arg.starts_with('$') {
            used.insert(arg.split('.').next().unwrap_or(arg).to_string());
        }
    }
    if let Some(target) = &cmd.pipeline_target {
        declared.insert(target.split('.').next().unwrap_or(target).to_string());
    }
}

fn extract_commands_step(step: &Step) -> Vec<&CommandCall> {
    let mut cmds = Vec::new();
    if let Some(body) = &step.body {
        extract_commands_stmt(body, &mut cmds);
    }
    for sub in &step.sub_steps {
        extract_commands_stmt(sub, &mut cmds);
    }
    cmds
}

fn extract_commands_stmt<'a>(node: &'a Stmt, out: &mut Vec<&'a CommandCall>) {
    match node {
        Stmt::Command(cmd) => out.push(cmd),
        Stmt::Conditional(cond) => {
            if let Some(a) = &cond.action {
                out.push(a);
            }
            for child in &cond.body {
                extract_commands_stmt(child, out);
            }
            for br in &cond.elif_branches {
                if let Some(a) = &br.action {
                    out.push(a);
                }
                for child in &br.body {
                    extract_commands_stmt(child, out);
                }
            }
            if let Some(else_br) = &cond.else_branch {
                if let Some(a) = &else_br.action {
                    out.push(a);
                }
                for child in &else_br.body {
                    extract_commands_stmt(child, out);
                }
            }
        }
        Stmt::ForLoop(fl) => {
            for child in &fl.body {
                extract_commands_stmt(child, out);
            }
        }
        Stmt::UntilLoop(ul) => {
            for child in &ul.body {
                extract_commands_stmt(child, out);
            }
        }
        Stmt::Parallel(pb) => {
            for child in &pb.branches {
                extract_commands_stmt(child, out);
            }
        }
        Stmt::Switch(sw) => {
            for (_, action) in &sw.arms {
                out.push(action);
            }
            if let Some(default) = &sw.default {
                out.push(default);
            }
        }
        Stmt::Map(mb) => out.push(&mb.command),
        Stmt::VarRef(_) | Stmt::Comment(_) => {}
    }
}

/// Flag a conditional branch carrying `!HALT` whose action is not an
/// `ESCALATE()`: a fatal stop must route through escalation in the same step.
fn check_halt_branch(branch: &Conditional, diags: &mut Vec<Diagnostic>, filename: &str) {
    if !branch.halt_flag {
        return;
    }
    let has_escalate = branch.action.as_ref().is_some_and(|a| a.name == "ESCALATE");
    if !has_escalate {
        diags.push(Diagnostic::new(
            Severity::Error,
            "E016",
            "!HALT requires an ESCALATE() action in the same step.",
            filename,
        ));
    }
}

fn find_halt_branches_stmt(node: &Stmt, diags: &mut Vec<Diagnostic>, filename: &str) {
    match node {
        Stmt::Conditional(cond) => {
            check_halt_branch(cond, diags, filename);
            for child in &cond.body {
                find_halt_branches_stmt(child, diags, filename);
            }
            for br in &cond.elif_branches {
                check_halt_branch(br, diags, filename);
                for child in &br.body {
                    find_halt_branches_stmt(child, diags, filename);
                }
            }
            if let Some(else_br) = &cond.else_branch {
                check_halt_branch(else_br, diags, filename);
                for child in &else_br.body {
                    find_halt_branches_stmt(child, diags, filename);
                }
            }
        }
        Stmt::ForLoop(fl) => {
            for child in &fl.body {
                find_halt_branches_stmt(child, diags, filename);
            }
        }
        Stmt::UntilLoop(ul) => {
            for child in &ul.body {
                find_halt_branches_stmt(child, diags, filename);
            }
        }
        Stmt::Parallel(pb) => {
            for child in &pb.branches {
                find_halt_branches_stmt(child, diags, filename);
            }
        }
        // Switch arms and map bodies carry command actions, not conditional
        // branches, so no `!HALT` flag can appear inside them.
        Stmt::Switch(_) | Stmt::Map(_) | Stmt::Command(_) | Stmt::VarRef(_) | Stmt::Comment(_) => {}
    }
}

/// Flag a `?SWITCH` with no value arms — it can only ever run its default (or
/// nothing), so the multi-branch construct adds no value.
fn find_empty_switches_stmt(node: &Stmt, diags: &mut Vec<Diagnostic>, filename: &str) {
    match node {
        Stmt::Switch(sw) => {
            if sw.arms.is_empty() {
                diags.push(Diagnostic::new(
                    Severity::Warning,
                    "W014",
                    "?SWITCH has no value arms.",
                    filename,
                ));
            }
        }
        Stmt::Conditional(cond) => {
            for child in &cond.body {
                find_empty_switches_stmt(child, diags, filename);
            }
            for br in &cond.elif_branches {
                for child in &br.body {
                    find_empty_switches_stmt(child, diags, filename);
                }
            }
            if let Some(else_br) = &cond.else_branch {
                for child in &else_br.body {
                    find_empty_switches_stmt(child, diags, filename);
                }
            }
        }
        Stmt::ForLoop(fl) => {
            for child in &fl.body {
                find_empty_switches_stmt(child, diags, filename);
            }
        }
        Stmt::UntilLoop(ul) => {
            for child in &ul.body {
                find_empty_switches_stmt(child, diags, filename);
            }
        }
        Stmt::Parallel(pb) => {
            for child in &pb.branches {
                find_empty_switches_stmt(child, diags, filename);
            }
        }
        Stmt::Map(_) | Stmt::Command(_) | Stmt::VarRef(_) | Stmt::Comment(_) => {}
    }
}

/// NL-23(b) entry: walk a statement transparently (nothing here is itself
/// forbidden), but once a `~FOR` body, `~PARALLEL` branch, or `?SWITCH` arm is
/// entered, switch to [`flag_restart_stmt`], which flags every `$restart`
/// target found beneath — including through further nesting.
fn restart_scope_stmt(node: &Stmt, diags: &mut Vec<Diagnostic>, filename: &str) {
    match node {
        Stmt::Command(_) => {}
        Stmt::Conditional(cond) => {
            for child in &cond.body {
                restart_scope_stmt(child, diags, filename);
            }
            for br in &cond.elif_branches {
                for child in &br.body {
                    restart_scope_stmt(child, diags, filename);
                }
            }
            if let Some(else_br) = &cond.else_branch {
                for child in &else_br.body {
                    restart_scope_stmt(child, diags, filename);
                }
            }
        }
        Stmt::ForLoop(fl) => {
            for child in &fl.body {
                flag_restart_stmt(child, diags, filename);
            }
        }
        // ~UNTIL is deliberately not forbidden by this rule (see the doc
        // comment on e019_restart_scope) — recurse transparently so a further
        // nested ~FOR/~PARALLEL/?SWITCH inside it is still caught.
        Stmt::UntilLoop(ul) => {
            for child in &ul.body {
                restart_scope_stmt(child, diags, filename);
            }
        }
        Stmt::Parallel(pb) => {
            for child in &pb.branches {
                flag_restart_stmt(child, diags, filename);
            }
        }
        Stmt::Switch(sw) => {
            for (_, action) in &sw.arms {
                flag_restart_command(action, diags, filename);
            }
            if let Some(default) = &sw.default {
                flag_restart_command(default, diags, filename);
            }
        }
        Stmt::Map(_) | Stmt::VarRef(_) | Stmt::Comment(_) => {}
    }
}

/// Unconditionally flags every `$restart`-targeting command reachable from
/// `node` — used once [`restart_scope_stmt`] has already entered a forbidden
/// container, so everything beneath it is in scope regardless of further
/// nesting shape.
fn flag_restart_stmt(node: &Stmt, diags: &mut Vec<Diagnostic>, filename: &str) {
    match node {
        Stmt::Command(cmd) => flag_restart_command(cmd, diags, filename),
        Stmt::Conditional(cond) => {
            if let Some(action) = &cond.action {
                flag_restart_command(action, diags, filename);
            }
            for child in &cond.body {
                flag_restart_stmt(child, diags, filename);
            }
            for br in &cond.elif_branches {
                if let Some(action) = &br.action {
                    flag_restart_command(action, diags, filename);
                }
                for child in &br.body {
                    flag_restart_stmt(child, diags, filename);
                }
            }
            if let Some(else_br) = &cond.else_branch {
                if let Some(action) = &else_br.action {
                    flag_restart_command(action, diags, filename);
                }
                for child in &else_br.body {
                    flag_restart_stmt(child, diags, filename);
                }
            }
        }
        Stmt::ForLoop(fl) => {
            for child in &fl.body {
                flag_restart_stmt(child, diags, filename);
            }
        }
        Stmt::UntilLoop(ul) => {
            for child in &ul.body {
                flag_restart_stmt(child, diags, filename);
            }
        }
        Stmt::Parallel(pb) => {
            for child in &pb.branches {
                flag_restart_stmt(child, diags, filename);
            }
        }
        Stmt::Switch(sw) => {
            for (_, action) in &sw.arms {
                flag_restart_command(action, diags, filename);
            }
            if let Some(default) = &sw.default {
                flag_restart_command(default, diags, filename);
            }
        }
        Stmt::Map(mb) => {
            if mb.target.as_deref() == Some("$restart") {
                diags.push(restart_scope_diagnostic(filename));
            }
        }
        Stmt::VarRef(_) | Stmt::Comment(_) => {}
    }
}

fn flag_restart_command(cmd: &CommandCall, diags: &mut Vec<Diagnostic>, filename: &str) {
    if cmd.pipeline_target.as_deref() == Some("$restart") {
        diags.push(restart_scope_diagnostic(filename));
    }
}

fn restart_scope_diagnostic(filename: &str) -> Diagnostic {
    Diagnostic::new(
        Severity::Error,
        "E019",
        "$restart is requestable only from a top-level run-boundary step, never from inside a ~FOR/~PARALLEL body or a ?SWITCH arm.",
        filename,
    )
}

fn find_until_loops_step<F: FnMut(&UntilLoop)>(step: &Step, f: &mut F) {
    if let Some(body) = &step.body {
        find_until_loops_stmt(body, f);
    }
    for sub in &step.sub_steps {
        find_until_loops_stmt(sub, f);
    }
}

fn find_until_loops_stmt<F: FnMut(&UntilLoop)>(node: &Stmt, f: &mut F) {
    match node {
        Stmt::UntilLoop(ul) => {
            f(ul);
            for child in &ul.body {
                find_until_loops_stmt(child, f);
            }
        }
        Stmt::ForLoop(fl) => {
            for child in &fl.body {
                find_until_loops_stmt(child, f);
            }
        }
        Stmt::Parallel(pb) => {
            for child in &pb.branches {
                find_until_loops_stmt(child, f);
            }
        }
        Stmt::Conditional(cond) => {
            if let Some(child) = cond.body.first() {
                find_until_loops_stmt(child, f);
            }
        }
        _ => {}
    }
}

fn max_conditional_depth(node: &Stmt, depth: usize) -> usize {
    match node {
        Stmt::Conditional(cond) => {
            let self_depth = depth + 1;
            let mut max = self_depth;
            for child in &cond.body {
                max = max.max(max_conditional_depth(child, self_depth));
            }
            if let Some(action) = &cond.action {
                max = max.max(max_conditional_depth(
                    &Stmt::Command(action.clone()),
                    self_depth,
                ));
            }
            for br in &cond.elif_branches {
                max = max.max(max_conditional_depth(&Stmt::Conditional(br.clone()), depth));
            }
            if let Some(else_br) = &cond.else_branch {
                max = max.max(max_conditional_depth(
                    &Stmt::Conditional(*else_br.clone()),
                    depth,
                ));
            }
            max
        }
        _ => depth,
    }
}

// ─── §config: shape check (NL-20) ─────────────────────────────────────────────

/// Why a proposed `§config` value set failed the shape check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigReason {
    /// A proposed key names no declared field.
    UnknownField,
    /// A `required` field has neither a proposed value nor a default.
    MissingRequired,
    /// A proposed value's runtime type does not match the field's declared type.
    TypeMismatch,
    /// A proposed (or defaulted) value falls outside its declared `range`.
    OutOfRange,
    /// A proposed (or defaulted) value is not a member of its declared `one_of` set.
    NotInEnum,
    /// A field's own declared `default` fails its declared type or constraint.
    BadDefault,
}

/// A single shape-check failure, naming the offending field and why (NL-20).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigViolation {
    /// The field name the violation concerns.
    pub field: String,
    /// Why the value set was rejected for this field.
    pub reason: ConfigReason,
}

/// A `§config` value set that passed the shape check (NL-20): every field
/// resolves to a value (proposed or defaulted), readable by name. `secret`
/// fields are retrievable via [`AcceptedConfig::get`] (the value is *usable*)
/// but this type exposes no separate accessor that renders a value into a
/// model-facing prompt — [`crate::workflows::run_with_config`] deliberately
/// does not merge `secret` fields into the workflow's `$in.config` surface,
/// so an ordinary step has no path to a secret at all (DC-9's write-only
/// guarantee, realized as an omission rather than a redaction filter).
#[derive(Debug, Clone, Default)]
pub struct AcceptedConfig {
    values: Vec<(String, Value, bool)>,
}

impl AcceptedConfig {
    /// The resolved value for `name`, if it is a declared field.
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.values
            .iter()
            .find(|(n, _, _)| n == name)
            .map(|(_, v, _)| v)
    }

    /// Whether `name` is a declared `secret` field.
    pub fn is_secret(&self, name: &str) -> bool {
        self.values.iter().any(|(n, _, s)| n == name && *s)
    }

    /// The non-secret fields, in declaration order — the set
    /// [`crate::workflows::run_with_config`] merges into `$in.config`.
    pub fn non_secret_fields(&self) -> impl Iterator<Item = (&str, &Value)> {
        self.values
            .iter()
            .filter(|(_, _, secret)| !secret)
            .map(|(n, v, _)| (n.as_str(), v))
    }
}

/// Coerce a raw config literal (from a declared `default` or a `range`/`one_of`
/// bound — always a raw `String` in the AST, per `ConfigField`'s doc) into a
/// typed [`Value`], directed by the field's declared primitive type. Returns
/// `None` when the literal cannot be coerced to that type (`BadDefault`).
fn coerce_config_literal(type_name: &str, raw: &str) -> Option<Value> {
    match type_name {
        "int" => raw.parse::<i64>().ok().map(Value::Int),
        "float" => raw.parse::<f64>().ok().map(Value::Float),
        "bool" => match raw {
            "true" => Some(Value::Bool(true)),
            "false" => Some(Value::Bool(false)),
            _ => None,
        },
        "null" => (raw == "null").then_some(Value::Null),
        // str/url/ts/any/list/obj: config field values captured by the parser
        // are always single literal tokens (no `[…]`/`{…}` value syntax), so
        // list/obj-typed fields carry no literal default/constraint members —
        // only a proposed `Value::List`/`Value::Map` from the caller is valid.
        _ => Some(Value::Text(raw.to_string())),
    }
}

/// Does `value`'s runtime shape match `type_name`?
fn config_type_matches(value: &Value, type_name: &str) -> bool {
    match type_name {
        "int" => matches!(value, Value::Int(_)),
        "float" => matches!(value, Value::Float(_) | Value::Int(_)),
        "bool" => matches!(value, Value::Bool(_)),
        "str" | "url" | "ts" => matches!(value, Value::Text(_)),
        "list" => matches!(value, Value::List(_)),
        "obj" => matches!(value, Value::Map(_)),
        "null" => matches!(value, Value::Null),
        // "any" and any non-canonical type name: unrestricted (advisory
        // unknown-type warnings are W013's concern, not this pre-run gate).
        _ => true,
    }
}

/// Does `value` satisfy `constraint`, given the field's declared type (used to
/// coerce the constraint's raw literal bounds/members)?
fn config_satisfies_constraint(
    value: &Value,
    constraint: &FieldConstraint,
    type_name: &str,
) -> bool {
    match constraint {
        FieldConstraint::Range { lo, hi } => {
            let (Some(lo_v), Some(hi_v)) = (
                coerce_config_literal(type_name, lo),
                coerce_config_literal(type_name, hi),
            ) else {
                return false;
            };
            match (value, &lo_v, &hi_v) {
                (Value::Int(v), Value::Int(l), Value::Int(h)) => v >= l && v <= h,
                (Value::Float(v), Value::Float(l), Value::Float(h)) => v >= l && v <= h,
                (Value::Int(v), Value::Float(l), Value::Float(h)) => {
                    (*v as f64) >= *l && (*v as f64) <= *h
                }
                (Value::Float(v), Value::Int(l), Value::Int(h)) => {
                    *v >= (*l as f64) && *v <= (*h as f64)
                }
                _ => false,
            }
        }
        FieldConstraint::OneOf(members) => members
            .iter()
            .any(|m| coerce_config_literal(type_name, m).as_ref() == Some(value)),
    }
}

/// Pure, pre-run shape check (NL-20 / DC-3 / DC-4): validate a proposed value
/// set against `decl` in one pass, reporting every violation and applying
/// none. On success, returns an [`AcceptedConfig`] where every field resolves
/// to a value — the proposed one, or the field's `default`, or `Value::Null`
/// for an optional field with neither. Never mutates any prior accepted set;
/// the caller decides what "prior" means (see
/// [`crate::workflows::run_with_config`]).
pub fn check_config_values(
    decl: &ConfigDecl,
    proposed: &[(String, Value)],
) -> std::result::Result<AcceptedConfig, Vec<ConfigViolation>> {
    let mut violations = Vec::new();

    for (name, _) in proposed {
        if !decl.fields.iter().any(|f| &f.name == name) {
            violations.push(ConfigViolation {
                field: name.clone(),
                reason: ConfigReason::UnknownField,
            });
        }
    }

    let mut accepted: Vec<(String, Value, bool)> = Vec::new();

    for field in &decl.fields {
        let default_value = field
            .default
            .as_deref()
            .map(|raw| coerce_config_literal(&field.type_name, raw));
        if let Some(None) = default_value {
            violations.push(ConfigViolation {
                field: field.name.clone(),
                reason: ConfigReason::BadDefault,
            });
        }
        if let (Some(Some(dv)), Some(c)) = (&default_value, &field.constraint)
            && !config_satisfies_constraint(dv, c, &field.type_name)
        {
            violations.push(ConfigViolation {
                field: field.name.clone(),
                reason: ConfigReason::BadDefault,
            });
        }

        match proposed.iter().find(|(n, _)| n == &field.name) {
            Some((_, value)) => {
                if !config_type_matches(value, &field.type_name) {
                    violations.push(ConfigViolation {
                        field: field.name.clone(),
                        reason: ConfigReason::TypeMismatch,
                    });
                    continue;
                }
                if let Some(constraint) = &field.constraint
                    && !config_satisfies_constraint(value, constraint, &field.type_name)
                {
                    let reason = match constraint {
                        FieldConstraint::Range { .. } => ConfigReason::OutOfRange,
                        FieldConstraint::OneOf(_) => ConfigReason::NotInEnum,
                    };
                    violations.push(ConfigViolation {
                        field: field.name.clone(),
                        reason,
                    });
                    continue;
                }
                accepted.push((field.name.clone(), value.clone(), field.secret));
            }
            None => match default_value.flatten() {
                Some(dv) => accepted.push((field.name.clone(), dv, field.secret)),
                None if field.required => violations.push(ConfigViolation {
                    field: field.name.clone(),
                    reason: ConfigReason::MissingRequired,
                }),
                None => accepted.push((field.name.clone(), Value::Null, field.secret)),
            },
        }
    }

    if violations.is_empty() {
        Ok(AcceptedConfig { values: accepted })
    } else {
        Err(violations)
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    const MINIMAL: &str = "\
§wf:test_wf v1.0
§runtime: { core: schema.nodus }
@steps:
  1. GEN(reply) → $out
  2. LOG($out)
";

    #[test]
    fn e001_fires_when_runtime_absent() {
        let src = "\
§wf:no_runtime v1.0
@steps:
  1. GEN(reply) → $out
";
        let ast = Parser::parse(src).expect("parse");
        let diags = Validator::validate(&ast, "no_runtime.nodus");
        let e001: Vec<_> = diags.iter().filter(|d| d.code == "E001").collect();
        assert!(!e001.is_empty(), "expected E001");
        assert!(e001.iter().all(|d| d.severity == Severity::Error));
    }

    #[test]
    fn no_e001_when_runtime_present() {
        let ast = Parser::parse(MINIMAL).expect("parse");
        let diags = Validator::validate(&ast, "test_wf.nodus");
        assert!(!diags.iter().any(|d| d.code == "E001"));
    }

    #[test]
    fn e005_fires_when_publish_before_validate() {
        let src = "\
§wf:pub_test v1.0
§runtime: { core: schema.nodus }
@steps:
  1. PUBLISH($out)
";
        let ast = Parser::parse(src).expect("parse");
        let diags = Validator::validate(&ast, "pub_test.nodus");
        assert!(
            diags.iter().any(|d| d.code == "E005"),
            "expected E005 for PUBLISH without VALIDATE"
        );
    }

    #[test]
    fn no_e005_when_validate_precedes_publish() {
        let src = "\
§wf:vp_test v1.0
§runtime: { core: schema.nodus }
@steps:
  1. VALIDATE($out)
  2. PUBLISH($out)
";
        let ast = Parser::parse(src).expect("parse");
        let diags = Validator::validate(&ast, "vp_test.nodus");
        assert!(!diags.iter().any(|d| d.code == "E005"));
    }

    #[test]
    fn e017_fires_on_unbounded_or_oversized_retry() {
        let missing = "\
§wf:retry_missing v1.0
§runtime: { core: schema.nodus }
@out: $out
@err: ESCALATE(human)
@steps:
  1. ~RETRY GEN(x) → $out
";
        let ast = Parser::parse(missing).expect("parse");
        let diags = Validator::validate(&ast, "retry_missing.nodus");
        assert!(
            diags
                .iter()
                .any(|d| d.code == "E017" && d.severity == Severity::Error),
            "expected E017 for ~RETRY with no bound; got: {diags:?}"
        );

        let oversized = "\
§wf:retry_big v1.0
§runtime: { core: schema.nodus }
@out: $out
@err: ESCALATE(human)
@steps:
  1. ~RETRY:15 GEN(x) → $out
";
        let ast = Parser::parse(oversized).expect("parse");
        let diags = Validator::validate(&ast, "retry_big.nodus");
        assert!(
            diags.iter().any(|d| d.code == "E017"),
            "expected E017 for ~RETRY:15 (over the cap of 10); got: {diags:?}"
        );
    }

    #[test]
    fn no_e017_for_valid_retry_bound() {
        let src = "\
§wf:retry_ok v1.0
§runtime: { core: schema.nodus }
@out: $out
@err: ESCALATE(human)
@steps:
  1. ~RETRY:3 GEN(x) → $out
";
        let ast = Parser::parse(src).expect("parse");
        let diags = Validator::validate(&ast, "retry_ok.nodus");
        assert!(
            !diags.iter().any(|d| d.code == "E017"),
            "a 1..=10 bound must not trip E017; got: {diags:?}"
        );
    }

    #[test]
    fn w014_fires_on_switch_with_no_arms() {
        let src = "\
§wf:empty_switch v1.0
§runtime: { core: schema.nodus }
@out: $out
@err: ESCALATE(human)
@steps:
  1. ?SWITCH $in.x:
    * → LOG(y)
  ~END
";
        let ast = Parser::parse(src).expect("parse");
        let diags = Validator::validate(&ast, "empty_switch.nodus");
        assert!(
            diags
                .iter()
                .any(|d| d.code == "W014" && d.severity == Severity::Warning),
            "expected W014 for a ?SWITCH with no value arms; got: {diags:?}"
        );
    }

    #[test]
    fn no_w014_when_switch_has_arms() {
        let src = "\
§wf:ok_switch v1.0
§runtime: { core: schema.nodus }
@out: $out
@err: ESCALATE(human)
@steps:
  1. ?SWITCH $in.x:
    urgent → LOG(y)
  ~END
";
        let ast = Parser::parse(src).expect("parse");
        let diags = Validator::validate(&ast, "ok_switch.nodus");
        assert!(
            !diags.iter().any(|d| d.code == "W014"),
            "a switch with value arms must not trip W014; got: {diags:?}"
        );
    }

    #[test]
    fn e016_fires_on_halt_without_escalate() {
        let src = "\
§wf:bad_halt v1.0
§runtime: { core: schema.nodus }
@out: $out
@err: ESCALATE(human)
@steps:
  1. ?IF 1 > 0 → GEN(x) !HALT
  2. LOG(done) → $out
";
        let ast = Parser::parse(src).expect("parse");
        let diags = Validator::validate(&ast, "bad_halt.nodus");
        assert!(
            diags
                .iter()
                .any(|d| d.code == "E016" && d.severity == Severity::Error),
            "expected E016 for !HALT without ESCALATE; got: {diags:?}"
        );
    }

    #[test]
    fn no_e016_when_halt_has_escalate() {
        let src = "\
§wf:good_halt v1.0
§runtime: { core: schema.nodus }
@out: $out
@err: ESCALATE(human)
@steps:
  1. ?IF 1 > 0 → ESCALATE(human) !HALT
  2. LOG(done) → $out
";
        let ast = Parser::parse(src).expect("parse");
        let diags = Validator::validate(&ast, "good_halt.nodus");
        assert!(
            !diags.iter().any(|d| d.code == "E016"),
            "ESCALATE alongside !HALT must not trip E016; got: {diags:?}"
        );
    }

    #[test]
    fn e009_fires_for_required_field_with_default() {
        // E009 requires !optional && default.is_some(). The parser auto-sets
        // optional = true when a default is present, so construct the AST directly.
        use crate::ast::{InputDecl, InputField, WorkflowFile};

        let wf = WorkflowFile {
            input_decl: Some(InputDecl {
                fields: vec![InputField {
                    name: "tone".to_string(),
                    optional: false, // required — but has a default (protocol error)
                    default: Some("warm".to_string()),
                    ..Default::default()
                }],
            }),
            ..Default::default()
        };
        let diags = Validator::validate(&wf, "");
        assert!(
            diags.iter().any(|d| d.code == "E009"),
            "expected E009 for required field with default; got: {:?}",
            diags.iter().map(|d| &d.code).collect::<Vec<_>>()
        );
    }

    #[test]
    fn e010_fires_for_until_missing_max() {
        use crate::ast::{Step, Stmt, UntilLoop, WorkflowFile};

        let ul = UntilLoop {
            condition: "$done = true".to_string(),
            max_iterations: None, // missing MAX
            body: vec![],
        };
        let wf = WorkflowFile {
            steps: vec![Step {
                number: 1,
                body: Some(Stmt::UntilLoop(ul)),
                ..Default::default()
            }],
            ..Default::default()
        };
        let diags = Validator::validate(&wf, "");
        assert!(
            diags.iter().any(|d| d.code == "E010"),
            "expected E010 for ~UNTIL without MAX"
        );
    }

    #[test]
    fn no_e010_when_max_present() {
        use crate::ast::{Step, Stmt, UntilLoop, WorkflowFile};

        let ul = UntilLoop {
            condition: "$done = true".to_string(),
            max_iterations: Some(5),
            body: vec![],
        };
        let wf = WorkflowFile {
            steps: vec![Step {
                number: 1,
                body: Some(Stmt::UntilLoop(ul)),
                ..Default::default()
            }],
            ..Default::default()
        };
        let diags = Validator::validate(&wf, "");
        assert!(!diags.iter().any(|d| d.code == "E010"));
    }

    #[test]
    fn e012_fires_when_name_differs_from_filename() {
        let ast = Parser::parse(MINIMAL).expect("parse");
        let diags = Validator::validate(&ast, "wrong_name.nodus");
        assert!(
            diags.iter().any(|d| d.code == "E012"),
            "expected E012 for name mismatch"
        );
    }

    #[test]
    fn w001_fires_when_no_err_handler() {
        let ast = Parser::parse(MINIMAL).expect("parse");
        let diags = Validator::validate(&ast, "test_wf.nodus");
        assert!(
            diags.iter().any(|d| d.code == "W001"),
            "expected W001 for missing @err"
        );
    }

    #[test]
    fn w002_fires_when_no_tests() {
        let ast = Parser::parse(MINIMAL).expect("parse");
        let diags = Validator::validate(&ast, "test_wf.nodus");
        assert!(diags.iter().any(|d| d.code == "W002"));
    }

    #[test]
    fn w007_fires_when_out_not_assigned() {
        let src = "\
§wf:out_test v1.0
§runtime: { core: schema.nodus }
@out: $result
@steps:
  1. GEN(reply) → $out
";
        let ast = Parser::parse(src).expect("parse");
        let diags = Validator::validate(&ast, "out_test.nodus");
        assert!(
            diags.iter().any(|d| d.code == "W007"),
            "expected W007 when $result is never assigned"
        );
    }

    #[test]
    fn i006_fires_when_version_absent() {
        let src = "§wf:no_ver\n§runtime: { core: schema.nodus }\n@steps:\n  1. GEN(x) → $out\n";
        let ast = Parser::parse(src).expect("parse");
        let diags = Validator::validate(&ast, "no_ver.nodus");
        assert!(
            diags
                .iter()
                .any(|d| d.code == "I006" && d.severity == Severity::Info)
        );
    }

    #[test]
    fn e013_fires_when_pipeline_target_is_reserved() {
        use crate::ast::{CommandCall, RuntimeBlock, Step, Stmt, WorkflowFile};

        let wf = WorkflowFile {
            runtime: Some(RuntimeBlock {
                core: "schema.nodus".to_string(),
                ..Default::default()
            }),
            steps: vec![Step {
                number: 1,
                body: Some(Stmt::Command(CommandCall {
                    name: "GEN".to_string(),
                    args: vec!["prompt".to_string()],
                    pipeline_target: Some("$in".to_string()),
                    ..Default::default()
                })),
                ..Default::default()
            }],
            ..Default::default()
        };
        let diags = Validator::validate(&wf, "");
        assert!(
            diags.iter().any(|d| d.code == "E013"),
            "expected E013 for pipeline target shadowing runtime-owned $in; got: {:?}",
            diags
                .iter()
                .map(|d| (&d.code, &d.message))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn no_e013_when_target_is_user_defined() {
        use crate::ast::{CommandCall, RuntimeBlock, Step, Stmt, WorkflowFile};

        let wf = WorkflowFile {
            runtime: Some(RuntimeBlock {
                core: "schema.nodus".to_string(),
                ..Default::default()
            }),
            steps: vec![Step {
                number: 1,
                body: Some(Stmt::Command(CommandCall {
                    name: "GEN".to_string(),
                    args: vec!["prompt".to_string()],
                    pipeline_target: Some("$result".to_string()),
                    ..Default::default()
                })),
                ..Default::default()
            }],
            ..Default::default()
        };
        let diags = Validator::validate(&wf, "");
        assert!(
            !diags.iter().any(|d| d.code == "E013"),
            "unexpected E013 for user-defined pipeline target"
        );
    }

    #[test]
    fn no_e013_for_writable_reserved_vars() {
        use crate::ast::{CommandCall, RuntimeBlock, Step, Stmt, WorkflowFile};

        for target in ["$out", "$draft", "$raw", "$quality"] {
            let wf = WorkflowFile {
                runtime: Some(RuntimeBlock {
                    core: "schema.nodus".to_string(),
                    ..Default::default()
                }),
                steps: vec![Step {
                    number: 1,
                    body: Some(Stmt::Command(CommandCall {
                        name: "GEN".to_string(),
                        args: vec!["prompt".to_string()],
                        pipeline_target: Some(target.to_string()),
                        ..Default::default()
                    })),
                    ..Default::default()
                }],
                ..Default::default()
            };
            let diags = Validator::validate(&wf, "");
            assert!(
                !diags.iter().any(|d| d.code == "E013"),
                "unexpected E013 for writable reserved variable {target}"
            );
        }
    }

    #[test]
    fn e014_fires_when_variable_used_before_assignment() {
        use crate::ast::{CommandCall, RuntimeBlock, Step, Stmt, WorkflowFile};

        // Step 1 uses $result; step 2 declares → $result — forward reference
        let wf = WorkflowFile {
            runtime: Some(RuntimeBlock {
                core: "schema.nodus".to_string(),
                ..Default::default()
            }),
            steps: vec![
                Step {
                    number: 1,
                    body: Some(Stmt::Command(CommandCall {
                        name: "LOG".to_string(),
                        args: vec!["$result".to_string()],
                        ..Default::default()
                    })),
                    ..Default::default()
                },
                Step {
                    number: 2,
                    body: Some(Stmt::Command(CommandCall {
                        name: "GEN".to_string(),
                        args: vec!["prompt".to_string()],
                        pipeline_target: Some("$result".to_string()),
                        ..Default::default()
                    })),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let diags = Validator::validate(&wf, "");
        assert!(
            diags.iter().any(|d| d.code == "E014"),
            "expected E014 for forward reference; got: {:?}",
            diags
                .iter()
                .map(|d| (&d.code, &d.message))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn no_e014_when_variable_assigned_in_prior_step() {
        use crate::ast::{CommandCall, RuntimeBlock, Step, Stmt, WorkflowFile};

        // Step 1 declares → $result; step 2 uses $result — correct order
        let wf = WorkflowFile {
            runtime: Some(RuntimeBlock {
                core: "schema.nodus".to_string(),
                ..Default::default()
            }),
            steps: vec![
                Step {
                    number: 1,
                    body: Some(Stmt::Command(CommandCall {
                        name: "GEN".to_string(),
                        args: vec!["prompt".to_string()],
                        pipeline_target: Some("$result".to_string()),
                        ..Default::default()
                    })),
                    ..Default::default()
                },
                Step {
                    number: 2,
                    body: Some(Stmt::Command(CommandCall {
                        name: "LOG".to_string(),
                        args: vec!["$result".to_string()],
                        ..Default::default()
                    })),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let diags = Validator::validate(&wf, "");
        assert!(
            !diags.iter().any(|d| d.code == "E014"),
            "unexpected E014 for correctly-ordered variable usage"
        );
    }

    #[test]
    fn all_severities_order_correctly() {
        assert!(Severity::Error < Severity::Warning);
        assert!(Severity::Warning < Severity::Info);
    }

    #[test]
    fn e015_fires_on_duplicate_test_names() {
        let src = "\
§wf:e015_wf v1.0
§runtime: { core: schema.nodus }
@in: { query }
@out: $out
@err: ESCALATE(human)
@steps:
  1. GEN($in.query) → $out
@test: smoke {
  input:
    query: hello
}
@test: smoke {
  input:
    query: world
}
";
        let ast = Parser::parse(src).expect("parse");
        let diags = Validator::validate(&ast, "e015_wf.nodus");
        assert!(
            diags.iter().any(|d| d.code == "E015"),
            "expected E015 for duplicate test name; got: {:?}",
            diags.iter().map(|d| &d.code).collect::<Vec<_>>()
        );
    }

    #[test]
    fn e015_absent_with_unique_test_names() {
        let src = "\
§wf:e015_ok v1.0
§runtime: { core: schema.nodus }
@in: { query }
@out: $out
@err: ESCALATE(human)
@steps:
  1. GEN($in.query) → $out
@test: alpha {
  input:
    query: hello
}
@test: beta {
  input:
    query: world
}
";
        let ast = Parser::parse(src).expect("parse");
        let diags = Validator::validate(&ast, "e015_ok.nodus");
        assert!(
            !diags.iter().any(|d| d.code == "E015"),
            "unexpected E015 for unique test names"
        );
    }

    #[test]
    fn w009_fires_when_test_block_has_no_expected() {
        let src = "\
§wf:w009_wf v1.0
§runtime: { core: schema.nodus }
@in: { query }
@out: $out
@err: ESCALATE(human)
@steps:
  1. GEN($in.query) → $out
@test: smoke {
  input:
    query: hello
}
";
        let ast = Parser::parse(src).expect("parse");
        let diags = Validator::validate(&ast, "w009_wf.nodus");
        assert!(
            diags.iter().any(|d| d.code == "W009"),
            "expected W009 for test block with no expected:; got: {:?}",
            diags.iter().map(|d| &d.code).collect::<Vec<_>>()
        );
    }

    #[test]
    fn w009_absent_when_expected_section_present() {
        let src = "\
§wf:w009_ok v1.0
§runtime: { core: schema.nodus }
@in: { query }
@out: $out
@err: ESCALATE(human)
@steps:
  1. GEN($in.query) → $out
@test: smoke {
  input:
    query: hello
  expected:
    $out: \"[STUB gen(hello) tone=brand]\"
}
";
        let ast = Parser::parse(src).expect("parse");
        let diags = Validator::validate(&ast, "w009_ok.nodus");
        assert!(
            !diags.iter().any(|d| d.code == "W009"),
            "unexpected W009 when expected: section is present"
        );
    }

    #[test]
    fn w015_fires_on_non_colon_pair_separator() {
        let src = "\
§wf:w015_wf v1.0
§runtime: { core: schema.nodus }
@in: { query }
@out: $out
@err: ESCALATE(human)
@steps:
  1. GEN($in.query) → $out
@test: smoke {
  expected: { status = SUCCESS }
}
";
        let ast = Parser::parse(src).expect("parse");
        let diags = Validator::validate(&ast, "w015_wf.nodus");
        assert!(
            diags.iter().any(|d| d.code == "W015"),
            "expected W015 for a non-':' pair separator; got: {:?}",
            diags.iter().map(|d| &d.code).collect::<Vec<_>>()
        );
    }

    #[test]
    fn w015_absent_on_conforming_colon_pairs() {
        let src = "\
§wf:w015_ok v1.0
§runtime: { core: schema.nodus }
@in: { query }
@out: $out
@err: ESCALATE(human)
@steps:
  1. GEN($in.query) → $out
@test: smoke {
  input: { query: hello }
  expected: { $out: hello }
}
";
        let ast = Parser::parse(src).expect("parse");
        let diags = Validator::validate(&ast, "w015_ok.nodus");
        assert!(
            !diags.iter().any(|d| d.code == "W015"),
            "unexpected W015 when all pairs use ':'; got: {:?}",
            diags.iter().map(|d| &d.code).collect::<Vec<_>>()
        );
    }

    #[test]
    fn w015_and_w009_co_fire_when_all_expected_pairs_are_dropped() {
        // §7: a block whose only `expected:` pairs are all non-conforming has
        // an `expected:` section in source but an empty one in the AST, so it
        // must emit BOTH W015 (the pairs were dropped) and W009 (nothing is
        // asserted) — the intended pairing, not a duplicate report.
        let src = "\
§wf:w015_w009_wf v1.0
§runtime: { core: schema.nodus }
@in: { query }
@out: $out
@err: ESCALATE(human)
@steps:
  1. GEN($in.query) → $out
@test: smoke {
  input: { query: hello }
  expected: { status = SUCCESS }
}
";
        let ast = Parser::parse(src).expect("parse");
        let diags = Validator::validate(&ast, "w015_w009_wf.nodus");
        let codes: Vec<_> = diags.iter().map(|d| d.code.as_str()).collect();
        assert!(codes.contains(&"W015"), "expected W015; got: {codes:?}");
        assert!(
            codes.contains(&"W009"),
            "expected W009 to co-fire since expected: is empty in the AST; got: {codes:?}"
        );
    }

    #[test]
    fn block_class_errors_identified() {
        let src = "\
§wf:block_test v1.0
@steps:
  1. PUBLISH($out)
";
        let ast = Parser::parse(src).expect("parse");
        let diags = Validator::validate(&ast, "block_test.nodus");
        let errors: Vec<_> = diags
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .collect();
        assert!(!errors.is_empty(), "block-class errors should be present");
        assert!(errors.iter().any(|d| d.code == "E001")); // runtime absent
        assert!(errors.iter().any(|d| d.code == "E005")); // publish without validate
    }

    #[test]
    fn registry_warnings_fire_for_unknown_vocabulary() {
        use crate::ast::{
            CommandCall, InputDecl, InputField, RuntimeBlock, Step, Stmt, WorkflowFile,
        };

        let wf = WorkflowFile {
            runtime: Some(RuntimeBlock {
                core: "schema.nodus".to_string(),
                ..Default::default()
            }),
            input_decl: Some(InputDecl {
                fields: vec![InputField {
                    name: "x".to_string(),
                    type_name: "widget".to_string(), // unknown type → W013
                    ..Default::default()
                }],
            }),
            steps: vec![Step {
                number: 1,
                body: Some(Stmt::Command(CommandCall {
                    name: "ANALYZE".to_string(),
                    args: vec!["$in.x".to_string()],
                    flags: vec!["bogusflag".to_string()], // unknown flag → W011
                    validators: vec!["nope".to_string()], // unknown validator → W012
                    pipeline_target: Some("$out".to_string()),
                    ..Default::default()
                })),
                ..Default::default()
            }],
            ..Default::default()
        };
        let diags = Validator::validate(&wf, "");
        assert!(
            diags.iter().any(|d| d.code == "W011"),
            "expected W011 unknown flag"
        );
        assert!(
            diags.iter().any(|d| d.code == "W012"),
            "expected W012 unknown validator"
        );
        assert!(
            diags.iter().any(|d| d.code == "W013"),
            "expected W013 unknown type"
        );
        // Advisory: registry findings are warnings, never errors.
        assert!(!diags.iter().any(|d| d.severity == Severity::Error
            && ["W011", "W012", "W013"].contains(&d.code.as_str())));
    }

    #[test]
    fn no_registry_warnings_for_known_vocabulary() {
        use crate::ast::{
            CommandCall, InputDecl, InputField, RuntimeBlock, Step, Stmt, WorkflowFile,
        };

        let wf = WorkflowFile {
            runtime: Some(RuntimeBlock {
                core: "schema.nodus".to_string(),
                ..Default::default()
            }),
            input_decl: Some(InputDecl {
                fields: vec![InputField {
                    name: "x".to_string(),
                    type_name: "str".to_string(),
                    ..Default::default()
                }],
            }),
            steps: vec![Step {
                number: 1,
                body: Some(Stmt::Command(CommandCall {
                    name: "ANALYZE".to_string(),
                    args: vec!["$in.x".to_string()],
                    flags: vec!["sentiment".to_string()],
                    validators: vec!["len:32".to_string()],
                    pipeline_target: Some("$out".to_string()),
                    ..Default::default()
                })),
                ..Default::default()
            }],
            ..Default::default()
        };
        let diags = Validator::validate(&wf, "");
        assert!(
            !diags
                .iter()
                .any(|d| ["W011", "W012", "W013"].contains(&d.code.as_str())),
            "known vocabulary must not warn; got: {:?}",
            diags.iter().map(|d| &d.code).collect::<Vec<_>>()
        );
    }

    // ── §config: shape check (NL-20) ────────────────────────────────────────

    fn sample_config_decl() -> ConfigDecl {
        use crate::ast::ConfigField;
        ConfigDecl {
            header: None,
            fields: vec![
                ConfigField {
                    name: "max_retries".to_string(),
                    type_name: "int".to_string(),
                    default: Some("3".to_string()),
                    constraint: Some(FieldConstraint::Range {
                        lo: "1".to_string(),
                        hi: "10".to_string(),
                    }),
                    ..Default::default()
                },
                ConfigField {
                    name: "api_key".to_string(),
                    type_name: "str".to_string(),
                    required: true,
                    secret: true,
                    ..Default::default()
                },
                ConfigField {
                    name: "level".to_string(),
                    type_name: "str".to_string(),
                    default: Some("medium".to_string()),
                    constraint: Some(FieldConstraint::OneOf(vec![
                        "low".to_string(),
                        "medium".to_string(),
                        "high".to_string(),
                    ])),
                    ..Default::default()
                },
            ],
        }
    }

    #[test]
    fn check_config_values_happy_path_uses_defaults_and_proposed() {
        let decl = sample_config_decl();
        let proposed = vec![("api_key".to_string(), Value::Text("sk-live".to_string()))];
        let accepted = check_config_values(&decl, &proposed).expect("shape check should pass");
        assert_eq!(accepted.get("max_retries"), Some(&Value::Int(3)));
        assert_eq!(
            accepted.get("level"),
            Some(&Value::Text("medium".to_string()))
        );
        assert_eq!(
            accepted.get("api_key"),
            Some(&Value::Text("sk-live".to_string()))
        );
        assert!(accepted.is_secret("api_key"));
        assert!(!accepted.is_secret("max_retries"));
    }

    #[test]
    fn check_config_values_unknown_field() {
        let decl = sample_config_decl();
        let proposed = vec![
            ("api_key".to_string(), Value::Text("k".to_string())),
            ("bogus".to_string(), Value::Int(1)),
        ];
        let violations = check_config_values(&decl, &proposed).expect_err("must reject");
        assert!(violations.contains(&ConfigViolation {
            field: "bogus".to_string(),
            reason: ConfigReason::UnknownField,
        }));
    }

    #[test]
    fn check_config_values_missing_required() {
        let decl = sample_config_decl();
        let violations = check_config_values(&decl, &[]).expect_err("must reject");
        assert!(violations.contains(&ConfigViolation {
            field: "api_key".to_string(),
            reason: ConfigReason::MissingRequired,
        }));
    }

    #[test]
    fn check_config_values_type_mismatch() {
        let decl = sample_config_decl();
        let proposed = vec![
            ("api_key".to_string(), Value::Text("k".to_string())),
            (
                "max_retries".to_string(),
                Value::Text("not_a_number".to_string()),
            ),
        ];
        let violations = check_config_values(&decl, &proposed).expect_err("must reject");
        assert!(violations.contains(&ConfigViolation {
            field: "max_retries".to_string(),
            reason: ConfigReason::TypeMismatch,
        }));
    }

    #[test]
    fn check_config_values_out_of_range() {
        let decl = sample_config_decl();
        let proposed = vec![
            ("api_key".to_string(), Value::Text("k".to_string())),
            ("max_retries".to_string(), Value::Int(99)),
        ];
        let violations = check_config_values(&decl, &proposed).expect_err("must reject");
        assert!(violations.contains(&ConfigViolation {
            field: "max_retries".to_string(),
            reason: ConfigReason::OutOfRange,
        }));
    }

    #[test]
    fn check_config_values_not_in_enum() {
        let decl = sample_config_decl();
        let proposed = vec![
            ("api_key".to_string(), Value::Text("k".to_string())),
            ("level".to_string(), Value::Text("extreme".to_string())),
        ];
        let violations = check_config_values(&decl, &proposed).expect_err("must reject");
        assert!(violations.contains(&ConfigViolation {
            field: "level".to_string(),
            reason: ConfigReason::NotInEnum,
        }));
    }

    #[test]
    fn check_config_values_bad_default_is_reported() {
        use crate::ast::ConfigField;
        let decl = ConfigDecl {
            header: None,
            fields: vec![ConfigField {
                name: "n".to_string(),
                type_name: "int".to_string(),
                default: Some("not_an_int".to_string()),
                ..Default::default()
            }],
        };
        let violations = check_config_values(&decl, &[]).expect_err("bad default must reject");
        assert!(violations.contains(&ConfigViolation {
            field: "n".to_string(),
            reason: ConfigReason::BadDefault,
        }));
    }

    #[test]
    fn check_config_values_reports_all_violations_applies_none() {
        let decl = sample_config_decl();
        // Two independent violations: missing required api_key + bad type on max_retries.
        let proposed = vec![("max_retries".to_string(), Value::Text("nope".to_string()))];
        let violations = check_config_values(&decl, &proposed).expect_err("must reject");
        assert_eq!(
            violations.len(),
            2,
            "both violations must be reported: {violations:?}"
        );
    }

    #[test]
    fn check_config_values_is_pure() {
        let decl = sample_config_decl();
        let proposed = vec![("api_key".to_string(), Value::Text("k".to_string()))];
        let a = check_config_values(&decl, &proposed).expect("pass");
        let b = check_config_values(&decl, &proposed).expect("pass");
        assert_eq!(a.get("max_retries"), b.get("max_retries"));
        assert_eq!(a.get("level"), b.get("level"));
    }

    #[test]
    fn accepted_config_non_secret_fields_excludes_secret() {
        let decl = sample_config_decl();
        let proposed = vec![("api_key".to_string(), Value::Text("k".to_string()))];
        let accepted = check_config_values(&decl, &proposed).expect("pass");
        let names: Vec<&str> = accepted.non_secret_fields().map(|(n, _)| n).collect();
        assert!(names.contains(&"max_retries"));
        assert!(names.contains(&"level"));
        assert!(
            !names.contains(&"api_key"),
            "secret field must not appear in non_secret_fields: {names:?}"
        );
    }

    #[test]
    fn e004_does_not_fire_on_map_implicit_it() {
        let src = "\
§wf:mapper v1.0
§runtime: { core: schema.nodus }
@out: $out
@err: ESCALATE(human)
@steps:
  1. ~MAP $in.items: GEN($it) → $out
";
        let ast = Parser::parse(src).expect("parse");
        let diags = Validator::validate(&ast, "mapper.nodus");
        assert!(
            diags.iter().all(|d| d.severity != Severity::Error),
            "~MAP's implicit $it binding must not raise any error diagnostic (E004 in particular); got: {diags:?}"
        );
    }

    #[test]
    fn e004_still_fires_on_it_used_outside_map() {
        let src = "\
§wf:stray_it v1.0
§runtime: { core: schema.nodus }
@out: $out
@err: ESCALATE(human)
@steps:
  1. LOG($it) → $out
";
        let ast = Parser::parse(src).expect("parse");
        let diags = Validator::validate(&ast, "stray_it.nodus");
        assert!(
            diags
                .iter()
                .any(|d| d.code == "E004" && d.severity == Severity::Error),
            "$it used outside a ~MAP must still be flagged as undeclared (the fix must not blanket-declare it); got: {diags:?}"
        );
    }

    // The next two tests were planned as a *fix* (Phase 20 Track B) on the
    // assumption that `collect_vars_stmt`'s `Stmt::Switch` arm only tracked
    // the scrutinee, mirroring the ~MAP/$it gap Phase 17 closed. Plan-time
    // grounding for that assumption used a `sed` line range that (unnoticed)
    // cut off mid-match-arm, before the arm-walking loop that was already
    // there. No fix was needed — these are regression tests confirming the
    // existing `collect_vars_cmd` calls over `sw.arms`/`sw.default` hold,
    // now that Phase 20 made `?SWITCH` arm targets reachable via the parser.

    #[test]
    fn e004_does_not_fire_on_switch_arm_bound_target_used_later() {
        let src = "\
§wf:dispatch v1.0
§runtime: { core: schema.nodus }
@in: { category?=urgent }
@out: $out
@err: ESCALATE(human)
@steps:
  1. ?SWITCH $in.category:
    urgent → GEN(x) → $picked
  ~END
  2. LOG($picked) → $out
";
        let ast = Parser::parse(src).expect("parse");
        let diags = Validator::validate(&ast, "dispatch.nodus");
        assert!(
            diags.iter().all(|d| d.code != "E004"),
            "a ?SWITCH arm's → $picked target must be declared, so a later step reading it must not raise E004; got: {diags:?}"
        );
    }

    #[test]
    fn e004_fires_on_switch_arm_actions_own_undeclared_variable() {
        let src = "\
§wf:dispatch v1.0
§runtime: { core: schema.nodus }
@out: $out
@err: ESCALATE(human)
@steps:
  1. ?SWITCH $category:
    urgent → GEN($stray) → $out
  ~END
";
        let ast = Parser::parse(src).expect("parse");
        let diags = Validator::validate(&ast, "dispatch.nodus");
        assert!(
            diags
                .iter()
                .any(|d| d.code == "E004" && d.severity == Severity::Error),
            "$stray is referenced only inside a ?SWITCH arm action and never assigned — must still be flagged (proves arm actions are walked for uses too); got: {diags:?}"
        );
    }

    #[test]
    fn e018_fires_on_unbounded_or_oversized_restart_max() {
        use crate::ast::RuntimeBlock;

        for n in [0u32, 11] {
            let wf = WorkflowFile {
                runtime: Some(RuntimeBlock {
                    core: "schema.nodus".to_string(),
                    restart_max: Some(n),
                    ..Default::default()
                }),
                ..Default::default()
            };
            let diags = Validator::validate(&wf, "");
            assert!(
                diags
                    .iter()
                    .any(|d| d.code == "E018" && d.severity == Severity::Error),
                "restart_max: {n} must raise E018; got: {diags:?}"
            );
        }
    }

    #[test]
    fn no_e018_for_valid_restart_max() {
        use crate::ast::RuntimeBlock;

        for n in [1u32, 10] {
            let wf = WorkflowFile {
                runtime: Some(RuntimeBlock {
                    core: "schema.nodus".to_string(),
                    restart_max: Some(n),
                    ..Default::default()
                }),
                ..Default::default()
            };
            let diags = Validator::validate(&wf, "");
            assert!(
                !diags.iter().any(|d| d.code == "E018"),
                "restart_max: {n} must not raise E018; got: {diags:?}"
            );
        }
    }

    #[test]
    fn e013_fires_when_pipeline_target_is_restart_count() {
        use crate::ast::{CommandCall, RuntimeBlock, Step, Stmt};

        let wf = WorkflowFile {
            runtime: Some(RuntimeBlock {
                core: "schema.nodus".to_string(),
                ..Default::default()
            }),
            steps: vec![Step {
                number: 1,
                body: Some(Stmt::Command(CommandCall {
                    name: "GEN".to_string(),
                    args: vec!["prompt".to_string()],
                    pipeline_target: Some("$restart_count".to_string()),
                    ..Default::default()
                })),
                ..Default::default()
            }],
            ..Default::default()
        };
        let diags = Validator::validate(&wf, "");
        assert!(
            diags
                .iter()
                .any(|d| d.code == "E013" && d.severity == Severity::Error),
            "→ $restart_count must be rejected — it is runtime-owned and unforgeable (NL-8/NL-23); got: {diags:?}"
        );
    }

    #[test]
    fn no_e013_for_restart_request_target() {
        use crate::ast::{CommandCall, RuntimeBlock, Step, Stmt};

        let wf = WorkflowFile {
            runtime: Some(RuntimeBlock {
                core: "schema.nodus".to_string(),
                restart_max: Some(3),
                ..Default::default()
            }),
            steps: vec![Step {
                number: 1,
                body: Some(Stmt::Command(CommandCall {
                    name: "GEN".to_string(),
                    args: vec!["prompt".to_string()],
                    pipeline_target: Some("$restart".to_string()),
                    ..Default::default()
                })),
                ..Default::default()
            }],
            ..Default::default()
        };
        let diags = Validator::validate(&wf, "");
        assert!(
            !diags.iter().any(|d| d.code == "E013"),
            "→ $restart must stay writable — a workflow must be able to request a restart; got: {diags:?}"
        );
    }

    #[test]
    fn e019_fires_on_restart_request_nested_in_for_loop() {
        use crate::ast::{CommandCall, ForLoop, RuntimeBlock, Step, Stmt};

        let wf = WorkflowFile {
            runtime: Some(RuntimeBlock {
                core: "schema.nodus".to_string(),
                restart_max: Some(3),
                ..Default::default()
            }),
            steps: vec![Step {
                number: 1,
                body: Some(Stmt::ForLoop(ForLoop {
                    variable: "$item".to_string(),
                    collection: "$in.items".to_string(),
                    body: vec![Stmt::Command(CommandCall {
                        name: "GEN".to_string(),
                        args: vec![],
                        pipeline_target: Some("$restart".to_string()),
                        ..Default::default()
                    })],
                })),
                ..Default::default()
            }],
            ..Default::default()
        };
        let diags = Validator::validate(&wf, "");
        assert!(
            diags
                .iter()
                .any(|d| d.code == "E019" && d.severity == Severity::Error),
            "a $restart request inside a ~FOR body must be rejected (NL-23(b)); got: {diags:?}"
        );
    }

    #[test]
    fn e019_fires_on_restart_request_nested_in_switch_arm() {
        use crate::ast::{CommandCall, RuntimeBlock, Step, Stmt, SwitchBlock};

        let wf = WorkflowFile {
            runtime: Some(RuntimeBlock {
                core: "schema.nodus".to_string(),
                restart_max: Some(3),
                ..Default::default()
            }),
            steps: vec![Step {
                number: 1,
                body: Some(Stmt::Switch(SwitchBlock {
                    scrutinee: "$in.x".to_string(),
                    arms: vec![(
                        "urgent".to_string(),
                        CommandCall {
                            name: "GEN".to_string(),
                            args: vec![],
                            pipeline_target: Some("$restart".to_string()),
                            ..Default::default()
                        },
                    )],
                    default: None,
                })),
                ..Default::default()
            }],
            ..Default::default()
        };
        let diags = Validator::validate(&wf, "");
        assert!(
            diags
                .iter()
                .any(|d| d.code == "E019" && d.severity == Severity::Error),
            "a $restart request inside a ?SWITCH arm must be rejected (NL-23(b)); got: {diags:?}"
        );
    }

    #[test]
    fn no_e019_for_top_level_restart_request() {
        use crate::ast::{CommandCall, RuntimeBlock, Step, Stmt};

        let wf = WorkflowFile {
            runtime: Some(RuntimeBlock {
                core: "schema.nodus".to_string(),
                restart_max: Some(3),
                ..Default::default()
            }),
            steps: vec![Step {
                number: 1,
                body: Some(Stmt::Command(CommandCall {
                    name: "GEN".to_string(),
                    args: vec![],
                    pipeline_target: Some("$restart".to_string()),
                    ..Default::default()
                })),
                ..Default::default()
            }],
            ..Default::default()
        };
        let diags = Validator::validate(&wf, "");
        assert!(
            !diags.iter().any(|d| d.code == "E019"),
            "a top-level $restart request must not be rejected; got: {diags:?}"
        );
    }
}
