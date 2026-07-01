use crate::cli::Command;
use crate::output::Context;

pub fn dispatch(command: Command, ctx: &Context) -> i32 {
    match command {
        Command::Init { path } => init::run(path, ctx),
        Command::Status => status::run(ctx),
        Command::Workflow { sub } => workflow::dispatch(sub, ctx),
        Command::Workspace { sub } => workspace::dispatch(sub, ctx),
        Command::Memory { sub } => memory::dispatch(sub, ctx),
        Command::Codegraph { sub } => codegraph_cmd::dispatch(sub, ctx),
        Command::Agent { sub } => agent::dispatch(sub, ctx),
        Command::Role { sub } => role::dispatch(sub, ctx),
        Command::Board { sub } => board::dispatch(sub, ctx),
        Command::Schedule { sub } => schedule::dispatch(sub, ctx),
        Command::Budget { sub } => budget_cmd::dispatch(sub, ctx),
        Command::Exec { sub } => exec::dispatch(sub, ctx),
        Command::Check { sub } => check::dispatch(sub, ctx),
        Command::Ext { sub } => ext::dispatch(sub, ctx),
        Command::Learn { sub } => learn::dispatch(sub, ctx),
        Command::Registry { sub } => registry::dispatch(sub, ctx),
        Command::Goal { sub } => goal::dispatch(sub, ctx),
        Command::Trigger { sub } => trigger::dispatch(sub, ctx),
        Command::Mission { sub } => mission::dispatch(sub, ctx),
        Command::Research { sub } => research::dispatch(sub, ctx),
        Command::Change { sub } => change::dispatch(sub, ctx),
    }
}

// ─── init ─────────────────────────────────────────────────────────────────────

mod init {
    use std::path::{Path, PathBuf};

    use cronus::state;

    use crate::output::Context;

    pub fn run(path: Option<PathBuf>, ctx: &Context) -> i32 {
        let target =
            path.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        run_at(&target, ctx)
    }

    fn run_at(target: &Path, ctx: &Context) -> i32 {
        let already = target.join("app.json").exists();
        match state::bootstrap_at(target) {
            Ok(()) => {
                if already {
                    if ctx.is_json() {
                        let p = json_escape(&target.display().to_string());
                        println!("{{\"result\":\"already-initialized\",\"path\":\"{p}\"}}");
                    } else {
                        println!("Workspace already initialized: {}", target.display());
                    }
                } else {
                    let abs = target
                        .canonicalize()
                        .unwrap_or_else(|_| target.to_path_buf());
                    if ctx.is_json() {
                        let p = json_escape(&abs.display().to_string());
                        println!("{{\"result\":\"initialized\",\"path\":\"{p}\"}}");
                    } else {
                        println!("Initialized: {}", abs.display());
                    }
                }
                0
            }
            Err(e) => {
                eprintln!("error: {e}");
                1
            }
        }
    }

    fn json_escape(s: &str) -> String {
        s.replace('\\', "\\\\").replace('"', "\\\"")
    }

    #[cfg(test)]
    mod tests {
        use std::fs;

        use crate::output::{Context, OutputFormat};

        use super::run_at;

        #[test]
        fn cmd_init_creates_state() {
            let tmp = std::env::temp_dir().join(format!("cronus-init-{}", std::process::id()));
            let _ = fs::remove_dir_all(&tmp);

            let ctx = Context::new(OutputFormat::Text);
            assert_eq!(run_at(&tmp, &ctx), 0, "init must exit 0 on success");
            assert!(
                tmp.join("app.json").is_file(),
                "app.json must exist after init"
            );

            let _ = fs::remove_dir_all(&tmp);
        }

        #[test]
        fn cmd_init_is_idempotent() {
            let tmp = std::env::temp_dir().join(format!("cronus-init-idem-{}", std::process::id()));
            let _ = fs::remove_dir_all(&tmp);

            let ctx = Context::new(OutputFormat::Text);
            assert_eq!(run_at(&tmp, &ctx), 0, "first init must succeed");
            assert_eq!(run_at(&tmp, &ctx), 0, "second init must also exit 0");

            let _ = fs::remove_dir_all(&tmp);
        }
    }
}

// ─── status ───────────────────────────────────────────────────────────────────

mod status {
    use std::path::Path;

    use cronus::paths::{Paths, Root};

    use crate::output::Context;

    pub fn run(ctx: &Context) -> i32 {
        let paths = Paths::os_native();
        let root = paths.resolve(Root::State);
        run_at(&root, ctx)
    }

    fn run_at(state_root: &Path, ctx: &Context) -> i32 {
        if !state_root.join("app.json").exists() {
            eprintln!("No workspace initialized. Run 'cronus init' first.");
            return 1;
        }
        let workspace = state_root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("default")
            .to_owned();
        if ctx.is_json() {
            println!("{{\"workspace\":\"{workspace}\",\"phase\":\"ready\"}}");
        } else {
            println!("workspace: {workspace}");
            println!("phase:     ready");
        }
        0
    }

    #[cfg(test)]
    mod tests {
        use std::fs;

        use crate::output::{Context, OutputFormat};

        use super::run_at;

        #[test]
        fn status_reads_state() {
            let tmp = std::env::temp_dir().join(format!("cronus-status-{}", std::process::id()));
            let _ = fs::remove_dir_all(&tmp);
            fs::create_dir_all(&tmp).unwrap();
            fs::write(tmp.join("app.json"), "{}\n").unwrap();

            let ctx = Context::new(OutputFormat::Text);
            assert_eq!(run_at(&tmp, &ctx), 0, "status must exit 0 when initialized");

            let _ = fs::remove_dir_all(&tmp);
        }

        #[test]
        fn status_exits_1_when_uninitialized() {
            let tmp =
                std::env::temp_dir().join(format!("cronus-status-uninit-{}", std::process::id()));
            let _ = fs::remove_dir_all(&tmp);

            let ctx = Context::new(OutputFormat::Text);
            assert_eq!(
                run_at(&tmp, &ctx),
                1,
                "status must exit 1 when not initialized"
            );
        }
    }
}

// ─── workflow ─────────────────────────────────────────────────────────────────

mod workflow {
    use std::path::{Path, PathBuf};

    use nodus::{
        executor::{LogEntry, RuntimeError, Status, Value},
        validator::{Diagnostic, Severity},
        workflows::{self, TranspileMode, ValidationReport},
    };

    use crate::cli::WorkflowCommand;
    use crate::output::Context;

    pub fn dispatch(sub: WorkflowCommand, ctx: &Context) -> i32 {
        match sub {
            WorkflowCommand::Scaffold { name, out } => scaffold(name, out, ctx),
            WorkflowCommand::Validate { file } => validate(file, ctx),
            WorkflowCommand::Run { file, input } => run(file, input, ctx),
            WorkflowCommand::Transpile {
                file,
                human,
                compact: _,
            } => transpile(file, human, ctx),
        }
    }

    // ── scaffold ──────────────────────────────────────────────────────────────

    fn scaffold(name: String, out: Option<PathBuf>, ctx: &Context) -> i32 {
        let dest = out.unwrap_or_else(|| PathBuf::from(format!("{name}.nodus")));
        scaffold_at(&name, &dest, ctx)
    }

    fn scaffold_at(name: &str, dest: &Path, ctx: &Context) -> i32 {
        if dest.exists() {
            eprintln!("error: File already exists: {}", dest.display());
            return 1;
        }
        let ast = workflows::scaffold(name);
        let source = nodus::transpiler::Transpiler::to_nodus(&ast);
        match std::fs::write(dest, &source) {
            Ok(()) => {
                let abs = dest.canonicalize().unwrap_or_else(|_| dest.to_path_buf());
                if ctx.is_json() {
                    let p = json_escape(&abs.display().to_string());
                    println!("{{\"result\":\"scaffolded\",\"path\":\"{p}\"}}");
                } else {
                    println!("Scaffolded: {}", abs.display());
                }
                0
            }
            Err(e) => {
                eprintln!("error: {e}");
                1
            }
        }
    }

    // ── validate ──────────────────────────────────────────────────────────────

    fn validate(file: PathBuf, ctx: &Context) -> i32 {
        let source = match std::fs::read_to_string(&file) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error: cannot read {}: {e}", file.display());
                return 1;
            }
        };
        let filename = file_stem(&file);
        validate_at(&source, &filename, ctx)
    }

    fn validate_at(source: &str, filename: &str, ctx: &Context) -> i32 {
        let report = match workflows::validate(source, filename) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("error: parse failed: {e}");
                return 1;
            }
        };
        render_validation(&report, ctx);
        if report.has_errors { 1 } else { 0 }
    }

    fn render_validation(report: &ValidationReport, ctx: &Context) {
        if ctx.is_json() {
            let errors = diags_to_json(
                report
                    .diagnostics
                    .iter()
                    .filter(|d| d.severity == Severity::Error),
            );
            let warnings = diags_to_json(
                report
                    .diagnostics
                    .iter()
                    .filter(|d| d.severity == Severity::Warning),
            );
            let infos = diags_to_json(
                report
                    .diagnostics
                    .iter()
                    .filter(|d| d.severity == Severity::Info),
            );
            println!("{{\"errors\":[{errors}],\"warnings\":[{warnings}],\"infos\":[{infos}]}}");
        } else if report.diagnostics.is_empty() {
            println!("No errors.");
        } else {
            for d in &report.diagnostics {
                println!("[{}] {} (line {})", d.code, d.message, d.line);
            }
        }
    }

    fn diags_to_json<'a>(iter: impl Iterator<Item = &'a Diagnostic>) -> String {
        iter.map(|d| {
            format!(
                "{{\"code\":\"{}\",\"message\":\"{}\",\"line\":{}}}",
                d.code,
                json_escape(&d.message),
                d.line,
            )
        })
        .collect::<Vec<_>>()
        .join(",")
    }

    // ── run ───────────────────────────────────────────────────────────────────

    fn run(file: PathBuf, input: Option<String>, ctx: &Context) -> i32 {
        let source = match std::fs::read_to_string(&file) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error: cannot read {}: {e}", file.display());
                return 1;
            }
        };
        let filename = file_stem(&file);
        let parsed_input = match input.as_deref() {
            None => None,
            Some(s) => match json_to_value(s) {
                Ok(v) => Some(v),
                Err(e) => {
                    eprintln!("error: --input is not valid JSON: {e}");
                    return 2;
                }
            },
        };
        run_at(&source, &filename, parsed_input, ctx)
    }

    fn run_at(source: &str, filename: &str, input: Option<Value>, ctx: &Context) -> i32 {
        match workflows::run(source, filename, input) {
            Err(diags) => {
                for d in &diags {
                    eprintln!("[{}] {} (line {})", d.code, d.message, d.line);
                }
                1
            }
            Ok(result) => {
                let exit_code = match result.status {
                    Status::Ok | Status::Partial => 0,
                    Status::Failed | Status::Aborted => 1,
                    // Suspended awaiting input (a workflow ASK/CONFIRM): the run is
                    // neither complete nor failed, so signal a distinct code so
                    // callers can tell a pause apart from an error.
                    Status::Paused => 2,
                };
                if ctx.is_json() {
                    println!("{}", render_run_result_json(&result));
                } else {
                    render_run_result_text(&result);
                }
                exit_code
            }
        }
    }

    fn render_run_result_text(result: &nodus::executor::RunResult) {
        println!("Status: {:?}", result.status);
        if !result.log.is_empty() {
            println!("Log:");
            for entry in &result.log {
                println!("  {}. [{}] {:?}", entry.step, entry.command, entry.result);
            }
        }
        if !matches!(result.out, Value::Null) {
            println!("Out: {:?}", result.out);
        }
        for e in &result.errors {
            eprintln!("  [{} step {}] {}", e.code, e.step, e.reason);
        }
    }

    fn render_run_result_json(result: &nodus::executor::RunResult) -> String {
        let status_str = format!("{:?}", result.status);
        let out_jv = value_to_json(&result.out);
        let log_jv: Vec<serde_json::Value> = result.log.iter()
            .map(|e: &LogEntry| serde_json::json!({"step": e.step, "command": e.command, "result": value_to_json(&e.result)}))
            .collect();
        let errors_jv: Vec<serde_json::Value> = result.errors.iter()
            .map(|e: &RuntimeError| serde_json::json!({"code": e.code, "step": e.step, "reason": e.reason}))
            .collect();
        let flags_jv: Vec<serde_json::Value> = result
            .flags
            .iter()
            .map(|f| serde_json::Value::String(f.clone()))
            .collect();
        serde_json::json!({
            "workflow": result.workflow,
            "status": status_str,
            "out": out_jv,
            "log": log_jv,
            "errors": errors_jv,
            "flags": flags_jv,
        })
        .to_string()
    }

    fn json_to_value(s: &str) -> Result<Value, String> {
        let jv: serde_json::Value = serde_json::from_str(s).map_err(|e| e.to_string())?;
        Ok(json_value_to_nodus(jv))
    }

    fn json_value_to_nodus(jv: serde_json::Value) -> Value {
        match jv {
            serde_json::Value::Null => Value::Null,
            serde_json::Value::Bool(b) => Value::Bool(b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Value::Int(i)
                } else {
                    Value::Float(n.as_f64().unwrap_or(0.0))
                }
            }
            serde_json::Value::String(s) => Value::Text(s),
            serde_json::Value::Array(arr) => {
                Value::List(arr.into_iter().map(json_value_to_nodus).collect())
            }
            serde_json::Value::Object(map) => Value::Map(
                map.into_iter()
                    .map(|(k, v)| (k, json_value_to_nodus(v)))
                    .collect(),
            ),
        }
    }

    fn value_to_json(v: &Value) -> serde_json::Value {
        match v {
            Value::Null => serde_json::Value::Null,
            Value::Bool(b) => serde_json::Value::Bool(*b),
            Value::Int(i) => serde_json::Value::Number((*i).into()),
            Value::Float(f) => serde_json::Number::from_f64(*f)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null),
            Value::Text(s) => serde_json::Value::String(s.clone()),
            Value::List(arr) => serde_json::Value::Array(arr.iter().map(value_to_json).collect()),
            Value::Map(entries) => {
                let mut obj = serde_json::Map::new();
                for (k, v) in entries {
                    obj.insert(k.clone(), value_to_json(v));
                }
                serde_json::Value::Object(obj)
            }
        }
    }

    // ── transpile ─────────────────────────────────────────────────────────────

    fn transpile(file: PathBuf, human: bool, ctx: &Context) -> i32 {
        let source = match std::fs::read_to_string(&file) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error: cannot read {}: {e}", file.display());
                return 1;
            }
        };
        let mode = if human {
            TranspileMode::Human
        } else {
            TranspileMode::Compact
        };
        transpile_at(&source, mode, ctx)
    }

    fn transpile_at(source: &str, mode: TranspileMode, _ctx: &Context) -> i32 {
        match workflows::transpile(source, mode) {
            Ok(output) => {
                print!("{output}");
                0
            }
            Err(e) => {
                eprintln!("error: {e}");
                1
            }
        }
    }

    // ── helpers ───────────────────────────────────────────────────────────────

    fn file_stem(path: &Path) -> String {
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown.nodus")
            .to_owned()
    }

    fn json_escape(s: &str) -> String {
        s.replace('\\', "\\\\").replace('"', "\\\"")
    }

    // ── tests ─────────────────────────────────────────────────────────────────

    #[cfg(test)]
    mod tests {
        use std::fs;

        use nodus::{executor::Value, workflows::TranspileMode};

        use crate::output::{Context, OutputFormat};

        use super::{run_at, scaffold_at, transpile_at, validate_at};

        const VALID: &str = "\
§wf:ok v1.0
§runtime: { core: schema.nodus }
@in: { x }
@out: $out
@err: ESCALATE(human)
@steps:
  1. GEN($in.x) → $out
  2. LOG($out)
";
        const INVALID: &str = "\
§wf:bad v1.0
@steps:
  1. GEN($in.x) → $out
";

        fn text_ctx() -> Context {
            Context::new(OutputFormat::Text)
        }

        // scaffold

        #[test]
        fn cmd_workflow_scaffold() {
            let dir = std::env::temp_dir().join(format!("cronus-scaffold-{}", std::process::id()));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).unwrap();
            let dest = dir.join("hello_world.nodus");

            assert_eq!(scaffold_at("hello_world", &dest, &text_ctx()), 0);
            assert!(dest.is_file(), "output file must be created");

            let source = fs::read_to_string(&dest).unwrap();
            nodus::parser::Parser::parse(&source).expect("scaffolded file must parse");

            let _ = fs::remove_dir_all(&dir);
        }

        #[test]
        fn cmd_workflow_scaffold_rejects_existing() {
            let dir =
                std::env::temp_dir().join(format!("cronus-scaffold-ex-{}", std::process::id()));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).unwrap();
            let dest = dir.join("existing.nodus");
            fs::write(&dest, "content").unwrap();

            assert_eq!(scaffold_at("existing", &dest, &text_ctx()), 1);

            let _ = fs::remove_dir_all(&dir);
        }

        // validate

        #[test]
        fn cmd_workflow_validate_clean() {
            assert_eq!(validate_at(VALID, "ok.nodus", &text_ctx()), 0);
        }

        #[test]
        fn cmd_workflow_validate_error() {
            assert_eq!(validate_at(INVALID, "bad.nodus", &text_ctx()), 1);
        }

        // run

        #[test]
        fn cmd_workflow_run() {
            assert_eq!(run_at(VALID, "ok.nodus", None, &text_ctx()), 0);
        }

        #[test]
        fn cmd_workflow_run_invalid_exits_1() {
            assert_eq!(run_at(INVALID, "bad.nodus", None, &text_ctx()), 1);
        }

        #[test]
        fn cmd_workflow_run_with_input() {
            let input = Value::Map(vec![("x".to_string(), Value::Text("hello".to_string()))]);
            assert_eq!(run_at(VALID, "ok.nodus", Some(input), &text_ctx()), 0);
        }

        // transpile

        #[test]
        fn cmd_workflow_transpile_compact() {
            assert_eq!(transpile_at(VALID, TranspileMode::Compact, &text_ctx()), 0);
        }

        #[test]
        fn cmd_workflow_transpile_human() {
            assert_eq!(transpile_at(VALID, TranspileMode::Human, &text_ctx()), 0);
        }
    }
}

// ─── workspace ────────────────────────────────────────────────────────────────

mod workspace {
    use std::path::{Path, PathBuf};

    use cronus::workspace::{WorkspaceId, WorkspaceManager, WorkspaceTemplate};

    use crate::cli::WorkspaceCommand;
    use crate::output::Context;

    pub fn dispatch(sub: WorkspaceCommand, ctx: &Context) -> i32 {
        match sub {
            WorkspaceCommand::Create { id, name, path } => create(id, name, path, ctx),
            WorkspaceCommand::List => list(ctx),
            WorkspaceCommand::Switch { id } => switch(id, ctx),
            WorkspaceCommand::Delete { id } => delete(id, ctx),
            WorkspaceCommand::Check { id } => check(id, ctx),
        }
    }

    fn db_path() -> PathBuf {
        cronus::paths::Paths::os_native()
            .resolve(cronus::paths::Root::State)
            .join("workspaces.db")
    }

    fn open_manager() -> Result<WorkspaceManager, String> {
        let p = db_path();
        if let Some(parent) = p.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        WorkspaceManager::open(&p).map_err(|e| e.to_string())
    }

    fn parse_id(s: &str) -> Result<WorkspaceId, String> {
        WorkspaceId::new(s).map_err(|e| e.to_string())
    }

    fn create(id: String, name: Option<String>, path: Option<PathBuf>, ctx: &Context) -> i32 {
        create_inner(&id, name.as_deref(), path.as_deref(), ctx)
    }

    fn create_inner(id: &str, name: Option<&str>, path: Option<&Path>, ctx: &Context) -> i32 {
        let ws_id = match parse_id(id) {
            Ok(i) => i,
            Err(e) => {
                eprintln!("error: {e}");
                return 1;
            }
        };
        let ws_name = name.unwrap_or(id);
        let ws_path = path
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        let mgr = match open_manager() {
            Ok(m) => m,
            Err(e) => {
                eprintln!("error: {e}");
                return 1;
            }
        };
        match mgr.create(&ws_id, ws_name, &ws_path, WorkspaceTemplate::Default) {
            Ok(ws) => {
                if ctx.is_json() {
                    println!("{{\"result\":\"created\",\"id\":\"{}\"}}", ws.id);
                } else {
                    println!("Created workspace: {} ({})", ws.id, ws.name);
                }
                0
            }
            Err(e) => {
                eprintln!("error: {e}");
                1
            }
        }
    }

    fn list(ctx: &Context) -> i32 {
        let mgr = match open_manager() {
            Ok(m) => m,
            Err(e) => {
                eprintln!("error: {e}");
                return 1;
            }
        };
        match mgr.list() {
            Ok(workspaces) => {
                let active = mgr.get_active().ok().flatten();
                if ctx.is_json() {
                    let items: Vec<String> = workspaces
                        .iter()
                        .map(|w| {
                            let is_active = active.as_ref().map(|a| a == &w.id).unwrap_or(false);
                            format!(
                                "{{\"id\":\"{}\",\"name\":\"{}\",\"active\":{is_active}}}",
                                w.id,
                                json_escape(&w.name)
                            )
                        })
                        .collect();
                    println!("[{}]", items.join(","));
                } else if workspaces.is_empty() {
                    println!("No workspaces.");
                } else {
                    for w in &workspaces {
                        let marker = if active.as_ref().map(|a| a == &w.id).unwrap_or(false) {
                            " *"
                        } else {
                            ""
                        };
                        println!("{}{marker}  {}", w.id, w.name);
                    }
                }
                0
            }
            Err(e) => {
                eprintln!("error: {e}");
                1
            }
        }
    }

    fn switch(id: String, ctx: &Context) -> i32 {
        let ws_id = match parse_id(&id) {
            Ok(i) => i,
            Err(e) => {
                eprintln!("error: {e}");
                return 1;
            }
        };
        let mgr = match open_manager() {
            Ok(m) => m,
            Err(e) => {
                eprintln!("error: {e}");
                return 1;
            }
        };
        match mgr.set_active(&ws_id) {
            Ok(()) => {
                if ctx.is_json() {
                    println!("{{\"result\":\"switched\",\"id\":\"{ws_id}\"}}");
                } else {
                    println!("Active workspace: {ws_id}");
                }
                0
            }
            Err(e) => {
                eprintln!("error: {e}");
                1
            }
        }
    }

    fn delete(id: String, ctx: &Context) -> i32 {
        let ws_id = match parse_id(&id) {
            Ok(i) => i,
            Err(e) => {
                eprintln!("error: {e}");
                return 1;
            }
        };
        let mgr = match open_manager() {
            Ok(m) => m,
            Err(e) => {
                eprintln!("error: {e}");
                return 1;
            }
        };
        match mgr.delete(&ws_id) {
            Ok(true) => {
                if ctx.is_json() {
                    println!("{{\"result\":\"deleted\",\"id\":\"{ws_id}\"}}");
                } else {
                    println!("Deleted workspace: {ws_id}");
                }
                0
            }
            Ok(false) => {
                eprintln!("error: workspace '{ws_id}' not found");
                1
            }
            Err(e) => {
                eprintln!("error: {e}");
                1
            }
        }
    }

    fn check(id: String, ctx: &Context) -> i32 {
        let ws_id = match parse_id(&id) {
            Ok(i) => i,
            Err(e) => {
                eprintln!("error: {e}");
                return 1;
            }
        };
        let mgr = match open_manager() {
            Ok(m) => m,
            Err(e) => {
                eprintln!("error: {e}");
                return 1;
            }
        };
        match mgr.check(&ws_id) {
            Ok(status) => {
                if ctx.is_json() {
                    println!(
                        "{{\"exists\":{},\"active\":{},\"path_exists\":{}}}",
                        status.exists, status.is_active, status.path_exists
                    );
                } else {
                    println!("exists:     {}", status.exists);
                    println!("active:     {}", status.is_active);
                    println!("path_ok:    {}", status.path_exists);
                }
                0
            }
            Err(e) => {
                eprintln!("error: {e}");
                1
            }
        }
    }

    fn json_escape(s: &str) -> String {
        s.replace('\\', "\\\\").replace('"', "\\\"")
    }

    #[cfg(test)]
    mod tests {
        use std::path::Path;

        use cronus::workspace::{WorkspaceId, WorkspaceManager, WorkspaceTemplate};

        use crate::output::{Context, OutputFormat};

        fn text_ctx() -> Context {
            Context::new(OutputFormat::Text)
        }

        #[test]
        fn workspace_create_and_list() {
            let mgr = WorkspaceManager::open_in_memory().unwrap();
            let id = WorkspaceId::new("test-ws").unwrap();
            mgr.create(&id, "Test", Path::new("/tmp"), WorkspaceTemplate::Default)
                .unwrap();

            let list = mgr.list().unwrap();
            assert_eq!(list.len(), 1);
            assert_eq!(list[0].id, id);
        }

        #[test]
        fn workspace_switch_and_active() {
            let mgr = WorkspaceManager::open_in_memory().unwrap();
            let id = WorkspaceId::new("ws-a").unwrap();
            mgr.create(&id, "A", Path::new("/a"), WorkspaceTemplate::Default)
                .unwrap();
            mgr.set_active(&id).unwrap();
            assert_eq!(mgr.get_active().unwrap().as_ref(), Some(&id));
        }

        #[test]
        fn workspace_delete_removes_entry() {
            let mgr = WorkspaceManager::open_in_memory().unwrap();
            let id = WorkspaceId::new("ws-del").unwrap();
            mgr.create(&id, "Del", Path::new("/d"), WorkspaceTemplate::Empty)
                .unwrap();
            assert!(mgr.delete(&id).unwrap());
            assert!(mgr.get(&id).unwrap().is_none());
        }

        #[test]
        fn invalid_id_rejected() {
            assert!(WorkspaceId::new("CAPS").is_err());
            assert!(WorkspaceId::new("x").is_err());
        }

        #[test]
        fn check_nonexistent_workspace() {
            let mgr = WorkspaceManager::open_in_memory().unwrap();
            let id = WorkspaceId::new("ghost").unwrap();
            let status = mgr.check(&id).unwrap();
            assert!(!status.exists);
            assert!(!status.is_active);
        }

        // Suppress unused import warning for text_ctx
        #[allow(dead_code)]
        fn _use_ctx() -> Context {
            text_ctx()
        }
    }
}

// ─── memory ───────────────────────────────────────────────────────────────────

mod memory {
    use cronus::memory::{MemoryEntry, MemoryKind, MemorySource, store::MemoryStore};

    use crate::cli::MemoryCommand;
    use crate::output::Context;

    fn open_store() -> Result<MemoryStore, String> {
        MemoryStore::open_in_memory().map_err(|e| e.to_string())
    }

    pub fn dispatch(sub: MemoryCommand, ctx: &Context) -> i32 {
        match sub {
            MemoryCommand::Store { key, value } => store_entry(key, value, ctx),
            MemoryCommand::Search { query } => search_entries(query, ctx),
            MemoryCommand::Forget { id } => forget_entry(id, ctx),
        }
    }

    fn store_entry(key: String, value: String, ctx: &Context) -> i32 {
        let store = match open_store() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error: {e}");
                return 1;
            }
        };
        let entry = MemoryEntry::new(
            MemoryKind::ProjectContext,
            MemorySource::System,
            key.clone(),
            value,
        );
        match store.add(entry) {
            Ok(id) => {
                if ctx.is_json() {
                    println!("{{\"result\":\"stored\",\"id\":\"{id}\"}}");
                } else {
                    println!("Stored: {key} ({id})");
                }
                0
            }
            Err(e) => {
                eprintln!("error: {e}");
                1
            }
        }
    }

    fn search_entries(query: String, ctx: &Context) -> i32 {
        let store = match open_store() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error: {e}");
                return 1;
            }
        };
        match store.search_fts(&query, 10) {
            Ok(entries) => {
                if ctx.is_json() {
                    let items: Vec<String> = entries
                        .iter()
                        .map(|e| format!("{{\"id\":\"{}\",\"title\":\"{}\"}}", e.id, e.title))
                        .collect();
                    println!("[{}]", items.join(","));
                } else if entries.is_empty() {
                    println!("No results for '{query}'.");
                } else {
                    for e in &entries {
                        println!("{}: {}", e.id, e.title);
                    }
                }
                0
            }
            Err(e) => {
                eprintln!("error: {e}");
                1
            }
        }
    }

    fn forget_entry(id: String, ctx: &Context) -> i32 {
        let store = match open_store() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error: {e}");
                return 1;
            }
        };
        match store.delete(&id) {
            Ok(true) => {
                if ctx.is_json() {
                    println!("{{\"result\":\"deleted\",\"id\":\"{id}\"}}");
                } else {
                    println!("Deleted: {id}");
                }
                0
            }
            Ok(false) => {
                eprintln!("error: entry '{id}' not found");
                1
            }
            Err(e) => {
                eprintln!("error: {e}");
                1
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::{forget_entry, search_entries, store_entry};
        use crate::output::{Context, OutputFormat};

        fn ctx() -> Context {
            Context::new(OutputFormat::Text)
        }

        #[test]
        fn memory_store_exits_0() {
            assert_eq!(
                store_entry("fact".into(), "the sky is blue".into(), &ctx()),
                0
            );
        }

        #[test]
        fn memory_search_exits_0_empty() {
            assert_eq!(search_entries("nothing".into(), &ctx()), 0);
        }

        #[test]
        fn memory_forget_exits_1_for_unknown() {
            assert_eq!(forget_entry("nonexistent-id".into(), &ctx()), 1);
        }
    }
}

// ─── codegraph ────────────────────────────────────────────────────────────────

mod codegraph_cmd {
    use codegraph::{
        extractor::{Extractor, RegexExtractor},
        index::{fts_search, migrate, store_symbols},
    };
    use rusqlite::Connection;
    use std::path::PathBuf;

    use crate::cli::CodegraphCommand;
    use crate::output::Context;

    fn open_db() -> Result<Connection, String> {
        let conn = Connection::open_in_memory().map_err(|e| e.to_string())?;
        migrate(&conn).map_err(|e| e.to_string())?;
        Ok(conn)
    }

    pub fn dispatch(sub: CodegraphCommand, ctx: &Context) -> i32 {
        match sub {
            CodegraphCommand::Index { path } => index_path(path, ctx),
            CodegraphCommand::Search { query } => search_graph(query, ctx),
        }
    }

    fn index_path(path: PathBuf, ctx: &Context) -> i32 {
        let conn = match open_db() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("error: {e}");
                return 1;
            }
        };
        let extractor = RegexExtractor;
        let mut total = 0usize;

        if path.is_file() {
            total += index_file(&conn, &path, &extractor);
        } else if path.is_dir()
            && let Ok(entries) = std::fs::read_dir(&path)
        {
            for entry in entries.filter_map(|e| e.ok()) {
                let p = entry.path();
                if p.extension().and_then(|e| e.to_str()) == Some("rs") {
                    total += index_file(&conn, &p, &extractor);
                }
            }
        }

        if ctx.is_json() {
            println!("{{\"result\":\"indexed\",\"symbols\":{total}}}");
        } else {
            println!("Indexed {total} symbols from {}", path.display());
        }
        0
    }

    fn index_file(conn: &Connection, path: &PathBuf, extractor: &RegexExtractor) -> usize {
        let source = std::fs::read_to_string(path).unwrap_or_default();
        let syms = extractor.extract(&source);
        let n = syms.len();
        let _ = store_symbols(conn, &path.display().to_string(), &syms);
        n
    }

    fn search_graph(query: String, ctx: &Context) -> i32 {
        let conn = match open_db() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("error: {e}");
                return 1;
            }
        };
        match fts_search(&conn, &query, 10) {
            Ok(results) => {
                if ctx.is_json() {
                    let items: Vec<String> = results
                        .iter()
                        .map(|s| {
                            format!(
                                "{{\"name\":\"{}\",\"file\":\"{}\",\"line\":{}}}",
                                s.name, s.file, s.line
                            )
                        })
                        .collect();
                    println!("[{}]", items.join(","));
                } else if results.is_empty() {
                    println!("No symbols matching '{query}'.");
                } else {
                    for s in &results {
                        println!("{}:{} — {}", s.file, s.line, s.name);
                    }
                }
                0
            }
            Err(e) => {
                eprintln!("error: {e}");
                1
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::{index_path, search_graph};
        use crate::output::{Context, OutputFormat};
        use std::path::PathBuf;

        fn ctx() -> Context {
            Context::new(OutputFormat::Text)
        }

        #[test]
        fn codegraph_index_nonexistent_path_exits_0_with_zero() {
            // Non-existent dir → exits 0, prints 0 symbols
            let p = PathBuf::from("/nonexistent/path/xyz");
            assert_eq!(index_path(p, &ctx()), 0);
        }

        #[test]
        fn codegraph_search_empty_db_exits_0() {
            assert_eq!(search_graph("alpha".into(), &ctx()), 0);
        }
    }
}

// ─── role ─────────────────────────────────────────────────────────────────────

mod role {
    use cronus::roles::{PRESET_CATALOG, RoleManager};

    use crate::cli::RoleCommand;
    use crate::output::Context;

    fn state_dir() -> std::path::PathBuf {
        cronus::paths::Paths::os_native().resolve(cronus::paths::Root::State)
    }

    fn open_manager() -> RoleManager {
        let state = state_dir();
        RoleManager::new(state.clone(), state.join("employees"))
    }

    pub fn dispatch(sub: RoleCommand, ctx: &Context) -> i32 {
        match sub {
            RoleCommand::List { presets } => list(presets, ctx),
            RoleCommand::Hire { preset, name } => hire(preset, name.as_deref(), ctx),
            RoleCommand::Show { id } => show(id, ctx),
            RoleCommand::Create { id, display_name } => create(id, display_name, ctx),
            RoleCommand::Fire { id } => fire(id, ctx),
        }
    }

    fn list(presets: bool, ctx: &Context) -> i32 {
        if presets {
            if ctx.is_json() {
                let items: Vec<String> = PRESET_CATALOG
                    .iter()
                    .map(|r| format!("{{\"id\":\"{}\",\"name\":\"{}\"}}", r.id, r.name))
                    .collect();
                println!("[{}]", items.join(","));
            } else {
                for r in PRESET_CATALOG {
                    println!("{}: {}", r.id, r.name);
                }
            }
            return 0;
        }
        let mgr = open_manager();
        match mgr.list_hired() {
            Ok(instances) if instances.is_empty() => {
                println!("No roles hired.");
                0
            }
            Ok(instances) => {
                for inst in &instances {
                    println!("{}: {}", inst.id, inst.display_name);
                }
                0
            }
            Err(_) => {
                println!("No roles hired.");
                0
            }
        }
    }

    fn hire(preset: String, name: Option<&str>, ctx: &Context) -> i32 {
        let mgr = open_manager();
        match mgr.hire(&preset, name) {
            Ok(inst) => {
                if ctx.is_json() {
                    println!("{{\"result\":\"hired\",\"id\":\"{}\"}}", inst.id);
                } else {
                    println!("Hired: {} ({})", inst.id, inst.display_name);
                }
                0
            }
            Err(e) => {
                eprintln!("error: {e}");
                1
            }
        }
    }

    fn show(id: String, _ctx: &Context) -> i32 {
        let mgr = open_manager();
        match mgr.get(&id) {
            Ok(Some(inst)) => {
                println!("id:   {}", inst.id);
                println!("name: {}", inst.display_name);
                0
            }
            Ok(None) => {
                eprintln!("error: role '{id}' not found");
                1
            }
            Err(e) => {
                eprintln!("error: {e}");
                1
            }
        }
    }

    fn create(id: String, display_name: String, ctx: &Context) -> i32 {
        let mgr = open_manager();
        match mgr.create_custom(&id, &display_name) {
            Ok(inst) => {
                if ctx.is_json() {
                    println!("{{\"result\":\"created\",\"id\":\"{}\"}}", inst.id);
                } else {
                    println!("Created: {} ({})", inst.id, inst.display_name);
                }
                0
            }
            Err(e) => {
                eprintln!("error: {e}");
                1
            }
        }
    }

    fn fire(id: String, ctx: &Context) -> i32 {
        let mgr = open_manager();
        match mgr.fire(&id) {
            Ok(()) => {
                if ctx.is_json() {
                    println!("{{\"result\":\"fired\",\"id\":\"{id}\"}}");
                } else {
                    println!("Fired: {id}");
                }
                0
            }
            Err(e) => {
                eprintln!("error: {e}");
                1
            }
        }
    }
}

// ─── board ────────────────────────────────────────────────────────────────────

mod board {
    use std::path::PathBuf;

    use cronus::kanban::{Board, CardState};
    use cronus::tool_security::now_ms;

    use crate::cli::BoardCommand;
    use crate::output::Context;

    fn board_path() -> PathBuf {
        cronus::paths::Paths::os_native()
            .resolve(cronus::paths::Root::State)
            .join("kanban")
    }

    fn open_board() -> Board {
        Board::new(board_path())
    }

    pub fn dispatch(sub: BoardCommand, ctx: &Context) -> i32 {
        match sub {
            BoardCommand::List => list(ctx),
            BoardCommand::Show { id } => show(id, ctx),
            BoardCommand::Add { id, task_ref } => add(id, task_ref, ctx),
            BoardCommand::Move { id, state, actor } => move_card(id, state, actor, ctx),
            BoardCommand::Block { id, reason } => block(id, reason, ctx),
            BoardCommand::Done { id, actor } => done(id, actor, ctx),
            BoardCommand::Archive => archive(ctx),
        }
    }

    fn list(ctx: &Context) -> i32 {
        let board = open_board();
        match board.list_cards() {
            Ok(cards) if cards.is_empty() => {
                if ctx.is_json() {
                    println!("[]");
                } else {
                    println!("No cards.");
                }
                0
            }
            Ok(cards) => {
                if ctx.is_json() {
                    let items: Vec<String> = cards
                        .iter()
                        .map(|c| {
                            format!("{{\"id\":\"{}\",\"state\":\"{}\"}}", c.id, c.state.as_str())
                        })
                        .collect();
                    println!("[{}]", items.join(","));
                } else {
                    for c in &cards {
                        println!("{}: {}", c.id, c.state.as_str());
                    }
                }
                0
            }
            Err(_) => {
                println!("No cards.");
                0
            }
        }
    }

    fn show(id: String, _ctx: &Context) -> i32 {
        let board = open_board();
        match board.get_card(&id) {
            Ok(Some(c)) => {
                println!("id:    {}", c.id);
                println!("state: {}", c.state.as_str());
                0
            }
            Ok(None) => {
                eprintln!("error: card '{id}' not found");
                1
            }
            Err(e) => {
                eprintln!("error: {e}");
                1
            }
        }
    }

    fn add(id: String, task_ref: String, ctx: &Context) -> i32 {
        let board = open_board();
        match board.add_card(&id, &task_ref, now_ms()) {
            Ok(card) => {
                if ctx.is_json() {
                    println!("{{\"result\":\"added\",\"id\":\"{}\"}}", card.id);
                } else {
                    println!("Added: {}", card.id);
                }
                0
            }
            Err(e) => {
                eprintln!("error: {e}");
                1
            }
        }
    }

    fn move_card(id: String, state: String, actor: String, ctx: &Context) -> i32 {
        let board = open_board();
        let to = match CardState::parse(&state) {
            Some(s) => s,
            None => {
                eprintln!("error: unknown state '{state}'");
                return 1;
            }
        };
        match board.move_card(&id, to, &actor, None, now_ms()) {
            Ok(card) => {
                if ctx.is_json() {
                    println!(
                        "{{\"result\":\"moved\",\"id\":\"{}\",\"state\":\"{}\"}}",
                        card.id,
                        card.state.as_str()
                    );
                } else {
                    println!("Moved {} → {}", card.id, card.state.as_str());
                }
                0
            }
            Err(e) => {
                eprintln!("error: {e}");
                1
            }
        }
    }

    fn block(id: String, reason: String, ctx: &Context) -> i32 {
        let board = open_board();
        match board.move_card(
            &id,
            CardState::Blocked,
            "cli",
            Some(reason.clone()),
            now_ms(),
        ) {
            Ok(card) => {
                if ctx.is_json() {
                    println!("{{\"result\":\"blocked\",\"id\":\"{}\"}}", card.id);
                } else {
                    println!("Blocked: {}", card.id);
                }
                0
            }
            Err(e) => {
                eprintln!("error: {e}");
                1
            }
        }
    }

    fn done(id: String, actor: String, ctx: &Context) -> i32 {
        let board = open_board();
        match board.move_card(&id, CardState::Done, &actor, None, now_ms()) {
            Ok(card) => {
                if ctx.is_json() {
                    println!("{{\"result\":\"done\",\"id\":\"{}\"}}", card.id);
                } else {
                    println!("Done: {}", card.id);
                }
                0
            }
            Err(e) => {
                eprintln!("error: {e}");
                1
            }
        }
    }

    fn archive(ctx: &Context) -> i32 {
        let board = open_board();
        match board.archive_done_cards() {
            Ok(n) => {
                if ctx.is_json() {
                    println!("{{\"result\":\"archived\",\"count\":{n}}}");
                } else {
                    println!("Archived {n} card(s).");
                }
                0
            }
            Err(e) => {
                eprintln!("error: {e}");
                1
            }
        }
    }
}

// ─── schedule ─────────────────────────────────────────────────────────────────

mod schedule {
    use std::path::PathBuf;

    use cronus::scheduler::{RecurrencePreset, Schedule, ScheduleAction, Scheduler};

    use crate::cli::ScheduleCommand;
    use crate::output::Context;

    fn sched_dir() -> PathBuf {
        cronus::paths::Paths::os_native()
            .resolve(cronus::paths::Root::State)
            .join("schedules")
    }

    fn open_scheduler() -> Result<Scheduler, String> {
        Scheduler::new(sched_dir()).map_err(|e| e.to_string())
    }

    pub fn dispatch(sub: ScheduleCommand, ctx: &Context) -> i32 {
        match sub {
            ScheduleCommand::List => list(ctx),
            ScheduleCommand::Add { id, name, preset } => add(id, name, preset, ctx),
            ScheduleCommand::Delete { id } => delete(id, ctx),
            ScheduleCommand::Run { id, prompt } => run(id, prompt, ctx),
        }
    }

    fn list(ctx: &Context) -> i32 {
        let sched = match open_scheduler() {
            Ok(s) => s,
            Err(_) => {
                println!("No schedules.");
                return 0;
            }
        };
        match sched.list() {
            Ok(items) if items.is_empty() => {
                if ctx.is_json() {
                    println!("[]");
                } else {
                    println!("No schedules.");
                }
                0
            }
            Ok(items) => {
                for s in &items {
                    println!("{}: {} ({})", s.id, s.name, s.kind.as_str());
                }
                0
            }
            Err(_) => {
                println!("No schedules.");
                0
            }
        }
    }

    fn add(id: String, name: String, preset_str: String, ctx: &Context) -> i32 {
        let preset = match preset_str.as_str() {
            "weekdays" => RecurrencePreset::Weekdays,
            "weekends" => RecurrencePreset::Weekends,
            _ => RecurrencePreset::Daily,
        };
        let sched = match open_scheduler() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error: {e}");
                return 1;
            }
        };
        let s = Schedule::recurring(&id, &name, &preset, ScheduleAction::Routine);
        match sched.add(&s) {
            Ok(()) => {
                if ctx.is_json() {
                    println!("{{\"result\":\"added\",\"id\":\"{id}\"}}");
                } else {
                    println!("Added: {id}");
                }
                0
            }
            Err(e) => {
                eprintln!("error: {e}");
                1
            }
        }
    }

    fn delete(id: String, ctx: &Context) -> i32 {
        let sched = match open_scheduler() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error: {e}");
                return 1;
            }
        };
        match sched.delete(&id) {
            Ok(()) => {
                if ctx.is_json() {
                    println!("{{\"result\":\"deleted\",\"id\":\"{id}\"}}");
                } else {
                    println!("Deleted: {id}");
                }
                0
            }
            Err(e) => {
                eprintln!("error: {e}");
                1
            }
        }
    }

    fn run(id: String, prompt: String, ctx: &Context) -> i32 {
        let sched = match open_scheduler() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error: {e}");
                return 1;
            }
        };
        match sched.fire(&id, &prompt) {
            Ok(session) => {
                if ctx.is_json() {
                    println!(
                        "{{\"result\":\"fired\",\"session_key\":\"{}\"}}",
                        session.session_key
                    );
                } else {
                    println!("Fired: {}", session.session_key);
                }
                0
            }
            Err(e) => {
                eprintln!("error: {e}");
                1
            }
        }
    }
}

// ─── budget ───────────────────────────────────────────────────────────────────

mod budget_cmd {
    use cronus::budget::{BudgetEngine, BudgetPeriod, BudgetPolicy};

    use crate::cli::BudgetCommand;
    use crate::output::Context;

    pub fn dispatch(sub: BudgetCommand, ctx: &Context) -> i32 {
        match sub {
            BudgetCommand::Show => show(ctx),
            BudgetCommand::Set { limit } => set(limit, ctx),
            BudgetCommand::Reset => reset(ctx),
        }
    }

    fn show(ctx: &Context) -> i32 {
        let engine = BudgetEngine::new();
        let spent = engine.spent_for("default");
        if ctx.is_json() {
            println!("{{\"spent\":{spent:.4}}}");
        } else {
            println!("spent: ${spent:.4}");
        }
        0
    }

    fn set(limit: f64, ctx: &Context) -> i32 {
        let mut engine = BudgetEngine::new();
        let policy = BudgetPolicy::workspace("default", limit, BudgetPeriod::Monthly);
        engine.add_policy(policy);
        if ctx.is_json() {
            println!("{{\"result\":\"set\",\"limit\":{limit:.4}}}");
        } else {
            println!("Budget limit set: ${limit:.4}/month");
        }
        0
    }

    fn reset(ctx: &Context) -> i32 {
        let mut engine = BudgetEngine::new();
        engine.reset();
        if ctx.is_json() {
            println!("{{\"result\":\"reset\"}}");
        } else {
            println!("Budget counters reset.");
        }
        0
    }
}

// ─── exec ─────────────────────────────────────────────────────────────────────

mod exec {
    use std::path::PathBuf;

    use cronus::exec_workspace::ExecWorkspaceManager;
    use cronus::tool_security::now_ms;

    use crate::cli::ExecCommand;
    use crate::output::Context;

    fn base_dir() -> PathBuf {
        cronus::paths::Paths::os_native()
            .resolve(cronus::paths::Root::State)
            .join("exec-workspaces")
    }

    pub fn dispatch(sub: ExecCommand, ctx: &Context) -> i32 {
        match sub {
            ExecCommand::List => list(ctx),
            ExecCommand::Create { ws_id, card_id } => create(ws_id, card_id, ctx),
            ExecCommand::Finalize { id } => finalize(id, ctx),
            ExecCommand::Discard { id } => discard(id, ctx),
        }
    }

    fn list(ctx: &Context) -> i32 {
        let mgr = ExecWorkspaceManager::new();
        let items = mgr.list();
        if items.is_empty() {
            if ctx.is_json() {
                println!("[]");
            } else {
                println!("No exec workspaces.");
            }
        } else {
            for w in items {
                println!("{}: {}", w.id, w.state.as_str());
            }
        }
        0
    }

    fn create(ws_id: String, card_id: String, ctx: &Context) -> i32 {
        let mut mgr = ExecWorkspaceManager::new();
        let dir = base_dir();
        let _ = std::fs::create_dir_all(&dir);
        match mgr.create(&ws_id, &card_id, &dir, now_ms()) {
            Ok(w) => {
                if ctx.is_json() {
                    println!("{{\"result\":\"created\",\"id\":\"{}\"}}", w.id);
                } else {
                    println!("Created: {}", w.id);
                }
                0
            }
            Err(e) => {
                eprintln!("error: {e}");
                1
            }
        }
    }

    fn finalize(id: String, ctx: &Context) -> i32 {
        let mut mgr = ExecWorkspaceManager::new();
        match mgr.finalize(&id, true, now_ms()) {
            Ok(()) => {
                if ctx.is_json() {
                    println!("{{\"result\":\"finalized\",\"id\":\"{id}\"}}");
                } else {
                    println!("Finalized: {id}");
                }
                0
            }
            Err(e) => {
                eprintln!("error: {e}");
                1
            }
        }
    }

    fn discard(id: String, ctx: &Context) -> i32 {
        let mut mgr = ExecWorkspaceManager::new();
        match mgr.discard(&id) {
            Ok(()) => {
                if ctx.is_json() {
                    println!("{{\"result\":\"discarded\",\"id\":\"{id}\"}}");
                } else {
                    println!("Discarded: {id}");
                }
                0
            }
            Err(e) => {
                eprintln!("error: {e}");
                1
            }
        }
    }
}

// ─── check ────────────────────────────────────────────────────────────────────

mod check {
    use std::path::PathBuf;

    use cronus::quality::{GateResultStore, detect_language};

    use crate::cli::CheckCommand;
    use crate::output::Context;

    pub fn dispatch(sub: CheckCommand, ctx: &Context) -> i32 {
        match sub {
            CheckCommand::Run { card_id, path } => run(card_id, path, ctx),
            CheckCommand::Show { card_id } => show(card_id, ctx),
            CheckCommand::History { card_id } => history(card_id, ctx),
        }
    }

    fn run(card_id: String, path: Option<PathBuf>, ctx: &Context) -> i32 {
        let root =
            path.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        let lang = detect_language(&root);
        if ctx.is_json() {
            println!(
                "{{\"card\":\"{card_id}\",\"language\":\"{}\"}}",
                lang.as_str()
            );
        } else {
            println!("card:     {card_id}");
            println!("language: {}", lang.as_str());
            println!("(gate runner seam — no tools invoked yet)");
        }
        0
    }

    fn show(card_id: String, _ctx: &Context) -> i32 {
        let store = GateResultStore::new();
        let results = store.results_for(&card_id);
        if results.is_empty() {
            println!("No gate results for '{card_id}'.");
        } else {
            for r in results {
                println!("{}: {}", r.gate.as_str(), r.status.as_str());
            }
        }
        0
    }

    fn history(card_id: String, _ctx: &Context) -> i32 {
        let store = GateResultStore::new();
        let results = store.results_for(&card_id);
        println!("{} gate result(s) for '{card_id}'.", results.len());
        0
    }
}

// ─── ext ──────────────────────────────────────────────────────────────────────

mod ext {
    use std::path::PathBuf;

    use cronus::extensions::{
        ExtensionKind, ExtensionManifest, ExtensionPermissions, ExtensionRegistry, ExtensionSource,
        ExtensionState,
    };

    use crate::cli::ExtCommand;
    use crate::output::Context;

    pub fn dispatch(sub: ExtCommand, ctx: &Context) -> i32 {
        match sub {
            ExtCommand::List => list(ctx),
            ExtCommand::Add { path } => add(path, ctx),
            ExtCommand::Remove { id } => remove(id, ctx),
            ExtCommand::Scan { path } => scan(path, ctx),
            ExtCommand::Activate { id } => activate(id, ctx),
            ExtCommand::Deactivate { id } => deactivate(id, ctx),
        }
    }

    fn list(ctx: &Context) -> i32 {
        let registry = ExtensionRegistry::new();
        let all = registry.list();
        if all.is_empty() {
            if ctx.is_json() {
                println!("[]");
            } else {
                println!("No extensions registered.");
            }
        } else {
            for (m, _s) in &all {
                println!("{}: {} ({})", m.id, m.name, m.version);
            }
        }
        0
    }

    fn add(path: PathBuf, ctx: &Context) -> i32 {
        let json = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error: {e}");
                return 1;
            }
        };
        let manifest = match parse_manifest(&json) {
            Some(m) => m,
            None => {
                eprintln!("error: invalid manifest (missing id, name, or version)");
                return 1;
            }
        };
        let mut registry = ExtensionRegistry::new();
        match registry.register(manifest) {
            Ok(()) => {
                if ctx.is_json() {
                    println!("{{\"result\":\"registered\"}}");
                } else {
                    println!("Extension registered.");
                }
                0
            }
            Err(e) => {
                eprintln!("error: {e}");
                1
            }
        }
    }

    fn remove(_id: String, _ctx: &Context) -> i32 {
        eprintln!("error: extension removal not yet supported");
        1
    }

    fn scan(path: PathBuf, _ctx: &Context) -> i32 {
        let content = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error: {e}");
                return 1;
            }
        };
        let fname = path.file_name().and_then(|n| n.to_str()).unwrap_or("ext");
        let result = cronus::tool_security::SkillScanner::scan_content(&content, fname);
        println!("safe: {}", result.is_safe);
        println!("risk_score: {}", result.risk_score);
        println!("findings: {}", result.findings.len());
        0
    }

    fn activate(id: String, ctx: &Context) -> i32 {
        let mut registry = ExtensionRegistry::new();
        // State machine: Discovered → Permitted → Active. Try the intermediate step first.
        let _ = registry.transition(&id, ExtensionState::Permitted);
        match registry.transition(&id, ExtensionState::Active) {
            Ok(()) => {
                if ctx.is_json() {
                    println!("{{\"result\":\"activated\",\"id\":\"{id}\"}}");
                } else {
                    println!("Activated: {id}");
                }
                0
            }
            Err(e) => {
                eprintln!("error: {e}");
                1
            }
        }
    }

    fn deactivate(id: String, ctx: &Context) -> i32 {
        let mut registry = ExtensionRegistry::new();
        match registry.transition(&id, ExtensionState::Inactive) {
            Ok(()) => {
                if ctx.is_json() {
                    println!("{{\"result\":\"deactivated\",\"id\":\"{id}\"}}");
                } else {
                    println!("Deactivated: {id}");
                }
                0
            }
            Err(e) => {
                eprintln!("error: {e}");
                1
            }
        }
    }

    fn parse_manifest(json: &str) -> Option<ExtensionManifest> {
        let id = extract_str(json, "id")?;
        let name = extract_str(json, "name").unwrap_or_else(|| id.clone());
        let version = extract_str(json, "version").unwrap_or_else(|| "0.0.0".to_string());
        Some(ExtensionManifest {
            id,
            kind: ExtensionKind::Skill,
            name,
            version,
            source: ExtensionSource::Custom,
            capabilities: Vec::new(),
            permissions: ExtensionPermissions::default(),
        })
    }

    fn extract_str(json: &str, key: &str) -> Option<String> {
        let pattern = format!("\"{}\":\"", key);
        let start = json.find(&pattern)? + pattern.len();
        let rest = &json[start..];
        let end = rest.find('"')?;
        Some(rest[..end].to_string())
    }
}

// ─── learn ────────────────────────────────────────────────────────────────────

mod learn {
    use cronus::learning::LearningApprovalGate;

    use crate::cli::LearnCommand;
    use crate::output::Context;

    pub fn dispatch(sub: LearnCommand, ctx: &Context) -> i32 {
        match sub {
            LearnCommand::List => list(ctx),
            LearnCommand::Approve { id } => approve(id, ctx),
            LearnCommand::Reject { id } => reject(id, ctx),
        }
    }

    fn list(ctx: &Context) -> i32 {
        let gate = LearningApprovalGate::new();
        let pending = gate.list_pending();
        if pending.is_empty() {
            if ctx.is_json() {
                println!("[]");
            } else {
                println!("No pending skill proposals.");
            }
        } else {
            for s in &pending {
                println!("{}: {} (confidence: {:.2})", s.id, s.trigger, s.confidence);
            }
        }
        0
    }

    fn approve(id: String, ctx: &Context) -> i32 {
        let mut gate = LearningApprovalGate::new();
        if gate.approve(&id) {
            if ctx.is_json() {
                println!("{{\"result\":\"approved\",\"id\":\"{id}\"}}");
            } else {
                println!("Approved: {id}");
            }
            0
        } else {
            eprintln!("error: skill proposal '{id}' not found");
            1
        }
    }

    fn reject(id: String, ctx: &Context) -> i32 {
        let mut gate = LearningApprovalGate::new();
        if gate.reject(&id) {
            if ctx.is_json() {
                println!("{{\"result\":\"rejected\",\"id\":\"{id}\"}}");
            } else {
                println!("Rejected: {id}");
            }
            0
        } else {
            eprintln!("error: skill proposal '{id}' not found");
            1
        }
    }
}

// ─── registry ─────────────────────────────────────────────────────────────────

mod registry {
    use cronus::agent_registry::AgentRegistry;

    use crate::cli::RegistryCommand;
    use crate::output::Context;

    pub fn dispatch(sub: RegistryCommand, ctx: &Context) -> i32 {
        match sub {
            RegistryCommand::List => list(ctx),
            RegistryCommand::Show { name } => show(name, ctx),
            RegistryCommand::Create { name, description } => create(name, description, ctx),
            RegistryCommand::Disable { name } => disable(name, ctx),
            RegistryCommand::Enable { name } => enable(name, ctx),
        }
    }

    fn list(ctx: &Context) -> i32 {
        let registry = AgentRegistry::new();
        let agents = registry.list_active();
        if ctx.is_json() {
            let items: Vec<String> = agents
                .iter()
                .map(|a| format!("{{\"name\":\"{}\"}}", a.name))
                .collect();
            println!("[{}]", items.join(","));
        } else {
            for a in &agents {
                println!("{}: {}", a.name, a.description.as_deref().unwrap_or(""));
            }
        }
        0
    }

    fn show(name: String, _ctx: &Context) -> i32 {
        let registry = AgentRegistry::new();
        match registry.resolve(&name) {
            Ok(a) => {
                println!("name:        {}", a.name);
                println!("description: {}", a.description.as_deref().unwrap_or(""));
                println!("mode:        {}", a.mode.as_str());
                0
            }
            Err(e) => {
                eprintln!("error: {e}");
                1
            }
        }
    }

    fn create(name: String, description: String, ctx: &Context) -> i32 {
        let mut registry = AgentRegistry::new();
        let def = AgentRegistry::generate_from_description(&name, &description);
        let def_name = def.name.clone();
        registry.register_custom(def);
        if ctx.is_json() {
            println!("{{\"result\":\"created\",\"name\":\"{def_name}\"}}");
        } else {
            println!("Created: {def_name}");
        }
        0
    }

    fn disable(name: String, ctx: &Context) -> i32 {
        let mut registry = AgentRegistry::new();
        registry.apply_user_config(&name, true, None);
        if ctx.is_json() {
            println!("{{\"result\":\"disabled\",\"name\":\"{name}\"}}");
        } else {
            println!("Disabled: {name}");
        }
        0
    }

    fn enable(name: String, ctx: &Context) -> i32 {
        let mut registry = AgentRegistry::new();
        registry.apply_user_config(&name, false, None);
        if ctx.is_json() {
            println!("{{\"result\":\"enabled\",\"name\":\"{name}\"}}");
        } else {
            println!("Enabled: {name}");
        }
        0
    }
}

// ─── agent ────────────────────────────────────────────────────────────────────

mod agent {
    use cronus::constitution::{IDENTITY_FILES, identity_paths};

    use crate::cli::AgentCommand;
    use crate::output::Context;

    pub fn dispatch(sub: AgentCommand, ctx: &Context) -> i32 {
        match sub {
            AgentCommand::Constitution => constitution(ctx),
            AgentCommand::Status => status(ctx),
        }
    }

    fn constitution(ctx: &Context) -> i32 {
        let workspace = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let paths = identity_paths(&workspace);
        if ctx.is_json() {
            let items: Vec<String> = IDENTITY_FILES
                .iter()
                .zip(paths.iter())
                .map(|(name, path)| {
                    let exists = path.exists();
                    let p = path
                        .display()
                        .to_string()
                        .replace('\\', "\\\\")
                        .replace('"', "\\\"");
                    format!("{{\"file\":\"{name}\",\"path\":\"{p}\",\"exists\":{exists}}}")
                })
                .collect();
            println!("[{}]", items.join(","));
        } else {
            for (name, path) in IDENTITY_FILES.iter().zip(paths.iter()) {
                let exists = if path.exists() { "✓" } else { "✗" };
                println!("{exists} {name}: {}", path.display());
            }
        }
        0
    }

    fn status(_ctx: &Context) -> i32 {
        println!("agent: no active session");
        0
    }

    #[cfg(test)]
    mod tests {
        use super::{constitution, status};
        use crate::output::{Context, OutputFormat};

        fn ctx() -> Context {
            Context::new(OutputFormat::Text)
        }

        #[test]
        fn agent_constitution_exits_0() {
            assert_eq!(constitution(&ctx()), 0);
        }

        #[test]
        fn agent_status_exits_0() {
            assert_eq!(status(&ctx()), 0);
        }
    }
}

// ─── goal ─────────────────────────────────────────────────────────────────────

mod goal {
    use crate::cli::GoalCommand;
    use crate::output::Context;

    pub fn dispatch(sub: GoalCommand, ctx: &Context) -> i32 {
        match sub {
            GoalCommand::Start {
                goal,
                budget,
                max_iter,
            } => start(&goal, budget, max_iter, ctx),
            GoalCommand::Stop { id } => stop(&id, ctx),
            GoalCommand::Status { id } => status(&id, ctx),
        }
    }

    fn start(goal: &str, budget: f64, max_iter: u32, ctx: &Context) -> i32 {
        if ctx.is_json() {
            println!(
                "{{\"status\":\"not_implemented\",\"command\":\"goal start\",\
                 \"goal\":\"{goal}\",\"budget\":{budget},\"max_iter\":{max_iter}}}"
            );
        } else {
            println!(
                "goal start: not yet implemented (goal={goal}, budget={budget}, max_iter={max_iter})"
            );
        }
        0
    }

    fn stop(id: &str, ctx: &Context) -> i32 {
        if ctx.is_json() {
            println!(
                "{{\"status\":\"not_implemented\",\"command\":\"goal stop\",\"id\":\"{id}\"}}"
            );
        } else {
            println!("goal stop: not yet implemented (id={id})");
        }
        0
    }

    fn status(id: &str, ctx: &Context) -> i32 {
        if ctx.is_json() {
            println!(
                "{{\"status\":\"not_implemented\",\"command\":\"goal status\",\"id\":\"{id}\"}}"
            );
        } else {
            println!("goal status: not yet implemented (id={id})");
        }
        0
    }
}

// ─── trigger ──────────────────────────────────────────────────────────────────

mod trigger {
    use crate::cli::TriggerCommand;
    use crate::output::Context;

    pub fn dispatch(sub: TriggerCommand, ctx: &Context) -> i32 {
        match sub {
            TriggerCommand::List => list(ctx),
            TriggerCommand::History { id } => history(&id, ctx),
            TriggerCommand::Replay { id } => replay(&id, ctx),
        }
    }

    fn list(ctx: &Context) -> i32 {
        if ctx.is_json() {
            println!("{{\"status\":\"not_implemented\",\"command\":\"trigger list\"}}");
        } else {
            println!("trigger list: not yet implemented");
        }
        0
    }

    fn history(id: &str, ctx: &Context) -> i32 {
        if ctx.is_json() {
            println!(
                "{{\"status\":\"not_implemented\",\"command\":\"trigger history\",\"id\":\"{id}\"}}"
            );
        } else {
            println!("trigger history: not yet implemented (id={id})");
        }
        0
    }

    fn replay(id: &str, ctx: &Context) -> i32 {
        if ctx.is_json() {
            println!(
                "{{\"status\":\"not_implemented\",\"command\":\"trigger replay\",\"id\":\"{id}\"}}"
            );
        } else {
            println!("trigger replay: not yet implemented (id={id})");
        }
        0
    }
}

// ─── mission ──────────────────────────────────────────────────────────────────

mod mission {
    use crate::cli::MissionCommand;
    use crate::output::Context;

    pub fn dispatch(sub: MissionCommand, ctx: &Context) -> i32 {
        match sub {
            MissionCommand::Start { task, mode } => start(&task, &mode, ctx),
            MissionCommand::Confirm { id } => confirm(&id, ctx),
            MissionCommand::Status { id } => status(&id, ctx),
            MissionCommand::List => list(ctx),
            MissionCommand::Resume { id } => resume(&id, ctx),
            MissionCommand::Abort { id } => abort(&id, ctx),
        }
    }

    fn start(task: &str, mode: &str, ctx: &Context) -> i32 {
        if ctx.is_json() {
            println!(
                "{{\"status\":\"not_implemented\",\"command\":\"mission start\",\
                 \"task\":\"{task}\",\"mode\":\"{mode}\"}}"
            );
        } else {
            println!("mission start: not yet implemented (task={task}, mode={mode})");
        }
        0
    }

    fn confirm(id: &str, ctx: &Context) -> i32 {
        if ctx.is_json() {
            println!(
                "{{\"status\":\"not_implemented\",\"command\":\"mission confirm\",\"id\":\"{id}\"}}"
            );
        } else {
            println!("mission confirm: not yet implemented (id={id})");
        }
        0
    }

    fn status(id: &str, ctx: &Context) -> i32 {
        if ctx.is_json() {
            println!(
                "{{\"status\":\"not_implemented\",\"command\":\"mission status\",\"id\":\"{id}\"}}"
            );
        } else {
            println!("mission status: not yet implemented (id={id})");
        }
        0
    }

    fn list(ctx: &Context) -> i32 {
        if ctx.is_json() {
            println!("{{\"status\":\"not_implemented\",\"command\":\"mission list\"}}");
        } else {
            println!("mission list: not yet implemented");
        }
        0
    }

    fn resume(id: &str, ctx: &Context) -> i32 {
        if ctx.is_json() {
            println!(
                "{{\"status\":\"not_implemented\",\"command\":\"mission resume\",\"id\":\"{id}\"}}"
            );
        } else {
            println!("mission resume: not yet implemented (id={id})");
        }
        0
    }

    fn abort(id: &str, ctx: &Context) -> i32 {
        if ctx.is_json() {
            println!(
                "{{\"status\":\"not_implemented\",\"command\":\"mission abort\",\"id\":\"{id}\"}}"
            );
        } else {
            println!("mission abort: not yet implemented (id={id})");
        }
        0
    }
}

// ─── research ─────────────────────────────────────────────────────────────────

mod research {
    use crate::cli::ResearchCommand;
    use crate::output::Context;

    pub fn dispatch(sub: ResearchCommand, ctx: &Context) -> i32 {
        match sub {
            ResearchCommand::Start {
                question,
                max_rounds,
            } => start(&question, max_rounds, ctx),
            ResearchCommand::Status { id } => status(&id, ctx),
            ResearchCommand::Report { id } => report(&id, ctx),
            ResearchCommand::List => list(ctx),
            ResearchCommand::Cancel { id } => cancel(&id, ctx),
        }
    }

    fn start(question: &str, max_rounds: u8, ctx: &Context) -> i32 {
        if ctx.is_json() {
            println!(
                "{{\"status\":\"not_implemented\",\"command\":\"research start\",\
                 \"question\":\"{question}\",\"max_rounds\":{max_rounds}}}"
            );
        } else {
            println!(
                "research start: not yet implemented (question={question}, max_rounds={max_rounds})"
            );
        }
        0
    }

    fn status(id: &str, ctx: &Context) -> i32 {
        if ctx.is_json() {
            println!(
                "{{\"status\":\"not_implemented\",\"command\":\"research status\",\"id\":\"{id}\"}}"
            );
        } else {
            println!("research status: not yet implemented (id={id})");
        }
        0
    }

    fn report(id: &str, ctx: &Context) -> i32 {
        if ctx.is_json() {
            println!(
                "{{\"status\":\"not_implemented\",\"command\":\"research report\",\"id\":\"{id}\"}}"
            );
        } else {
            println!("research report: not yet implemented (id={id})");
        }
        0
    }

    fn list(ctx: &Context) -> i32 {
        if ctx.is_json() {
            println!("{{\"status\":\"not_implemented\",\"command\":\"research list\"}}");
        } else {
            println!("research list: not yet implemented");
        }
        0
    }

    fn cancel(id: &str, ctx: &Context) -> i32 {
        if ctx.is_json() {
            println!(
                "{{\"status\":\"not_implemented\",\"command\":\"research cancel\",\"id\":\"{id}\"}}"
            );
        } else {
            println!("research cancel: not yet implemented (id={id})");
        }
        0
    }
}

// ─── change ───────────────────────────────────────────────────────────────────

mod change {
    use crate::cli::ChangeCommand;
    use crate::output::Context;

    pub fn dispatch(sub: ChangeCommand, ctx: &Context) -> i32 {
        match sub {
            ChangeCommand::Graph => graph(ctx),
            ChangeCommand::Next => next(ctx),
            ChangeCommand::Split { id } => split(&id, ctx),
            ChangeCommand::Status { id } => status(&id, ctx),
        }
    }

    fn graph(ctx: &Context) -> i32 {
        if ctx.is_json() {
            println!("{{\"status\":\"not_implemented\",\"command\":\"change graph\"}}");
        } else {
            println!("change graph: not yet implemented");
        }
        0
    }

    fn next(ctx: &Context) -> i32 {
        if ctx.is_json() {
            println!("{{\"status\":\"not_implemented\",\"command\":\"change next\"}}");
        } else {
            println!("change next: not yet implemented");
        }
        0
    }

    fn split(id: &str, ctx: &Context) -> i32 {
        if ctx.is_json() {
            println!(
                "{{\"status\":\"not_implemented\",\"command\":\"change split\",\"id\":\"{id}\"}}"
            );
        } else {
            println!("change split: not yet implemented (id={id})");
        }
        0
    }

    fn status(id: &str, ctx: &Context) -> i32 {
        if ctx.is_json() {
            println!(
                "{{\"status\":\"not_implemented\",\"command\":\"change status\",\"id\":\"{id}\"}}"
            );
        } else {
            println!("change status: not yet implemented (id={id})");
        }
        0
    }
}
