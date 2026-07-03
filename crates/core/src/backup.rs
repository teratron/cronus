//! Backup & restore (STO-1/2/5/6/7): copy the mutable state tier — minus
//! secrets and regenerable cache — to a self-contained destination, and drop
//! it back to resume from later. "Nothing extra": the program tier is never
//! backed up (it is reinstallable), and secrets never leave the device
//! through this path.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Top-level state-tier entries excluded from every backup by default
/// (§4.1): the secret file and regenerable cache. `logs` is excluded by
/// default too but is the one entry a caller may opt back in (§4.1 "optional").
const ALWAYS_EXCLUDED: &[&str] = &[".env", "cache"];
const LOGS_ENTRY: &str = "logs";

/// What to leave out of a backup, beyond the always-excluded secret file
/// and cache directory.
#[derive(Debug, Clone, Copy, Default)]
pub struct BackupOptions {
    /// `logs` is excluded by default; set true to include it anyway.
    pub include_logs: bool,
}

/// A reference to one backup: its id (also its directory name) and location.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupRef {
    pub id: String,
    pub path: PathBuf,
    pub created_at_unix: u64,
}

fn excluded_top_level_names(options: BackupOptions) -> Vec<&'static str> {
    let mut names: Vec<&'static str> = ALWAYS_EXCLUDED.to_vec();
    if !options.include_logs {
        names.push(LOGS_ENTRY);
    }
    names
}

/// Recursively copy `from` into `to`, skipping any entry whose *top-level*
/// name (relative to the original `from` root) is in `excluded`. Creates
/// `to` and any needed parent directories.
fn copy_tree_excluding(
    from: &Path,
    to: &Path,
    excluded: &[&str],
    is_top_level: bool,
) -> io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if is_top_level && excluded.contains(&name_str.as_ref()) {
            continue;
        }
        let src = entry.path();
        let dst = to.join(&name);
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            copy_tree_excluding(&src, &dst, excluded, false)?;
        } else if file_type.is_file() {
            fs::copy(&src, &dst)?;
        }
        // Symlinks are neither: skipped rather than followed, so a backup
        // never silently escapes the state tier through a link target.
    }
    Ok(())
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Back up `state_root` into a fresh timestamped directory under
/// `backups_dir` (or `dest` if given explicitly, `--to <path>`), excluding
/// secrets/cache/logs per `options`. Self-contained: nothing outside the
/// returned directory is needed to restore it.
pub fn create(
    state_root: &Path,
    backups_dir: &Path,
    dest: Option<&Path>,
    options: BackupOptions,
) -> io::Result<BackupRef> {
    let created_at_unix = unix_now();
    let id = format!("backup-{created_at_unix}");
    let target = dest
        .map(Path::to_path_buf)
        .unwrap_or_else(|| backups_dir.join(&id));

    let excluded = excluded_top_level_names(options);
    copy_tree_excluding(state_root, &target, &excluded, true)?;

    Ok(BackupRef {
        id,
        path: target,
        created_at_unix,
    })
}

/// List backups found directly under `backups_dir` (each is one directory,
/// named `backup-<unix-seconds>`), most recent first.
pub fn list(backups_dir: &Path) -> io::Result<Vec<BackupRef>> {
    if !backups_dir.exists() {
        return Ok(Vec::new());
    }
    let mut refs = Vec::new();
    for entry in fs::read_dir(backups_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(created_at_unix) = name.strip_prefix("backup-").and_then(|ts| ts.parse().ok())
        else {
            continue;
        };
        refs.push(BackupRef {
            id: name,
            path: entry.path(),
            created_at_unix,
        });
    }
    refs.sort_by_key(|backup_ref| std::cmp::Reverse(backup_ref.created_at_unix));
    Ok(refs)
}

/// Restore-by-copy (STO-7): drop a backup's contents into `dest`, ready for
/// the runtime to resume from. `dest` is created if missing; existing files
/// at colliding paths are overwritten (a fresh restore target is expected).
pub fn restore(backup: &BackupRef, dest: &Path) -> io::Result<()> {
    copy_tree_excluding(&backup.path, dest, &[], true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("cronus-backup-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write(path: &Path, contents: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, contents).unwrap();
    }

    /// A representative state tier: included content plus everything §4.1
    /// says must be excluded.
    fn seed_state_tier(root: &Path) {
        write(&root.join("config.json"), "{\"theme\":\"dark\"}");
        write(&root.join("AGENTS.md"), "# agents");
        write(&root.join("memory/notes.db"), "memory-bytes");
        write(&root.join("workspaces/acme/board.json"), "[]");
        write(&root.join(".env"), "API_KEY=super-secret-value");
        write(&root.join("cache/embeddings.bin"), "regenerable-bytes");
        write(&root.join("logs/daemon.log"), "log-line");
    }

    #[test]
    fn backup_includes_state_content_and_excludes_secrets_and_cache() {
        let state_root = temp_dir("state");
        let backups_dir = temp_dir("backups");
        seed_state_tier(&state_root);

        let backup_ref = create(&state_root, &backups_dir, None, BackupOptions::default()).unwrap();

        assert!(backup_ref.path.join("config.json").exists());
        assert!(backup_ref.path.join("AGENTS.md").exists());
        assert!(backup_ref.path.join("memory/notes.db").exists());
        assert!(backup_ref.path.join("workspaces/acme/board.json").exists());

        assert!(
            !backup_ref.path.join(".env").exists(),
            "secrets must never be backed up (STO-6)"
        );
        assert!(
            !backup_ref.path.join("cache").exists(),
            "regenerable cache is excluded"
        );
        assert!(
            !backup_ref.path.join("logs").exists(),
            "logs excluded by default"
        );

        let secret_leaked = fs::read_to_string(backup_ref.path.join("config.json"))
            .unwrap_or_default()
            .contains("super-secret-value");
        assert!(!secret_leaked);

        let _ = fs::remove_dir_all(&state_root);
        let _ = fs::remove_dir_all(&backups_dir);
    }

    #[test]
    fn logs_can_be_opted_back_in() {
        let state_root = temp_dir("state-logs");
        let backups_dir = temp_dir("backups-logs");
        seed_state_tier(&state_root);

        let backup_ref = create(
            &state_root,
            &backups_dir,
            None,
            BackupOptions { include_logs: true },
        )
        .unwrap();

        assert!(backup_ref.path.join("logs/daemon.log").exists());
        assert!(
            !backup_ref.path.join(".env").exists(),
            "secrets are never opted back in"
        );

        let _ = fs::remove_dir_all(&state_root);
        let _ = fs::remove_dir_all(&backups_dir);
    }

    #[test]
    fn explicit_destination_overrides_the_default_backups_directory() {
        let state_root = temp_dir("state-dest");
        let backups_dir = temp_dir("backups-dest");
        let chosen = temp_dir("chosen-dest");
        seed_state_tier(&state_root);

        let backup_ref = create(
            &state_root,
            &backups_dir,
            Some(&chosen),
            BackupOptions::default(),
        )
        .unwrap();
        assert_eq!(backup_ref.path, chosen);
        assert!(backup_ref.path.join("config.json").exists());

        let _ = fs::remove_dir_all(&state_root);
        let _ = fs::remove_dir_all(&backups_dir);
        let _ = fs::remove_dir_all(&chosen);
    }

    #[test]
    fn list_returns_backups_most_recent_first() {
        let state_root = temp_dir("state-list");
        let backups_dir = temp_dir("backups-list");
        seed_state_tier(&state_root);

        let older = BackupRef {
            id: "backup-1000".into(),
            path: backups_dir.join("backup-1000"),
            created_at_unix: 1000,
        };
        let newer = BackupRef {
            id: "backup-2000".into(),
            path: backups_dir.join("backup-2000"),
            created_at_unix: 2000,
        };
        fs::create_dir_all(&older.path).unwrap();
        fs::create_dir_all(&newer.path).unwrap();
        // A non-backup directory must be ignored, not misparsed.
        fs::create_dir_all(backups_dir.join("not-a-backup")).unwrap();

        let listed = list(&backups_dir).unwrap();
        assert_eq!(listed.len(), 2);
        assert_eq!(listed[0].id, "backup-2000", "most recent first");
        assert_eq!(listed[1].id, "backup-1000");

        let _ = fs::remove_dir_all(&state_root);
        let _ = fs::remove_dir_all(&backups_dir);
    }

    #[test]
    fn listing_a_missing_backups_directory_returns_empty_not_an_error() {
        let missing =
            std::env::temp_dir().join(format!("cronus-backup-missing-{}", std::process::id()));
        let _ = fs::remove_dir_all(&missing);
        assert_eq!(list(&missing).unwrap(), Vec::new());
    }

    #[test]
    fn restore_round_trips_a_backup_into_a_resumable_state_tier() {
        let state_root = temp_dir("state-rt");
        let backups_dir = temp_dir("backups-rt");
        let restore_target = temp_dir("restored-rt");
        seed_state_tier(&state_root);

        let backup_ref = create(&state_root, &backups_dir, None, BackupOptions::default()).unwrap();
        restore(&backup_ref, &restore_target).unwrap();

        assert_eq!(
            fs::read_to_string(restore_target.join("config.json")).unwrap(),
            fs::read_to_string(state_root.join("config.json")).unwrap()
        );
        assert!(restore_target.join("memory/notes.db").exists());
        assert!(restore_target.join("workspaces/acme/board.json").exists());
        assert!(
            !restore_target.join(".env").exists(),
            "a restored tier still carries no secrets"
        );

        let _ = fs::remove_dir_all(&state_root);
        let _ = fs::remove_dir_all(&backups_dir);
        let _ = fs::remove_dir_all(&restore_target);
    }

    #[test]
    fn a_backup_is_self_contained_and_restorable_after_the_source_is_gone() {
        let state_root = temp_dir("state-sc");
        let backups_dir = temp_dir("backups-sc");
        let restore_target = temp_dir("restored-sc");
        seed_state_tier(&state_root);

        let backup_ref = create(&state_root, &backups_dir, None, BackupOptions::default()).unwrap();
        fs::remove_dir_all(&state_root).unwrap(); // the original state tier is gone

        restore(&backup_ref, &restore_target).unwrap();
        assert!(
            restore_target.join("config.json").exists(),
            "the backup needed nothing outside itself"
        );

        let _ = fs::remove_dir_all(&backups_dir);
        let _ = fs::remove_dir_all(&restore_target);
    }
}
