use std::path::Path;

use cronus::workspace::{WorkspaceId, WorkspaceManager, WorkspaceTemplate};

fn mgr() -> WorkspaceManager {
    WorkspaceManager::open_in_memory().expect("in-memory workspace manager must open")
}

// ── WorkspaceId validation ────────────────────────────────────────────────────

#[test]
fn valid_ids_accepted() {
    let ok = ["my-project", "project123", "a0", "x-y-z", "abc"];
    for id in ok {
        assert!(WorkspaceId::new(id).is_ok(), "'{id}' should be valid");
    }
}

#[test]
fn invalid_ids_rejected() {
    let bad = [
        ("a", "too short"),
        ("-start", "starts with dash"),
        ("end-", "ends with dash"),
        ("UPPER", "uppercase"),
        ("has space", "contains space"),
        ("has_underscore", "contains underscore"),
    ];
    for (id, reason) in bad {
        assert!(WorkspaceId::new(id).is_err(), "'{id}' should be invalid: {reason}");
    }
}

#[test]
fn max_length_id_accepted() {
    let id = "a".repeat(32) + "-" + &"b".repeat(31);
    assert!(id.len() == 64);
    assert!(WorkspaceId::new(id).is_ok());
}

#[test]
fn over_max_length_rejected() {
    let id = "a".repeat(65);
    assert!(WorkspaceId::new(id).is_err());
}

// ── CRUD ─────────────────────────────────────────────────────────────────────

#[test]
fn create_and_retrieve() {
    let m = mgr();
    let id = WorkspaceId::new("my-ws").unwrap();
    let ws = m.create(&id, "My WS", Path::new("/projects/my"), WorkspaceTemplate::Default).unwrap();

    assert_eq!(ws.id, id);
    assert_eq!(ws.name, "My WS");
    assert_eq!(ws.template, "Default");

    let got = m.get(&id).unwrap().expect("workspace must exist after create");
    assert_eq!(got.id, id);
    assert_eq!(got.name, "My WS");
}

#[test]
fn get_missing_returns_none() {
    let m = mgr();
    let id = WorkspaceId::new("ghost").unwrap();
    assert!(m.get(&id).unwrap().is_none());
}

#[test]
fn duplicate_create_returns_already_exists() {
    let m = mgr();
    let id = WorkspaceId::new("dup-ws").unwrap();
    m.create(&id, "First", Path::new("/a"), WorkspaceTemplate::Empty).unwrap();
    let err = m.create(&id, "Second", Path::new("/b"), WorkspaceTemplate::Default);
    assert!(
        matches!(err, Err(cronus::workspace::WorkspaceError::AlreadyExists(_))),
        "duplicate create must return AlreadyExists"
    );
}

#[test]
fn list_empty_when_no_workspaces() {
    let m = mgr();
    assert!(m.list().unwrap().is_empty());
}

#[test]
fn list_returns_all() {
    let m = mgr();
    for i in 0..5u8 {
        let id = WorkspaceId::new(format!("ws-{i}")).unwrap();
        m.create(&id, &format!("WS {i}"), Path::new("/tmp"), WorkspaceTemplate::Empty).unwrap();
    }
    assert_eq!(m.list().unwrap().len(), 5);
}

#[test]
fn delete_returns_true_on_existing() {
    let m = mgr();
    let id = WorkspaceId::new("to-del").unwrap();
    m.create(&id, "Del", Path::new("/d"), WorkspaceTemplate::Default).unwrap();
    assert!(m.delete(&id).unwrap());
    assert!(m.get(&id).unwrap().is_none());
}

#[test]
fn delete_returns_false_on_missing() {
    let m = mgr();
    let id = WorkspaceId::new("no-such").unwrap();
    assert!(!m.delete(&id).unwrap());
}

// ── Active workspace ──────────────────────────────────────────────────────────

#[test]
fn no_active_initially() {
    let m = mgr();
    assert!(m.get_active().unwrap().is_none());
}

#[test]
fn set_and_get_active() {
    let m = mgr();
    let id = WorkspaceId::new("active-one").unwrap();
    m.create(&id, "Active One", Path::new("/a"), WorkspaceTemplate::Default).unwrap();
    m.set_active(&id).unwrap();
    let active = m.get_active().unwrap().expect("active workspace must be set");
    assert_eq!(active, id);
}

#[test]
fn set_active_is_idempotent() {
    let m = mgr();
    let id = WorkspaceId::new("idem-ws").unwrap();
    m.create(&id, "Idem", Path::new("/i"), WorkspaceTemplate::Default).unwrap();
    m.set_active(&id).unwrap();
    m.set_active(&id).unwrap(); // second call must not fail
    assert_eq!(m.get_active().unwrap().unwrap(), id);
}

#[test]
fn set_active_switches_between_workspaces() {
    let m = mgr();
    let id_a = WorkspaceId::new("ws-alpha").unwrap();
    let id_b = WorkspaceId::new("ws-beta").unwrap();
    m.create(&id_a, "Alpha", Path::new("/a"), WorkspaceTemplate::Default).unwrap();
    m.create(&id_b, "Beta", Path::new("/b"), WorkspaceTemplate::Default).unwrap();

    m.set_active(&id_a).unwrap();
    assert_eq!(m.get_active().unwrap().unwrap(), id_a);

    m.set_active(&id_b).unwrap();
    assert_eq!(m.get_active().unwrap().unwrap(), id_b);
}

#[test]
fn set_active_nonexistent_fails() {
    let m = mgr();
    let id = WorkspaceId::new("ghost-ws").unwrap();
    assert!(
        matches!(m.set_active(&id), Err(cronus::workspace::WorkspaceError::NotFound(_))),
        "set_active on nonexistent workspace must return NotFound"
    );
}

#[test]
fn delete_clears_active() {
    let m = mgr();
    let id = WorkspaceId::new("will-vanish").unwrap();
    m.create(&id, "Vanish", Path::new("/v"), WorkspaceTemplate::Default).unwrap();
    m.set_active(&id).unwrap();
    m.delete(&id).unwrap();
    // Active should now be empty
    assert!(m.get_active().unwrap().is_none());
}

// ── Check ─────────────────────────────────────────────────────────────────────

#[test]
fn check_existing_workspace() {
    let m = mgr();
    let id = WorkspaceId::new("chk-ws").unwrap();
    m.create(&id, "Check", Path::new("/tmp"), WorkspaceTemplate::Default).unwrap();
    m.set_active(&id).unwrap();

    let status = m.check(&id).unwrap();
    assert!(status.exists, "workspace must exist");
    assert!(status.is_active, "workspace must be active");
}

#[test]
fn check_nonexistent_workspace() {
    let m = mgr();
    let id = WorkspaceId::new("absent").unwrap();
    let status = m.check(&id).unwrap();
    assert!(!status.exists);
    assert!(!status.is_active);
    assert!(!status.path_exists);
}

// ── Template ─────────────────────────────────────────────────────────────────

#[test]
fn template_stored_correctly() {
    let m = mgr();
    let id = WorkspaceId::new("tmpl-ws").unwrap();
    m.create(&id, "Tmpl", Path::new("/t"), WorkspaceTemplate::Empty).unwrap();
    let ws = m.get(&id).unwrap().unwrap();
    assert_eq!(ws.template, "Empty");
}
