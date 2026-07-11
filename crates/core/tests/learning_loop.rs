use cronus_core::learning::{
    CandidateSkill, CandidateStatus, LearningApprovalGate, LearningConfig, NoOpReviewFork,
    PostTurnReviewFork, try_review,
};

fn sample_skill(id: &str, confidence: f64) -> CandidateSkill {
    CandidateSkill {
        id: id.to_string(),
        trigger: "when user asks about X".to_string(),
        content: "do Y".to_string(),
        confidence,
        source_session_id: "sess-1".to_string(),
    }
}

// ── LearningConfig defaults ──────────────────────────────────────────────────

#[test]
fn default_config_is_enabled() {
    let cfg = LearningConfig::default();
    assert!(cfg.enabled);
}

#[test]
fn default_config_min_turns_is_five() {
    let cfg = LearningConfig::default();
    assert_eq!(cfg.min_turns_before_review, 5);
}

#[test]
fn default_config_min_confidence_is_seventy_percent() {
    let cfg = LearningConfig::default();
    assert!((cfg.min_confidence_to_propose - 0.7).abs() < 1e-9);
}

// ── LearningApprovalGate ─────────────────────────────────────────────────────

#[test]
fn submit_adds_candidate_as_pending() {
    let mut gate = LearningApprovalGate::new();
    gate.submit(sample_skill("sk-1", 0.9));
    assert_eq!(gate.status("sk-1"), Some(CandidateStatus::Pending));
}

#[test]
fn approve_transitions_to_approved() {
    let mut gate = LearningApprovalGate::new();
    gate.submit(sample_skill("sk-2", 0.85));
    assert!(gate.approve("sk-2"));
    assert_eq!(gate.status("sk-2"), Some(CandidateStatus::Approved));
}

#[test]
fn reject_transitions_to_rejected() {
    let mut gate = LearningApprovalGate::new();
    gate.submit(sample_skill("sk-3", 0.8));
    assert!(gate.reject("sk-3"));
    assert_eq!(gate.status("sk-3"), Some(CandidateStatus::Rejected));
}

#[test]
fn approve_unknown_returns_false() {
    let mut gate = LearningApprovalGate::new();
    assert!(!gate.approve("no-such-id"));
}

#[test]
fn reject_unknown_returns_false() {
    let mut gate = LearningApprovalGate::new();
    assert!(!gate.reject("no-such-id"));
}

#[test]
fn status_returns_none_for_unknown() {
    let gate = LearningApprovalGate::new();
    assert_eq!(gate.status("unknown"), None);
}

#[test]
fn list_pending_returns_only_pending() {
    let mut gate = LearningApprovalGate::new();
    gate.submit(sample_skill("sk-a", 0.9));
    gate.submit(sample_skill("sk-b", 0.8));
    gate.submit(sample_skill("sk-c", 0.75));
    gate.approve("sk-a");

    let pending = gate.list_pending();
    assert_eq!(pending.len(), 2, "only sk-b and sk-c should remain pending");
    assert!(!pending.iter().any(|s| s.id == "sk-a"));
}

// ── NoOpReviewFork ────────────────────────────────────────────────────────────

#[test]
fn noop_fork_always_returns_none() {
    let fork = NoOpReviewFork;
    assert!(fork.fork_review(0).is_none());
    assert!(fork.fork_review(100).is_none());
}

// ── try_review ────────────────────────────────────────────────────────────────

#[test]
fn try_review_returns_none_when_disabled() {
    let cfg = LearningConfig {
        enabled: false,
        ..LearningConfig::default()
    };
    let fork = NoOpReviewFork;
    let mut gate = LearningApprovalGate::new();
    assert!(try_review(&cfg, &fork, 100, &mut gate).is_none());
}

#[test]
fn try_review_returns_none_below_min_turns() {
    let cfg = LearningConfig::default(); // min_turns = 5
    let fork = NoOpReviewFork;
    let mut gate = LearningApprovalGate::new();
    assert!(try_review(&cfg, &fork, 3, &mut gate).is_none());
}

#[test]
fn try_review_returns_none_when_fork_yields_none() {
    let cfg = LearningConfig::default();
    let fork = NoOpReviewFork; // always returns None
    let mut gate = LearningApprovalGate::new();
    assert!(try_review(&cfg, &fork, 10, &mut gate).is_none());
}

/// Adapter that always proposes a skill with a given confidence.
struct FixedConfidenceFork(f64);
impl PostTurnReviewFork for FixedConfidenceFork {
    fn fork_review(&self, _: usize) -> Option<CandidateSkill> {
        Some(CandidateSkill {
            id: "auto-sk".to_string(),
            trigger: "trigger".to_string(),
            content: "content".to_string(),
            confidence: self.0,
            source_session_id: "s".to_string(),
        })
    }
}

#[test]
fn try_review_returns_none_when_confidence_below_threshold() {
    let cfg = LearningConfig::default(); // threshold = 0.7
    let fork = FixedConfidenceFork(0.5);
    let mut gate = LearningApprovalGate::new();
    assert!(try_review(&cfg, &fork, 10, &mut gate).is_none());
}

#[test]
fn try_review_submits_high_confidence_skill() {
    let cfg = LearningConfig::default();
    let fork = FixedConfidenceFork(0.9);
    let mut gate = LearningApprovalGate::new();

    let id = try_review(&cfg, &fork, 10, &mut gate);
    assert!(id.is_some(), "should return candidate ID");
    assert_eq!(gate.status("auto-sk"), Some(CandidateStatus::Pending));
}
