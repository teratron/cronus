use cronus::autonomy::{
    classify_command, evaluate, ApprovalDecision, ApprovalManager, ActionTracker,
    AutonomyError, AutonomyLevel, CommandRiskLevel, GateDecision, RESOLVED_ENTRY_GRACE_MS,
};
use std::time::{Duration, Instant};

// ── Autonomy levels ───────────────────────────────────────────────────────────

#[test]
fn three_autonomy_levels_exist() {
    let _ = AutonomyLevel::Supervised;
    let _ = AutonomyLevel::SemiAutonomous;
    let _ = AutonomyLevel::Autonomous;
}

// ── SecurityPolicy gate matrix ────────────────────────────────────────────────

#[test]
fn supervised_low_risk_is_allowed() {
    assert_eq!(evaluate(AutonomyLevel::Supervised, CommandRiskLevel::Low), GateDecision::Allow);
}

#[test]
fn supervised_medium_risk_requires_approval() {
    assert_eq!(
        evaluate(AutonomyLevel::Supervised, CommandRiskLevel::Medium),
        GateDecision::RequireApproval
    );
}

#[test]
fn supervised_high_risk_requires_approval() {
    assert_eq!(
        evaluate(AutonomyLevel::Supervised, CommandRiskLevel::High),
        GateDecision::RequireApproval
    );
}

#[test]
fn supervised_critical_risk_is_denied() {
    assert_eq!(evaluate(AutonomyLevel::Supervised, CommandRiskLevel::Critical), GateDecision::Deny);
}

#[test]
fn semi_autonomous_high_risk_requires_approval() {
    assert_eq!(
        evaluate(AutonomyLevel::SemiAutonomous, CommandRiskLevel::High),
        GateDecision::RequireApproval
    );
}

#[test]
fn semi_autonomous_critical_risk_is_denied() {
    assert_eq!(
        evaluate(AutonomyLevel::SemiAutonomous, CommandRiskLevel::Critical),
        GateDecision::Deny
    );
}

#[test]
fn autonomous_high_risk_is_allowed() {
    assert_eq!(evaluate(AutonomyLevel::Autonomous, CommandRiskLevel::High), GateDecision::Allow);
}

#[test]
fn autonomous_critical_risk_requires_approval() {
    assert_eq!(
        evaluate(AutonomyLevel::Autonomous, CommandRiskLevel::Critical),
        GateDecision::RequireApproval
    );
}

// ── CommandRiskLevel classifier ───────────────────────────────────────────────

#[test]
fn rm_rf_is_critical() {
    assert_eq!(classify_command("rm -rf /home/user"), CommandRiskLevel::Critical);
}

#[test]
fn drop_table_is_critical() {
    assert_eq!(classify_command("DROP TABLE users"), CommandRiskLevel::Critical);
}

#[test]
fn read_only_query_is_low() {
    assert_eq!(classify_command("read config.toml"), CommandRiskLevel::Low);
}

#[test]
fn list_files_is_low() {
    assert_eq!(classify_command("ls /home"), CommandRiskLevel::Low);
}

#[test]
fn write_file_is_high() {
    assert_eq!(classify_command("write output.txt"), CommandRiskLevel::High);
}

// ── ActionTracker rolling cap ─────────────────────────────────────────────────

#[test]
fn action_tracker_allows_up_to_cap() {
    let mut tracker = ActionTracker::new(5);
    let now = Instant::now();
    for _ in 0..5 {
        tracker.record(now).unwrap();
    }
    assert_eq!(tracker.count(now), 5);
}

#[test]
fn action_tracker_rejects_over_cap() {
    let mut tracker = ActionTracker::new(3);
    let now = Instant::now();
    tracker.record(now).unwrap();
    tracker.record(now).unwrap();
    tracker.record(now).unwrap();
    let result = tracker.record(now);
    assert_eq!(result, Err(AutonomyError::RateLimitExceeded));
}

#[test]
fn action_tracker_resets_after_window() {
    // Use a 100ms window to avoid Instant underflow on Windows.
    let mut tracker = ActionTracker::new_with_window(2, Duration::from_millis(100));
    let now = Instant::now();
    tracker.record(now).unwrap();
    tracker.record(now).unwrap();
    // Advance past the 100ms window
    let future = now + Duration::from_millis(101);
    assert_eq!(tracker.count(future), 0, "old entries must expire after window");
    // Should be able to record again
    tracker.record(future).unwrap();
}

// ── ApprovalGate ──────────────────────────────────────────────────────────────

#[test]
fn approval_gate_create_and_register_are_separate() {
    let mut mgr = ApprovalManager::new();
    let id = mgr.create("delete /tmp/x".into(), CommandRiskLevel::High, 600_000);
    // Gate is pending
    assert_eq!(mgr.decision(id, Instant::now()), None);

    mgr.register(id, ApprovalDecision::Approved).unwrap();
    assert_eq!(mgr.decision(id, Instant::now()), Some(ApprovalDecision::Approved));
}

#[test]
fn approval_gate_register_is_idempotent() {
    let mut mgr = ApprovalManager::new();
    let id = mgr.create("cmd".into(), CommandRiskLevel::Low, 600_000);
    mgr.register(id, ApprovalDecision::Approved).unwrap();
    // Second register call — must not error
    mgr.register(id, ApprovalDecision::Denied).unwrap();
    // Decision remains Approved (first write wins)
    assert_eq!(mgr.decision(id, Instant::now()), Some(ApprovalDecision::Approved));
}

#[test]
fn approval_gate_expired_auto_denies() {
    let mut mgr = ApprovalManager::new();
    let id = mgr.create("cmd".into(), CommandRiskLevel::Medium, 1); // 1ms TTL
    let now = Instant::now() + Duration::from_millis(2); // 2ms later
    assert_eq!(mgr.decision(id, now), Some(ApprovalDecision::Denied));
}

#[test]
fn approval_gate_register_known_id_returns_ok() {
    let mut mgr = ApprovalManager::new();
    let id = mgr.create("c".into(), CommandRiskLevel::Low, 600_000);
    let result = mgr.register(id, ApprovalDecision::Approved);
    assert!(result.is_ok());
}

#[test]
fn resolved_entry_grace_prevents_immediate_eviction() {
    let mut mgr = ApprovalManager::new();
    let id = mgr.create("cmd".into(), CommandRiskLevel::Low, 600_000);
    mgr.register(id, ApprovalDecision::Approved).unwrap();

    // GC at time T = now (within grace)
    mgr.gc(Instant::now());
    // Gate still available within grace window
    assert!(mgr.decision(id, Instant::now()).is_some());
}

#[test]
fn gc_evicts_resolved_gates_past_grace() {
    let mut mgr = ApprovalManager::new();
    let id = mgr.create("cmd".into(), CommandRiskLevel::Low, 600_000);
    mgr.register(id, ApprovalDecision::Approved).unwrap();

    // GC at time T = now + RESOLVED_ENTRY_GRACE_MS + 1 (past grace)
    let past_grace = Instant::now() + Duration::from_millis(RESOLVED_ENTRY_GRACE_MS + 1);
    mgr.gc(past_grace);
    assert_eq!(mgr.decision(id, past_grace), None, "evicted gate must return None");
}

// ── Constants ─────────────────────────────────────────────────────────────────

#[test]
fn resolved_entry_grace_is_15_seconds() {
    const { assert!(RESOLVED_ENTRY_GRACE_MS == 15_000) }
}
