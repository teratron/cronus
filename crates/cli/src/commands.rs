use crate::cli::Command;
use crate::output::Context;

pub fn dispatch(command: Command, ctx: &Context) -> i32 {
    match command {
        Command::Init { path } => init::run(path, ctx),
        Command::Status => status::run(ctx),
        Command::Workflow { sub } => workflow::dispatch(sub, ctx),
    }
}

// ─── init ─────────────────────────────────────────────────────────────────────

mod init {
    use std::path::{Path, PathBuf};

    use cronus_core::state;

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

    use cronus_core::paths::{Paths, Root};

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
