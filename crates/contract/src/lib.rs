//! `cronus-contract` — the ports tier of the crate topology (§4.1/§4.2):
//! shared types plus the seam traits domain code depends on and
//! adapter crates implement. Zero external dependencies, by construction —
//! nothing here may ever depend on I/O, a platform service, or a C library
//! (§4.3).
//!
//! This crate holds no logic of its own beyond what a data type's own
//! invariants require (id generation, display formatting, weight lookup). It
//! exists so `cronus-domain` and the adapter crates (`cronus-store-local`,
//! `cronus-auth-local`) can share a vocabulary without either depending on
//! the other.

// ── Memory types ─────────────────────────────────────────────────────────────
//
// Moved from `crates/core/src/memory/mod.rs`. `MemoryEntry` is the payload the
// `MemorySearch` / `UserDataStore` seam traits below carry across the
// domain/adapter boundary (§4.5); its field types travel with it.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

static ID_COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn now_secs() -> u64 {
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

/// Where a memory sits on the processing-depth axis (MC-1), orthogonal to
/// scope. Refinement flows one way, `raw -> working -> consolidated`;
/// consolidation never rewrites raw evidence, so any consolidated claim can
/// be checked against what actually happened.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryDepth {
    /// Verbatim captured evidence (transcripts, source documents) — immutable.
    Raw,
    /// Recent, lightly-processed notes organized by occurrence.
    Working,
    /// Durable, reusable abstraction — the recallable long-term corpus.
    Consolidated,
}

impl MemoryDepth {
    pub fn as_str(self) -> &'static str {
        match self {
            MemoryDepth::Raw => "Raw",
            MemoryDepth::Working => "Working",
            MemoryDepth::Consolidated => "Consolidated",
        }
    }

    pub fn from_db_str(s: &str) -> Option<Self> {
        match s {
            "Raw" => Some(MemoryDepth::Raw),
            "Working" => Some(MemoryDepth::Working),
            "Consolidated" => Some(MemoryDepth::Consolidated),
            _ => None,
        }
    }
}

/// A memory's reversible lifecycle state (MI-9), orthogonal to MEM-5 decay.
/// Decay may lower an item's ranking in any state, but MUST NOT delete an
/// item whose state is `Paused` or `Archived` — a deliberate shelving is a
/// value signal that overrides automatic pruning. `Deleted` is not a stored
/// variant: it is realized by the existing targeted forget (row removal),
/// per MI-9's own table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleState {
    /// In the default recall set.
    Active,
    /// Temporarily and reversibly excluded from recall, no data loss.
    Paused,
    /// Retained but out of the default recall set; opt-in to include.
    Archived,
}

impl LifecycleState {
    pub fn as_str(self) -> &'static str {
        match self {
            LifecycleState::Active => "Active",
            LifecycleState::Paused => "Paused",
            LifecycleState::Archived => "Archived",
        }
    }

    pub fn from_db_str(s: &str) -> Option<Self> {
        match s {
            "Active" => Some(LifecycleState::Active),
            "Paused" => Some(LifecycleState::Paused),
            "Archived" => Some(LifecycleState::Archived),
            _ => None,
        }
    }
}

/// The typed outcome of a captured experience (MI-13's read side over MI-7's
/// write side) — `None` on every ordinary memory (the honest default for the
/// entire pre-existing corpus), `Some(_)` only for an item `distill_run`
/// produced. Orthogonal to `kind`: a distilled run is still classified by
/// `kind` for what it's *about* (`ProjectContext`, typically), while this
/// says how the attempt *went* — the axis `act_with_experience`'s reuse gate
/// actually needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExperienceOutcome {
    /// Reusable directly when the gate passes.
    Success,
    /// Never reused as a solution — injected as an avoid signal.
    Failure,
    /// Injected as guidance, never reused as a solution.
    Insight,
}

impl ExperienceOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            ExperienceOutcome::Success => "Success",
            ExperienceOutcome::Failure => "Failure",
            ExperienceOutcome::Insight => "Insight",
        }
    }

    pub fn from_db_str(s: &str) -> Option<Self> {
        match s {
            "Success" => Some(ExperienceOutcome::Success),
            "Failure" => Some(ExperienceOutcome::Failure),
            "Insight" => Some(ExperienceOutcome::Insight),
            _ => None,
        }
    }
}

/// The subject-of-memory lens (MI-6 ext): who the memory is *about*,
/// orthogonal to `actor` (who said it) and to `source` (how it entered the
/// system). A closed 2-variant vocabulary — no third-party subject exists
/// in this single-tenant model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemorySubject {
    User,
    AgentSelf,
}

impl MemorySubject {
    pub fn as_str(self) -> &'static str {
        match self {
            MemorySubject::User => "User",
            MemorySubject::AgentSelf => "AgentSelf",
        }
    }

    pub fn from_db_str(s: &str) -> Option<Self> {
        match s {
            "User" => Some(MemorySubject::User),
            "AgentSelf" => Some(MemorySubject::AgentSelf),
            _ => None,
        }
    }
}

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
    pub depth: MemoryDepth,
    pub lifecycle_state: LifecycleState,
    pub experience_outcome: Option<ExperienceOutcome>,
    /// Who said this (MI-6) — distinct from `workspace_id` (scope
    /// ownership) and `source` (how it entered the system).
    pub actor: Option<String>,
    /// A hard void-after instant (MI-6) — complements MEM-5 decay rather
    /// than replacing it; decay lowers ranking, `expiry` removes the item
    /// from default recall outright once passed.
    pub expiry: Option<u64>,
    /// The subject-of-memory lens (MI-6 ext).
    pub subject: Option<MemorySubject>,
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
            // A single-shot `new()` call already represents a discrete,
            // finished fact — every pre-existing call site (auth, CLI `cronus
            // memory store`, session capture) writes exactly this shape, so
            // defaulting to `Consolidated` preserves that behavior exactly.
            // `Raw`/`Working` are for the future ingestion pipeline (MC-1)
            // and are opted into via `with_depth`.
            depth: MemoryDepth::Consolidated,
            lifecycle_state: LifecycleState::Active,
            experience_outcome: None,
            actor: None,
            expiry: None,
            subject: None,
        }
    }

    pub fn with_workspace(mut self, id: impl Into<String>) -> Self {
        self.workspace_id = Some(id.into());
        self
    }

    pub fn with_depth(mut self, depth: MemoryDepth) -> Self {
        self.depth = depth;
        self
    }

    /// Marks this entry as a captured experience (MI-7 write side) so
    /// MI-13's reuse gate can later recall it typed. Ordinary memories never
    /// call this — `experience_outcome` stays `None`.
    pub fn with_experience_outcome(mut self, outcome: ExperienceOutcome) -> Self {
        self.experience_outcome = Some(outcome);
        self
    }

    /// MI-6: attribute this capture to who said it.
    pub fn with_actor(mut self, actor: impl Into<String>) -> Self {
        self.actor = Some(actor.into());
        self
    }

    /// MI-6: a hard void-after instant, distinct from MEM-5 decay.
    pub fn with_expiry(mut self, expiry: u64) -> Self {
        self.expiry = Some(expiry);
        self
    }

    /// MI-6 ext: the subject-of-memory lens.
    pub fn with_subject(mut self, subject: MemorySubject) -> Self {
        self.subject = Some(subject);
        self
    }

    /// Effective trust score after applying the verification state weight.
    pub fn effective_trust(&self) -> f64 {
        self.trust_score * self.verification_state.weight()
    }
}

// ── StateStore seam ──────────────────────────────────────────────────────────
//
// Moved from `crates/core/src/store.rs`.

/// A durable key-value store the engine resumes from after a restart.
pub trait StateStore {
    /// Persist a value; durable once this returns `Ok`.
    fn put(&mut self, key: &str, value: &str) -> std::io::Result<()>;
    /// Read a value previously stored, if present.
    fn get(&self, key: &str) -> Option<String>;
}

// ── ModelProvider seam ───────────────────────────────────────────────────────
//
// Moved from `crates/core/src/router/provider.rs`. Only the types the trait
// signature itself references travel with it — `ProviderError`,
// `RoutingRequest`, and `RouteDecision` are router-internal and stay in
// `cronus-domain`.

/// Health state reported by a provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderHealth {
    Healthy,
    Degraded,
    Unavailable,
}

/// Provider tier used for routing preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProviderTier {
    /// Locally-hosted model (lowest cost, highest privacy).
    Local = 0,
    /// Small, fast cloud model.
    Economy = 1,
    /// Standard cloud model.
    Standard = 2,
    /// Large, high-capability cloud model.
    Premium = 3,
}

/// Category of task being routed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskType {
    CodeGeneration,
    Analysis,
    Summarization,
    QA,
    Chat,
}

/// The `ModelProvider` trait — implemented by each backend.
///
/// All methods take `&self` — providers are stateless from the router's
/// perspective; mutable circuit state lives in `RouterPool`.
pub trait ModelProvider: Send + Sync {
    /// Unique stable identifier (e.g. "openai-gpt4o", "local-llama3").
    fn id(&self) -> &str;

    /// Current health as reported by the provider's own health check.
    fn health(&self) -> ProviderHealth;

    /// Maximum tokens this provider accepts in context.
    fn context_window(&self) -> u32;

    /// Approximate cost per 1k tokens (output) in USD.
    fn cost_per_1k_tokens(&self) -> f64;

    /// Median observed latency in milliseconds.
    fn latency_p50_ms(&self) -> u64;

    /// Provider tier for routing priority.
    fn tier(&self) -> ProviderTier;

    /// Returns how well this provider handles the given task type (0.0–1.0).
    fn task_fit(&self, task: TaskType) -> f64;
}

// ── CheckpointWriter seam ────────────────────────────────────────────────────
//
// Moved from `crates/core/src/checkpoint.rs`.

/// The three canonical checkpoint files for a session.
#[derive(Debug, Clone)]
pub struct CheckpointPaths {
    /// Full session context JSON.
    pub context: std::path::PathBuf,
    /// Extracted memory facts (plain text).
    pub memory: std::path::PathBuf,
    /// Human-readable session notes.
    pub notes: std::path::PathBuf,
}

impl CheckpointPaths {
    pub fn new(state_dir: &std::path::Path) -> Self {
        let base = state_dir.join("checkpoint");
        CheckpointPaths {
            context: base.clone(),
            memory: base.join("memory"),
            notes: base.join("notes.md"),
        }
    }

    pub fn fork(state_dir: &std::path::Path, fork_id: &str) -> Self {
        let base = state_dir.join(format!("checkpoint-fork-{fork_id}"));
        CheckpointPaths {
            context: base.clone(),
            memory: base.join("memory"),
            notes: base.join("notes.md"),
        }
    }
}

#[derive(Debug)]
pub enum CheckpointError {
    Io(std::io::Error),
}

impl std::fmt::Display for CheckpointError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CheckpointError::Io(e) => write!(f, "checkpoint I/O error: {e}"),
        }
    }
}

impl std::error::Error for CheckpointError {}

impl From<std::io::Error> for CheckpointError {
    fn from(e: std::io::Error) -> Self {
        CheckpointError::Io(e)
    }
}

/// Seam trait for writing checkpoints (wired by agent-registry later).
pub trait CheckpointWriter: Send + Sync {
    fn write(&self, paths: &CheckpointPaths, body: &str) -> Result<(), CheckpointError>;
}

// ── Compactor seam ───────────────────────────────────────────────────────────
//
// Moved from `crates/core/src/context_mgmt.rs`.

/// Where a context entry falls in the eviction order (least-important last).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TrimPriority {
    OrphanedToolResult,
    ToolUsePair,
    NonProtectedThinking,
    NonProtectedAssistant,
    NonProtectedUser,
    CompactionMarker,
    ModelChangeMarker,
    Protected, // never trimmed — invariant
}

/// One turn's worth of context, with its trim priority and token cost.
#[derive(Debug, Clone)]
pub struct ContextEntry {
    pub role: String,
    pub body: String,
    pub token_count: u64,
    pub protected: bool,
    pub priority: TrimPriority,
}

impl ContextEntry {
    pub fn new(role: impl Into<String>, body: impl Into<String>, token_count: u64) -> Self {
        ContextEntry {
            role: role.into(),
            body: body.into(),
            token_count,
            protected: false,
            priority: TrimPriority::NonProtectedUser,
        }
    }

    pub fn with_priority(mut self, p: TrimPriority) -> Self {
        self.priority = p;
        self
    }

    pub fn protect(mut self) -> Self {
        self.protected = true;
        self.priority = TrimPriority::Protected;
        self
    }
}

/// Seam trait for LLM-driven compaction (wiring deferred).
pub trait Compactor: Send + Sync {
    fn compact(&self, context: &[ContextEntry], keep_recent_tokens: u64) -> Result<String, String>;
}

// ── BusSender seam ───────────────────────────────────────────────────────────
//
// Moved from `crates/core/src/inbox.rs`.

/// Bus events emitted by the inbox module.
#[derive(Debug, Clone, PartialEq)]
pub enum BusEvent {
    InboxArrived { recipient_id: String, count: u32 },
}

/// Seam trait for bus event delivery (real bus wiring deferred).
pub trait BusSender: Send + Sync {
    fn send(&self, event: BusEvent);
}

/// No-op bus sender (stub). Lives beside the trait it implements — both
/// `cronus-domain` and `cronus-store-local` need a bus sender, and neither
/// may depend on the other, so a null-object default belongs in the ports
/// tier both already depend on.
pub struct NoOpBusSender;

impl BusSender for NoOpBusSender {
    fn send(&self, _: BusEvent) {}
}

/// Capturing bus sender for tests.
pub struct CaptureBusSender {
    pub events: std::sync::Mutex<Vec<BusEvent>>,
}

impl Default for CaptureBusSender {
    fn default() -> Self {
        Self::new()
    }
}

impl CaptureBusSender {
    pub fn new() -> Self {
        CaptureBusSender {
            events: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub fn captured(&self) -> Vec<BusEvent> {
        self.events.lock().unwrap().clone()
    }
}

impl BusSender for CaptureBusSender {
    fn send(&self, event: BusEvent) {
        self.events.lock().unwrap().push(event);
    }
}

// ── DN-2 provider-plane seams ────────────────────────────────────────────────
//
// New trait declarations (§4.5). Illustrative shape, not
// a final signature — no implementation exists yet; `cronus-store-local`
// (`UserDataStore`) and `cronus-auth-local` (`AuthProvider`/`IdentityProvider`)
// implement these in a later phase task. The plain `String` error is
// deliberately minimal for the same reason: these are net-new APIs, not an
// existing one being redesigned, so committing to a richer error type now
// would be speculative.

/// MI-2: first-class temporal recall modes over the bi-temporal record —
/// resolved against valid-time (`AsOf`) or transaction-time (`ChangedSince`,
/// `Recent`), never conflating the two.
#[derive(Debug, Clone, Copy)]
pub enum TemporalMode {
    /// "What did we hold true at instant T?" — valid-time window containing T.
    AsOf(u64),
    /// "What is new or changed since checkpoint C?" — transaction-time > C.
    ChangedSince(u64),
    /// "What are the newest N, regardless of age?" — transaction-time desc.
    Recent,
}

/// MI-8: the field a structured predicate compares — closed to the columns
/// `MemoryEntry` actually has. The vocabulary is backend-agnostic; this enum
/// names *which* field, the backend decides how to index or compare it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PredicateField {
    Kind,
    Source,
    WorkspaceId,
    TrustScore,
    Confidence,
    CreatedAt,
    ValidAt,
    ExperienceOutcome,
}

/// A comparison operand (MI-8).
#[derive(Debug, Clone)]
pub enum PredicateValue {
    Text(String),
    Number(f64),
}

/// MI-8: a small, closed structured-comparison vocabulary — equals/
/// not-equals, ordering, set membership, text containment — combinable via
/// AND/OR/NOT, composable with (never replacing) the fuzzy multi-signal
/// fusion and the temporal modes. Fixed at this layer precisely so it stays
/// backend-agnostic: a backend that cannot express a combinator natively
/// falls back to post-fetch evaluation rather than silently dropping the
/// constraint (SQLite, the only backend today, expresses every variant here
/// natively — the fallback contract is honored vacuously, not exercised).
#[derive(Debug, Clone)]
pub enum FieldPredicate {
    Eq(PredicateField, PredicateValue),
    Ne(PredicateField, PredicateValue),
    Gt(PredicateField, PredicateValue),
    Ge(PredicateField, PredicateValue),
    Lt(PredicateField, PredicateValue),
    Le(PredicateField, PredicateValue),
    In(PredicateField, Vec<PredicateValue>),
    NotIn(PredicateField, Vec<PredicateValue>),
    Contains(PredicateField, String),
    ContainsCi(PredicateField, String),
    And(Vec<FieldPredicate>),
    Or(Vec<FieldPredicate>),
    Not(Box<FieldPredicate>),
}

/// The read half of the user-data plane: full-text search over stored
/// memories, independent of where they are persisted.
pub trait MemorySearch {
    fn search_fts(&self, query: &str, limit: usize) -> Result<Vec<MemoryEntry>, String>;

    /// MI-2: temporal recall modes over the bi-temporal record, composing
    /// with the same trust/lifecycle defaults as `search_fts`.
    fn recall_temporal(&self, mode: TemporalMode, limit: usize)
    -> Result<Vec<MemoryEntry>, String>;

    /// MI-8: the closed structured-comparison vocabulary.
    fn recall_structured(
        &self,
        predicate: &FieldPredicate,
        limit: usize,
    ) -> Result<Vec<MemoryEntry>, String>;
}

/// The DN-2 user-data plane (§4.5). `MemorySearch` is one facet; a full
/// implementation also covers write, prune, and export (DN-7 portability).
///
/// No `Send + Sync` bound: the on-device default wraps a `rusqlite::Connection`,
/// which is not `Sync` (SQLite connections are not shared across threads
/// without external synchronization). The illustrative sketch this trait
/// started from assumed it; the concrete implementation proved it wrong.
pub trait UserDataStore: MemorySearch {
    fn put(&self, entry: &MemoryEntry) -> Result<(), String>;
    fn export(&self) -> Result<Vec<MemoryEntry>, String>; // DN-7: always able to come home
}

/// The DN-2 authentication plane (§4.5).
pub trait AuthProvider: Send + Sync {
    fn authenticate(&self, principal: &str, credential: &str) -> Result<bool, String>;
}

/// The DN-2 principal-identity plane (§4.5).
pub trait IdentityProvider: Send + Sync {
    fn current_principal(&self) -> Option<String>;
}
