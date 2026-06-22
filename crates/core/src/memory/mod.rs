//! Long-term memory subsystem.
//!
//! Memories are bi-temporal (valid_at / created_at), trust-scored,
//! full-text searchable via FTS5, and linked into session chains.
//! HRR (holographic reduced representation) encoding is a seam —
//! the stub returns a zeroed vector; the real encoder ships with
//! sqlite-vec in a later phase.

pub mod chain;
pub mod consolidation;
pub mod encryption;
pub mod store;
pub mod trust;

pub use store::MemoryStore;

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// ── ID generation ─────────────────────────────────────────────────────────────

static ID_COUNTER: AtomicU64 = AtomicU64::new(0);

pub(crate) fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs()
}

fn generate_id() -> String {
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_millis() as u64;
    let c = ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("mem_{t:016x}_{c:08x}")
}

// ── Error ─────────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum MemoryError {
    Database(rusqlite::Error),
}

impl std::fmt::Display for MemoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryError::Database(e) => write!(f, "memory database error: {e}"),
        }
    }
}

impl std::error::Error for MemoryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            MemoryError::Database(e) => Some(e),
        }
    }
}

impl From<rusqlite::Error> for MemoryError {
    fn from(e: rusqlite::Error) -> Self {
        MemoryError::Database(e)
    }
}

pub type Result<T> = std::result::Result<T, MemoryError>;

// ── MemoryId ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MemoryId(String);

impl MemoryId {
    pub fn new() -> Self {
        MemoryId(generate_id())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for MemoryId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<String> for MemoryId {
    fn from(s: String) -> Self {
        MemoryId(s)
    }
}

impl std::fmt::Display for MemoryId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

// ── MemoryKind ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryKind {
    ArchitecturalDecision,
    DebugContext,
    KnownIssue,
    Convention,
    ProjectContext,
}

impl MemoryKind {
    pub fn as_str(self) -> &'static str {
        match self {
            MemoryKind::ArchitecturalDecision => "ArchitecturalDecision",
            MemoryKind::DebugContext => "DebugContext",
            MemoryKind::KnownIssue => "KnownIssue",
            MemoryKind::Convention => "Convention",
            MemoryKind::ProjectContext => "ProjectContext",
        }
    }

    pub fn from_db_str(s: &str) -> Option<Self> {
        match s {
            "ArchitecturalDecision" => Some(MemoryKind::ArchitecturalDecision),
            "DebugContext" => Some(MemoryKind::DebugContext),
            "KnownIssue" => Some(MemoryKind::KnownIssue),
            "Convention" => Some(MemoryKind::Convention),
            "ProjectContext" => Some(MemoryKind::ProjectContext),
            _ => None,
        }
    }
}

// ── MemorySource ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemorySource {
    Agent,
    User,
    Git,
    System,
    Import,
}

impl MemorySource {
    pub fn as_str(self) -> &'static str {
        match self {
            MemorySource::Agent => "Agent",
            MemorySource::User => "User",
            MemorySource::Git => "Git",
            MemorySource::System => "System",
            MemorySource::Import => "Import",
        }
    }

    pub fn from_db_str(s: &str) -> Option<Self> {
        match s {
            "Agent" => Some(MemorySource::Agent),
            "User" => Some(MemorySource::User),
            "Git" => Some(MemorySource::Git),
            "System" => Some(MemorySource::System),
            "Import" => Some(MemorySource::Import),
            _ => None,
        }
    }
}

// ── VerificationState ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationState {
    Untested,
    Claimed,
    TestedInProject,
    ValidatedCrossProject,
}

impl VerificationState {
    pub fn weight(self) -> f64 {
        match self {
            VerificationState::Untested => 0.30,
            VerificationState::Claimed => 0.50,
            VerificationState::TestedInProject => 0.70,
            VerificationState::ValidatedCrossProject => 1.00,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            VerificationState::Untested => "Untested",
            VerificationState::Claimed => "Claimed",
            VerificationState::TestedInProject => "TestedInProject",
            VerificationState::ValidatedCrossProject => "ValidatedCrossProject",
        }
    }

    pub fn from_db_str(s: &str) -> Option<Self> {
        match s {
            "Untested" => Some(VerificationState::Untested),
            "Claimed" => Some(VerificationState::Claimed),
            "TestedInProject" => Some(VerificationState::TestedInProject),
            "ValidatedCrossProject" => Some(VerificationState::ValidatedCrossProject),
            _ => None,
        }
    }
}

// ── CodeChangeType / SuggestedAction ─────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodeChangeType {
    Deleted,
    SignatureChanged,
    MajorRefactor,
    MinorEdit,
    Renamed,
    Moved,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SuggestedAction {
    Invalidate(f64),
    Review(f64),
    Update(f64),
    None,
}

impl CodeChangeType {
    pub fn suggested_action(self) -> SuggestedAction {
        match self {
            CodeChangeType::Deleted => SuggestedAction::Invalidate(1.0),
            CodeChangeType::SignatureChanged => SuggestedAction::Review(0.7),
            CodeChangeType::MajorRefactor => SuggestedAction::Review(0.5),
            CodeChangeType::MinorEdit => SuggestedAction::Update(0.2),
            CodeChangeType::Renamed => SuggestedAction::Update(0.3),
            CodeChangeType::Moved => SuggestedAction::Update(0.2),
        }
    }
}

// ── MemoryEntry ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct MemoryEntry {
    pub id: MemoryId,
    pub kind: MemoryKind,
    pub source: MemorySource,
    pub title: String,
    pub body: String,
    pub confidence: f64,
    pub trust_score: f64,
    pub valid_at: u64,
    pub created_at: u64,
    pub superseded_at: Option<u64>,
    pub workspace_id: Option<String>,
    pub verification_state: VerificationState,
}

impl MemoryEntry {
    pub fn new(
        kind: MemoryKind,
        source: MemorySource,
        title: impl Into<String>,
        body: impl Into<String>,
    ) -> Self {
        let now = now_secs();
        MemoryEntry {
            id: MemoryId::new(),
            kind,
            source,
            title: title.into(),
            body: body.into(),
            confidence: 1.0,
            trust_score: 0.5,
            valid_at: now,
            created_at: now,
            superseded_at: None,
            workspace_id: None,
            verification_state: VerificationState::Untested,
        }
    }

    pub fn with_workspace(mut self, id: impl Into<String>) -> Self {
        self.workspace_id = Some(id.into());
        self
    }

    /// Effective trust score after applying the verification state weight.
    pub fn effective_trust(&self) -> f64 {
        self.trust_score * self.verification_state.weight()
    }
}

// ── TrustUpdate ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TrustUpdate {
    pub positive: bool,
    pub new_verification_state: Option<VerificationState>,
}

impl TrustUpdate {
    pub fn positive() -> Self {
        TrustUpdate {
            positive: true,
            new_verification_state: None,
        }
    }

    pub fn negative() -> Self {
        TrustUpdate {
            positive: false,
            new_verification_state: None,
        }
    }

    pub fn with_verification(state: VerificationState) -> Self {
        TrustUpdate {
            positive: true,
            new_verification_state: Some(state),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_id_uniqueness() {
        let a = MemoryId::new();
        let b = MemoryId::new();
        assert_ne!(a, b, "sequential IDs must be unique");
    }

    #[test]
    fn verification_state_weights_ordered() {
        assert!(VerificationState::Untested.weight() < VerificationState::Claimed.weight());
        assert!(VerificationState::Claimed.weight() < VerificationState::TestedInProject.weight());
        assert!(
            VerificationState::TestedInProject.weight()
                < VerificationState::ValidatedCrossProject.weight()
        );
        assert_eq!(VerificationState::ValidatedCrossProject.weight(), 1.0);
    }

    #[test]
    fn code_change_type_mapping() {
        assert_eq!(
            CodeChangeType::Deleted.suggested_action(),
            SuggestedAction::Invalidate(1.0)
        );
        assert_eq!(
            CodeChangeType::SignatureChanged.suggested_action(),
            SuggestedAction::Review(0.7)
        );
        assert_eq!(
            CodeChangeType::MinorEdit.suggested_action(),
            SuggestedAction::Update(0.2)
        );
    }

    #[test]
    fn memory_entry_effective_trust() {
        let mut entry =
            MemoryEntry::new(MemoryKind::Convention, MemorySource::Agent, "test", "body");
        entry.trust_score = 1.0;
        entry.verification_state = VerificationState::Untested;
        assert!((entry.effective_trust() - 0.30).abs() < f64::EPSILON);

        entry.verification_state = VerificationState::ValidatedCrossProject;
        assert!((entry.effective_trust() - 1.0).abs() < f64::EPSILON);
    }
}
