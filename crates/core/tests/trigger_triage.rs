use cronus_core::trigger_triage::{
    CONFIDENCE_THRESHOLD, ContentHash, DeduplicateCache, RateLimiter, SourceType, TriageDecision,
    TriggerEnvelope, TriggerPayload, rule_classify, triage,
};

fn make_envelope(id: &str, source: SourceType, excerpt: &str) -> TriggerEnvelope {
    TriggerEnvelope::new(id, source, TriggerPayload::new(excerpt), 1_000_000, "ws1")
}

fn make_event(id: &str, kind: &str) -> TriggerEnvelope {
    TriggerEnvelope::new(
        id,
        SourceType::Event,
        TriggerPayload::new("something happened").with_event_kind(kind),
        1_000_000,
        "ws1",
    )
}

// ── Rule classifier ───────────────────────────────────────────────────────────

#[test]
fn cron_source_becomes_spawn_reactor() {
    let env = make_envelope("e1", SourceType::Cron, "run backup");
    let decision = rule_classify(&env);
    assert!(
        matches!(decision, TriageDecision::SpawnReactor { .. }),
        "cron should produce SpawnReactor, got {:?}",
        decision
    );
}

#[test]
fn event_with_error_kind_becomes_spawn_orchestrator() {
    let env = make_event("e2", "service.error");
    let decision = rule_classify(&env);
    assert!(
        matches!(decision, TriageDecision::SpawnOrchestrator { .. }),
        "error event should produce SpawnOrchestrator, got {:?}",
        decision
    );
}

#[test]
fn event_with_fail_kind_becomes_spawn_orchestrator() {
    let env = make_event("e3", "pipeline.fail");
    let decision = rule_classify(&env);
    assert!(matches!(decision, TriageDecision::SpawnOrchestrator { .. }));
}

#[test]
fn event_without_error_kind_becomes_notify() {
    let env = make_event("e4", "deployment.started");
    let decision = rule_classify(&env);
    assert!(matches!(decision, TriageDecision::Notify { .. }));
}

#[test]
fn chat_message_becomes_spawn_reactor() {
    let env = make_envelope("e5", SourceType::ChatMessage, "help me");
    assert!(matches!(
        rule_classify(&env),
        TriageDecision::SpawnReactor { .. }
    ));
}

#[test]
fn webhook_becomes_spawn_reactor() {
    let env = make_envelope("e6", SourceType::Webhook, "payload data");
    assert!(matches!(
        rule_classify(&env),
        TriageDecision::SpawnReactor { .. }
    ));
}

#[test]
fn sub_agent_spawn_becomes_spawn_reactor() {
    let env = make_envelope("e7", SourceType::SubAgentSpawn, "child task");
    assert!(matches!(
        rule_classify(&env),
        TriageDecision::SpawnReactor { .. }
    ));
}

// ── Dedup cache ───────────────────────────────────────────────────────────────

#[test]
fn duplicate_event_within_window_is_dropped() {
    let mut cache = DeduplicateCache::new();
    let env = make_envelope("e8", SourceType::Event, "duplicate signal");
    cache.record(&env, 1000);
    // Same payload within the 60-second window
    assert!(cache.is_duplicate(&env, 1030));
}

#[test]
fn duplicate_event_outside_window_is_allowed() {
    let mut cache = DeduplicateCache::new();
    let env = make_envelope("e9", SourceType::Event, "repeat signal");
    cache.record(&env, 1000);
    // 61 seconds later — outside the 60s window
    assert!(!cache.is_duplicate(&env, 1061));
}

#[test]
fn cron_bypasses_dedup_window_is_zero() {
    let mut cache = DeduplicateCache::new();
    let env = make_envelope("e10", SourceType::Cron, "schedule run");
    cache.record(&env, 1000);
    // Cron has dedup window = 0, so never a duplicate
    assert!(!cache.is_duplicate(&env, 1001));
}

#[test]
fn different_payloads_not_deduplicated() {
    let mut cache = DeduplicateCache::new();
    let env_a = make_envelope("e11", SourceType::Event, "signal A");
    let env_b = make_envelope("e12", SourceType::Event, "signal B");
    cache.record(&env_a, 1000);
    assert!(!cache.is_duplicate(&env_b, 1005));
}

// ── ContentHash ───────────────────────────────────────────────────────────────

#[test]
fn content_hash_same_payload_is_equal() {
    let p1 = TriggerPayload::new("identical excerpt");
    let p2 = TriggerPayload::new("identical excerpt");
    assert_eq!(
        ContentHash::from_payload(&p1),
        ContentHash::from_payload(&p2)
    );
}

#[test]
fn content_hash_different_payload_differs() {
    let p1 = TriggerPayload::new("excerpt A");
    let p2 = TriggerPayload::new("excerpt B");
    assert_ne!(
        ContentHash::from_payload(&p1),
        ContentHash::from_payload(&p2)
    );
}

#[test]
fn content_hash_same_excerpt_different_event_kind_differs() {
    let p1 = TriggerPayload::new("same text").with_event_kind("alpha");
    let p2 = TriggerPayload::new("same text").with_event_kind("beta");
    assert_ne!(
        ContentHash::from_payload(&p1),
        ContentHash::from_payload(&p2)
    );
}

// ── Rate limiter ──────────────────────────────────────────────────────────────

#[test]
fn rate_limiter_blocks_after_exceeding_limit() {
    let mut rl = RateLimiter::new();
    rl.set_limit(SourceType::Webhook, 2);
    // Two allowed
    assert!(!rl.is_rate_limited(SourceType::Webhook, 1000));
    assert!(!rl.is_rate_limited(SourceType::Webhook, 1000));
    // Third in the same second exceeds limit=2
    assert!(rl.is_rate_limited(SourceType::Webhook, 1000));
}

#[test]
fn rate_limiter_resets_after_window() {
    let mut rl = RateLimiter::new();
    rl.set_limit(SourceType::Webhook, 1);
    assert!(!rl.is_rate_limited(SourceType::Webhook, 1000));
    assert!(rl.is_rate_limited(SourceType::Webhook, 1000)); // exceeded
    // A new 60-second window resets the counter
    assert!(!rl.is_rate_limited(SourceType::Webhook, 1061));
}

#[test]
fn chat_message_has_no_rate_limit_by_default() {
    let mut rl = RateLimiter::new();
    // ChatMessage default rpm=0 → unlimited
    for i in 0..200u64 {
        assert!(!rl.is_rate_limited(SourceType::ChatMessage, 1000 + i));
    }
}

// ── Full triage pipeline ──────────────────────────────────────────────────────

#[test]
fn oversized_payload_is_dropped() {
    let mut cache = DeduplicateCache::new();
    let mut rl = RateLimiter::new();
    let huge = "x".repeat(2001);
    let env = make_envelope("e20", SourceType::ChatMessage, &huge);
    let decision = triage(&env, &mut cache, &mut rl, 1000);
    assert!(
        matches!(decision, TriageDecision::Drop { .. }),
        "oversized payload must be dropped, got {:?}",
        decision
    );
}

#[test]
fn empty_payload_is_dropped() {
    let mut cache = DeduplicateCache::new();
    let mut rl = RateLimiter::new();
    let env = make_envelope("e21", SourceType::ChatMessage, "");
    let decision = triage(&env, &mut cache, &mut rl, 1000);
    assert!(matches!(decision, TriageDecision::Drop { .. }));
}

#[test]
fn rate_exceeded_envelope_is_dropped() {
    let mut cache = DeduplicateCache::new();
    let mut rl = RateLimiter::new();
    rl.set_limit(SourceType::Webhook, 1);
    let env_a = make_envelope("e22", SourceType::Webhook, "first");
    let env_b = make_envelope("e23", SourceType::Webhook, "second");
    let _ = triage(&env_a, &mut cache, &mut rl, 1000);
    let decision = triage(&env_b, &mut cache, &mut rl, 1000);
    assert!(matches!(decision, TriageDecision::Drop { .. }));
}

#[test]
fn duplicate_event_envelope_is_dropped_by_triage() {
    let mut cache = DeduplicateCache::new();
    let mut rl = RateLimiter::new();
    let env = make_envelope("e24", SourceType::Event, "the same event");
    let _ = triage(&env, &mut cache, &mut rl, 1000);
    let env2 = make_envelope("e25", SourceType::Event, "the same event");
    let decision = triage(&env2, &mut cache, &mut rl, 1005);
    assert!(matches!(decision, TriageDecision::Drop { .. }));
}

#[test]
fn confidence_threshold_constant_value() {
    assert_eq!(CONFIDENCE_THRESHOLD, 0.8);
}
