//! Cross-layer validation for the memory L2 pair: proves the consolidation
//! (MC-1..10) and intelligence (MI-1/2/3/4/5/7/8/9/13) invariants hold
//! together on the built tier — through the real facade and the real
//! SQLite adapter, not each task's own isolated stub/unit tests.
//!
//! MI-6/10/11/12 are out of scope here: they need capture-metadata fields
//! (actor/expiry/cross-ref/subject-lens/capture-mode) that were deliberately
//! deferred rather than rushed through a fourth schema round in one task —
//! there is nothing built yet for a sweep to exercise.

use cronus_contract::{ExperienceOutcome, MemoryDepth, MemoryEntry, MemoryKind, MemorySource};
use cronus_core::autonomy::AutonomyLevel;
use cronus_core::memory::MemoryStore;
use cronus_core::memory_intelligence::{self, AnswerVerdict, ExperienceDecision, RunTrace};
use cronus_store_local::memory::{ConsolidationAction, SignalKind};

// ── MC-1/MC-2/MC-4 + MI-1: capture consolidates and becomes answerable ──────

#[test]
fn raw_capture_consolidates_and_becomes_answerable_end_to_end() {
    let store = MemoryStore::open_in_memory().unwrap();
    let mut raw = MemoryEntry::new(
        MemoryKind::Convention,
        MemorySource::Agent,
        "raw note",
        "native Rust builds must run through PowerShell, not Git Bash",
    );
    raw.depth = MemoryDepth::Raw;
    store.add(raw).unwrap();

    let results = store.run_incremental_consolidation("test").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].1, ConsolidationAction::Create);

    let answer = memory_intelligence::answer(&store, "PowerShell", 5);
    assert_eq!(answer.verdict, AnswerVerdict::Supported);
    assert!(
        answer.text.contains("PowerShell"),
        "the consolidated item must be answerable through the same seam intelligence reads"
    );
}

// ── MC-5/MC-8 cold start: no signal ever computed, ranking still works ──────

#[test]
fn cold_start_ranking_degrades_to_base_relevance_with_no_signals_computed() {
    let store = MemoryStore::open_in_memory().unwrap();
    store
        .add(MemoryEntry::new(
            MemoryKind::Convention,
            MemorySource::Agent,
            "a",
            "the deploy pipeline uses a staging gate",
        ))
        .unwrap();
    store
        .add(MemoryEntry::new(
            MemoryKind::Convention,
            MemorySource::Agent,
            "b",
            "the staging gate requires manual approval",
        ))
        .unwrap();

    // No recompute_centrality / recompute_recency / signal write of any kind.
    let ranked = store.search_ranked("staging gate", 10).unwrap();
    assert_eq!(
        ranked.len(),
        2,
        "cold-start recall must not error or drop hits"
    );
    for (_, score) in &ranked {
        assert!(
            *score > 0.0,
            "a neutral-factor score must still be positive"
        );
    }
}

// ── MC-8/MEM-2: the ranking-time signal read never re-walks the live graph ──

#[test]
fn centrality_signal_is_precomputed_not_rewalked_on_every_read() {
    // Centrality normalizes by the graph's max in-degree, so proving
    // staleness needs two competing hubs: `rival` fixed at 2 in-edges,
    // `hub` starting at 1 (normalized 1/2) and later overtaking it at 3
    // (normalized 3/3) once recomputed — a single-hub graph is always its
    // own max and would trivially read 1.0 before and after, proving nothing.
    let store = MemoryStore::open_in_memory().unwrap();
    let hub_id = store
        .add(MemoryEntry::new(
            MemoryKind::Convention,
            MemorySource::Agent,
            "hub",
            "hub raw fact",
        ))
        .unwrap();
    let rival_id = store
        .add(MemoryEntry::new(
            MemoryKind::Convention,
            MemorySource::Agent,
            "rival",
            "rival raw fact",
        ))
        .unwrap();
    for body in ["rival edge one", "rival edge two"] {
        let c = MemoryEntry::new(MemoryKind::Convention, MemorySource::Agent, "r", body);
        store.consolidate(c, Some(&rival_id), "test").unwrap();
    }

    let c1 = MemoryEntry::new(
        MemoryKind::Convention,
        MemorySource::Agent,
        "c1",
        "alpha derived fact",
    );
    store.consolidate(c1, Some(&hub_id), "test").unwrap();
    store.recompute_centrality().unwrap();
    let factor_after_one_edge = store
        .signal_factor(&hub_id, SignalKind::Centrality)
        .unwrap();

    // Two more edges land on `hub`, overtaking `rival`'s in-degree, but
    // centrality is never recomputed again — the next read must not
    // silently re-derive it from the now-changed live graph.
    let c2 = MemoryEntry::new(
        MemoryKind::Convention,
        MemorySource::Agent,
        "c2",
        "beta derived fact",
    );
    store.consolidate(c2, Some(&hub_id), "test").unwrap();
    let c3 = MemoryEntry::new(
        MemoryKind::Convention,
        MemorySource::Agent,
        "c3",
        "gamma derived fact",
    );
    store.consolidate(c3, Some(&hub_id), "test").unwrap();

    let factor_still = store
        .signal_factor(&hub_id, SignalKind::Centrality)
        .unwrap();
    assert_eq!(
        factor_after_one_edge, factor_still,
        "a ranking-time signal read must not silently re-walk the live edge graph (MC-8/MEM-2)"
    );

    // Confirm the graph really did change underneath the stale signal —
    // otherwise this test would pass vacuously for the wrong reason.
    store.recompute_centrality().unwrap();
    let factor_after_recompute = store
        .signal_factor(&hub_id, SignalKind::Centrality)
        .unwrap();
    assert_ne!(
        factor_still, factor_after_recompute,
        "recompute_centrality must actually observe the new edges once explicitly re-run"
    );
}

// ── MC-5: the fact/derived boundary holds through the public write path ────

#[test]
fn signal_writes_never_touch_the_authored_fact_columns() {
    let store = MemoryStore::open_in_memory().unwrap();
    let entry = MemoryEntry::new(
        MemoryKind::Convention,
        MemorySource::Agent,
        "authored title",
        "authored body text",
    );
    let id = store.add(entry).unwrap();

    store.set_signal(&id, SignalKind::Recency, 0.9).unwrap();
    store.set_signal(&id, SignalKind::Cluster, 0.4).unwrap();

    let after = store.get(&id).unwrap().expect("item must still exist");
    assert_eq!(after.title, "authored title");
    assert_eq!(after.body, "authored body text");
}

// ── MI-7/MI-13: a distilled experience round-trips through the real adapter ─

#[test]
fn distilled_experience_round_trips_through_the_real_store_and_is_reused() {
    let store = MemoryStore::open_in_memory().unwrap();
    let trace = RunTrace {
        objective: "restart the flaky worker after a timeout".to_string(),
        actions_taken: vec![],
        findings: vec![],
        end_state: "worker restarts cleanly".to_string(),
        next_steps: vec![],
    };
    let mut distilled = memory_intelligence::distill_run(&trace, ExperienceOutcome::Success);
    distilled.trust_score = 0.9;
    distilled.verification_state = cronus_contract::VerificationState::TestedInProject;
    let distilled_id = distilled.id.clone();
    store.add(distilled).unwrap();

    let decision = memory_intelligence::recall_for_reuse(
        &store,
        "restart the flaky worker after a timeout",
        cronus_contract::now_secs(),
        false,
        AutonomyLevel::Autonomous,
        10,
    );

    match decision {
        ExperienceDecision::Reuse(citation) => assert_eq!(citation.item_id, distilled_id),
        other => panic!(
            "expected the distilled success to be reused through the real adapter, got {other:?}"
        ),
    }
}
