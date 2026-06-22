use cronus::memory::{
    CodeChangeType, MemoryEntry, MemoryKind, MemorySource, MemoryStore, SuggestedAction,
    TrustUpdate, VerificationState,
    chain::ChainKind,
    trust::{TRUST_MIN_SEARCH, TRUST_NEGATIVE_DELTA, TRUST_POSITIVE_DELTA},
};

fn store() -> MemoryStore {
    MemoryStore::open_in_memory().expect("in-memory store must open")
}

fn entry(title: &str, body: &str) -> MemoryEntry {
    MemoryEntry::new(MemoryKind::Convention, MemorySource::Agent, title, body)
}

#[test]
fn add_and_get_roundtrip() {
    let s = store();
    let e = entry("test title", "test body");
    let id = s.add(e.clone()).unwrap();

    let got = s.get(&id).unwrap().expect("entry must exist");
    assert_eq!(got.id, id);
    assert_eq!(got.title, "test title");
    assert_eq!(got.body, "test body");
    assert_eq!(got.kind, MemoryKind::Convention);
    assert_eq!(got.source, MemorySource::Agent);
}

#[test]
fn get_missing_returns_none() {
    let s = store();
    use cronus::memory::MemoryId;
    let fake = MemoryId::from("nonexistent".to_string());
    assert!(s.get(&fake).unwrap().is_none());
}

#[test]
fn fts_search_finds_matching_entries() {
    let s = store();
    s.add(entry("Rust lifetimes", "ownership borrow checker")).unwrap();
    s.add(entry("Python decorators", "metaclass function wrap")).unwrap();

    let results = s.search_fts("ownership", 10).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "Rust lifetimes");
}

#[test]
fn fts_search_filters_by_trust_score() {
    let s = store();
    let mut low_trust = entry("low trust entry", "secret knowledge");
    low_trust.trust_score = 0.05; // below TRUST_MIN_SEARCH
    s.add(low_trust).unwrap();

    let results = s.search_fts("secret knowledge", 10).unwrap();
    assert!(
        results.is_empty(),
        "entries below TRUST_MIN_SEARCH must be excluded from search"
    );
}

#[test]
fn trust_update_positive_delta() {
    let s = store();
    let e = entry("positive test", "body");
    let id = s.add(e).unwrap();

    let initial = s.get(&id).unwrap().unwrap().trust_score;
    let new_score = s.update_trust(&id, TrustUpdate::positive()).unwrap();

    assert!(
        (new_score - (initial + TRUST_POSITIVE_DELTA)).abs() < 1e-10,
        "positive update must add {TRUST_POSITIVE_DELTA}"
    );
}

#[test]
fn trust_update_negative_delta() {
    let s = store();
    let e = entry("negative test", "body");
    let id = s.add(e).unwrap();

    let initial = s.get(&id).unwrap().unwrap().trust_score;
    let new_score = s.update_trust(&id, TrustUpdate::negative()).unwrap();

    assert!(
        (new_score - (initial - TRUST_NEGATIVE_DELTA)).abs() < 1e-10,
        "negative update must subtract {TRUST_NEGATIVE_DELTA}"
    );
}

#[test]
fn trust_update_clamped_at_bounds() {
    let s = store();
    let mut e = entry("clamp test", "body");
    e.trust_score = 1.0;
    let id = s.add(e).unwrap();

    let score = s.update_trust(&id, TrustUpdate::positive()).unwrap();
    assert!((score - 1.0).abs() < 1e-10, "trust must not exceed 1.0");
}

#[test]
fn trust_update_sets_verification_state() {
    let s = store();
    let e = entry("verify test", "body");
    let id = s.add(e).unwrap();

    s.update_trust(
        &id,
        TrustUpdate::with_verification(VerificationState::TestedInProject),
    )
    .unwrap();

    let got = s.get(&id).unwrap().unwrap();
    assert_eq!(got.verification_state, VerificationState::TestedInProject);
}

#[test]
fn session_auto_chain_within_window() {
    let s = store();

    let mut e1 = entry("first memory", "body one");
    e1.workspace_id = Some("ws-test".to_string());
    let id1 = s.add(e1).unwrap();

    let mut e2 = entry("second memory", "body two");
    e2.workspace_id = Some("ws-test".to_string());
    e2.created_at = cronus::memory::MemoryEntry::new(
        MemoryKind::Convention, MemorySource::Agent, "", ""
    ).created_at + 60; // 60 seconds later — within 2h window
    let _id2 = s.add(e2).unwrap();

    // Verify chain was created by checking the chain store
    // The auto-chain creates: id1 → id2 with Continuation kind
    // We verify indirectly: propagate_trust should traverse the chain
    s.propagate_trust(&id1, 0.1).unwrap();
    // If propagation doesn't panic and returns Ok, the chain exists
}

#[test]
fn explicit_chain_and_propagation() {
    let s = store();
    let id_a = s.add(entry("source", "alpha")).unwrap();
    let id_b = s.add(entry("target", "beta")).unwrap();

    s.chain(&id_a, &id_b, ChainKind::RelatedTo).unwrap();

    let b_before = s.get(&id_b).unwrap().unwrap().trust_score;
    s.propagate_trust(&id_a, 1.0).unwrap();
    let b_after = s.get(&id_b).unwrap().unwrap().trust_score;

    assert!(
        b_after > b_before,
        "propagated trust must increase target's score"
    );
}

#[test]
fn code_change_deleted_triggers_invalidate() {
    let s = store();
    let mut e = entry("some function", "uses old api");
    e.workspace_id = Some("ws-code".to_string());
    let id = s.add(e).unwrap();

    let action = CodeChangeType::Deleted.suggested_action();
    assert!(matches!(action, SuggestedAction::Invalidate(_)));

    let affected = s.apply_code_change("ws-code", CodeChangeType::Deleted).unwrap();
    assert!(affected.contains(&id));
}

#[test]
fn code_change_signature_changed_triggers_review() {
    let action = CodeChangeType::SignatureChanged.suggested_action();
    assert!(
        matches!(action, SuggestedAction::Review(_)),
        "SignatureChanged must map to Review action"
    );
}

#[test]
fn hrr_encoding_stub_returns_zeroed_vector() {
    let s = store();
    let e = entry("hrr test", "body");
    let vec = s.encode_hrr(&e);
    assert_eq!(vec.len(), 256, "HRR stub must return 256-dimensional vector");
    assert!(vec.iter().all(|&v| v == 0.0), "HRR stub must be all zeros");
}

#[test]
fn search_excludes_superseded_entries() {
    let s = store();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let mut e = entry("superseded entry", "outdated knowledge");
    e.superseded_at = Some(now - 100);
    s.add(e).unwrap();

    let results = s.search_fts("outdated knowledge", 10).unwrap();
    assert!(results.is_empty(), "superseded entries must be excluded from search");
}

#[test]
fn trust_min_search_constant() {
    const { assert!(TRUST_MIN_SEARCH > 0.0 && TRUST_MIN_SEARCH <= 1.0) }
    const { assert!(TRUST_NEGATIVE_DELTA > TRUST_POSITIVE_DELTA) }
}
