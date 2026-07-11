use cronus_core::exec_workspace::{ExecError, ExecState, ExecWorkspaceManager, make_slug};
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn tmp_base(label: &str) -> std::path::PathBuf {
    let pid = std::process::id();
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    let p = std::env::temp_dir().join(format!("cronus-exec-{pid}-{id}-{label}"));
    std::fs::create_dir_all(&p).unwrap();
    p
}

// ── make_slug ────────────────────────────────────────────────────────────────

#[test]
fn slug_contains_all_three_components() {
    let slug = make_slug("ws1", "card42", 1000);
    assert_eq!(slug, "ws1-card42-1000");
}

#[test]
fn slug_is_kebab_separated() {
    let slug = make_slug("ws", "card", 99);
    assert!(slug.contains('-'));
    // format is {ws_id}-{card_id}-{ts}: exactly 2 separator dashes
    assert_eq!(slug.matches('-').count(), 2);
}

// ── create / get / list ──────────────────────────────────────────────────────

#[test]
fn create_workspace_returns_active_state() {
    let base = tmp_base("create");
    let mut mgr = ExecWorkspaceManager::new();
    let ws = mgr.create("ws", "card1", &base, 100).unwrap();
    assert_eq!(ws.state, ExecState::Active);
    assert_eq!(ws.ws_id, "ws");
    assert_eq!(ws.card_id, "card1");
}

#[test]
fn create_workspace_directory_exists_on_disk() {
    let base = tmp_base("dir-exists");
    let mut mgr = ExecWorkspaceManager::new();
    let ws = mgr.create("ws", "c1", &base, 200).unwrap();
    assert!(
        ws.path.exists(),
        "workspace directory must be created on disk"
    );
}

#[test]
fn get_returns_workspace_by_id() {
    let base = tmp_base("get");
    let mut mgr = ExecWorkspaceManager::new();
    let ws = mgr.create("ws", "c1", &base, 300).unwrap();
    let fetched = mgr.get(&ws.id).unwrap();
    assert_eq!(fetched.id, ws.id);
}

#[test]
fn get_returns_none_for_unknown_id() {
    let mgr = ExecWorkspaceManager::new();
    assert!(mgr.get("no-such-id").is_none());
}

#[test]
fn list_returns_all_workspaces() {
    let base = tmp_base("list");
    let mut mgr = ExecWorkspaceManager::new();
    mgr.create("ws", "c1", &base, 1).unwrap();
    mgr.create("ws", "c2", &base, 2).unwrap();
    assert_eq!(mgr.list().len(), 2);
}

#[test]
fn duplicate_slug_returns_already_exists_error() {
    let base = tmp_base("dup");
    let mut mgr = ExecWorkspaceManager::new();
    mgr.create("ws", "c1", &base, 42).unwrap();
    let result = mgr.create("ws", "c1", &base, 42);
    assert!(matches!(result, Err(ExecError::AlreadyExists(_))));
}

// ── is_pristine ──────────────────────────────────────────────────────────────

#[test]
fn new_workspace_is_pristine() {
    let base = tmp_base("pristine");
    let mut mgr = ExecWorkspaceManager::new();
    let ws = mgr.create("ws", "c1", &base, 500).unwrap();
    assert!(mgr.is_pristine(&ws.id).unwrap());
}

#[test]
fn workspace_with_file_is_not_pristine() {
    let base = tmp_base("dirty");
    let mut mgr = ExecWorkspaceManager::new();
    let ws = mgr.create("ws", "c1", &base, 600).unwrap();
    std::fs::write(ws.path.join("output.txt"), "data").unwrap();
    assert!(!mgr.is_pristine(&ws.id).unwrap());
}

// ── git_push (no-remote-git contract) ───────────────────────────────────────

#[test]
fn git_push_always_returns_forbidden() {
    let base = tmp_base("git-push");
    let mut mgr = ExecWorkspaceManager::new();
    let ws = mgr.create("ws", "c1", &base, 700).unwrap();
    let result = mgr.git_push(&ws.id);
    assert!(matches!(result, Err(ExecError::RemoteGitForbidden)));
}

// ── finalize ─────────────────────────────────────────────────────────────────

#[test]
fn finalize_marks_workspace_as_finalized() {
    let base = tmp_base("finalize");
    let mut mgr = ExecWorkspaceManager::new();
    let ws = mgr.create("ws", "c1", &base, 800).unwrap();
    mgr.finalize(&ws.id, true, 900).unwrap();
    let fetched = mgr.get(&ws.id).unwrap();
    assert_eq!(fetched.state, ExecState::Finalized);
    assert_eq!(fetched.finalized_at, Some(900));
}

#[test]
fn finalize_fails_when_gate_not_passed() {
    let base = tmp_base("gate-fail");
    let mut mgr = ExecWorkspaceManager::new();
    let ws = mgr.create("ws", "c1", &base, 100).unwrap();
    let result = mgr.finalize(&ws.id, false, 200);
    assert!(matches!(result, Err(ExecError::GateNotPassed)));
}

// ── discard ──────────────────────────────────────────────────────────────────

#[test]
fn discard_marks_as_discarded_and_removes_directory() {
    let base = tmp_base("discard");
    let mut mgr = ExecWorkspaceManager::new();
    let ws = mgr.create("ws", "c1", &base, 100).unwrap();
    let path = ws.path.clone();
    mgr.discard(&ws.id).unwrap();
    assert!(!path.exists(), "directory should be removed on discard");
    assert_eq!(mgr.get(&ws.id).unwrap().state, ExecState::Discarded);
}
