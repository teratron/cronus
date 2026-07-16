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

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
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

// ── InferenceBackend seam ────────────────────────────────────────────────────
//
// The streaming call surface (MR-2/MR-8). Distinct from `ModelProvider`
// above: that trait is routing metadata (id/health/cost/latency/tier/
// task_fit — what the router scores) and has no method that performs a
// call. A concrete provider in `cronus-model-local` implements both traits —
// two facets of one object, score vs. call — never one subsuming the other.

/// A request to generate against a named model.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GenerateRequest {
    /// The model name/tag this request targets (resolves via the router's
    /// `api_base`, not looked up here).
    pub model: String,
    pub prompt: String,
    /// Backend-specific generation parameters (temperature, top_p, ...) as
    /// opaque key/value pairs — mirrors the modifier convention `nodus`
    /// already uses for its own provider seam.
    pub parameters: Vec<(String, String)>,
}

/// One event in a generation stream (MR-8).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamEvent {
    Token(String),
    ToolCall {
        name: String,
        arguments: String,
    },
    Usage {
        prompt_tokens: u64,
        completion_tokens: u64,
    },
    /// Terminal: the stream completed normally.
    Done,
    /// Terminal: the stream ended abnormally, including on cancellation.
    Error(InferenceError),
}

/// The wire-failure taxonomy a transport maps onto (§4.5). Deliberately flat
/// and small — retry/rotate/fallback policy over these variants lives in
/// `l2-model-error-recovery`, not here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InferenceError {
    ConnectRefused,
    Timeout,
    ClientError(u16),
    ServerError(u16),
    MalformedStream(String),
    /// The caller's `CancelHandle` was set mid-call.
    Cancelled,
    /// The backend has no support for the attempted operation (MR-6/MR-9:
    /// reported honestly, never silently emulated).
    Unsupported,
}

/// Static facts about a model as reported by its serving backend (MR-3/MR-12).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ModelDescriptor {
    pub name: String,
    pub digest: Option<String>,
    pub size_bytes: Option<u64>,
    pub parameters: Option<String>,
}

/// A residency instruction for an explicit load/unload lifecycle (MR-6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResidencyHint {
    KeepAliveSecs(u64),
    UnloadNow,
}

/// One event in a model-acquisition stream (MR-4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PullProgress {
    Downloading {
        bytes_done: u64,
        bytes_total: Option<u64>,
    },
    Done {
        digest: Option<String>,
    },
    Error(InferenceError),
}

/// A cooperative cancellation flag shared between a caller and the worker
/// driving a `generate_stream` call. Cloning shares the same underlying flag
/// — the caller keeps one clone to call `cancel()` while another is moved
/// into the streaming call.
#[derive(Debug, Clone)]
pub struct CancelHandle(Arc<AtomicBool>);

impl CancelHandle {
    pub fn new() -> Self {
        CancelHandle(Arc::new(AtomicBool::new(false)))
    }

    /// Request cancellation. The next event the stream yields MUST be
    /// `StreamEvent::Error(InferenceError::Cancelled)`, followed by
    /// termination (no further events).
    pub fn cancel(&self) {
        self.0.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

impl Default for CancelHandle {
    fn default() -> Self {
        Self::new()
    }
}

/// The streaming inference call surface (MR-2/MR-8). Implemented by a
/// concrete endpoint-profile provider in `cronus-model-local`; the nodus
/// `ModelProvider` (`generate`/`analyze` → `String`) is satisfied by a
/// host-side bridge that collapses this stream, never by implementing this
/// trait a second time.
pub trait InferenceBackend: Send + Sync {
    /// Blocking pull-iterator: the caller advances it to drive the call.
    /// Cancelling `cancel` mid-stream yields exactly one
    /// `Error(Cancelled)` and then `None`.
    fn generate_stream(
        &self,
        request: GenerateRequest,
        cancel: CancelHandle,
    ) -> Box<dyn Iterator<Item = StreamEvent> + Send>;

    fn embed(&self, model: &str, input: &str) -> Result<Vec<f32>, InferenceError>;

    fn describe(&self, model: &str) -> Result<ModelDescriptor, InferenceError>;

    /// Acquire a model by name; progress-streamed, never a hidden stall.
    fn pull(&self, model: &str) -> Box<dyn Iterator<Item = PullProgress> + Send>;

    fn set_residency(&self, model: &str, hint: ResidencyHint) -> Result<(), InferenceError>;
}

#[cfg(test)]
mod inference_tests {
    use super::*;

    /// A scripted backend: yields three tokens then `Done`, unless
    /// cancelled mid-stream, in which case it yields exactly one
    /// `Error(Cancelled)` and stops.
    struct ScriptedBackend;

    impl InferenceBackend for ScriptedBackend {
        fn generate_stream(
            &self,
            _request: GenerateRequest,
            cancel: CancelHandle,
        ) -> Box<dyn Iterator<Item = StreamEvent> + Send> {
            let tokens = ["Hello", ", ", "world"];
            let mut idx = 0usize;
            let mut finished = false;
            Box::new(std::iter::from_fn(move || {
                if finished {
                    return None;
                }
                if cancel.is_cancelled() {
                    finished = true;
                    return Some(StreamEvent::Error(InferenceError::Cancelled));
                }
                if idx < tokens.len() {
                    let tok = tokens[idx].to_string();
                    idx += 1;
                    Some(StreamEvent::Token(tok))
                } else {
                    finished = true;
                    Some(StreamEvent::Done)
                }
            }))
        }

        fn embed(&self, _model: &str, _input: &str) -> Result<Vec<f32>, InferenceError> {
            Err(InferenceError::Unsupported)
        }

        fn describe(&self, model: &str) -> Result<ModelDescriptor, InferenceError> {
            Ok(ModelDescriptor {
                name: model.to_string(),
                ..Default::default()
            })
        }

        fn pull(&self, _model: &str) -> Box<dyn Iterator<Item = PullProgress> + Send> {
            Box::new(std::iter::once(PullProgress::Done { digest: None }))
        }

        fn set_residency(&self, _model: &str, _hint: ResidencyHint) -> Result<(), InferenceError> {
            Ok(())
        }
    }

    #[test]
    fn generate_stream_runs_to_done_uncancelled() {
        let backend = ScriptedBackend;
        let cancel = CancelHandle::new();
        let events: Vec<StreamEvent> = backend
            .generate_stream(GenerateRequest::default(), cancel)
            .collect();

        assert_eq!(
            events,
            vec![
                StreamEvent::Token("Hello".to_string()),
                StreamEvent::Token(", ".to_string()),
                StreamEvent::Token("world".to_string()),
                StreamEvent::Done,
            ]
        );
    }

    #[test]
    fn cancel_mid_stream_yields_single_cancelled_error_then_stops() {
        let backend = ScriptedBackend;
        let cancel = CancelHandle::new();
        let mut stream = backend.generate_stream(GenerateRequest::default(), cancel.clone());

        // Pull two tokens, then cancel mid-stream.
        assert_eq!(stream.next(), Some(StreamEvent::Token("Hello".to_string())));
        assert_eq!(stream.next(), Some(StreamEvent::Token(", ".to_string())));
        cancel.cancel();

        assert_eq!(
            stream.next(),
            Some(StreamEvent::Error(InferenceError::Cancelled))
        );
        assert_eq!(
            stream.next(),
            None,
            "no events after the terminal Cancelled"
        );
    }

    #[test]
    fn unsupported_capability_reported_honestly_not_emulated() {
        let backend = ScriptedBackend;
        assert_eq!(
            backend.embed("any-model", "text"),
            Err(InferenceError::Unsupported)
        );
    }

    #[test]
    fn cancel_handle_clone_shares_the_same_flag() {
        let cancel = CancelHandle::new();
        let clone = cancel.clone();
        assert!(!clone.is_cancelled());
        cancel.cancel();
        assert!(
            clone.is_cancelled(),
            "clone must observe cancellation through the shared flag"
        );
    }
}

// ── Project wiki types ────────────────────────────────────────────────────────
//
// The client-facing project wiki (l2-project-wiki) is a derived projection
// CACHE: pages are rows written only by the office regeneration pipeline and
// reconstructable from ground truth (PW-3). These are the payload types the
// store persists; the SQLite store itself lives in `cronus-store-local`.

/// The fixed page-kind hierarchy (l2-project-wiki §4.1). The client wiki is
/// navigable overview → area → detail via `WikiPage::parent_id` + `ord`; the
/// kinds are closed so no page sits far from the overview (PW-6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WikiPageKind {
    Overview,
    Area,
    Decisions,
    Howto,
    Glossary,
    Changelog,
}

impl WikiPageKind {
    pub fn as_str(self) -> &'static str {
        match self {
            WikiPageKind::Overview => "overview",
            WikiPageKind::Area => "area",
            WikiPageKind::Decisions => "decisions",
            WikiPageKind::Howto => "howto",
            WikiPageKind::Glossary => "glossary",
            WikiPageKind::Changelog => "changelog",
        }
    }

    pub fn from_db_str(s: &str) -> Option<Self> {
        match s {
            "overview" => Some(WikiPageKind::Overview),
            "area" => Some(WikiPageKind::Area),
            "decisions" => Some(WikiPageKind::Decisions),
            "howto" => Some(WikiPageKind::Howto),
            "glossary" => Some(WikiPageKind::Glossary),
            "changelog" => Some(WikiPageKind::Changelog),
            _ => None,
        }
    }
}

/// One citation backing a wiki page's claims (PW-4). Every substantive claim
/// must resolve to a citation; the regeneration pipeline drops an uncited
/// section rather than persisting it. `source_kind` names what is cited (e.g.
/// `decision`, `work_product`, `board_item`, `ledger_fact`); `source_id`
/// references the specific ground-truth record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiCitation {
    pub source_kind: String,
    pub source_id: String,
}

impl WikiCitation {
    pub fn new(source_kind: impl Into<String>, source_id: impl Into<String>) -> Self {
        WikiCitation {
            source_kind: source_kind.into(),
            source_id: source_id.into(),
        }
    }
}

/// A client-facing wiki page — a derived projection row (PW-1…PW-6), never a
/// source of truth: reconstructable from ground truth by `rebuild` (PW-3).
///
/// Optional structure is absent by default: a freshly-built page is a root
/// (`parent_id = None`), first in order (`ord = 0`), fresh (`stale = false`),
/// and uncited (`citations` empty) until the regeneration pipeline attributes
/// and places it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiPage {
    pub id: String,
    pub office_id: String,
    /// `None` for the overview root; otherwise the parent in the nav tree (PW-6).
    pub parent_id: Option<String>,
    /// Sibling ordering under `parent_id`.
    pub ord: i64,
    pub kind: WikiPageKind,
    pub title: String,
    /// Generated plain-language content (PW-1).
    pub body: String,
    /// Sources backing the page (PW-4); non-empty once the pipeline attributes it.
    pub citations: Vec<WikiCitation>,
    /// Hash of the inputs this page was generated from (PW-5).
    pub source_fingerprint: String,
    pub generated_at: u64,
    /// `true` when the current source fingerprint differs from the stored one (PW-5).
    pub stale: bool,
}

impl WikiPage {
    /// A minimal page with structure absent by default (root, ord 0, uncited,
    /// fresh). Callers set `parent_id`/`ord`/`citations`/`source_fingerprint`
    /// as the regeneration pipeline places and attributes it.
    pub fn new(
        id: impl Into<String>,
        office_id: impl Into<String>,
        kind: WikiPageKind,
        title: impl Into<String>,
        body: impl Into<String>,
    ) -> Self {
        WikiPage {
            id: id.into(),
            office_id: office_id.into(),
            parent_id: None,
            ord: 0,
            kind,
            title: title.into(),
            body: body.into(),
            citations: Vec::new(),
            source_fingerprint: String::new(),
            generated_at: now_secs(),
            stale: false,
        }
    }
}

/// One entry in a page's change history (PW-5), appended newest-first by the
/// regeneration pipeline. `page_id` is `None` for an office-level change not
/// tied to a single page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiChangelogEntry {
    pub id: String,
    pub office_id: String,
    pub page_id: Option<String>,
    /// Human-readable "what changed".
    pub change: String,
    pub at: u64,
}

/// The wiki store seam the office regeneration pipeline writes through
/// (l2-project-wiki §4.2). Lives in the ports tier so `cronus-domain` (the
/// pipeline) never depends on `cronus-store-local` (the SQLite realization);
/// the client has no handle to this — only the curator-owned pipeline does
/// (PW-2). The plain `String` error mirrors the other DN-2 seams.
pub trait WikiCache {
    fn get_page(&self, id: &str) -> Result<Option<WikiPage>, String>;

    /// Apply one regeneration **transactionally** (PW-3): upsert every page
    /// and append every changelog entry as a single all-or-nothing unit. On
    /// any failure, nothing is written — the prior rows stay intact and
    /// correctly marked (a failed regeneration never leaves a half-written
    /// projection).
    fn apply_regeneration(
        &self,
        pages: &[WikiPage],
        changelog: &[WikiChangelogEntry],
    ) -> Result<(), String>;

    /// Every page belonging to an office — the input to the freshness sweep.
    fn pages_for_office(&self, office_id: &str) -> Result<Vec<WikiPage>, String>;

    /// Mark pages stale (PW-5): their sources drifted since generation and no
    /// regeneration has caught up, so the UI must show a stale marker rather
    /// than silently presenting them as current.
    fn mark_stale(&self, page_ids: &[String]) -> Result<(), String>;

    /// Change history newest-first (PW-5), at most `limit` entries.
    fn changelog(&self, office_id: &str, limit: usize) -> Result<Vec<WikiChangelogEntry>, String>;
}

/// The **read-only** client-facing wiki surface (PW-2/PW-6). The client is
/// handed a `&dyn WikiReadSurface`, which — by having no write method at all —
/// makes "the client can never curate the wiki" a compile-time property, not a
/// convention: there is simply no API to mutate a row through this trait. The
/// curator pipeline uses [`WikiCache`]; the client uses only this.
pub trait WikiReadSurface {
    /// One page by id, or `None`.
    fn page(&self, id: &str) -> Result<Option<WikiPage>, String>;

    /// The direct children of `parent_id` (or the roots when `None`), ordered
    /// for navigation — the overview → area → detail tree (PW-6).
    fn children(&self, office_id: &str, parent_id: Option<&str>) -> Result<Vec<WikiPage>, String>;

    /// Full-text search over page title + body, best matches first (PW-6).
    fn search(&self, office_id: &str, query: &str, limit: usize) -> Result<Vec<WikiPage>, String>;

    /// Change history newest-first (PW-5), at most `limit` entries.
    fn changelog(&self, office_id: &str, limit: usize) -> Result<Vec<WikiChangelogEntry>, String>;
}
