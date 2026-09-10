//! Execution workspace — isolated per-card environments with git worktree lifecycle.
//!
//! Slug naming: `<ws_id>-<card_id>-<unix_ts>`. No-remote-git contract enforced.
//! Finalize write-back gate defers to the quality pipeline seam (wiring deferred).

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecState {
    Active,
    Finalizing,
    Finalized,
    Discarded,
}

impl ExecState {
    pub fn as_str(&self) -> &'static str {
        match self {
            ExecState::Active => "active",
            ExecState::Finalizing => "finalizing",
            ExecState::Finalized => "finalized",
            ExecState::Discarded => "discarded",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExecWorkspace {
    pub id: String,
    pub card_id: String,
    pub ws_id: String,
    pub slug: String,
    pub path: PathBuf,
    pub state: ExecState,
    pub created_at: u64,
    pub finalized_at: Option<u64>,
}

#[derive(Debug)]
pub enum ExecError {
    AlreadyExists(String),
    NotFound(String),
    RemoteGitForbidden,
    GateNotPassed,
    /// A `ws_id` or `card_id` that is not a safe single path segment. The
    /// slug `<ws_id>-<card_id>-<ts>` becomes a directory name under the exec
    /// root, so a `..` or a separator would let a caller create a directory
    /// anywhere the process can write.
    InvalidId(String),
    Io(std::io::Error),
}

impl std::fmt::Display for ExecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecError::AlreadyExists(s) => write!(f, "exec workspace already exists: {s}"),
            ExecError::NotFound(s) => write!(f, "exec workspace not found: {s}"),
            ExecError::RemoteGitForbidden => {
                write!(f, "remote git operations are forbidden in exec workspaces")
            }
            ExecError::GateNotPassed => write!(f, "cannot finalize: quality gates have not passed"),
            ExecError::InvalidId(s) => {
                write!(f, "invalid id {s:?}: must be a single path segment")
            }
            ExecError::Io(e) => write!(f, "I/O error: {e}"),
        }
    }
}

/// Reject an id that would not stay inside the exec root as part of a
/// directory name — empty, `.`/`..`, or containing `/`, `\`, a NUL/control
/// byte, or a `..` run.
fn safe_segment(id: &str) -> bool {
    !id.is_empty()
        && id != "."
        && id != ".."
        && !id.contains('/')
        && !id.contains('\\')
        && !id.contains('\0')
        && !id.contains("..")
        && !id.chars().any(|c| c.is_control())
        && std::path::Path::new(id).components().count() == 1
        && !std::path::Path::new(id).is_absolute()
}

impl std::error::Error for ExecError {}
impl From<std::io::Error> for ExecError {
    fn from(e: std::io::Error) -> Self {
        ExecError::Io(e)
    }
}

pub type Result<T> = std::result::Result<T, ExecError>;

/// Slug format: `{ws_id}-{card_id}-{unix_ts}` (kebab, lowercase).
pub fn make_slug(ws_id: &str, card_id: &str, ts: u64) -> String {
    format!("{ws_id}-{card_id}-{ts}")
}

/// In-memory exec workspace manager (SQLite-backed in production).
#[derive(Debug, Default)]
pub struct ExecWorkspaceManager {
    workspaces: Vec<ExecWorkspace>,
}

impl ExecWorkspaceManager {
    pub fn new() -> Self {
        ExecWorkspaceManager::default()
    }

    pub fn create(
        &mut self,
        ws_id: &str,
        card_id: &str,
        base_dir: &Path,
        now: u64,
    ) -> Result<ExecWorkspace> {
        if !safe_segment(ws_id) {
            return Err(ExecError::InvalidId(ws_id.to_string()));
        }
        if !safe_segment(card_id) {
            return Err(ExecError::InvalidId(card_id.to_string()));
        }
        let slug = make_slug(ws_id, card_id, now);
        if self.workspaces.iter().any(|w| w.slug == slug) {
            return Err(ExecError::AlreadyExists(slug));
        }
        let path = base_dir.join(&slug);
        std::fs::create_dir_all(&path)?;
        let ws = ExecWorkspace {
            id: slug.clone(),
            card_id: card_id.to_string(),
            ws_id: ws_id.to_string(),
            slug,
            path,
            state: ExecState::Active,
            created_at: now,
            finalized_at: None,
        };
        self.workspaces.push(ws.clone());
        Ok(ws)
    }

    pub fn get(&self, id: &str) -> Option<&ExecWorkspace> {
        self.workspaces.iter().find(|w| w.id == id)
    }

    pub fn list(&self) -> &[ExecWorkspace] {
        &self.workspaces
    }

    /// Check if the worktree is pristine (no untracked or modified files).
    /// Current implementation: checks whether the directory is empty.
    pub fn is_pristine(&self, id: &str) -> Result<bool> {
        let ws = self
            .workspaces
            .iter()
            .find(|w| w.id == id)
            .ok_or_else(|| ExecError::NotFound(id.to_string()))?;

        let entries: Vec<_> = std::fs::read_dir(&ws.path)?.flatten().collect();
        Ok(entries.is_empty())
    }

    /// Attempt a remote git push — always denied (no-remote-git contract).
    pub fn git_push(&self, _id: &str) -> Result<()> {
        Err(ExecError::RemoteGitForbidden)
    }

    /// Finalize: checks quality gate seam, marks as Finalized.
    /// Gate seam returns Pass for now (real check wiring deferred).
    pub fn finalize(&mut self, id: &str, gate_passed: bool, now: u64) -> Result<()> {
        if !gate_passed {
            return Err(ExecError::GateNotPassed);
        }
        let ws = self
            .workspaces
            .iter_mut()
            .find(|w| w.id == id)
            .ok_or_else(|| ExecError::NotFound(id.to_string()))?;
        ws.state = ExecState::Finalized;
        ws.finalized_at = Some(now);
        Ok(())
    }

    /// Discard: remove the worktree directory and mark as Discarded.
    pub fn discard(&mut self, id: &str) -> Result<()> {
        let ws = self
            .workspaces
            .iter_mut()
            .find(|w| w.id == id)
            .ok_or_else(|| ExecError::NotFound(id.to_string()))?;
        if ws.path.exists() {
            std::fs::remove_dir_all(&ws.path)?;
        }
        ws.state = ExecState::Discarded;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp() -> PathBuf {
        std::env::temp_dir().join(format!(
            "cronus-execws-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ))
    }

    #[test]
    fn create_rejects_traversal_ids_and_writes_nothing_outside_the_root() {
        let root = tmp();
        std::fs::create_dir_all(&root).unwrap();
        let base = root.join("exec");
        let mut mgr = ExecWorkspaceManager::new();

        for (ws, card) in [
            ("../../../../pwned", "c"),
            ("ws", "../../pwned"),
            ("a/b", "c"),
            ("a\\b", "c"),
            ("", "c"),
            ("..", "c"),
        ] {
            let err = mgr.create(ws, card, &base, 1).unwrap_err();
            assert!(
                matches!(err, ExecError::InvalidId(_)),
                "{ws:?}/{card:?} must be rejected, got {err:?}"
            );
        }
        assert!(
            !root.parent().unwrap().join("pwned-c-1").exists(),
            "a traversal id must not create a directory outside the exec root"
        );

        // A clean pair still works.
        assert!(mgr.create("ws-1", "card-1", &base, 1).is_ok());
        let _ = std::fs::remove_dir_all(&root);
    }
}
