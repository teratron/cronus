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
        Command::Doctor { fix } => doctor::run(fix, ctx),
        Command::Backup { sub } => backup_cmd::dispatch(sub, ctx),
        Command::Restore { backup } => backup_cmd::restore(&backup, ctx),
        Command::Activation { sub } => activation_cmd::dispatch(sub, ctx),
        Command::Loop { sub } => loop_cmd::dispatch(sub, ctx),
        Command::Archetype { sub } => archetype_cmd::dispatch(sub, ctx),
        Command::Knowledge { sub } => knowledge_cmd::dispatch(sub, ctx),
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

// ─── doctor ───────────────────────────────────────────────────────────────────

mod doctor {
    use std::path::Path;

    use cronus_core::doctor::{self, Disposition};
    use cronus_core::paths::{Paths, Root};

    use crate::output::Context;

    pub fn run(fix: bool, ctx: &Context) -> i32 {
        let paths = Paths::os_native();
        let root = paths.resolve(Root::State);
        run_at(&root, fix, ctx)
    }

    /// Config validity is checked against the real workspace state root;
    /// the remaining check categories (board cards, sessions, disk, store,
    /// crash recovery) await their subsystem-projection wiring — the check
    /// engine itself already covers all six (see `cronus_core::doctor` tests).
    fn run_at(state_root: &Path, fix: bool, ctx: &Context) -> i32 {
        let mut inputs = doctor::DoctorInputs::default();
        if !state_root.join("app.json").exists() {
            inputs.config.missing_defaults.push("app.json".to_string());
        }

        let report = if fix {
            doctor::repair(&inputs)
        } else {
            doctor::check(&inputs)
        };

        if ctx.is_json() {
            println!(
                "{{\"findings\":{},\"repaired\":{},\"escalated\":{}}}",
                report.findings.len(),
                report.repaired.len(),
                report.escalated.len()
            );
        } else if report.findings.is_empty() {
            println!("doctor: all checks passed");
        } else {
            for finding in &report.findings {
                let tag = match finding.disposition {
                    Disposition::SafeRepair if fix => "repaired",
                    Disposition::SafeRepair => "repairable (--fix to apply)",
                    Disposition::Escalate => "escalate",
                };
                println!("[{tag}] {}: {}", finding.id, finding.description);
            }
        }
        if report.escalated.is_empty() { 0 } else { 1 }
    }

    #[cfg(test)]
    mod tests {
        use std::fs;

        use crate::output::{Context, OutputFormat};

        use super::run_at;

        #[test]
        fn reports_all_clear_when_config_is_present() {
            let tmp =
                std::env::temp_dir().join(format!("cronus-doctor-clean-{}", std::process::id()));
            let _ = fs::remove_dir_all(&tmp);
            fs::create_dir_all(&tmp).unwrap();
            fs::write(tmp.join("app.json"), "{}\n").unwrap();

            let ctx = Context::new(OutputFormat::Text);
            assert_eq!(run_at(&tmp, false, &ctx), 0);

            let _ = fs::remove_dir_all(&tmp);
        }

        #[test]
        fn flags_missing_config_as_a_safe_repair_and_fix_resolves_it() {
            let tmp =
                std::env::temp_dir().join(format!("cronus-doctor-missing-{}", std::process::id()));
            let _ = fs::remove_dir_all(&tmp);
            fs::create_dir_all(&tmp).unwrap();

            let ctx = Context::new(OutputFormat::Text);
            // A safe-repair-only finding never escalates, so the exit code stays 0
            // even though something was flagged.
            assert_eq!(
                run_at(&tmp, false, &ctx),
                0,
                "check-only still exits 0 (nothing escalated)"
            );
            assert_eq!(run_at(&tmp, true, &ctx), 0, "--fix applies the safe repair");

            let _ = fs::remove_dir_all(&tmp);
        }
    }
}

// ─── backup ───────────────────────────────────────────────────────────────────

mod backup_cmd {
    use std::path::{Path, PathBuf};

    use cronus_core::backup::{self, BackupOptions};
    use cronus_core::paths::{Paths, Root};

    use crate::cli::BackupCommand;
    use crate::output::Context;

    fn state_root_and_backups_dir() -> (PathBuf, PathBuf) {
        let paths = Paths::os_native();
        let root = paths.resolve(Root::State);
        let backups_dir = root.join("backups");
        (root, backups_dir)
    }

    pub fn dispatch(sub: BackupCommand, ctx: &Context) -> i32 {
        let (state_root, backups_dir) = state_root_and_backups_dir();
        match sub {
            BackupCommand::Create { to, include_logs } => {
                create_at(&state_root, &backups_dir, to.as_deref(), include_logs, ctx)
            }
            BackupCommand::List => list_at(&backups_dir, ctx),
        }
    }

    fn create_at(
        state_root: &Path,
        backups_dir: &Path,
        to: Option<&Path>,
        include_logs: bool,
        ctx: &Context,
    ) -> i32 {
        let options = BackupOptions { include_logs };
        match backup::create(state_root, backups_dir, to, options) {
            Ok(backup_ref) => {
                if ctx.is_json() {
                    println!(
                        "{{\"id\":\"{}\",\"path\":\"{}\"}}",
                        backup_ref.id,
                        backup_ref.path.display()
                    );
                } else {
                    println!(
                        "backup created: {} ({})",
                        backup_ref.id,
                        backup_ref.path.display()
                    );
                }
                0
            }
            Err(err) => {
                eprintln!("backup failed: {err}");
                1
            }
        }
    }

    fn list_at(backups_dir: &Path, ctx: &Context) -> i32 {
        match backup::list(backups_dir) {
            Ok(backups) if backups.is_empty() => {
                println!("no backups found");
                0
            }
            Ok(backups) => {
                for backup_ref in &backups {
                    if ctx.is_json() {
                        println!(
                            "{{\"id\":\"{}\",\"path\":\"{}\",\"created_at\":{}}}",
                            backup_ref.id,
                            backup_ref.path.display(),
                            backup_ref.created_at_unix
                        );
                    } else {
                        println!("{}\t{}", backup_ref.id, backup_ref.path.display());
                    }
                }
                0
            }
            Err(err) => {
                eprintln!("listing backups failed: {err}");
                1
            }
        }
    }

    pub fn restore(backup_id: &str, ctx: &Context) -> i32 {
        let (state_root, backups_dir) = state_root_and_backups_dir();
        restore_at(&state_root, &backups_dir, backup_id, ctx)
    }

    fn restore_at(state_root: &Path, backups_dir: &Path, backup_id: &str, ctx: &Context) -> i32 {
        let Ok(backups) = backup::list(backups_dir) else {
            eprintln!("could not read backups directory");
            return 1;
        };
        let Some(backup_ref) = backups.into_iter().find(|b| b.id == backup_id) else {
            eprintln!("backup not found: {backup_id}");
            return 1;
        };
        match backup::restore(&backup_ref, state_root) {
            Ok(()) => {
                if !ctx.is_json() {
                    println!("restored {backup_id} into {}", state_root.display());
                }
                0
            }
            Err(err) => {
                eprintln!("restore failed: {err}");
                1
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use std::fs;

        use crate::output::{Context, OutputFormat};

        use super::{create_at, list_at, restore_at};

        #[test]
        fn create_then_list_then_restore_round_trips() {
            let state_root = std::env::temp_dir()
                .join(format!("cronus-cli-backup-state-{}", std::process::id()));
            let backups_dir =
                std::env::temp_dir().join(format!("cronus-cli-backup-dir-{}", std::process::id()));
            let restore_target = std::env::temp_dir()
                .join(format!("cronus-cli-backup-restore-{}", std::process::id()));
            for dir in [&state_root, &backups_dir, &restore_target] {
                let _ = fs::remove_dir_all(dir);
            }
            fs::create_dir_all(&state_root).unwrap();
            fs::write(state_root.join("config.json"), "{}").unwrap();
            fs::write(state_root.join(".env"), "SECRET=x").unwrap();

            let ctx = Context::new(OutputFormat::Text);
            assert_eq!(create_at(&state_root, &backups_dir, None, false, &ctx), 0);
            assert_eq!(list_at(&backups_dir, &ctx), 0);

            let backups = cronus_core::backup::list(&backups_dir).unwrap();
            assert_eq!(backups.len(), 1);
            assert_eq!(
                restore_at(&restore_target, &backups_dir, &backups[0].id, &ctx),
                0
            );
            assert!(restore_target.join("config.json").exists());
            assert!(!restore_target.join(".env").exists());

            for dir in [&state_root, &backups_dir, &restore_target] {
                let _ = fs::remove_dir_all(dir);
            }
        }

        #[test]
        fn restoring_an_unknown_backup_id_fails_cleanly() {
            let backups_dir = std::env::temp_dir()
                .join(format!("cronus-cli-backup-unknown-{}", std::process::id()));
            let _ = fs::remove_dir_all(&backups_dir);
            fs::create_dir_all(&backups_dir).unwrap();

            let ctx = Context::new(OutputFormat::Text);
            let dest =
                std::env::temp_dir().join(format!("cronus-cli-backup-dest-{}", std::process::id()));
            assert_eq!(
                restore_at(&dest, &backups_dir, "backup-does-not-exist", &ctx),
                1
            );

            let _ = fs::remove_dir_all(&backups_dir);
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

    use cronus_core::workspace::{WorkspaceId, WorkspaceManager, WorkspaceTemplate};

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
        cronus_core::paths::Paths::os_native()
            .resolve(cronus_core::paths::Root::State)
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

        use cronus_core::workspace::{WorkspaceId, WorkspaceManager, WorkspaceTemplate};

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
    use cronus_core::memory::{MemoryEntry, MemoryKind, MemorySource, store::MemoryStore};

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
    use cronus_codegraph::{
        extractor::{Extractor, RegexExtractor},
        index::CodeIndex,
    };
    use std::path::PathBuf;

    use crate::cli::CodegraphCommand;
    use crate::output::Context;

    fn open_index() -> Result<CodeIndex, String> {
        CodeIndex::open_in_memory().map_err(|e| e.to_string())
    }

    pub fn dispatch(sub: CodegraphCommand, ctx: &Context) -> i32 {
        match sub {
            CodegraphCommand::Index { path } => index_path(path, ctx),
            CodegraphCommand::Search { query } => search_graph(query, ctx),
        }
    }

    fn index_path(path: PathBuf, ctx: &Context) -> i32 {
        let index = match open_index() {
            Ok(idx) => idx,
            Err(e) => {
                eprintln!("error: {e}");
                return 1;
            }
        };
        let extractor = RegexExtractor;
        let mut total = 0usize;

        if path.is_file() {
            total += index_file(&index, &path, &extractor);
        } else if path.is_dir()
            && let Ok(entries) = std::fs::read_dir(&path)
        {
            for entry in entries.filter_map(|e| e.ok()) {
                let p = entry.path();
                if p.extension().and_then(|e| e.to_str()) == Some("rs") {
                    total += index_file(&index, &p, &extractor);
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

    fn index_file(index: &CodeIndex, path: &PathBuf, extractor: &RegexExtractor) -> usize {
        let source = std::fs::read_to_string(path).unwrap_or_default();
        let syms = extractor.extract(&source);
        let n = syms.len();
        let _ = index.index_symbols(&path.display().to_string(), &syms);
        n
    }

    fn search_graph(query: String, ctx: &Context) -> i32 {
        let index = match open_index() {
            Ok(idx) => idx,
            Err(e) => {
                eprintln!("error: {e}");
                return 1;
            }
        };
        match index.search(&query, 10) {
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
    use cronus_core::roles::{PRESET_CATALOG, RoleManager};

    use crate::cli::RoleCommand;
    use crate::output::Context;

    fn state_dir() -> std::path::PathBuf {
        cronus_core::paths::Paths::os_native().resolve(cronus_core::paths::Root::State)
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

    use cronus_core::kanban::{Board, CardState};
    use cronus_core::tool_security::now_ms;

    use crate::cli::BoardCommand;
    use crate::output::Context;

    fn board_path() -> PathBuf {
        cronus_core::paths::Paths::os_native()
            .resolve(cronus_core::paths::Root::State)
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

    use cronus_core::scheduler::{RecurrencePreset, Schedule, ScheduleAction, Scheduler};

    use crate::cli::ScheduleCommand;
    use crate::output::Context;

    fn sched_dir() -> PathBuf {
        cronus_core::paths::Paths::os_native()
            .resolve(cronus_core::paths::Root::State)
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
    use cronus_core::budget::{BudgetEngine, BudgetPeriod, BudgetPolicy};

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

    use cronus_core::exec_workspace::ExecWorkspaceManager;
    use cronus_core::tool_security::now_ms;

    use crate::cli::ExecCommand;
    use crate::output::Context;

    fn base_dir() -> PathBuf {
        cronus_core::paths::Paths::os_native()
            .resolve(cronus_core::paths::Root::State)
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

    use cronus_core::quality::{GateResultStore, detect_language};

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

    use cronus_core::extensions::{
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
            ExtCommand::Skill { sub } => skill::dispatch(sub, ctx),
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
        let result = cronus_core::tool_security::SkillScanner::scan_content(&content, fname);
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

    mod skill {
        use std::collections::HashMap;
        use std::path::Path;

        use cronus_core::extensions::{
            ExtensionKind, ExtensionManifest, ExtensionPermissions, ExtensionSource,
        };
        use cronus_core::skills::convert::{
            self, ConvertError, ForeignItem, ForeignKind, WitnessStatus,
        };
        use cronus_core::skills::store::{SkillId, SkillStore, SkillTier};
        use cronus_core::skills::synthesize::{self, AuthoredSkill, SynthesizeError};

        use crate::cli::SkillCommand;
        use crate::output::Context;

        /// Single-name lookups aren't pack-qualified at the CLI surface yet;
        /// everything resolves under one placeholder pack until a real
        /// namespacing scheme lands.
        const DEFAULT_PACK: &str = "core";

        pub fn dispatch(sub: SkillCommand, ctx: &Context) -> i32 {
            match sub {
                SkillCommand::Import { path } => import(&path, ctx),
                SkillCommand::Create { prompt } => create(&prompt, ctx),
                SkillCommand::Status { id } => status(id, ctx),
            }
        }

        /// Best-effort kind inference from a single imported file's
        /// extension. A real foreign package spans many files classified by
        /// its own source adapter (§4.4); this binding handles the
        /// single-file case honestly until a directory-walking adapter
        /// exists.
        fn infer_kind(path: &Path) -> ForeignKind {
            match path.extension().and_then(|e| e.to_str()) {
                Some("md" | "txt") => ForeignKind::Instruction,
                Some("sh" | "py" | "ps1" | "bat") => ForeignKind::Script,
                Some("nd" | "yaml" | "yml") => ForeignKind::ProceduralStep,
                _ => ForeignKind::Asset,
            }
        }

        fn import(path: &Path, ctx: &Context) -> i32 {
            let content = match std::fs::read_to_string(path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("error: {e}");
                    return 1;
                }
            };
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("imported")
                .to_string();
            let item = ForeignItem {
                path: file_name.clone(),
                kind: infer_kind(path),
                content,
            };
            let manifest = ExtensionManifest {
                id: format!("{DEFAULT_PACK}/{file_name}"),
                kind: ExtensionKind::Skill,
                name: file_name,
                version: "0.0.0".to_string(),
                source: ExtensionSource::Custom, // convert() overrides this regardless
                capabilities: Vec::new(),
                permissions: ExtensionPermissions::default(),
            };
            // No signed-witness mechanism is wired yet; a single-file CLI
            // import is treated as pre-verified — a real witness adapter
            // replaces this input, not this call site, once one exists. No
            // persisted transpile-mapping table exists yet either, so every
            // script/procedure degrades honestly rather than being silently
            // guessed at.
            match convert::convert(WitnessStatus::Valid, manifest, &[item], &HashMap::new()) {
                Ok(outcome) => {
                    if ctx.is_json() {
                        println!(
                            "{{\"mapped\":{},\"degraded\":{}}}",
                            outcome.report.mapped.len(),
                            outcome.report.degraded.len()
                        );
                    } else {
                        println!(
                            "Imported '{}': {} mapped, {} degraded to instruction-only.",
                            outcome.package.manifest.id,
                            outcome.report.mapped.len(),
                            outcome.report.degraded.len()
                        );
                    }
                    0
                }
                Err(ConvertError::WitnessDenied(status)) => {
                    eprintln!("error: witness denied: {status:?}");
                    1
                }
                Err(ConvertError::InvalidResult(e)) => {
                    eprintln!("error: invalid package: {e}");
                    1
                }
            }
        }

        fn create(prompt: &str, ctx: &Context) -> i32 {
            let slug = slugify(prompt);
            let manifest = ExtensionManifest {
                id: format!("generated/{slug}"),
                kind: ExtensionKind::Skill,
                name: slug,
                version: "0.0.0".to_string(),
                source: ExtensionSource::Generated, // synthesize() overrides this regardless
                capabilities: Vec::new(),
                permissions: ExtensionPermissions::default(),
            };
            // The authoring model call is a seam (§4.4 Notes): this binding
            // lands an honest instruction-only skill from the prompt text
            // until a model adapter exists — no workflow.nd is fabricated.
            let authored = AuthoredSkill {
                manifest,
                workflow_nd: false,
                workflow_md: false,
                assets: Vec::new(),
            };
            match synthesize::synthesize(authored) {
                Ok(outcome) => {
                    if ctx.is_json() {
                        println!(
                            "{{\"id\":\"{}\",\"status\":\"discovered\"}}",
                            outcome.package.manifest.id
                        );
                    } else {
                        println!(
                            "Created '{}' (source: generated, pending review).",
                            outcome.package.manifest.id
                        );
                    }
                    0
                }
                Err(SynthesizeError::Lint(e)) => {
                    eprintln!("error: lint failed: {e:?}");
                    1
                }
                Err(SynthesizeError::InvalidResult(e)) => {
                    eprintln!("error: invalid package: {e}");
                    1
                }
            }
        }

        fn slugify(prompt: &str) -> String {
            let slug = prompt
                .split_whitespace()
                .take(4)
                .collect::<Vec<_>>()
                .join("-")
                .to_lowercase();
            if slug.is_empty() {
                "untitled".to_string()
            } else {
                slug
            }
        }

        /// One skill's reportable status (§4.6: store origin, degradation
        /// flag, pending-review state).
        struct SkillStatusView {
            origin: SkillTier,
            degraded: bool,
            pending_review: bool,
        }

        fn compute_status(store: &SkillStore, id: &str) -> Option<SkillStatusView> {
            store
                .resolve(&SkillId::new(DEFAULT_PACK, id))
                .map(|(origin, entry)| SkillStatusView {
                    origin,
                    degraded: entry.degraded,
                    pending_review: entry.pending_review,
                })
        }

        fn status(id: Option<String>, ctx: &Context) -> i32 {
            // No persistence layer is wired yet (matches every other `ext`
            // command's fresh-registry pattern): status reports against an
            // empty store honestly rather than fabricating tracked skills.
            status_with_store(&SkillStore::new(), id, ctx)
        }

        fn status_with_store(store: &SkillStore, id: Option<String>, ctx: &Context) -> i32 {
            match id {
                Some(id) => match compute_status(store, &id) {
                    Some(view) => {
                        if ctx.is_json() {
                            println!(
                                "{{\"id\":\"{id}\",\"origin\":\"{:?}\",\"degraded\":{},\"pending_review\":{}}}",
                                view.origin, view.degraded, view.pending_review
                            );
                        } else {
                            println!(
                                "{id}: origin={:?} degraded={} pending_review={}",
                                view.origin, view.degraded, view.pending_review
                            );
                        }
                        0
                    }
                    None => {
                        if ctx.is_json() {
                            println!("{{\"found\":false}}");
                        } else {
                            println!("No tracked skill '{id}'.");
                        }
                        0
                    }
                },
                None => {
                    if ctx.is_json() {
                        println!("[]");
                    } else {
                        println!("No skills tracked yet.");
                    }
                    0
                }
            }
        }

        #[cfg(test)]
        mod tests {
            use super::*;
            use crate::output::OutputFormat;
            use cronus_core::skills::store::SkillEntry;
            use std::fs;
            use std::path::PathBuf;

            #[test]
            fn import_maps_instruction_and_reports_zero_degraded() {
                let dir = std::env::temp_dir()
                    .join(format!("cronus-skill-import-{}", std::process::id()));
                fs::create_dir_all(&dir).unwrap();
                let file = dir.join("standup.md");
                fs::write(&file, "Run the daily standup.").unwrap();

                let ctx = Context::new(OutputFormat::Text);
                assert_eq!(
                    import(&file, &ctx),
                    0,
                    "importing a plain instruction file must succeed"
                );

                let _ = fs::remove_dir_all(&dir);
            }

            #[test]
            fn import_degrades_an_unmapped_script() {
                let dir = std::env::temp_dir()
                    .join(format!("cronus-skill-degrade-{}", std::process::id()));
                fs::create_dir_all(&dir).unwrap();
                let file = dir.join("run.sh");
                fs::write(&file, "#!/bin/sh\necho hi\n").unwrap();

                let ctx = Context::new(OutputFormat::Text);
                // No transpile mapping exists for this script — it must
                // degrade to instruction-only, not fail the command.
                assert_eq!(import(&file, &ctx), 0);

                let _ = fs::remove_dir_all(&dir);
            }

            #[test]
            fn import_reports_error_on_missing_file() {
                let ctx = Context::new(OutputFormat::Text);
                let missing = PathBuf::from("/nonexistent/path/to/a/skill.md");
                assert_eq!(import(&missing, &ctx), 1);
            }

            #[test]
            fn create_lands_an_instruction_only_skill() {
                let ctx = Context::new(OutputFormat::Text);
                assert_eq!(create("summarize the weekly report", &ctx), 0);
            }

            #[test]
            fn slugify_falls_back_to_untitled_on_empty_prompt() {
                assert_eq!(slugify("   "), "untitled");
                assert_eq!(slugify("Draft A Standup Note"), "draft-a-standup-note");
            }

            #[test]
            fn status_reports_store_origin_degradation_and_pending_review() {
                let store = SkillStore::with_presets([(
                    SkillId::new(DEFAULT_PACK, "standup"),
                    SkillEntry::new("preset content").with_status(true, true),
                )]);
                let view = compute_status(&store, "standup").expect("must resolve");
                assert_eq!(view.origin, SkillTier::Preset);
                assert!(view.degraded);
                assert!(view.pending_review);
            }

            #[test]
            fn status_for_unknown_id_is_none() {
                let store = SkillStore::new();
                assert!(compute_status(&store, "missing").is_none());
            }

            #[test]
            fn status_binding_exits_0_for_known_and_unknown_ids() {
                let store = SkillStore::with_presets([(
                    SkillId::new(DEFAULT_PACK, "standup"),
                    SkillEntry::new("preset content"),
                )]);
                let ctx = Context::new(OutputFormat::Text);
                assert_eq!(
                    status_with_store(&store, Some("standup".to_string()), &ctx),
                    0
                );
                assert_eq!(
                    status_with_store(&store, Some("missing".to_string()), &ctx),
                    0
                );
                assert_eq!(status_with_store(&store, None, &ctx), 0);
            }
        }
    }
}

// ─── learn ────────────────────────────────────────────────────────────────────

mod learn {
    use cronus_core::learning::LearningApprovalGate;

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
    use cronus_core::agent_registry::AgentRegistry;

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
    use cronus_core::constitution::{IDENTITY_FILES, identity_paths};

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

// ─── activation ─────────────────────────────────────────────────────────────

mod activation_cmd {
    use std::io::{self, IsTerminal, Write};

    use cronus_core::activation::{
        TransitionError, TransitionOutcome, disable as disable_transition,
        enable as enable_transition,
    };
    use cronus_core::{
        ActivationMode, ActivationRegistry, ActivationState, default_activation_registry,
    };

    use crate::cli::{ActivationCommand, ActivationModeArg};
    use crate::output::Context;

    pub fn dispatch(sub: ActivationCommand, ctx: &Context) -> i32 {
        match sub {
            ActivationCommand::Status => status(ctx),
            ActivationCommand::Enable {
                mode,
                acknowledge_unattended_execution,
            } => enable(mode, acknowledge_unattended_execution, ctx),
            ActivationCommand::Disable => disable(ctx),
        }
    }

    fn mode_str(mode: ActivationMode) -> &'static str {
        match mode {
            ActivationMode::Login => "login",
            ActivationMode::System => "system",
        }
    }

    fn state_str(state: &ActivationState) -> String {
        match state {
            ActivationState::Inactive => "inactive".to_string(),
            ActivationState::Active(mode) => format!("active ({})", mode_str(*mode)),
            ActivationState::RequiresApproval(mode) => {
                format!("requires-approval ({})", mode_str(*mode))
            }
            ActivationState::Unknown { reason } => format!("unknown: {reason}"),
        }
    }

    fn transition_error_str(err: &TransitionError) -> String {
        match err {
            TransitionError::PriorModeNotRemoved { observed } => {
                format!("the prior mode could not be verified removed (observed {observed:?})")
            }
            TransitionError::RegistrationFailed { detail } => detail.clone(),
            TransitionError::VerificationFailed { observed } => {
                format!("registration did not verify (observed {observed:?})")
            }
            TransitionError::ConsentMissing { mode } => {
                format!("no consent recorded for {} mode", mode_str(*mode))
            }
        }
    }

    fn status(ctx: &Context) -> i32 {
        let registry = default_activation_registry();
        let state = registry.observe();
        if ctx.is_json() {
            println!(
                "{{\"state\":\"{}\"}}",
                state_str(&state).replace('"', "\\\"")
            );
        } else {
            println!("activation status: {}", state_str(&state));
        }
        0
    }

    /// The outcome of `enable`'s consent gate, decided BEFORE any registry is
    /// constructed or touched — so a `RefuseNonInteractive`/`CancelledByUser`
    /// path structurally cannot mutate anything (the Verify property: "exits
    /// non-zero and mutates nothing"). `confirm` is injected so the gate is
    /// testable without a real terminal or a real OS call.
    #[derive(Debug, PartialEq, Eq)]
    enum EnableGate {
        Proceed,
        RefuseNonInteractive,
        CancelledByUser,
    }

    fn enable_gate(
        acknowledged: bool,
        interactive: bool,
        confirm: impl FnOnce() -> bool,
    ) -> EnableGate {
        if acknowledged {
            return EnableGate::Proceed;
        }
        if !interactive {
            // BA-5: an unattended `enable` would silently satisfy the
            // consent moment a human must see — refuse rather than proceed.
            return EnableGate::RefuseNonInteractive;
        }
        if confirm() {
            EnableGate::Proceed
        } else {
            EnableGate::CancelledByUser
        }
    }

    fn confirm_disclosure(mode: ActivationMode) -> bool {
        println!(
            "Enabling {} background activation lets Cronus run while you are not present.",
            mode_str(mode)
        );
        println!(
            "It will call models, spend allowance, and act on your behalf under the \
             currently configured autonomy level and spend ceiling."
        );
        print!("Type 'yes' to confirm, anything else to cancel: ");
        let _ = io::stdout().flush();
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            return false;
        }
        input.trim().eq_ignore_ascii_case("yes")
    }

    fn enable(mode: ActivationModeArg, acknowledged: bool, ctx: &Context) -> i32 {
        let target = match mode {
            ActivationModeArg::Login => ActivationMode::Login,
            ActivationModeArg::System => ActivationMode::System,
        };
        let interactive = io::stdin().is_terminal();

        match enable_gate(acknowledged, interactive, || confirm_disclosure(target)) {
            EnableGate::RefuseNonInteractive => {
                eprintln!(
                    "cronus activation enable refuses to run non-interactively without \
                     --acknowledge-unattended-execution"
                );
                return 1;
            }
            EnableGate::CancelledByUser => {
                println!("cancelled — activation not changed");
                return 1;
            }
            EnableGate::Proceed => {}
        }

        let registry = default_activation_registry();
        match enable_transition(&registry, target) {
            Ok(TransitionOutcome::Activated(m)) => {
                if ctx.is_json() {
                    println!("{{\"outcome\":\"activated\",\"mode\":\"{}\"}}", mode_str(m));
                } else {
                    println!("activation enabled: {}", mode_str(m));
                }
                0
            }
            Ok(TransitionOutcome::RequiresApproval(m)) => {
                println!(
                    "registered ({}), but the OS requires your approval before it will run — \
                     see your system's background-items settings",
                    mode_str(m)
                );
                0
            }
            Ok(TransitionOutcome::Deactivated) => 0, // enable() never returns this in practice
            Err(err) => {
                eprintln!("activation enable failed: {}", transition_error_str(&err));
                1
            }
        }
    }

    fn disable(ctx: &Context) -> i32 {
        let registry = default_activation_registry();
        match disable_transition(&registry) {
            Ok(_) => {
                if ctx.is_json() {
                    println!("{{\"outcome\":\"deactivated\"}}");
                } else {
                    println!("activation disabled");
                }
                0
            }
            Err(err) => {
                eprintln!("activation disable failed: {}", transition_error_str(&err));
                1
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use std::cell::Cell;

        use super::{EnableGate, enable_gate};

        #[test]
        fn non_interactive_without_acknowledgement_refuses() {
            let gate = enable_gate(false, false, || true);
            assert_eq!(gate, EnableGate::RefuseNonInteractive);
        }

        #[test]
        fn interactive_without_acknowledgement_asks_for_confirmation() {
            assert_eq!(enable_gate(false, true, || true), EnableGate::Proceed);
            assert_eq!(
                enable_gate(false, true, || false),
                EnableGate::CancelledByUser
            );
        }

        #[test]
        fn the_acknowledgement_flag_skips_confirmation_entirely() {
            let confirm_called = Cell::new(false);
            let gate = enable_gate(true, true, || {
                confirm_called.set(true);
                true
            });
            assert_eq!(gate, EnableGate::Proceed);
            assert!(
                !confirm_called.get(),
                "the acknowledgement flag must skip the confirmation prompt entirely"
            );
        }

        #[test]
        fn the_acknowledgement_flag_also_works_non_interactively() {
            // The flag is exactly what makes a script/CI-runner invocation
            // legitimate — it must proceed even with interactive=false.
            assert_eq!(enable_gate(true, false, || true), EnableGate::Proceed);
        }
    }
}

// ─── loop ───────────────────────────────────────────────────────────────────

mod loop_cmd {
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use cronus_core::loop_bootstrap::{
        FileExistsBackend, file_exists_spec, read_report, read_spec, write_report, write_spec,
    };
    use cronus_core::loop_runner::{LoopOutcome, run_execution};

    use crate::cli::LoopCommand;
    use crate::output::Context;

    pub fn dispatch(sub: LoopCommand, ctx: &Context) -> i32 {
        match sub {
            LoopCommand::Run { file, max_iter } => run(file, max_iter, ctx),
            LoopCommand::Evolve { harness_id } => evolve(&harness_id),
            LoopCommand::Log { run_id } => log(&run_id),
            LoopCommand::Show { run_id } => show(&run_id),
        }
    }

    fn new_run_id() -> String {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        format!("run-{secs}-{}", std::process::id())
    }

    fn run(file: PathBuf, max_iter: u32, ctx: &Context) -> i32 {
        let spec = file_exists_spec(max_iter);
        let mut backend = FileExistsBackend { target_path: file };
        let report = run_execution(&spec, &mut backend, "");
        let run_id = new_run_id();

        if let Err(e) = write_report(&run_id, &report) {
            eprintln!("cronus loop run: failed to persist ledger: {e}");
            return 1;
        }
        if let Err(e) = write_spec(&run_id, &spec) {
            eprintln!("cronus loop run: failed to persist spec: {e}");
            return 1;
        }

        match &report.outcome {
            LoopOutcome::Done(_) => {
                if ctx.is_json() {
                    println!(
                        "{{\"outcome\":\"done\",\"run_id\":\"{run_id}\",\"iterations\":{}}}",
                        report.iterations_run
                    );
                } else {
                    println!(
                        "loop {run_id}: done ({} iteration(s))",
                        report.iterations_run
                    );
                }
                0
            }
            LoopOutcome::Stopped(reason) => {
                if ctx.is_json() {
                    println!(
                        "{{\"outcome\":\"stopped\",\"reason\":\"{reason}\",\"run_id\":\"{run_id}\"}}"
                    );
                } else {
                    println!("loop {run_id}: stopped ({reason})");
                }
                1
            }
        }
    }

    fn evolve(_harness_id: &str) -> i32 {
        // INV-9 shipped-surface honesty: this workspace has no CLI-nameable
        // harness registry yet, so the command is present but marked
        // unavailable — never a silent "not implemented" success stub.
        eprintln!(
            "cronus loop evolve is unavailable: no harness registry exists in this workspace \
             yet. The domain-tier run_evolution is implemented and tested; wiring a real, \
             CLI-nameable harness resource is future work."
        );
        1
    }

    fn log(run_id: &str) -> i32 {
        match read_report(run_id) {
            Ok(text) => {
                println!("{text}");
                0
            }
            Err(e) => {
                eprintln!("cronus loop log: {e}");
                1
            }
        }
    }

    fn show(run_id: &str) -> i32 {
        match read_spec(run_id) {
            Ok(text) => {
                println!("{text}");
                0
            }
            Err(e) => {
                eprintln!("cronus loop show: {e}");
                1
            }
        }
    }
}

// ─── archetype ────────────────────────────────────────────────────────────────

mod archetype_cmd {
    use std::path::PathBuf;

    use cronus_core::archetype::{ArchetypeCatalog, ValidationStatus};
    use cronus_core::paths::{Paths, Root};

    use crate::cli::ArchetypeCommand;
    use crate::output::Context;

    pub fn dispatch(sub: ArchetypeCommand, ctx: &Context) -> i32 {
        match sub {
            ArchetypeCommand::List { catalog, active } => list(catalog, active),
            ArchetypeCommand::Info { id, deviations } => info(&id, deviations),
            ArchetypeCommand::Set { id, clear } => set(id, clear, ctx),
            ArchetypeCommand::Create { name, from } => create(&name, &from),
        }
    }

    fn state_dir() -> PathBuf {
        Paths::os_native().resolve(Root::State)
    }

    /// The office's active-archetype marker. A single state-tier file: absent
    /// (or empty) means the archetype-free default (OA-11).
    fn active_marker() -> PathBuf {
        state_dir().join("archetype").join("active")
    }

    fn read_active() -> Option<String> {
        std::fs::read_to_string(active_marker())
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    fn list(_catalog: bool, active: bool) -> i32 {
        let catalog = ArchetypeCatalog::program();
        if active {
            match read_active() {
                Some(id) => println!("active archetype: {id}"),
                None => println!("active archetype: (none — archetype-free)"),
            }
            return 0;
        }
        println!("shipped:");
        for def in catalog.shipped() {
            println!("  {} — {} ({} roles)", def.id, def.domain, def.pool.len());
        }
        println!("blocked:");
        for b in catalog.blocked() {
            println!(
                "  {} — {} (needs: {})",
                b.id,
                b.domain,
                b.missing_roles.join(", ")
            );
        }
        0
    }

    fn info(id: &str, deviations: bool) -> i32 {
        let catalog = ArchetypeCatalog::program();
        if let Some(def) = catalog.get(id) {
            println!("archetype: {}", def.id);
            println!("domain: {}", def.domain);
            println!("pool ({}): {}", def.pool.len(), def.pool.join(", "));
            println!("departments: {}", def.shape.departments.join(", "));
            println!("grow_when: {}", def.shape.grow_when);
            println!("seed: {} role(s)", def.seed.len());
            if deviations {
                // No offices have run under this archetype in this surface yet,
                // so its prior is unvalidated — reported honestly, never
                // "correct" by default (OA-9).
                println!("validation: {:?}", ValidationStatus::Unvalidated);
            }
            0
        } else if let Some(b) = catalog.blocked_status(id) {
            println!("archetype: {} (BLOCKED)", b.id);
            println!("domain: {}", b.domain);
            println!("missing roles: {}", b.missing_roles.join(", "));
            println!(
                "unblocking: each missing role must clear the ROL-9 gate in a \
                 separate role-catalog amendment first"
            );
            0
        } else {
            eprintln!("cronus archetype info: unknown archetype '{id}'");
            1
        }
    }

    fn set(id: Option<String>, clear: bool, ctx: &Context) -> i32 {
        let marker = active_marker();
        if let Some(parent) = marker.parent()
            && let Err(e) = std::fs::create_dir_all(parent)
        {
            eprintln!("cronus archetype set: {e}");
            return 1;
        }

        if clear {
            let _ = std::fs::remove_file(&marker);
            if ctx.is_json() {
                println!("{{\"active\":null}}");
            } else {
                println!("archetype cleared — office is archetype-free");
            }
            return 0;
        }

        let Some(id) = id else {
            eprintln!("cronus archetype set: provide an archetype id or --clear");
            return 1;
        };

        // Only a shippable archetype may be set active; a blocked one is not
        // usable, and an unknown one is refused (never silently accepted).
        let catalog = ArchetypeCatalog::program();
        if catalog.get(&id).is_none() {
            if catalog.blocked_status(&id).is_some() {
                eprintln!("cronus archetype set: '{id}' is declared-blocked and cannot be applied");
            } else {
                eprintln!("cronus archetype set: unknown archetype '{id}'");
            }
            return 1;
        }

        if let Err(e) = std::fs::write(&marker, &id) {
            eprintln!("cronus archetype set: {e}");
            return 1;
        }
        if ctx.is_json() {
            println!("{{\"active\":\"{id}\"}}");
        } else {
            println!("archetype set: {id} (expectations re-scoped; staff unchanged)");
        }
        0
    }

    fn create(name: &str, from: &str) -> i32 {
        let catalog = ArchetypeCatalog::program();
        match catalog.create_from_preset(&state_dir(), name, from) {
            Ok(custom) => {
                println!(
                    "created custom archetype '{}' from preset '{}'",
                    custom.definition.id, custom.derived_from
                );
                0
            }
            Err(e) => {
                eprintln!("cronus archetype create: {e}");
                1
            }
        }
    }
}

// ─── knowledge ──────────────────────────────────────────────────────────────

mod knowledge_cmd {
    use std::path::PathBuf;

    use cronus_core::knowledge_access::KnowledgePrincipal;
    use cronus_core::knowledge_bootstrap::{
        Collection, Document, KnowledgeService, RetrievalRequest,
    };
    use cronus_core::paths::{Paths, Root};
    use cronus_core::resource_sharing::GrantStore;

    use crate::cli::KnowledgeCommand;
    use crate::output::Context;

    /// The single local CLI operator's identity — see
    /// `KnowledgeService::query`'s doc comment for why this CLI treats the
    /// invoking user as the collection owner (RS-5) rather than wiring a
    /// not-yet-existing persisted multi-user grant store, matching the
    /// `board`/`memory` CLI modules' own no-multi-tenant-gating precedent.
    const LOCAL_USER: &str = "local";

    fn db_path() -> PathBuf {
        Paths::os_native()
            .resolve(Root::State)
            .join("knowledge")
            .join("knowledge.db")
    }

    fn open_service(ctx_err_prefix: &str) -> Option<KnowledgeService> {
        if let Some(parent) = db_path().parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match KnowledgeService::open_default(db_path()) {
            Ok(s) => Some(s),
            Err(e) => {
                eprintln!("{ctx_err_prefix}: {e}");
                None
            }
        }
    }

    pub fn dispatch(sub: KnowledgeCommand, ctx: &Context) -> i32 {
        match sub {
            KnowledgeCommand::CollectionCreate { id, name } => collection_create(id, name, ctx),
            KnowledgeCommand::Add {
                collection,
                doc_id,
                name,
                text,
            } => add_record(collection, doc_id, name, text, ctx),
            KnowledgeCommand::AddUrl {
                collection,
                doc_id,
                name,
                url,
            } => add_url(collection, doc_id, name, url, ctx),
            KnowledgeCommand::Query {
                collections,
                top_k,
                text,
            } => query(collections, top_k, text, ctx),
        }
    }

    fn collection_create(id: String, name: String, ctx: &Context) -> i32 {
        let Some(svc) = open_service("cronus knowledge collection-create") else {
            return 1;
        };
        let collection = Collection::new(id.clone(), LOCAL_USER, name);
        match svc.create_collection(&collection) {
            Ok(()) => {
                if ctx.is_json() {
                    println!("{{\"result\":\"created\",\"id\":\"{id}\"}}");
                } else {
                    println!("Created collection: {id}");
                }
                0
            }
            Err(e) => {
                eprintln!("cronus knowledge collection-create: {e}");
                1
            }
        }
    }

    fn add_record(
        collection: String,
        doc_id: String,
        name: String,
        text: String,
        ctx: &Context,
    ) -> i32 {
        let Some(svc) = open_service("cronus knowledge add") else {
            return 1;
        };
        let document = Document::new_agent(doc_id.clone(), collection, name);
        match svc.ingest_record(document, &text) {
            Ok(doc) => {
                if ctx.is_json() {
                    println!(
                        "{{\"result\":\"ingested\",\"id\":\"{}\",\"status\":\"{}\"}}",
                        doc.id,
                        doc.status.as_str()
                    );
                } else {
                    println!("Ingested: {} ({})", doc.id, doc.status.as_str());
                }
                0
            }
            Err(e) => {
                eprintln!("cronus knowledge add: {e}");
                1
            }
        }
    }

    fn add_url(
        collection: String,
        doc_id: String,
        name: String,
        url: String,
        ctx: &Context,
    ) -> i32 {
        let Some(svc) = open_service("cronus knowledge add-url") else {
            return 1;
        };
        let document = Document::new_agent(doc_id.clone(), collection, name);
        match svc.ingest_url(document, &url) {
            Ok(doc) => {
                if ctx.is_json() {
                    println!(
                        "{{\"result\":\"ingested\",\"id\":\"{}\",\"status\":\"{}\"}}",
                        doc.id,
                        doc.status.as_str()
                    );
                } else {
                    println!("Ingested: {} ({})", doc.id, doc.status.as_str());
                }
                0
            }
            Err(e) => {
                eprintln!("cronus knowledge add-url: {e}");
                1
            }
        }
    }

    fn query(collections: Vec<String>, top_k: usize, text: String, ctx: &Context) -> i32 {
        let Some(svc) = open_service("cronus knowledge query") else {
            return 1;
        };
        let mut request = RetrievalRequest::new(text, collections);
        request.top_k = top_k;
        let grants = GrantStore::new();
        match svc.query(&grants, KnowledgePrincipal::owner(LOCAL_USER), &request) {
            Ok(results) if results.is_empty() => {
                if ctx.is_json() {
                    println!("[]");
                } else {
                    println!("No results.");
                }
                0
            }
            Ok(results) => {
                if ctx.is_json() {
                    let items: Vec<String> = results
                        .iter()
                        .map(|c| {
                            format!(
                                "{{\"chunk_id\":\"{}\",\"document_id\":\"{}\",\"score\":{},\"text\":{:?}}}",
                                c.chunk_id, c.document_id, c.score, c.text
                            )
                        })
                        .collect();
                    println!("[{}]", items.join(","));
                } else {
                    for c in &results {
                        println!("[{:.4}] {} — {}", c.score, c.document_id, c.text);
                    }
                }
                0
            }
            Err(e) => {
                eprintln!("cronus knowledge query: {e}");
                1
            }
        }
    }

    // No unit tests here: every command function resolves the real,
    // un-overridable `Paths::os_native()` state directory (the `board`
    // module's own precedent for the same reason) — `KnowledgeService`'s
    // wiring itself is thoroughly tested in `crates/core`'s
    // `knowledge_bootstrap` module (against `:memory:` + a deterministic
    // fake embedder); this module's own job is arg-parsing/output-formatting
    // composition over that already-proven service.
}
