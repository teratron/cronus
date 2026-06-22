//! Learning loop — post-turn background review fork and skill generation.
//!
//! Phase 5 stub: review fork seam returns None; approval gate is in-memory.
//! Real LLM sub-agent wires in Phase 6.

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct LearningConfig {
    pub enabled: bool,
    pub min_turns_before_review: u32,
    pub min_confidence_to_propose: f64,
}

impl Default for LearningConfig {
    fn default() -> Self {
        LearningConfig {
            enabled: true,
            min_turns_before_review: 5,
            min_confidence_to_propose: 0.7,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CandidateSkill {
    pub id: String,
    pub trigger: String,
    pub content: String,
    pub confidence: f64,
    pub source_session_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidateStatus {
    Pending,
    Approved,
    Rejected,
}

pub trait PostTurnReviewFork: Send + Sync {
    fn fork_review(&self, session_history_len: usize) -> Option<CandidateSkill>;
}

pub struct NoOpReviewFork;

impl PostTurnReviewFork for NoOpReviewFork {
    fn fork_review(&self, _session_history_len: usize) -> Option<CandidateSkill> {
        None
    }
}

/// In-memory approval gate for candidate skills.
#[derive(Debug, Default)]
pub struct LearningApprovalGate {
    candidates: HashMap<String, (CandidateSkill, CandidateStatus)>,
}

impl LearningApprovalGate {
    pub fn new() -> Self {
        LearningApprovalGate::default()
    }

    pub fn submit(&mut self, skill: CandidateSkill) {
        let id = skill.id.clone();
        self.candidates.insert(id, (skill, CandidateStatus::Pending));
    }

    pub fn approve(&mut self, id: &str) -> bool {
        if let Some((_, status)) = self.candidates.get_mut(id) {
            *status = CandidateStatus::Approved;
            return true;
        }
        false
    }

    pub fn reject(&mut self, id: &str) -> bool {
        if let Some((_, status)) = self.candidates.get_mut(id) {
            *status = CandidateStatus::Rejected;
            return true;
        }
        false
    }

    pub fn list_pending(&self) -> Vec<&CandidateSkill> {
        self.candidates
            .values()
            .filter(|(_, s)| *s == CandidateStatus::Pending)
            .map(|(skill, _)| skill)
            .collect()
    }

    pub fn status(&self, id: &str) -> Option<CandidateStatus> {
        self.candidates.get(id).map(|(_, s)| *s)
    }
}

/// Try to trigger a post-turn review. Respects LearningConfig guards.
pub fn try_review(
    config: &LearningConfig,
    fork: &dyn PostTurnReviewFork,
    session_history_len: usize,
    gate: &mut LearningApprovalGate,
) -> Option<String> {
    if !config.enabled {
        return None;
    }
    if session_history_len < config.min_turns_before_review as usize {
        return None;
    }
    let skill = fork.fork_review(session_history_len)?;
    if skill.confidence < config.min_confidence_to_propose {
        return None;
    }
    let id = skill.id.clone();
    gate.submit(skill);
    Some(id)
}
