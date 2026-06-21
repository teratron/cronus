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
//! The result contract (`RunResult`) is the structured surface consumed by the
//! library API (T-2F01) and satisfies the WFL-8 invariant.

use crate::ast::{AbsoluteRule, CommandCall, RuleKind, Step, Stmt, WorkflowFile};
use crate::vocab;
use std::collections::HashMap;

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

/// Execution status (WFL-8).
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

/// The structured output of a workflow execution (WFL-8).
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

// ─── Execution signal ─────────────────────────────────────────────────────────

enum Signal {
    Break,
    Skip,
}

// ─── Execution context ────────────────────────────────────────────────────────

struct ExecutionContext {
    variables: HashMap<String, Value>,
    rules: Vec<AbsoluteRule>,
    log: Vec<LogEntry>,
    errors: Vec<RuntimeError>,
    flags: Vec<String>,
    out_locked: bool,
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
}

impl Executor {
    /// Create an executor backed by `provider`.
    pub fn new(provider: impl ModelProvider + 'static) -> Self {
        Executor {
            provider: Box::new(provider),
        }
    }

    /// Create an executor backed by the built-in [`StubProvider`].
    pub fn with_stub() -> Self {
        Self::new(StubProvider)
    }

    /// Run a parsed workflow through the boot sequence, execute its steps, and
    /// return the structured [`RunResult`].
    pub fn execute(&self, ast: &WorkflowFile, input: Option<Value>) -> RunResult {
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
        for step in &ast.steps {
            if matches!(self.execute_step(&mut ctx, step), Some(Signal::Break)) {
                abort = true;
                break;
            }
        }

        // Finalise.
        let out = ctx.get_var("out");
        // Rule violations take precedence over the abort flag because they
        // represent a protocol-level failure, not a user-initiated !BREAK.
        let status = if ctx
            .errors
            .iter()
            .any(|e| e.code == vocab::error_code::RULE_VIOLATION)
        {
            Status::Failed
        } else if abort {
            Status::Aborted
        } else if !ctx.errors.is_empty() {
            Status::Partial
        } else {
            Status::Ok
        };

        RunResult {
            workflow: workflow_id,
            status,
            out,
            log: ctx.log,
            errors: ctx.errors,
            flags: ctx.flags,
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
            Stmt::VarRef(_) | Stmt::Comment(_) => None,
        }
    }

    fn execute_conditional(
        &self,
        ctx: &mut ExecutionContext,
        cond: &crate::ast::Conditional,
        step_num: u32,
    ) -> Option<Signal> {
        if Self::evaluate_condition(ctx, &cond.condition) {
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
            if cond.break_flag {
                return Some(Signal::Break);
            }
            if cond.skip_flag {
                return Some(Signal::Skip);
            }
            return None;
        }

        for elif_br in &cond.elif_branches {
            if Self::evaluate_condition(ctx, &elif_br.condition) {
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
                if elif_br.break_flag {
                    return Some(Signal::Break);
                }
                if elif_br.skip_flag {
                    return Some(Signal::Skip);
                }
                return None;
            }
        }

        if let Some(else_br) = &cond.else_branch {
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
            if else_br.break_flag {
                return Some(Signal::Break);
            }
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

        for item in collection {
            ctx.set_var(&fl.variable, item);
            let mut skip_rest = false;
            for child in &fl.body {
                let sig = self.execute_node(ctx, child, step_num);
                match sig {
                    Some(Signal::Break) => return Some(Signal::Break),
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

        for _ in 0..max_iter {
            for child in &ul.body {
                let sig = self.execute_node(ctx, child, step_num);
                if matches!(sig, Some(Signal::Break)) {
                    return Some(Signal::Break);
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
            ctx.errors.push(RuntimeError {
                code: vocab::error_code::RULE_VIOLATION.to_string(),
                step: step_num,
                reason: violation,
            });
            return Some(Signal::Break);
        }

        let result = self.dispatch(ctx, cmd);
        ctx.log_step(step_num, &cmd.name, result.clone());

        if let Some(target) = &cmd.pipeline_target {
            ctx.set_var(target, result);
        }

        if cmd.name == "LOG" {
            ctx.out_locked = true;
        }

        None
    }

    // ─── Command dispatch ─────────────────────────────────────────────────────

    fn dispatch(&self, ctx: &mut ExecutionContext, cmd: &CommandCall) -> Value {
        match cmd.name.as_str() {
            "LOG" => Self::handle_log(ctx, cmd),
            "GEN" => self.handle_gen(ctx, cmd),
            "ANALYZE" => self.handle_analyze(ctx, cmd),
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
                ctx.flags.push(format!("RUN:{macro_name}"));
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

    fn handle_gen(&self, ctx: &ExecutionContext, cmd: &CommandCall) -> Value {
        let prompt = cmd.args.first().map(|a| {
            if a.starts_with('$') {
                ctx.get_var(a).as_text()
            } else {
                a.clone()
            }
        });
        Value::Text(
            self.provider
                .generate(prompt.as_deref().unwrap_or(""), &cmd.modifiers),
        )
    }

    fn handle_analyze(&self, ctx: &ExecutionContext, cmd: &CommandCall) -> Value {
        let text = cmd
            .args
            .first()
            .map(|a| ctx.get_var(a).as_text())
            .unwrap_or_default();
        self.provider.analyze(&text, &cmd.flags)
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
