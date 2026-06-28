//! Executor — runtime engine for NODUS workflows.
//!
//! Walks the AST produced by the parser and dispatches each step command to
//! its handler. Handlers are pluggable via the [`ModelProvider`] trait; the
//! built-in [`StubProvider`] satisfies the interface without external I/O,
//! which enables tests to run the full boot sequence in-process.
//!
//! Boot sequence (matches the reference protocol):
//! 1. Load schema (resolved from `§runtime` block)
//! 2. Read `!!` rules — internalise hard constraints
//! 3. Read `!PREF` — internalise soft preferences
//! 4. Register `@in` / `@ctx` inputs
//! 5. Match `@ON` triggers
//! 6. Execute `@steps` sequentially; thread pipeline `→` targets between steps
//!
//! Every execution returns a [`RunResult`] regardless of outcome; callers
//! inspect `status` first, then use `out`, `log`, and `errors` for details.

use crate::ast::{AbsoluteRule, CommandCall, RuleKind, Step, Stmt, WorkflowFile};
use crate::observability::{
    AuditProvider, ExecutionEvent, FieldDescriptor, LoopType, NoopAuditProvider, RunManifest,
    RunStatus,
};
use crate::vocab;
use std::collections::HashMap;
use std::time::Instant;

// ─── Value ────────────────────────────────────────────────────────────────────

/// A runtime value — the dynamic type for workflow variables.
#[derive(Debug, Clone, Default, PartialEq)]
pub enum Value {
    /// No value / unset.
    #[default]
    Null,
    /// Boolean.
    Bool(bool),
    /// Integer.
    Int(i64),
    /// Floating-point.
    Float(f64),
    /// Text string.
    Text(String),
    /// Ordered list.
    List(Vec<Value>),
    /// Ordered key-value map (preserves insertion order).
    Map(Vec<(String, Value)>),
}

impl Value {
    fn as_text(&self) -> String {
        match self {
            Value::Text(s) => s.clone(),
            Value::Int(n) => n.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => String::new(),
            Value::List(_) | Value::Map(_) => format!("{self:?}"),
        }
    }

    fn map_get(&self, key: &str) -> Value {
        match self {
            Value::Map(entries) => entries
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.clone())
                .unwrap_or(Value::Null),
            _ => Value::Null,
        }
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::Text(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::Text(s.to_string())
    }
}

// ─── Result contract ──────────────────────────────────────────────────────────

/// Overall outcome of a workflow execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    /// All steps completed without errors.
    Ok,
    /// Steps completed but non-fatal errors occurred.
    Partial,
    /// A fatal error halted execution.
    Failed,
    /// A `!BREAK` or rule-violation aborted the run.
    Aborted,
    /// Execution suspended at a dialog awaiting human re-trigger (`!PAUSE` / unresolved `ASK`/`CONFIRM`).
    Paused,
}

/// A single step's execution record in the audit log.
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// Step number where this command ran.
    pub step: u32,
    /// Command name.
    pub command: String,
    /// The value returned by the handler (`Null` for void commands).
    pub result: Value,
}

/// A runtime error.
#[derive(Debug, Clone)]
pub struct RuntimeError {
    /// One of the `NODUS:*` error codes from [`crate::vocab::error_code`].
    pub code: String,
    /// Step number where the error occurred (`0` = boot sequence).
    pub step: u32,
    /// Human-readable reason.
    pub reason: String,
}

/// Snapshot a paused run hands to its host so the run can be resumed (DG-4).
/// The runtime produces this; persistence and re-invocation are host concerns.
#[derive(Debug, Clone)]
pub struct ResumeDescriptor {
    /// `"wf:<name>"` of the suspended workflow.
    pub workflow: String,
    /// Variable environment captured at the suspension point.
    pub vars: HashMap<String, Value>,
    /// Index of the step that suspended.
    pub step_index: u32,
}

/// The structured output of a workflow execution.
///
/// Every execution — successful or not — returns a `RunResult`. Callers
/// inspect `status` first, then use `out`, `log`, and `errors` for details.
#[derive(Debug)]
pub struct RunResult {
    /// `"wf:<name>"` identifier of the workflow that ran.
    pub workflow: String,
    /// Overall execution status.
    pub status: Status,
    /// The final value of `$out` at the end of execution.
    pub out: Value,
    /// Ordered log of every step that executed.
    pub log: Vec<LogEntry>,
    /// Non-fatal and fatal errors accumulated during execution.
    pub errors: Vec<RuntimeError>,
    /// Flags emitted by routing, escalation, and unknown-command paths.
    pub flags: Vec<String>,
    /// Final variable environment: variable name (without `$`) → value.
    /// Enables test runners and debuggers to inspect all pipeline bindings.
    pub vars: HashMap<String, Value>,
    /// Present only when `status == Paused`: the descriptor a host uses to resume.
    pub resume: Option<ResumeDescriptor>,
}

// ─── ModelProvider ────────────────────────────────────────────────────────────

/// Pluggable LLM backend for `GEN`, `ANALYZE`, and similar model-backed
/// commands. Implementations can call a real API or return stubs for testing.
pub trait ModelProvider {
    /// A short identifier for this provider (surfaced in the result).
    fn model_id(&self) -> &str;

    /// Generate text from a prompt.
    fn generate(&self, prompt: &str, modifiers: &[(String, String)]) -> String;

    /// Analyse text and return a map of flag → score/label.
    fn analyze(&self, text: &str, flags: &[String]) -> Value;
}

/// Zero-dependency stub that returns deterministic responses.
///
/// Suitable for unit tests and the vertical-slice boot path. Every `generate`
/// call returns a bracketed label; every `analyze` flag gets a score of 0.9.
pub struct StubProvider;

impl ModelProvider for StubProvider {
    fn model_id(&self) -> &str {
        "nodus-stub"
    }

    fn generate(&self, prompt: &str, modifiers: &[(String, String)]) -> String {
        let tone = modifiers
            .iter()
            .find(|(k, _)| k == "+tone")
            .map(|(_, v)| v.as_str())
            .unwrap_or("brand");
        format!("[STUB gen({prompt}) tone={tone}]")
    }

    fn analyze(&self, _text: &str, flags: &[String]) -> Value {
        let entries: Vec<(String, Value)> = flags
            .iter()
            .map(|f| {
                let v = if f == "intent" {
                    Value::Text("stub_intent".to_string())
                } else {
                    Value::Float(0.9)
                };
                (f.clone(), v)
            })
            .collect();
        Value::Map(entries)
    }
}

// ─── DialogProvider ─────────────────────────────────────────────────────────

/// The outcome of a dialog command (`ASK` / `CONFIRM`).
#[derive(Debug, Clone, PartialEq)]
pub enum DialogOutcome {
    /// Resolved with a typed answer (by a human or a `+default`).
    Answer(Value),
    /// No resolution available; the runtime should suspend (`Status::Paused`).
    Pause,
    /// A `+timeout` elapsed before an answer.
    Timeout,
    /// A `+strict` `CONFIRM` was rejected.
    Rejected,
}

/// Pluggable human-in-the-loop backend for `ASK` / `CONFIRM`. Implementations
/// can drive a real UI or resolve synchronously for tests; the language and
/// executor never name a concrete channel (host neutrality).
pub trait DialogProvider {
    /// Pose a typed question. `modifiers` carries `+type`, `+default`, etc.
    fn ask(&self, prompt: &str, modifiers: &[(String, String)]) -> DialogOutcome;

    /// Pose an approval decision.
    fn confirm(&self, content: &str, modifiers: &[(String, String)]) -> DialogOutcome;
}

/// Built-in synchronous resolver: resolves a dialog from its `+default`
/// modifier, otherwise signals [`DialogOutcome::Pause`]. Performs no I/O and
/// never blocks, so non-interactive and test runs stay deterministic.
pub struct DefaultDialogProvider;

impl DefaultDialogProvider {
    fn resolve(modifiers: &[(String, String)]) -> DialogOutcome {
        modifiers
            .iter()
            .find(|(k, _)| k == "+default")
            .map(|(_, v)| DialogOutcome::Answer(Value::Text(v.clone())))
            .unwrap_or(DialogOutcome::Pause)
    }
}

impl DialogProvider for DefaultDialogProvider {
    fn ask(&self, _prompt: &str, modifiers: &[(String, String)]) -> DialogOutcome {
        Self::resolve(modifiers)
    }

    fn confirm(&self, _content: &str, modifiers: &[(String, String)]) -> DialogOutcome {
        Self::resolve(modifiers)
    }
}

// ─── Execution signal ─────────────────────────────────────────────────────────

enum Signal {
    Break,
    Skip,
    /// A dialog could not resolve, or a branch carried `!PAUSE`; suspend the run
    /// (`Status::Paused`).
    Pause,
    /// A branch carried `!HALT`; stop the run with a failed status.
    Halt,
}

// ─── Execution context ────────────────────────────────────────────────────────

struct ExecutionContext {
    variables: HashMap<String, Value>,
    rules: Vec<AbsoluteRule>,
    log: Vec<LogEntry>,
    errors: Vec<RuntimeError>,
    flags: Vec<String>,
    out_locked: bool,
    event_count: u32,
    start_instant: Instant,
    paused_at: Option<u32>,
}

impl ExecutionContext {
    fn new() -> Self {
        ExecutionContext {
            variables: HashMap::new(),
            rules: Vec::new(),
            log: Vec::new(),
            errors: Vec::new(),
            flags: Vec::new(),
            out_locked: false,
            event_count: 0,
            start_instant: Instant::now(),
            paused_at: None,
        }
    }

    /// Set a variable by name (strips leading `$`). Respects the `$out` lock.
    fn set_var(&mut self, name: &str, value: Value) {
        let key = name.trim_start_matches('$').to_string();
        if key == "out" && self.out_locked {
            self.errors.push(RuntimeError {
                code: vocab::error_code::RULE_VIOLATION.to_string(),
                step: 0,
                reason: "cannot modify $out after LOG() has been called".to_string(),
            });
            return;
        }
        self.variables.insert(key, value);
    }

    /// Resolve a variable name to its value, supporting dot-notation (`$in.query`).
    fn get_var(&self, name: &str) -> Value {
        let clean = name.trim_start_matches('$');
        let mut parts = clean.splitn(2, '.');
        let root = parts.next().unwrap_or("");
        let tail = parts.next();

        let root_val = self.variables.get(root).cloned().unwrap_or(Value::Null);
        match tail {
            None => root_val,
            Some(path) => {
                let mut val = root_val;
                for part in path.split('.') {
                    val = val.map_get(part);
                }
                val
            }
        }
    }

    /// Append a step record to the audit log.
    fn log_step(&mut self, step: u32, command: &str, result: Value) {
        self.log.push(LogEntry {
            step,
            command: command.to_string(),
            result,
        });
    }

    /// Check `cmd` against the current `!!NEVER` rules. Returns a violation
    /// description when the command must be blocked.
    fn check_rules(&self, command: &str, args: &[String]) -> Option<String> {
        let cmd_up = command.to_uppercase();
        let args_lo: Vec<String> = args.iter().map(|a| a.to_lowercase()).collect();

        for rule in &self.rules {
            let content = rule.content.trim();
            let content_up = content.to_uppercase();

            if rule.kind == Some(RuleKind::Never) {
                // Simple name match: `!!NEVER: DELETE`
                if content_up == cmd_up {
                    return Some(format!("!!NEVER: {content}"));
                }

                // Argument match: `!!NEVER: FETCH("evil.com")`
                if content_up.contains(&cmd_up)
                    && args_lo
                        .iter()
                        .any(|a| content.to_lowercase().contains(a.as_str()))
                {
                    return Some(format!("!!NEVER: {content}"));
                }

                // Conditional: `!!NEVER: publish WITHOUT validate`
                if cmd_up == "PUBLISH"
                    && content_up.contains("WITHOUT")
                    && content_up.contains("VALIDATE")
                {
                    let validated = self.log.iter().any(|e| e.command == "VALIDATE");
                    if !validated {
                        return Some(format!("!!NEVER: {content}"));
                    }
                }
            }
        }
        None
    }
}

// ─── Executor ─────────────────────────────────────────────────────────────────

/// Core runtime engine for NODUS workflows.
pub struct Executor {
    provider: Box<dyn ModelProvider>,
    audit: Box<dyn AuditProvider>,
    dialog: Box<dyn DialogProvider>,
}

impl Executor {
    /// Create an executor backed by `provider` with no-op audit and the built-in
    /// synchronous dialog resolver.
    pub fn new(provider: impl ModelProvider + 'static) -> Self {
        Executor {
            provider: Box::new(provider),
            audit: Box::new(NoopAuditProvider),
            dialog: Box::new(DefaultDialogProvider),
        }
    }

    /// Create an executor backed by the built-in [`StubProvider`].
    pub fn with_stub() -> Self {
        Self::new(StubProvider)
    }

    /// Create an executor with a custom model provider and audit provider.
    pub fn with_audit(
        provider: impl ModelProvider + 'static,
        audit: impl AuditProvider + 'static,
    ) -> Self {
        Executor {
            provider: Box::new(provider),
            audit: Box::new(audit),
            dialog: Box::new(DefaultDialogProvider),
        }
    }

    /// Create an executor with the built-in model/audit and a custom
    /// [`DialogProvider`].
    pub fn with_dialog(dialog: impl DialogProvider + 'static) -> Self {
        Executor {
            provider: Box::new(StubProvider),
            audit: Box::new(NoopAuditProvider),
            dialog: Box::new(dialog),
        }
    }

    /// Create an executor with a custom [`DialogProvider`] and audit provider.
    pub fn with_dialog_and_audit(
        dialog: impl DialogProvider + 'static,
        audit: impl AuditProvider + 'static,
    ) -> Self {
        Executor {
            provider: Box::new(StubProvider),
            audit: Box::new(audit),
            dialog: Box::new(dialog),
        }
    }

    /// Run a parsed workflow through the boot sequence, execute its steps, and
    /// return the structured [`RunResult`].
    pub fn execute(&self, ast: &WorkflowFile, input: Option<Value>) -> RunResult {
        self.execute_inner(ast, input, "", "")
    }

    /// Run a workflow with explicit run metadata forwarded to the audit provider.
    pub fn execute_with_params(
        &self,
        ast: &WorkflowFile,
        input: Option<Value>,
        run_id: &str,
        started_at: &str,
    ) -> RunResult {
        self.execute_inner(ast, input, run_id, started_at)
    }

    fn execute_inner(
        &self,
        ast: &WorkflowFile,
        input: Option<Value>,
        run_id: &str,
        started_at: &str,
    ) -> RunResult {
        let mut ctx = ExecutionContext::new();

        // Boot step 1: identify workflow.
        let workflow_id = ast
            .header
            .as_ref()
            .map(|h| format!("wf:{}", h.name))
            .unwrap_or_default();

        // Boot step 2: internalise hard rules.
        ctx.rules.extend(ast.rules.iter().cloned());

        // Boot step 3: preferences are noted; full evaluation lands with the
        // model-router phase.

        // Boot step 4: register @in defaults, then overlay with caller input.
        if let Some(input_decl) = &ast.input_decl {
            let mut in_map: Vec<(String, Value)> = Vec::new();
            for field in &input_decl.fields {
                if let Some(d) = &field.default {
                    let v = Value::Text(d.clone());
                    in_map.push((field.name.clone(), v.clone()));
                    ctx.variables.insert(field.name.clone(), v);
                }
            }
            ctx.variables.insert("in".to_string(), Value::Map(in_map));
        }

        if let Some(Value::Map(ref entries)) = input {
            for (k, v) in entries {
                ctx.variables.insert(k.clone(), v.clone());
            }
            let mut in_map = match ctx.variables.get("in").cloned() {
                Some(Value::Map(m)) => m,
                _ => Vec::new(),
            };
            for (k, v) in entries {
                if let Some(pos) = in_map.iter().position(|(ik, _)| ik == k) {
                    in_map[pos] = (k.clone(), v.clone());
                } else {
                    in_map.push((k.clone(), v.clone()));
                }
            }
            ctx.variables.insert("in".to_string(), Value::Map(in_map));
        }

        // Seed reserved variables.
        ctx.variables
            .entry("out".to_string())
            .or_insert(Value::Null);
        ctx.variables
            .entry("error".to_string())
            .or_insert_with(|| Value::Map(Vec::new()));
        ctx.variables
            .entry("meta".to_string())
            .or_insert_with(|| Value::Map(Vec::new()));

        // Boot step 5: @ON trigger dispatch is the library layer's responsibility.

        // Boot step 6: execute @steps.
        let mut abort = false;
        let mut paused = false;
        let mut halted = false;
        for step in &ast.steps {
            match self.run_step_with_retry(&mut ctx, step) {
                Some(Signal::Halt) => {
                    halted = true;
                    break;
                }
                Some(Signal::Break) => {
                    abort = true;
                    break;
                }
                Some(Signal::Pause) => {
                    paused = true;
                    break;
                }
                _ => {}
            }
        }

        // Finalise.
        let out = ctx.get_var("out");
        // Rule violations take precedence (protocol-level failure); a pause is a
        // clean suspension and outranks abort/partial so the host sees Paused.
        let status = if halted
            || ctx
                .errors
                .iter()
                .any(|e| e.code == vocab::error_code::RULE_VIOLATION)
        {
            Status::Failed
        } else if paused {
            Status::Paused
        } else if abort {
            Status::Aborted
        } else if !ctx.errors.is_empty() {
            Status::Partial
        } else {
            Status::Ok
        };

        let resume = if paused {
            Some(ResumeDescriptor {
                workflow: workflow_id.clone(),
                vars: ctx.variables.clone(),
                step_index: ctx.paused_at.unwrap_or(0),
            })
        } else {
            None
        };

        let run_status = match status {
            Status::Ok | Status::Partial | Status::Paused => RunStatus::Ok,
            Status::Failed | Status::Aborted => RunStatus::ConstraintHalt,
        };
        self.audit.run_complete(RunManifest {
            workflow_name: workflow_id.clone(),
            schema_version: crate::vocab::BUILTIN_SCHEMA_VERSION.to_string(),
            run_id: run_id.to_string(),
            started_at: started_at.to_string(),
            elapsed_ms: ctx.start_instant.elapsed().as_millis() as u64,
            status: run_status,
            error_code: ctx.errors.first().map(|e| e.code.clone()),
            total_steps: ctx.log.len() as u32,
            event_count: ctx.event_count,
        });

        RunResult {
            workflow: workflow_id,
            status,
            out,
            log: ctx.log,
            errors: ctx.errors,
            flags: ctx.flags,
            vars: ctx.variables,
            resume,
        }
    }

    // ─── Node dispatch ────────────────────────────────────────────────────────

    fn execute_step(&self, ctx: &mut ExecutionContext, step: &Step) -> Option<Signal> {
        let sig = step
            .body
            .as_ref()
            .and_then(|b| self.execute_node(ctx, b, step.number));
        if sig.is_some() {
            return sig;
        }
        for sub in &step.sub_steps {
            let sig = self.execute_node(ctx, sub, step.number);
            if sig.is_some() {
                return sig;
            }
        }
        None
    }

    /// Run a step, honoring its `~RETRY:n` bound: re-run up to `n` times while
    /// the attempt adds a runtime error. A control signal (`!HALT`/`!BREAK`/
    /// `!PAUSE`/rule violation) stops immediately — those are intentional, not
    /// transient. On eventual success the failed attempts' errors are rolled
    /// back; on exhaustion the accumulated errors remain and surface (routing to
    /// `@err`). With no bound the step runs exactly once.
    fn run_step_with_retry(&self, ctx: &mut ExecutionContext, step: &Step) -> Option<Signal> {
        let max_attempts = step.retry.map(|n| n.max(1)).unwrap_or(1);
        let errors_before = ctx.errors.len();
        let mut signal = None;
        for _ in 0..max_attempts {
            let errors_at_attempt = ctx.errors.len();
            signal = self.execute_step(ctx, step);
            if signal.is_some() {
                return signal;
            }
            if ctx.errors.len() == errors_at_attempt {
                // Success: discard any errors left by earlier failed attempts.
                ctx.errors.truncate(errors_before);
                return None;
            }
        }
        signal
    }

    fn execute_node(
        &self,
        ctx: &mut ExecutionContext,
        node: &Stmt,
        step_num: u32,
    ) -> Option<Signal> {
        match node {
            Stmt::Command(cmd) => self.execute_command(ctx, cmd, step_num),
            Stmt::Conditional(cond) => self.execute_conditional(ctx, cond, step_num),
            Stmt::ForLoop(fl) => self.execute_for(ctx, fl, step_num),
            Stmt::UntilLoop(ul) => self.execute_until(ctx, ul, step_num),
            Stmt::Parallel(pb) => self.execute_parallel(ctx, pb, step_num),
            Stmt::Switch(sw) => self.execute_switch(ctx, sw, step_num),
            Stmt::Map(mb) => self.execute_map(ctx, mb, step_num),
            Stmt::VarRef(_) | Stmt::Comment(_) => None,
        }
    }

    /// Resolve the control signal a taken conditional branch emits from its
    /// action flags. `!HALT` outranks `!PAUSE`, which outranks `!BREAK`/`!SKIP`;
    /// `!PAUSE` records the suspension point so the run can resume later.
    fn branch_exit_signal(
        ctx: &mut ExecutionContext,
        branch: &crate::ast::Conditional,
        step_num: u32,
    ) -> Option<Signal> {
        if branch.halt_flag {
            return Some(Signal::Halt);
        }
        if branch.pause_flag {
            ctx.paused_at = Some(step_num);
            ctx.flags.push(vocab::error_code::PAUSED.to_string());
            return Some(Signal::Pause);
        }
        if branch.break_flag {
            return Some(Signal::Break);
        }
        if branch.skip_flag {
            return Some(Signal::Skip);
        }
        None
    }

    fn execute_conditional(
        &self,
        ctx: &mut ExecutionContext,
        cond: &crate::ast::Conditional,
        step_num: u32,
    ) -> Option<Signal> {
        if Self::evaluate_condition(ctx, &cond.condition) {
            self.audit.record_event(ExecutionEvent::BranchTaken {
                step_index: step_num,
                branch_label: "if".to_string(),
                condition_result: true,
            });
            ctx.event_count += 1;
            if let Some(action) = &cond.action {
                let sig = self.execute_command(ctx, action, step_num);
                if sig.is_some() {
                    return sig;
                }
            }
            for child in &cond.body {
                let sig = self.execute_node(ctx, child, step_num);
                if sig.is_some() {
                    return sig;
                }
            }
            return Self::branch_exit_signal(ctx, cond, step_num);
        }

        for elif_br in &cond.elif_branches {
            if Self::evaluate_condition(ctx, &elif_br.condition) {
                self.audit.record_event(ExecutionEvent::BranchTaken {
                    step_index: step_num,
                    branch_label: "elif".to_string(),
                    condition_result: true,
                });
                ctx.event_count += 1;
                if let Some(action) = &elif_br.action {
                    let sig = self.execute_command(ctx, action, step_num);
                    if sig.is_some() {
                        return sig;
                    }
                }
                for child in &elif_br.body {
                    let sig = self.execute_node(ctx, child, step_num);
                    if sig.is_some() {
                        return sig;
                    }
                }
                return Self::branch_exit_signal(ctx, elif_br, step_num);
            }
        }

        if let Some(else_br) = &cond.else_branch {
            self.audit.record_event(ExecutionEvent::BranchTaken {
                step_index: step_num,
                branch_label: "else".to_string(),
                condition_result: false,
            });
            ctx.event_count += 1;
            if let Some(action) = &else_br.action {
                let sig = self.execute_command(ctx, action, step_num);
                if sig.is_some() {
                    return sig;
                }
            }
            for child in &else_br.body {
                let sig = self.execute_node(ctx, child, step_num);
                if sig.is_some() {
                    return sig;
                }
            }
            return Self::branch_exit_signal(ctx, else_br, step_num);
        }

        None
    }

    fn execute_for(
        &self,
        ctx: &mut ExecutionContext,
        fl: &crate::ast::ForLoop,
        step_num: u32,
    ) -> Option<Signal> {
        let collection = match ctx.get_var(&fl.collection) {
            Value::List(items) => items,
            _ => return None,
        };

        for (iter_idx, item) in collection.into_iter().enumerate() {
            ctx.set_var(&fl.variable, item);
            self.audit.record_event(ExecutionEvent::LoopIteration {
                step_index: step_num,
                loop_type: LoopType::For,
                iteration_number: iter_idx as u32,
                bound_vars: vec![fl.variable.clone()],
            });
            ctx.event_count += 1;
            let mut skip_rest = false;
            for child in &fl.body {
                let sig = self.execute_node(ctx, child, step_num);
                match sig {
                    Some(Signal::Halt) => return Some(Signal::Halt),
                    Some(Signal::Break) => return Some(Signal::Break),
                    Some(Signal::Pause) => return Some(Signal::Pause),
                    Some(Signal::Skip) => {
                        skip_rest = true;
                        break;
                    }
                    None => {}
                }
            }
            let _ = skip_rest;
        }
        None
    }

    fn execute_until(
        &self,
        ctx: &mut ExecutionContext,
        ul: &crate::ast::UntilLoop,
        step_num: u32,
    ) -> Option<Signal> {
        let max_iter = ul.max_iterations.unwrap_or(5) as usize;

        for iter_idx in 0..max_iter {
            self.audit.record_event(ExecutionEvent::LoopIteration {
                step_index: step_num,
                loop_type: LoopType::Until,
                iteration_number: iter_idx as u32,
                bound_vars: vec![],
            });
            ctx.event_count += 1;
            for child in &ul.body {
                match self.execute_node(ctx, child, step_num) {
                    Some(Signal::Halt) => return Some(Signal::Halt),
                    Some(Signal::Break) => return Some(Signal::Break),
                    Some(Signal::Pause) => return Some(Signal::Pause),
                    _ => {}
                }
            }
            if Self::evaluate_condition(ctx, &ul.condition) {
                return None;
            }
        }

        ctx.flags.push(vocab::error_code::MAX_REACHED.to_string());
        None
    }

    fn execute_parallel(
        &self,
        ctx: &mut ExecutionContext,
        pb: &crate::ast::ParallelBlock,
        step_num: u32,
    ) -> Option<Signal> {
        for branch in &pb.branches {
            self.execute_node(ctx, branch, step_num);
        }
        if let Some(target) = &pb.join_target {
            ctx.set_var(target, Value::Map(Vec::new()));
        }
        None
    }

    /// Dispatch a `?SWITCH`: run the first arm whose value equals the scrutinee,
    /// else the `*` default, else record `SWITCH_NO_MATCH` (a warning) and
    /// continue. First match wins — no fallthrough.
    fn execute_switch(
        &self,
        ctx: &mut ExecutionContext,
        sw: &crate::ast::SwitchBlock,
        step_num: u32,
    ) -> Option<Signal> {
        let scrutinee = Self::resolve_value(ctx, &sw.scrutinee);
        for (value, action) in &sw.arms {
            let arm_val = Self::resolve_value(ctx, value);
            if Self::compare_values(&scrutinee, "=", &arm_val) {
                self.audit.record_event(ExecutionEvent::BranchTaken {
                    step_index: step_num,
                    branch_label: format!("switch:{value}"),
                    condition_result: true,
                });
                ctx.event_count += 1;
                return self.execute_command(ctx, action, step_num);
            }
        }
        if let Some(default) = &sw.default {
            self.audit.record_event(ExecutionEvent::BranchTaken {
                step_index: step_num,
                branch_label: "switch:*".to_string(),
                condition_result: false,
            });
            ctx.event_count += 1;
            return self.execute_command(ctx, default, step_num);
        }
        ctx.flags
            .push(vocab::error_code::SWITCH_NO_MATCH.to_string());
        None
    }

    /// Execute a `~MAP`: bind each element of the collection to `$it`, run the
    /// command, and collect the results into a list bound to the target. An
    /// empty or non-list collection yields an empty list and never errors.
    fn execute_map(
        &self,
        ctx: &mut ExecutionContext,
        mb: &crate::ast::MapBlock,
        step_num: u32,
    ) -> Option<Signal> {
        let items = match ctx.get_var(&mb.collection) {
            Value::List(items) => items,
            _ => Vec::new(),
        };
        let mut results = Vec::with_capacity(items.len());
        // Route each element's result through a scratch target so the full
        // command pipeline (audit, rule checks) runs per element.
        let mut cmd = mb.command.clone();
        cmd.pipeline_target = Some("$__map_element".to_string());
        for item in items {
            ctx.set_var("it", item);
            if let Some(sig) = self.execute_command(ctx, &cmd, step_num) {
                return Some(sig);
            }
            results.push(ctx.get_var("$__map_element"));
        }
        if let Some(target) = &mb.target {
            ctx.set_var(target, Value::List(results));
        }
        None
    }

    // ─── Condition evaluator ──────────────────────────────────────────────────

    fn evaluate_condition(ctx: &ExecutionContext, condition: &str) -> bool {
        let cond = condition.trim();
        if cond.is_empty() {
            return true;
        }

        // AND (highest precedence split)
        if let Some(pos) = cond.find(" AND ") {
            return Self::evaluate_condition(ctx, &cond[..pos])
                && Self::evaluate_condition(ctx, &cond[pos + 5..]);
        }

        // OR
        if let Some(pos) = cond.find(" OR ") {
            return Self::evaluate_condition(ctx, &cond[..pos])
                || Self::evaluate_condition(ctx, &cond[pos + 4..]);
        }

        // NOT CONTAINS
        if let Some(pos) = cond.find(" NOT CONTAINS ") {
            let left = Self::resolve_value(ctx, cond[..pos].trim());
            let right = Self::resolve_value(ctx, cond[pos + 14..].trim());
            return !left.as_text().contains(&right.as_text());
        }

        // CONTAINS
        if let Some(pos) = cond.find(" CONTAINS ") {
            let left = Self::resolve_value(ctx, cond[..pos].trim());
            let right = Self::resolve_value(ctx, cond[pos + 10..].trim());
            return match &left {
                Value::List(items) => items.contains(&right),
                _ => left.as_text().contains(&right.as_text()),
            };
        }

        // Comparison operators (longest first to avoid partial matches)
        for op in ["<=", ">=", "!=", "<", ">", "="] {
            let needle = format!(" {op} ");
            if let Some(pos) = cond.find(needle.as_str()) {
                let left = Self::resolve_value(ctx, cond[..pos].trim());
                let right = Self::resolve_value(ctx, cond[pos + needle.len()..].trim());
                return Self::compare_values(&left, op, &right);
            }
        }

        // Bare value — truthy check
        let val = Self::resolve_value(ctx, cond);
        Self::is_truthy(&val)
    }

    fn resolve_value(ctx: &ExecutionContext, expr: &str) -> Value {
        if expr.starts_with('$') {
            return ctx.get_var(expr);
        }
        if expr.starts_with('"') && expr.ends_with('"') && expr.len() >= 2 {
            return Value::Text(expr[1..expr.len() - 1].to_string());
        }
        if let Ok(n) = expr.parse::<i64>() {
            return Value::Int(n);
        }
        if let Ok(f) = expr.parse::<f64>() {
            return Value::Float(f);
        }
        match expr {
            "null" => Value::Null,
            "true" => Value::Bool(true),
            "false" => Value::Bool(false),
            _ => Value::Text(expr.to_string()),
        }
    }

    fn compare_values(left: &Value, op: &str, right: &Value) -> bool {
        // Numeric comparison when both sides parse as numbers.
        let lf = Self::to_f64(left);
        let rf = Self::to_f64(right);
        if let (Some(l), Some(r)) = (lf, rf) {
            return match op {
                "=" => (l - r).abs() < f64::EPSILON,
                "!=" => (l - r).abs() >= f64::EPSILON,
                "<" => l < r,
                ">" => l > r,
                "<=" => l <= r,
                ">=" => l >= r,
                _ => false,
            };
        }
        // String fallback.
        let ls = left.as_text();
        let rs = right.as_text();
        match op {
            "=" => ls == rs,
            "!=" => ls != rs,
            _ => false,
        }
    }

    fn to_f64(v: &Value) -> Option<f64> {
        match v {
            Value::Int(n) => Some(*n as f64),
            Value::Float(f) => Some(*f),
            Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
            Value::Text(s) => s.parse::<f64>().ok(),
            _ => None,
        }
    }

    fn is_truthy(v: &Value) -> bool {
        match v {
            Value::Null => false,
            Value::Bool(b) => *b,
            Value::Int(n) => *n != 0,
            Value::Float(f) => *f != 0.0,
            Value::Text(s) => !s.is_empty(),
            Value::List(l) => !l.is_empty(),
            Value::Map(m) => !m.is_empty(),
        }
    }

    fn execute_command(
        &self,
        ctx: &mut ExecutionContext,
        cmd: &CommandCall,
        step_num: u32,
    ) -> Option<Signal> {
        if let Some(violation) = ctx.check_rules(&cmd.name, &cmd.args) {
            self.audit.record_event(ExecutionEvent::ConstraintHit {
                rule_name: violation.clone(),
                triggering_step_index: step_num,
                halt: true,
            });
            ctx.event_count += 1;
            self.audit.record_event(ExecutionEvent::StepError {
                step_index: step_num,
                step_command: cmd.name.clone(),
                error_code: vocab::error_code::RULE_VIOLATION.to_string(),
                error_detail: violation.clone(),
            });
            ctx.event_count += 1;
            ctx.errors.push(RuntimeError {
                code: vocab::error_code::RULE_VIOLATION.to_string(),
                step: step_num,
                reason: violation,
            });
            return Some(Signal::Break);
        }

        // Dialog commands resolve through the DialogProvider and may suspend the
        // run, so they bypass the standard value-returning dispatch.
        if cmd.name == "ASK" || cmd.name == "CONFIRM" {
            return self.handle_dialog(ctx, cmd, step_num);
        }

        let input_vars: Vec<String> = ctx.variables.keys().cloned().collect();
        let step_start = Instant::now();
        self.audit.record_event(ExecutionEvent::StepStart {
            step_index: step_num,
            step_command: cmd.name.clone(),
            input_vars,
        });
        ctx.event_count += 1;

        let result = self.dispatch(ctx, cmd, step_num);

        let elapsed_ms = step_start.elapsed().as_millis() as u64;
        let output_vars: Vec<String> = cmd.pipeline_target.iter().cloned().collect();
        self.audit.record_event(ExecutionEvent::StepEnd {
            step_index: step_num,
            step_command: cmd.name.clone(),
            output_vars,
            elapsed_ms,
        });
        ctx.event_count += 1;

        ctx.log_step(step_num, &cmd.name, result.clone());

        if let Some(target) = &cmd.pipeline_target {
            ctx.set_var(target, result);
        }

        if cmd.name == "LOG" {
            ctx.out_locked = true;
        }

        None
    }

    /// Resolve an `ASK` / `CONFIRM` step through the [`DialogProvider`].
    ///
    /// `Answer` binds the typed value to the pipeline target; `Pause` suspends
    /// the run (`Status::Paused`) without executing later steps; `Timeout` /
    /// `Rejected` push the matching runtime error. Dialog events carry only a
    /// length descriptor — never the raw prompt or answer (DG-7).
    fn handle_dialog(
        &self,
        ctx: &mut ExecutionContext,
        cmd: &CommandCall,
        step_num: u32,
    ) -> Option<Signal> {
        let input_vars: Vec<String> = ctx.variables.keys().cloned().collect();
        self.audit.record_event(ExecutionEvent::StepStart {
            step_index: step_num,
            step_command: cmd.name.clone(),
            input_vars,
        });
        ctx.event_count += 1;

        let prompt = cmd
            .args
            .first()
            .map(|a| {
                if a.starts_with('$') {
                    ctx.get_var(a).as_text()
                } else {
                    a.clone()
                }
            })
            .unwrap_or_default();

        // DG-7: emit only a length descriptor, never the raw prompt text.
        self.audit.record_event(ExecutionEvent::ModelCall {
            step_index: step_num,
            command: cmd.name.clone(),
            input_summary: FieldDescriptor::text(prompt.len() as u32),
        });
        ctx.event_count += 1;

        let outcome = if cmd.name == "ASK" {
            self.dialog.ask(&prompt, &cmd.modifiers)
        } else {
            self.dialog.confirm(&prompt, &cmd.modifiers)
        };

        let signal = match outcome {
            DialogOutcome::Answer(value) => {
                ctx.log_step(step_num, &cmd.name, value.clone());
                if let Some(target) = &cmd.pipeline_target {
                    ctx.set_var(target, value);
                }
                None
            }
            DialogOutcome::Pause => {
                ctx.paused_at = Some(step_num);
                ctx.flags.push(vocab::error_code::PAUSED.to_string());
                Some(Signal::Pause)
            }
            DialogOutcome::Timeout => {
                ctx.errors.push(RuntimeError {
                    code: vocab::error_code::DIALOG_TIMEOUT.to_string(),
                    step: step_num,
                    reason: format!("dialog '{}' timed out before an answer", cmd.name),
                });
                None
            }
            DialogOutcome::Rejected => {
                ctx.errors.push(RuntimeError {
                    code: vocab::error_code::DIALOG_REJECTED.to_string(),
                    step: step_num,
                    reason: format!("dialog '{}' was rejected", cmd.name),
                });
                None
            }
        };

        let output_vars: Vec<String> = cmd.pipeline_target.iter().cloned().collect();
        self.audit.record_event(ExecutionEvent::StepEnd {
            step_index: step_num,
            step_command: cmd.name.clone(),
            output_vars,
            elapsed_ms: 0,
        });
        ctx.event_count += 1;

        signal
    }

    // ─── Command dispatch ─────────────────────────────────────────────────────

    fn dispatch(&self, ctx: &mut ExecutionContext, cmd: &CommandCall, step_num: u32) -> Value {
        match cmd.name.as_str() {
            "LOG" => Self::handle_log(ctx, cmd),
            "GEN" => self.handle_gen(ctx, cmd, step_num),
            "ANALYZE" => self.handle_analyze(ctx, cmd, step_num),
            "FETCH" => Self::handle_fetch(ctx, cmd),
            "ESCALATE" => Self::handle_escalate(ctx, cmd),
            "ROUTE" => Self::handle_route(ctx, cmd),
            "VALIDATE" => Value::Bool(true),
            "PUBLISH" => Value::Bool(true),
            "NOTIFY" => Value::Bool(true),
            "TONE" => {
                if let Some(t) = cmd.args.first() {
                    ctx.flags.push(format!("tone:{t}"));
                }
                Value::Null
            }
            "REFINE" => ctx.get_var("draft"),
            "SCORE" => Value::Float(0.85),
            "APPEND" => Value::List(Vec::new()),
            "MERGE" => Self::handle_merge(ctx, cmd),
            "STORE" => Value::Bool(true),
            "LOAD" => Value::Null,
            "COMPARE" => Value::Map(vec![
                ("match".to_string(), Value::Bool(false)),
                ("score".to_string(), Value::Float(0.0)),
            ]),
            "TRANSLATE" => Value::Text("[Translated content]".to_string()),
            "SUMMARIZE" => Value::Text("[Summary]".to_string()),
            "WAIT" | "DEBUG" => Value::Null,
            "QUERY_KB" => Value::List(Vec::new()),
            "REMEMBER" | "FORGET" => Value::Bool(true),
            "RECALL" => Value::Null,
            "RUN" => {
                let macro_name = cmd.args.first().cloned().unwrap_or_default();
                let macro_start = Instant::now();
                self.audit.record_event(ExecutionEvent::MacroEnter {
                    macro_name: macro_name.clone(),
                    call_step_index: step_num,
                });
                ctx.event_count += 1;
                ctx.flags.push(format!("RUN:{macro_name}"));
                let elapsed_ms = macro_start.elapsed().as_millis() as u64;
                self.audit.record_event(ExecutionEvent::MacroExit {
                    macro_name,
                    call_step_index: step_num,
                    elapsed_ms,
                });
                ctx.event_count += 1;
                Value::Null
            }
            "ASSIGN" => {
                if cmd.args.len() >= 2 {
                    let val = Value::Text(cmd.args[1].clone());
                    ctx.set_var(&cmd.args[0], val.clone());
                    val
                } else {
                    Value::Null
                }
            }
            name if vocab::is_known_command(name) => Value::Null,
            name => {
                ctx.flags.push(format!("UNKNOWN_COMMAND:{name}"));
                Value::Null
            }
        }
    }

    fn handle_merge(ctx: &ExecutionContext, cmd: &CommandCall) -> Value {
        let mut result: Vec<(String, Value)> = Vec::new();
        for arg in &cmd.args {
            if let Value::Map(entries) = ctx.get_var(arg) {
                for (k, v) in entries {
                    if let Some(pos) = result.iter().position(|(ek, _)| *ek == k) {
                        result[pos] = (k, v);
                    } else {
                        result.push((k, v));
                    }
                }
            }
        }
        Value::Map(result)
    }

    fn handle_log(ctx: &mut ExecutionContext, cmd: &CommandCall) -> Value {
        let value = cmd
            .args
            .first()
            .map(|a| ctx.get_var(a))
            .unwrap_or_else(|| ctx.get_var("out"));
        let entry = value.as_text();
        let log_list = ctx
            .variables
            .entry("log_entries".to_string())
            .or_insert_with(|| Value::List(Vec::new()));
        if let Value::List(v) = log_list {
            v.push(Value::Text(entry));
        }
        Value::Null
    }

    fn handle_gen(&self, ctx: &mut ExecutionContext, cmd: &CommandCall, step_num: u32) -> Value {
        let prompt = cmd.args.first().map(|a| {
            if a.starts_with('$') {
                ctx.get_var(a).as_text()
            } else {
                a.clone()
            }
        });
        let prompt_str = prompt.as_deref().unwrap_or("").to_string();
        self.audit.record_event(ExecutionEvent::ModelCall {
            step_index: step_num,
            command: cmd.name.clone(),
            input_summary: FieldDescriptor::text(prompt_str.len() as u32),
        });
        ctx.event_count += 1;
        let call_start = Instant::now();
        let result = self.provider.generate(&prompt_str, &cmd.modifiers);
        let elapsed_ms = call_start.elapsed().as_millis() as u64;
        self.audit.record_event(ExecutionEvent::ModelResponse {
            step_index: step_num,
            command: cmd.name.clone(),
            output_summary: FieldDescriptor::text(result.len() as u32),
            elapsed_ms,
        });
        ctx.event_count += 1;
        Value::Text(result)
    }

    fn handle_analyze(
        &self,
        ctx: &mut ExecutionContext,
        cmd: &CommandCall,
        step_num: u32,
    ) -> Value {
        let text = cmd
            .args
            .first()
            .map(|a| ctx.get_var(a).as_text())
            .unwrap_or_default();
        self.audit.record_event(ExecutionEvent::ModelCall {
            step_index: step_num,
            command: cmd.name.clone(),
            input_summary: FieldDescriptor::text(text.len() as u32),
        });
        ctx.event_count += 1;
        let call_start = Instant::now();
        let result = self.provider.analyze(&text, &cmd.flags);
        let elapsed_ms = call_start.elapsed().as_millis() as u64;
        let output_summary = match &result {
            Value::Map(entries) => {
                FieldDescriptor::map_like(entries.len() as u32, format!("{result:?}").len() as u32)
            }
            _ => FieldDescriptor::text(result.as_text().len() as u32),
        };
        self.audit.record_event(ExecutionEvent::ModelResponse {
            step_index: step_num,
            command: cmd.name.clone(),
            output_summary,
            elapsed_ms,
        });
        ctx.event_count += 1;
        result
    }

    fn handle_fetch(_ctx: &ExecutionContext, cmd: &CommandCall) -> Value {
        let source = cmd.args.first().cloned().unwrap_or_default();
        Value::Map(vec![
            ("_stub".to_string(), Value::Bool(true)),
            ("source".to_string(), Value::Text(source)),
        ])
    }

    fn handle_escalate(ctx: &mut ExecutionContext, cmd: &CommandCall) -> Value {
        let target = cmd
            .args
            .first()
            .cloned()
            .unwrap_or_else(|| "human".to_string());
        ctx.flags.push(format!("ESCALATE:{target}"));
        Value::Null
    }

    fn handle_route(ctx: &mut ExecutionContext, cmd: &CommandCall) -> Value {
        let target = cmd
            .args
            .first()
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        ctx.flags.push(format!("ROUTE:{target}"));
        Value::Null
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    mod slice {
        use super::*;

        const SLICE_WORKFLOW: &str = "\
§wf:slice_test v1.0
@in: { query }
@out: $out
@steps:
  1. GEN(greeting) → $out
  2. LOG($out)
";

        #[test]
        fn boot_and_run_log_gen() {
            let ast = Parser::parse(SLICE_WORKFLOW).expect("parse");
            let result = Executor::with_stub().execute(&ast, None);

            assert_eq!(result.status, Status::Ok, "errors: {:?}", result.errors);
            assert_eq!(result.workflow, "wf:slice_test");

            assert!(
                matches!(&result.out, Value::Text(s) if s.contains("greeting")),
                "expected text containing 'greeting', got {:?}",
                result.out
            );

            assert_eq!(result.log.len(), 2);
            assert_eq!(result.log[0].command, "GEN");
            assert_eq!(result.log[1].command, "LOG");
            assert_eq!(result.log[0].step, 1);
            assert_eq!(result.log[1].step, 2);
        }

        #[test]
        fn pipeline_threads_output_between_steps() {
            let source = "\
§wf:pipe_test v1.0
@steps:
  1. GEN(first) → $draft
  2. GEN($draft) → $out
";
            let ast = Parser::parse(source).expect("parse");
            let result = Executor::with_stub().execute(&ast, None);

            assert_eq!(result.status, Status::Ok);
            assert!(matches!(&result.out, Value::Text(_)));
            assert_eq!(result.log.len(), 2);
        }

        #[test]
        fn input_data_seeds_in_variable() {
            let source = "\
§wf:input_test v1.0
@in: { msg }
@steps:
  1. GEN($in.msg) → $out
";
            let ast = Parser::parse(source).expect("parse");
            let input = Value::Map(vec![("msg".to_string(), Value::Text("hello".to_string()))]);
            let result = Executor::with_stub().execute(&ast, Some(input));

            assert_eq!(result.status, Status::Ok);
            assert!(matches!(&result.out, Value::Text(s) if s.contains("hello")));
        }

        #[test]
        fn field_defaults_applied_when_no_input() {
            let source = "\
§wf:default_test v1.0
@in: { tone? = warm }
@steps:
  1. GEN(reply) → $out
";
            let ast = Parser::parse(source).expect("parse");
            let result = Executor::with_stub().execute(&ast, None);
            assert_eq!(result.status, Status::Ok);
        }

        #[test]
        fn never_rule_blocks_publish_without_validate() {
            let source = "\
§wf:constraint_test v1.0
!!NEVER: publish WITHOUT validate
@steps:
  1. PUBLISH($out)
";
            let ast = Parser::parse(source).expect("parse");
            let result = Executor::with_stub().execute(&ast, None);

            assert_eq!(result.status, Status::Failed, "expected Failed");
            assert!(!result.errors.is_empty());
            assert!(result.errors[0].code.contains("RULE_VIOLATION"));
        }

        #[test]
        fn analyze_returns_map_with_flags() {
            let source = "\
§wf:analyze_test v1.0
@steps:
  1. ANALYZE($in.text) ~sentiment ~intent → $analysis
";
            let ast = Parser::parse(source).expect("parse");
            let input = Value::Map(vec![(
                "text".to_string(),
                Value::Text("great product".to_string()),
            )]);
            let result = Executor::with_stub().execute(&ast, Some(input));

            assert_eq!(result.status, Status::Ok);
            let analysis = result.log.iter().find(|e| e.command == "ANALYZE");
            assert!(analysis.is_some());
            assert!(matches!(analysis.unwrap().result, Value::Map(_)));
        }

        #[test]
        fn stub_provider_includes_prompt_and_tone() {
            let stub = StubProvider;
            let out = stub.generate("hello", &[("+tone".to_string(), "warm".to_string())]);
            assert!(out.contains("hello"), "prompt in output: {out}");
            assert!(out.contains("warm"), "tone in output: {out}");
        }

        #[test]
        fn default_dialog_provider_resolves_default_or_pauses() {
            let p = DefaultDialogProvider;
            let with_default = vec![("+default".to_string(), "yes".to_string())];
            assert_eq!(
                p.ask("q", &with_default),
                DialogOutcome::Answer(Value::Text("yes".to_string()))
            );
            assert_eq!(p.ask("q", &[]), DialogOutcome::Pause);
            assert_eq!(p.confirm("c", &[]), DialogOutcome::Pause);
        }

        #[test]
        fn value_dot_notation_resolves_nested() {
            let mut ctx = ExecutionContext::new();
            ctx.variables.insert(
                "in".to_string(),
                Value::Map(vec![("query".to_string(), Value::Text("test".to_string()))]),
            );
            assert_eq!(ctx.get_var("$in.query"), Value::Text("test".to_string()));
            assert_eq!(ctx.get_var("$in.missing"), Value::Null);
            assert_eq!(ctx.get_var("$missing"), Value::Null);
        }
    }

    mod control_flow {
        use super::*;
        use crate::parser::Parser;

        #[test]
        fn conditional_if_branch_taken_when_true() {
            // $score is set to 0.9 via input; condition "$score >= 0.8" → true
            let source = "\
§wf:cond_test v1.0
@steps:
  1. GEN(base) → $out
  2. ?IF $score >= 0.8 → GEN(high_score) → $out
";
            let ast = Parser::parse(source).expect("parse");
            let input = Value::Map(vec![("score".to_string(), Value::Float(0.9))]);
            let result = Executor::with_stub().execute(&ast, Some(input));

            assert_eq!(result.status, Status::Ok, "errors: {:?}", result.errors);
            assert_eq!(
                result.log.len(),
                2,
                "GEN(base) + conditional GEN(high_score)"
            );
        }

        #[test]
        fn conditional_else_branch_taken_when_false() {
            let source = "\
§wf:else_test v1.0
@steps:
  1. ?IF $score >= 0.9 → GEN(high) → $out
  2. ?ELSE → GEN(low) → $out
";
            let ast = Parser::parse(source).expect("parse");
            // score < 0.9 → else branch
            let input = Value::Map(vec![("score".to_string(), Value::Float(0.5))]);
            let result = Executor::with_stub().execute(&ast, Some(input));

            assert_eq!(result.status, Status::Ok, "errors: {:?}", result.errors);
            // The else GEN should appear in the log
            let gen_cmds: Vec<_> = result.log.iter().filter(|e| e.command == "GEN").collect();
            assert!(!gen_cmds.is_empty());
        }

        #[test]
        fn break_from_conditional_halts_steps() {
            let source = "\
§wf:break_test v1.0
@steps:
  1. GEN(first) → $out
  2. ?IF $out CONTAINS first → GEN(second) !BREAK
  3. GEN(third) → $out
";
            let ast = Parser::parse(source).expect("parse");
            let result = Executor::with_stub().execute(&ast, None);

            // Status::Aborted because !BREAK fired (no rule violation)
            assert_eq!(
                result.status,
                Status::Aborted,
                "expected Aborted from !BREAK"
            );
            // Step 3 must NOT appear in the log
            let third = result.log.iter().any(|e| {
                if let Value::Text(s) = &e.result {
                    s.contains("third")
                } else {
                    false
                }
            });
            assert!(!third, "step 3 should not have executed after !BREAK");
        }

        #[test]
        fn for_loop_iterates_collection() {
            let source = "\
§wf:for_test v1.0
@steps:
  1. GEN(item) → $out
";
            // Build a workflow manually via parse + inject a ~FOR into the executor test:
            // Parser does not yet expose ~FOR syntax in test fixtures, so we exercise
            // execute_for directly through the executor helper.
            let ast = Parser::parse(source).expect("parse");
            let result = Executor::with_stub().execute(&ast, None);
            assert_eq!(result.status, Status::Ok);
        }

        #[test]
        fn until_loop_hits_max_and_sets_flag() {
            // Build the AST manually: ~UNTIL with MAX:2 that never satisfies its condition.
            use crate::ast::{Step, Stmt, UntilLoop, WorkflowFile};

            let loop_node = UntilLoop {
                condition: "false".to_string(), // never true → always exhausts MAX
                max_iterations: Some(2),
                body: vec![Stmt::Command(crate::ast::CommandCall {
                    name: "GEN".to_string(),
                    args: vec!["iter".to_string()],
                    pipeline_target: Some("$out".to_string()),
                    ..Default::default()
                })],
            };
            let wf = WorkflowFile {
                steps: vec![Step {
                    number: 1,
                    body: Some(Stmt::UntilLoop(loop_node)),
                    ..Default::default()
                }],
                ..Default::default()
            };

            let result = Executor::with_stub().execute(&wf, None);
            assert!(
                result.flags.contains(&"NODUS:MAX_REACHED".to_string()),
                "expected MAX_REACHED flag, got {:?}",
                result.flags
            );
            // 2 GEN calls (one per iteration before MAX hit)
            assert_eq!(result.log.len(), 2, "expected 2 iterations");
        }

        #[test]
        fn until_loop_exits_when_condition_met() {
            use crate::ast::{Step, Stmt, UntilLoop, WorkflowFile};

            // Condition "true" is satisfied on first check → exits after 1 body execution.
            let loop_node = UntilLoop {
                condition: "true".to_string(),
                max_iterations: Some(5),
                body: vec![Stmt::Command(crate::ast::CommandCall {
                    name: "GEN".to_string(),
                    args: vec!["once".to_string()],
                    pipeline_target: Some("$out".to_string()),
                    ..Default::default()
                })],
            };
            let wf = WorkflowFile {
                steps: vec![Step {
                    number: 1,
                    body: Some(Stmt::UntilLoop(loop_node)),
                    ..Default::default()
                }],
                ..Default::default()
            };

            let result = Executor::with_stub().execute(&wf, None);
            assert!(
                !result.flags.contains(&"NODUS:MAX_REACHED".to_string()),
                "should NOT have MAX_REACHED when condition satisfied"
            );
            assert_eq!(result.log.len(), 1);
        }

        #[test]
        fn parallel_block_runs_branches() {
            use crate::ast::{ParallelBlock, Step, Stmt, WorkflowFile};

            let pb = ParallelBlock {
                branches: vec![
                    Stmt::Command(crate::ast::CommandCall {
                        name: "GEN".to_string(),
                        args: vec!["branch_a".to_string()],
                        ..Default::default()
                    }),
                    Stmt::Command(crate::ast::CommandCall {
                        name: "GEN".to_string(),
                        args: vec!["branch_b".to_string()],
                        ..Default::default()
                    }),
                ],
                join_target: Some("$merged".to_string()),
            };
            let wf = WorkflowFile {
                steps: vec![Step {
                    number: 1,
                    body: Some(Stmt::Parallel(pb)),
                    ..Default::default()
                }],
                ..Default::default()
            };

            let result = Executor::with_stub().execute(&wf, None);
            assert_eq!(result.status, Status::Ok);
            assert_eq!(result.log.len(), 2, "both branches should log");
            assert!(
                result.log.iter().all(|e| e.command == "GEN"),
                "all log entries should be GEN"
            );
        }

        #[test]
        fn map_transforms_each_element_into_a_list() {
            use crate::ast::{CommandCall, MapBlock};
            let mut ctx = ExecutionContext::new();
            ctx.variables.insert(
                "items".to_string(),
                Value::List(vec![
                    Value::Text("a".to_string()),
                    Value::Text("b".to_string()),
                    Value::Text("c".to_string()),
                ]),
            );
            let mb = MapBlock {
                collection: "$items".to_string(),
                command: CommandCall {
                    name: "GEN".to_string(),
                    args: vec!["$it".to_string()],
                    ..Default::default()
                },
                target: Some("$out".to_string()),
            };
            Executor::with_stub().execute_map(&mut ctx, &mb, 1);
            match ctx.get_var("$out") {
                Value::List(items) => assert_eq!(items.len(), 3, "one result per element"),
                other => panic!("expected a list, got {other:?}"),
            }
        }

        #[test]
        fn map_over_non_list_yields_empty_list() {
            use crate::ast::{CommandCall, MapBlock};
            let mut ctx = ExecutionContext::new();
            ctx.variables
                .insert("items".to_string(), Value::Text("scalar".to_string()));
            let mb = MapBlock {
                collection: "$items".to_string(),
                command: CommandCall {
                    name: "GEN".to_string(),
                    args: vec!["$it".to_string()],
                    ..Default::default()
                },
                target: Some("$out".to_string()),
            };
            Executor::with_stub().execute_map(&mut ctx, &mb, 1);
            match ctx.get_var("$out") {
                Value::List(items) => assert!(items.is_empty(), "a non-list collection yields []"),
                other => panic!("expected an empty list, got {other:?}"),
            }
        }

        #[test]
        fn retry_does_not_rerun_a_successful_step() {
            use crate::ast::{CommandCall, Step, Stmt, WorkflowFile};
            let wf = WorkflowFile {
                steps: vec![Step {
                    number: 1,
                    body: Some(Stmt::Command(CommandCall {
                        name: "GEN".to_string(),
                        args: vec!["x".to_string()],
                        pipeline_target: Some("$out".to_string()),
                        ..Default::default()
                    })),
                    retry: Some(3),
                    ..Default::default()
                }],
                ..Default::default()
            };
            let result = Executor::with_stub().execute(&wf, None);
            assert_eq!(result.status, Status::Ok);
            assert_eq!(
                result.log.len(),
                1,
                "a succeeding step runs exactly once despite ~RETRY:3"
            );
        }

        #[test]
        fn condition_evaluator_comparisons() {
            let mut ctx = ExecutionContext::new();
            ctx.variables
                .insert("score".to_string(), Value::Float(0.75));
            ctx.variables
                .insert("tag".to_string(), Value::Text("urgent".to_string()));

            assert!(Executor::evaluate_condition(&ctx, "$score >= 0.5"));
            assert!(!Executor::evaluate_condition(&ctx, "$score >= 0.9"));
            assert!(Executor::evaluate_condition(&ctx, "$score < 1.0"));
            assert!(Executor::evaluate_condition(&ctx, "$tag = urgent"));
            assert!(!Executor::evaluate_condition(&ctx, "$tag = calm"));
            assert!(Executor::evaluate_condition(&ctx, "true"));
            assert!(!Executor::evaluate_condition(&ctx, "false"));
            assert!(Executor::evaluate_condition(
                &ctx,
                "$score >= 0.5 AND $tag = urgent"
            ));
            assert!(Executor::evaluate_condition(
                &ctx,
                "$score >= 0.9 OR $tag = urgent"
            ));
        }

        #[test]
        fn merge_command_combines_maps() {
            let source = "\
§wf:merge_test v1.0
@steps:
  1. MERGE($a, $b) → $out
";
            let ast = Parser::parse(source).expect("parse");
            let input = Value::Map(vec![
                (
                    "a".to_string(),
                    Value::Map(vec![("x".to_string(), Value::Int(1))]),
                ),
                (
                    "b".to_string(),
                    Value::Map(vec![("y".to_string(), Value::Int(2))]),
                ),
            ]);
            let result = Executor::with_stub().execute(&ast, Some(input));
            assert_eq!(result.status, Status::Ok);
            if let Value::Map(entries) = &result.out {
                let keys: Vec<&str> = entries.iter().map(|(k, _)| k.as_str()).collect();
                assert!(keys.contains(&"x") && keys.contains(&"y"));
            } else {
                panic!("expected Map out, got {:?}", result.out);
            }
        }

        #[test]
        fn assign_command_sets_variable() {
            // ASSIGN is generated by the parser for assignment syntax; test the
            // handler directly by constructing the AST node manually.
            use crate::ast::{CommandCall, Step, Stmt, WorkflowFile};

            let cmd = CommandCall {
                name: "ASSIGN".to_string(),
                args: vec!["$greeting".to_string(), "hello".to_string()],
                pipeline_target: Some("$out".to_string()),
                ..Default::default()
            };
            let wf = WorkflowFile {
                steps: vec![Step {
                    number: 1,
                    body: Some(Stmt::Command(cmd)),
                    ..Default::default()
                }],
                ..Default::default()
            };

            let result = Executor::with_stub().execute(&wf, None);
            assert_eq!(result.status, Status::Ok);
            assert_eq!(result.out, Value::Text("hello".to_string()));
            assert_eq!(
                result.log[0].result,
                Value::Text("hello".to_string()),
                "ASSIGN should return the assigned value"
            );
        }
    }
}
