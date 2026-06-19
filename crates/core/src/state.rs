//! State-tier bootstrap: create the mutable state skeleton, idempotently.
//!
//! Only the state tier is written; the program tier is never touched (STO-1).
//! Secrets are seeded as `.env.example` only — never `.env` (STO-6).

use crate::paths::{Paths, Root};
use std::fs;
use std::io;
use std::path::Path;

/// Directories created in a fresh state tier (per the filesystem layout).
const STATE_DIRS: &[&str] = &[
    "memory/notes",
    "skills",
    "employees",
    "workspaces",
    "backups",
];

/// Files seeded if absent: (relative name, contents).
const STATE_FILES: &[(&str, &str)] = &[
    (
        "app.json",
        "{\n  \"locale\": \"en\",\n  \"theme\": \"system\"\n}\n",
    ),
    ("config.json", "{}\n"),
    (
        ".env.example",
        "# Local secrets — copy to .env (never committed).\n",
    ),
    ("AGENTS.md", "# Agents Instructions\n"),
];

/// Bootstrap the state tier at the resolved `Root::State`.
pub fn bootstrap(paths: &Paths) -> io::Result<()> {
    bootstrap_at(&paths.resolve(Root::State))
}

/// Bootstrap the state tier at an explicit root. Idempotent: existing files are kept.
pub fn bootstrap_at(root: &Path) -> io::Result<()> {
    fs::create_dir_all(root)?;
    for dir in STATE_DIRS {
        fs::create_dir_all(root.join(dir))?;
    }
    for (name, contents) in STATE_FILES {
        let path = root.join(name);
        if !path.exists() {
            fs::write(&path, contents)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(tag: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("cronus-test-{tag}-{}", std::process::id()))
    }

    #[test]
    fn bootstrap_creates_tree_and_is_idempotent() {
        let root = temp_root("bootstrap");
        let _ = fs::remove_dir_all(&root);

        bootstrap_at(&root).expect("first bootstrap");
        assert!(root.join("memory/notes").is_dir());
        assert!(root.join("workspaces").is_dir());
        assert!(root.join(".env.example").is_file());
        assert!(!root.join(".env").exists(), "must never seed a real .env");

        // Mutate a file, then re-run: an idempotent run must not overwrite it.
        fs::write(root.join("config.json"), "{\"keep\":true}\n").unwrap();
        bootstrap_at(&root).expect("second bootstrap");
        let kept = fs::read_to_string(root.join("config.json")).unwrap();
        assert!(kept.contains("keep"), "existing files preserved");

        let _ = fs::remove_dir_all(&root);
    }
}
