// ─── init ─────────────────────────────────────────────────────────────────────

pub(crate) mod init {
    use std::path::{Path, PathBuf};

    use cronus_core::state;

    use crate::output::{Context, json_escape};

    pub fn run(path: Option<PathBuf>, ctx: &Context) -> i32 {
        let dir =
            path.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        // The workspace state skeleton lives in a single `.cronus/` directory,
        // not scattered across the target directory's top level — running
        // `cronus init` in an existing project must not drop `app.json`,
        // `AGENTS.md`, `employees/`, `skills/` … next to the project's own files.
        run_at(&dir.join(".cronus"), ctx)
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
                    let shown = cronus_core::paths::display_clean(&abs);
                    if ctx.is_json() {
                        println!(
                            "{{\"result\":\"initialized\",\"path\":\"{}\"}}",
                            json_escape(&shown)
                        );
                    } else {
                        println!("Initialized: {shown}");
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

/// The workspace root every verb — installation half (`status`, `doctor`)
/// and every semantic invocable alike — resolves its state against: a thin
/// re-export of the one shared resolver (`cronus_domain::paths`, F-02) so
/// every surface scopes to the same project a user is actually standing in,
/// never a second, locally-reinvented notion of "the workspace."
pub(crate) use cronus_core::paths::resolve_workspace_root;
#[cfg(test)]
pub(crate) use cronus_core::paths::resolve_workspace_root_from;

pub(crate) mod status {
    use std::path::Path;

    use crate::output::Context;

    pub fn run(ctx: &Context) -> i32 {
        run_at(&crate::commands::resolve_workspace_root(), ctx)
    }

    fn run_at(state_root: &Path, ctx: &Context) -> i32 {
        if !state_root.join("app.json").exists() {
            eprintln!("No workspace initialized. Run 'cronus init' first.");
            return 1;
        }
        // For a `.cronus` state directory the workspace's name is its parent
        // (the project directory), not the literal ".cronus".
        let name_source = if state_root.file_name().and_then(|n| n.to_str()) == Some(".cronus") {
            state_root.parent().unwrap_or(state_root)
        } else {
            state_root
        };
        let workspace = name_source
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("default")
            .to_owned();
        if ctx.is_json() {
            println!(
                "{{\"workspace\":\"{}\",\"phase\":\"ready\"}}",
                crate::output::json_escape(&workspace)
            );
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

        /// `init` scaffolds the current directory (or a `--path`),
        /// not the OS state tier — so `status`, resolving from that directory
        /// or any descendant, must find that `app.json` rather than reporting
        /// "No workspace initialized" forever.
        #[test]
        fn status_resolves_an_initialized_ancestor_over_the_fallback() {
            use crate::commands::resolve_workspace_root_from as resolve_root_from;

            let root = std::env::temp_dir().join(format!(
                "cronus-status-anc-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_nanos())
                    .unwrap_or(0)
            ));
            let nested = root.join("a").join("b");
            fs::create_dir_all(&nested).unwrap();
            fs::write(root.join("app.json"), "{}\n").unwrap();
            let fallback = root.join("nonexistent-state-tier");

            let resolved = resolve_root_from(Some(&nested), fallback.clone());
            assert_eq!(
                resolved, root,
                "must resolve the ancestor holding app.json, not the fallback"
            );

            // A start dir with no app.json in any ancestor -> the fallback.
            let bare = std::env::temp_dir().join(format!(
                "cronus-status-bare-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_nanos())
                    .unwrap_or(0)
            ));
            fs::create_dir_all(&bare).unwrap();
            assert_eq!(
                resolve_root_from(Some(&bare), fallback.clone()),
                fallback,
                "with no app.json in any ancestor, resolve to the fallback"
            );
            let _ = fs::remove_dir_all(&bare);

            let ctx = Context::new(OutputFormat::Text);
            assert_eq!(
                run_at(&root, &ctx),
                0,
                "status must exit 0 for the resolved initialized root"
            );
            let _ = fs::remove_dir_all(&root);
        }
    }
}

// ─── doctor ───────────────────────────────────────────────────────────────────

pub(crate) mod doctor {
    use std::path::Path;

    use cronus_core::doctor::{self, Disposition};

    use crate::output::Context;

    pub fn run(fix: bool, ctx: &Context) -> i32 {
        run_at(&crate::commands::resolve_workspace_root(), fix, ctx)
    }

    /// Config validity is checked against the real workspace state root;
    /// the remaining check categories (board cards, sessions, disk, store,
    /// crash recovery) await their subsystem-projection wiring — the check
    /// engine itself already covers all six (see `cronus_core::doctor` tests).
    ///
    /// An `app.json` absent at the *resolved* root means no workspace was
    /// ever initialized there (`resolve_workspace_root` already searched
    /// every ancestor) — the same condition `status` reports, not a
    /// "repairable" defect in a workspace that doesn't exist. Checked first
    /// and reported the same way `status` does, before any doctor finding
    /// is built: the only signal this handler ever wired into
    /// `ConfigSignal::missing_defaults` *was* this same existence check, so
    /// nothing downstream loses coverage by handling it here instead.
    fn run_at(state_root: &Path, fix: bool, ctx: &Context) -> i32 {
        if !state_root.join("app.json").exists() {
            eprintln!("No workspace initialized. Run 'cronus init' first.");
            return 1;
        }
        let inputs = doctor::DoctorInputs::default();

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

        /// A directory with no `app.json` at the resolved root has no
        /// workspace at all — `doctor` must say so (matching `status`'s own
        /// message) rather than diagnose a "repairable" defect in a
        /// workspace that was never initialized (F-11).
        #[test]
        fn reports_no_workspace_rather_than_a_repairable_defect_when_uninitialized() {
            let tmp =
                std::env::temp_dir().join(format!("cronus-doctor-missing-{}", std::process::id()));
            let _ = fs::remove_dir_all(&tmp);
            fs::create_dir_all(&tmp).unwrap();

            let ctx = Context::new(OutputFormat::Text);
            assert_eq!(
                run_at(&tmp, false, &ctx),
                1,
                "no workspace here must exit 1, the same as status"
            );
            assert_eq!(
                run_at(&tmp, true, &ctx),
                1,
                "--fix does not conjure a workspace into existence either"
            );

            let _ = fs::remove_dir_all(&tmp);
        }
    }
}

// ─── backup ───────────────────────────────────────────────────────────────────

pub(crate) mod backup_cmd {
    use std::path::{Path, PathBuf};

    use cronus_core::backup::{self, BackupOptions};

    use crate::output::{Context, json_escape};

    /// What a backup snapshots: the resolved workspace root (F-02), the same
    /// project-scoped state `board`/`memory`/`knowledge`/… now write to —
    /// not the OS-native tier alone, which a per-project workspace user's
    /// actual data no longer lives in. `backups_dir` stays nested under it,
    /// preserving the existing on-disk relationship; the destination guard
    /// in `cronus_domain::backup` keeps a backup from ever copying itself
    /// into itself regardless of where this resolves.
    fn state_root_and_backups_dir() -> (PathBuf, PathBuf) {
        let root = cronus_core::paths::resolve_workspace_root();
        let backups_dir = root.join("backups");
        (root, backups_dir)
    }

    // Reached directly from `crate::installation::dispatch` now — the
    // installation half's own generated grammar owns the `backup` group, so
    // no `BackupCommand`-shaped wrapper is needed here any more.
    pub(crate) fn create(to: Option<PathBuf>, include_logs: bool, ctx: &Context) -> i32 {
        let (state_root, backups_dir) = state_root_and_backups_dir();
        create_at(&state_root, &backups_dir, to.as_deref(), include_logs, ctx)
    }

    pub(crate) fn list(ctx: &Context) -> i32 {
        let (_, backups_dir) = state_root_and_backups_dir();
        list_at(&backups_dir, ctx)
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
                        json_escape(&backup_ref.id),
                        json_escape(&backup_ref.path.display().to_string())
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
                if ctx.is_json() {
                    println!("[]");
                } else {
                    println!("no backups found");
                }
                0
            }
            Ok(backups) if ctx.is_json() => {
                let rows: Vec<String> = backups
                    .iter()
                    .map(|b| {
                        format!(
                            "{{\"id\":\"{}\",\"path\":\"{}\",\"created_at\":{}}}",
                            json_escape(&b.id),
                            json_escape(&b.path.display().to_string()),
                            b.created_at_unix
                        )
                    })
                    .collect();
                println!("[{}]", rows.join(","));
                0
            }
            Ok(backups) => {
                for backup_ref in &backups {
                    println!("{}\t{}", backup_ref.id, backup_ref.path.display());
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

// ─── workspace ────────────────────────────────────────────────────────────────

pub(crate) mod workspace {
    use std::path::{Path, PathBuf};

    use cronus_core::workspace::{WorkspaceId, WorkspaceManager, WorkspaceTemplate};

    use crate::output::Context;

    // Reached directly from `crate::installation::dispatch` now — the
    // installation half's own generated grammar owns the `workspace` group,
    // so no `WorkspaceCommand`-shaped wrapper is needed here any more.

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

    pub(crate) fn create(
        id: String,
        name: Option<String>,
        path: Option<PathBuf>,
        ctx: &Context,
    ) -> i32 {
        create_inner(&id, name.as_deref(), path.as_deref(), ctx)
    }

    fn create_inner(id: &str, name: Option<&str>, path: Option<&Path>, ctx: &Context) -> i32 {
        if cronus_core::dev_office_workspace::is_reserved_dev_workspace_id(id) {
            eprintln!("error: '{id}' is a reserved system workspace id and cannot be created here");
            return 1;
        }
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
                    println!(
                        "{{\"result\":\"created\",\"id\":\"{}\"}}",
                        crate::output::json_escape(&ws.id.to_string())
                    );
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

    pub(crate) fn list(ctx: &Context) -> i32 {
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
                                json_escape(&w.id.to_string()),
                                json_escape(&w.name)
                            )
                        })
                        .collect();
                    println!("[{}]", items.join(","));
                } else if workspaces.is_empty() {
                    println!("No results.");
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

    pub(crate) fn switch(id: String, ctx: &Context) -> i32 {
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
                    println!(
                        "{{\"result\":\"switched\",\"id\":\"{}\"}}",
                        crate::output::json_escape(&ws_id.to_string())
                    );
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

    pub(crate) fn delete(id: String, ctx: &Context) -> i32 {
        if cronus_core::dev_office_workspace::is_reserved_dev_workspace_id(&id) {
            eprintln!("error: '{id}' is a reserved system workspace and cannot be deleted here");
            return 1;
        }
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
                    println!(
                        "{{\"result\":\"deleted\",\"id\":\"{}\"}}",
                        crate::output::json_escape(&ws_id.to_string())
                    );
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

    pub(crate) fn check(id: String, ctx: &Context) -> i32 {
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

// ─── ext ──────────────────────────────────────────────────────────────────────

pub(crate) mod ext {
    use std::path::PathBuf;

    use cronus_core::extensions::{
        ExtensionKind, ExtensionManifest, ExtensionPermissions, ExtensionRegistry, ExtensionSource,
        ExtensionState,
    };

    use crate::output::Context;

    // Reached directly from `crate::installation::dispatch` now — the
    // installation half's own generated grammar owns the `ext` group
    // (including its nested `skill` sub-group), so no `ExtCommand`-shaped
    // wrapper is needed here any more.

    pub(crate) fn list(ctx: &Context) -> i32 {
        let registry = ExtensionRegistry::new();
        let all = registry.list();
        if all.is_empty() {
            if ctx.is_json() {
                println!("[]");
            } else {
                println!("No extensions registered.");
            }
        } else {
            // Known residual, not fixed here: unlike the empty-list branch
            // above, this one ignores `--format json` entirely and always
            // prints prose — corrected separately, after convergence,
            // alongside the same defect at every other site that discards
            // the requested format.
            for (m, _s) in &all {
                println!("{}: {} ({})", m.id, m.name, m.version);
            }
        }
        0
    }

    pub(crate) fn add(path: PathBuf, ctx: &Context) -> i32 {
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

    pub(crate) fn remove(_id: String, _ctx: &Context) -> i32 {
        eprintln!("error: extension removal not yet supported");
        1
    }

    // Known residual, not fixed here: `ctx` is accepted but never
    // consulted, so `--format json` is silently discarded — corrected
    // separately, after convergence, alongside the same defect at every
    // other site that discards the requested format.
    pub(crate) fn scan(path: PathBuf, _ctx: &Context) -> i32 {
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

    pub(crate) fn activate(id: String, ctx: &Context) -> i32 {
        let mut registry = ExtensionRegistry::new();
        // State machine: Discovered → Permitted → Active. Try the intermediate step first.
        let _ = registry.transition(&id, ExtensionState::Permitted);
        match registry.transition(&id, ExtensionState::Active) {
            Ok(()) => {
                if ctx.is_json() {
                    // Known residual, not fixed here: `id` is
                    // interpolated into a hand-built JSON literal without
                    // escaping — a `"` or `\` in an extension id would
                    // emit invalid JSON. Corrected separately, after
                    // convergence, by real `Outcome` serialization.
                    println!(
                        "{{\"result\":\"activated\",\"id\":\"{}\"}}",
                        crate::output::json_escape(&id)
                    );
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

    pub(crate) fn deactivate(id: String, ctx: &Context) -> i32 {
        let mut registry = ExtensionRegistry::new();
        match registry.transition(&id, ExtensionState::Inactive) {
            Ok(()) => {
                if ctx.is_json() {
                    println!(
                        "{{\"result\":\"deactivated\",\"id\":\"{}\"}}",
                        crate::output::json_escape(&id)
                    );
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

    pub(crate) mod skill {
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

        use crate::output::Context;

        /// Single-name lookups aren't pack-qualified at the CLI surface yet;
        /// everything resolves under one placeholder pack until a real
        /// namespacing scheme lands.
        const DEFAULT_PACK: &str = "core";

        // Reached directly from `crate::installation::dispatch` now — the
        // installation half's own generated grammar owns `ext skill`, so no
        // `SkillCommand`-shaped wrapper is needed here any more.

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

        pub(crate) fn import(path: &Path, ctx: &Context) -> i32 {
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

        pub(crate) fn create(prompt: &str, ctx: &Context) -> i32 {
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

        pub(crate) fn status(id: Option<String>, ctx: &Context) -> i32 {
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

// ─── registry ─────────────────────────────────────────────────────────────────

pub(crate) mod registry {
    use cronus_core::agent_registry::AgentRegistry;

    use crate::output::Context;

    // Reached directly from `crate::installation::dispatch` now — the
    // installation half's own generated grammar owns the `registry` group,
    // so no `RegistryCommand`-shaped wrapper is needed here any more.

    pub(crate) fn list(ctx: &Context) -> i32 {
        let registry = AgentRegistry::load();
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

    // Known residual, not fixed here: `ctx` is accepted but never
    // consulted, so `--format json` is silently discarded — corrected
    // separately, after convergence, alongside the same defect at every
    // other site that discards the requested format.
    pub(crate) fn show(name: String, _ctx: &Context) -> i32 {
        let registry = AgentRegistry::load();
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

    pub(crate) fn create(name: String, description: String, ctx: &Context) -> i32 {
        let mut registry = AgentRegistry::load();
        let def = AgentRegistry::generate_from_description(&name, &description);
        let def_name = def.name.clone();
        registry.register_custom(def);
        if let Err(e) = registry.save() {
            eprintln!("error: could not persist the agent registry: {e}");
            return 1;
        }
        if ctx.is_json() {
            // Known residual, not fixed here: `def_name` is interpolated
            // into a hand-built JSON literal without escaping — a `"` or
            // `\` in an agent name would emit invalid JSON. Corrected
            // separately, after convergence, by real `Outcome`
            // serialization.
            println!(
                "{{\"result\":\"created\",\"name\":\"{}\"}}",
                crate::output::json_escape(&def_name)
            );
        } else {
            println!("Created: {def_name}");
        }
        0
    }

    pub(crate) fn disable(name: String, ctx: &Context) -> i32 {
        let mut registry = AgentRegistry::load();
        registry.apply_user_config(&name, true, None);
        if let Err(e) = registry.save() {
            eprintln!("error: could not persist the agent registry: {e}");
            return 1;
        }
        if ctx.is_json() {
            println!(
                "{{\"result\":\"disabled\",\"name\":\"{}\"}}",
                crate::output::json_escape(&name)
            );
        } else {
            println!("Disabled: {name}");
        }
        0
    }

    pub(crate) fn enable(name: String, ctx: &Context) -> i32 {
        let mut registry = AgentRegistry::load();
        registry.apply_user_config(&name, false, None);
        if let Err(e) = registry.save() {
            eprintln!("error: could not persist the agent registry: {e}");
            return 1;
        }
        if ctx.is_json() {
            println!(
                "{{\"result\":\"enabled\",\"name\":\"{}\"}}",
                crate::output::json_escape(&name)
            );
        } else {
            println!("Enabled: {name}");
        }
        0
    }
}

// ─── activation ─────────────────────────────────────────────────────────────

pub(crate) mod activation_cmd {
    use std::io::{self, IsTerminal, Write};

    use cronus_core::activation::{
        TransitionError, TransitionOutcome, disable as disable_transition,
        enable as enable_transition,
    };
    use cronus_core::{
        ActivationMode, ActivationRegistry, ActivationState, default_activation_registry,
    };

    use crate::cli::ActivationModeArg;
    use crate::output::Context;

    // Reached directly from `crate::installation::dispatch` now — the
    // installation half's own generated grammar owns the `activation`
    // group, so no `ActivationCommand`-shaped wrapper is needed here any
    // more.

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

    pub(crate) fn status(ctx: &Context) -> i32 {
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

    pub(crate) fn enable(mode: ActivationModeArg, acknowledged: bool, ctx: &Context) -> i32 {
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
                // Known residual, not fixed here: ignores `--format json`
                // and always prints prose — corrected separately, after
                // convergence, alongside the same defect at every other
                // site that discards the requested format.
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
                // Known residual, not fixed here: unlike the `Activated`
                // arm just above, this one ignores `--format json` and
                // always prints prose — corrected separately, after
                // convergence, alongside the same defect at every other
                // site that discards the requested format.
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

    pub(crate) fn disable(ctx: &Context) -> i32 {
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

// ─── archetype ────────────────────────────────────────────────────────────────

pub(crate) mod archetype_cmd {
    use std::path::PathBuf;

    use cronus_core::archetype::{ArchetypeCatalog, ValidationStatus};
    use cronus_core::paths::{Paths, Root};

    use crate::output::{Context, json_escape};

    // Reached directly from `crate::installation::dispatch` now — the
    // installation half's own generated grammar owns the `archetype`
    // group, so no `ArchetypeCommand`-shaped wrapper is needed here any
    // more.

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

    fn json_str_array(items: impl Iterator<Item = String>) -> String {
        let inner: Vec<String> = items.map(|s| format!("\"{}\"", json_escape(&s))).collect();
        format!("[{}]", inner.join(","))
    }

    pub(crate) fn list(catalog_only: bool, active: bool, ctx: &Context) -> i32 {
        let _ = catalog_only;
        let catalog = ArchetypeCatalog::program();
        if active {
            let current = read_active();
            if ctx.is_json() {
                match &current {
                    Some(id) => println!("{{\"active\":\"{}\"}}", json_escape(id)),
                    None => println!("{{\"active\":null}}"),
                }
            } else {
                match &current {
                    Some(id) => println!("active archetype: {id}"),
                    None => println!("active archetype: (none — archetype-free)"),
                }
            }
            return 0;
        }
        if ctx.is_json() {
            let shipped = catalog
                .shipped()
                .iter()
                .map(|def| {
                    format!(
                        "{{\"id\":\"{}\",\"domain\":\"{}\",\"roles\":{}}}",
                        json_escape(&def.id),
                        json_escape(&def.domain),
                        def.pool.len()
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            let blocked = catalog
                .blocked()
                .iter()
                .map(|b| {
                    format!(
                        "{{\"id\":\"{}\",\"domain\":\"{}\",\"missing_roles\":{}}}",
                        json_escape(&b.id),
                        json_escape(&b.domain),
                        json_str_array(b.missing_roles.iter().cloned())
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            println!("{{\"shipped\":[{shipped}],\"blocked\":[{blocked}]}}");
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

    pub(crate) fn info(id: &str, deviations: bool, ctx: &Context) -> i32 {
        let catalog = ArchetypeCatalog::program();
        if let Some(def) = catalog.get(id) {
            if ctx.is_json() {
                let validation = if deviations {
                    format!(",\"validation\":\"{:?}\"", ValidationStatus::Unvalidated)
                } else {
                    String::new()
                };
                println!(
                    "{{\"id\":\"{}\",\"domain\":\"{}\",\"pool\":{},\"departments\":{},\
                     \"grow_when\":\"{}\",\"seed\":{}{validation}}}",
                    json_escape(&def.id),
                    json_escape(&def.domain),
                    json_str_array(def.pool.iter().cloned()),
                    json_str_array(def.shape.departments.iter().cloned()),
                    json_escape(&def.shape.grow_when),
                    def.seed.len(),
                );
                return 0;
            }
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
            if ctx.is_json() {
                println!(
                    "{{\"id\":\"{}\",\"blocked\":true,\"domain\":\"{}\",\"missing_roles\":{}}}",
                    json_escape(&b.id),
                    json_escape(&b.domain),
                    json_str_array(b.missing_roles.iter().cloned())
                );
                return 0;
            }
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

    pub(crate) fn set(id: Option<String>, clear: bool, ctx: &Context) -> i32 {
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
            println!("{{\"active\":\"{}\"}}", json_escape(&id));
        } else {
            println!("archetype set: {id} (expectations re-scoped; staff unchanged)");
        }
        0
    }

    pub(crate) fn create(name: &str, from: &str, ctx: &Context) -> i32 {
        let catalog = ArchetypeCatalog::program();
        match catalog.create_from_preset(&state_dir(), name, from) {
            Ok(custom) => {
                if ctx.is_json() {
                    println!(
                        "{{\"id\":\"{}\",\"derived_from\":\"{}\"}}",
                        json_escape(&custom.definition.id),
                        json_escape(&custom.derived_from)
                    );
                } else {
                    println!(
                        "created custom archetype '{}' from preset '{}'",
                        custom.definition.id, custom.derived_from
                    );
                }
                0
            }
            Err(e) => {
                eprintln!("cronus archetype create: {e}");
                1
            }
        }
    }
}

// ─── dev ──────────────────────────────────────────────────────────────────────

pub(crate) mod dev_office_cmd {
    use std::path::{Path, PathBuf};

    use cronus_core::auth::{DeveloperAdmissionStore, HumanPrincipal};
    use cronus_core::dev_office::{AdmissionReader, AdmissionTier, DevOfficeGate, GateInputs};
    use cronus_core::dev_office_gate::{AuthLocalAdmissionReader, repo_authenticity};
    use cronus_core::paths::{Paths, Root};

    use crate::output::Context;

    /// The shipped default (DVO-5): the feedback tier is off. A build/deploy
    /// opt-in, not something this CLI exposes as a runtime flag.
    const FEEDBACK_TIER_ENABLED: bool = false;

    fn admission_path() -> PathBuf {
        Paths::os_native()
            .resolve(Root::State)
            .join("dev_office")
            .join("admission.txt")
    }

    fn resolve_tier(cwd: &Path) -> AdmissionTier {
        let repo = repo_authenticity(cwd);
        let reader = AuthLocalAdmissionReader::open(admission_path());
        let inputs = GateInputs {
            repo,
            admitted: reader.is_admitted(),
            feedback_tier_enabled: FEEDBACK_TIER_ENABLED,
        };
        DevOfficeGate::resolve(&inputs)
    }

    fn tier_label(tier: AdmissionTier) -> &'static str {
        match tier {
            AdmissionTier::Absent => "absent",
            AdmissionTier::Feedback => "feedback",
            AdmissionTier::Elevated => "elevated",
        }
    }

    // Reached directly from `crate::installation::dispatch` now — the
    // installation half's own generated grammar owns the three-verb `dev`
    // group, so no `DevCommand`-shaped wrapper is needed here any more.
    pub(crate) fn status(ctx: &Context) -> i32 {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let label = tier_label(resolve_tier(&cwd));
        if ctx.is_json() {
            println!("{{\"tier\":\"{label}\"}}");
        } else {
            println!("Developer office: {label}");
        }
        0
    }

    /// The CLI operator running this command themselves IS the DVO-3 human
    /// principal — a legitimate admission path, distinct from an *agent*
    /// self-granting through a tool call (no such tool call exists).
    pub(crate) fn admit(ctx: &Context) -> i32 {
        let path = admission_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match DeveloperAdmissionStore::open(path).mint(&HumanPrincipal::assert_human_operated()) {
            Ok(()) => {
                if ctx.is_json() {
                    println!("{{\"result\":\"admitted\"}}");
                } else {
                    println!("Developer office admission granted.");
                }
                0
            }
            Err(e) => {
                eprintln!("cronus dev admit: {e}");
                1
            }
        }
    }

    pub(crate) fn revoke(ctx: &Context) -> i32 {
        let path = admission_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match DeveloperAdmissionStore::open(path).revoke(&HumanPrincipal::assert_human_operated()) {
            Ok(()) => {
                if ctx.is_json() {
                    println!("{{\"result\":\"revoked\"}}");
                } else {
                    println!("Developer office admission revoked.");
                }
                0
            }
            Err(e) => {
                eprintln!("cronus dev revoke: {e}");
                1
            }
        }
    }

    // No unit tests here: every command function resolves the real,
    // un-overridable `Paths::os_native()` state directory (the `board`/
    // `knowledge_cmd` precedent) — `DevOfficeGate::resolve`/`repo_authenticity`/
    // `DeveloperAdmissionStore` are all thoroughly tested in `crates/domain`
    // and `crates/core`; this module's own job is arg-parsing/output-formatting
    // composition over that already-proven machinery, validated by a real CLI
    // smoke run instead.
}
