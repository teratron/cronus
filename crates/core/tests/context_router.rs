use cronus_core::{
    context_router::{ContextRouter, RuleEntry, RulePrecedence, SessionAction, SessionContext},
    memory::MemoryStore,
};
use std::path::PathBuf;

fn store() -> MemoryStore {
    MemoryStore::open_in_memory().expect("in-memory store must open")
}

fn nonexistent_dir() -> PathBuf {
    std::env::temp_dir().join("cronus-ctx-router-test-nonexistent-xyz")
}

#[test]
fn empty_bundle_when_no_memories_or_rules() {
    let s = store();
    let dir = nonexistent_dir();
    let router = ContextRouter::new(&s, &dir, &dir);
    let bundle = router.assemble("anything", 10).unwrap();
    assert!(bundle.is_empty());
}

#[test]
fn memory_entries_included_in_bundle() {
    use cronus_core::memory::{MemoryEntry, MemoryKind, MemorySource};
    let s = store();
    let e = MemoryEntry::new(
        MemoryKind::Convention,
        MemorySource::Agent,
        "FTS test",
        "context router keyword",
    );
    s.add(e).unwrap();

    let dir = nonexistent_dir();
    let router = ContextRouter::new(&s, &dir, &dir);
    let bundle = router.assemble("keyword", 10).unwrap();
    assert_eq!(bundle.memories.len(), 1);
    assert_eq!(bundle.memories[0].title, "FTS test");
}

#[test]
fn session_continue_attaches_context() {
    let s = store();
    let dir = nonexistent_dir();
    let router = ContextRouter::new(&s, &dir, &dir);
    let bundle = router.assemble("query", 5).unwrap();
    let ctx = SessionContext {
        session_id: "s1".into(),
        entries: vec!["hello".into()],
    };
    let bundle = router.with_session(bundle, ctx, SessionAction::Continue);
    assert!(bundle.session.is_some());
    assert_eq!(bundle.session.unwrap().session_id, "s1");
}

#[test]
fn session_retire_clears_context() {
    let s = store();
    let dir = nonexistent_dir();
    let router = ContextRouter::new(&s, &dir, &dir);
    let bundle = router.assemble("query", 5).unwrap();
    let ctx = SessionContext {
        session_id: "s2".into(),
        entries: vec![],
    };
    let bundle = router.with_session(bundle, ctx, SessionAction::Retire);
    assert!(bundle.session.is_none());
}

#[test]
fn rules_workspace_precedence_is_highest() {
    let a = RuleEntry {
        path: PathBuf::from("a"),
        body: String::new(),
        precedence: RulePrecedence::Workspace,
    };
    let b = RuleEntry {
        path: PathBuf::from("b"),
        body: String::new(),
        precedence: RulePrecedence::Global,
    };
    assert!(a.precedence > b.precedence);
}

#[test]
fn rules_project_precedence_between_workspace_and_global() {
    assert!(RulePrecedence::Project > RulePrecedence::Global);
    assert!(RulePrecedence::Workspace > RulePrecedence::Project);
}

#[test]
fn bundle_session_starts_none() {
    let s = store();
    let dir = nonexistent_dir();
    let router = ContextRouter::new(&s, &dir, &dir);
    let bundle = router.assemble("query", 5).unwrap();
    assert!(bundle.session.is_none());
}
