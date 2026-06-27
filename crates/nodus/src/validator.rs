//! Validator — structural lint and schema-vocabulary checks.
//!
//! Runs 30 rules against a parsed [`WorkflowFile`] AST and returns a flat list
//! of [`Diagnostic`]s. Rules are grouped by severity:
//!
//! - **Error** (E001–E015): block execution when found.
//! - **Warning** (W001–W013): workflow runs but has unsafe or incomplete patterns.
//! - **Info** (I001–I006): style suggestions.
//!
//! AST nodes carry no source positions in this iteration; diagnostics therefore
//! report `line = 0, column = 0`. When the parser is extended to attach spans,
//! the diagnostic helpers below will naturally carry real positions.

use crate::ast::{Conditional, ForLoop, ParallelBlock, Stmt, UntilLoop, WorkflowFile};
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
    /// Run all 30 lint rules against `ast` and return accumulated diagnostics.
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
        d.extend(Self::w011_known_vocabulary(ast, filename));

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

    // ─── Warnings ─────────────────────────────────────────────────────────────

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
        Stmt::VarRef(_) | Stmt::Comment(_) => {}
    }
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
}
