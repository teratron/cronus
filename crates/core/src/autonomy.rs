//! Agent autonomy — ladder, SecurityPolicy gate, CommandRiskLevel classifier,
//! ActionTracker rolling cap, and ApprovalGate manager.

use std::collections::HashMap;
use std::time::{Duration, Instant};

// ── Autonomy level ────────────────────────────────────────────────────────────

/// Three-rung autonomy ladder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutonomyLevel {
    Supervised,
    SemiAutonomous,
    Autonomous,
}

// ── Command risk ──────────────────────────────────────────────────────────────

/// Risk classification for a proposed agent command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandRiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Classify a command string into a `CommandRiskLevel`.
///
/// Rules (highest-matching wins):
/// - Contains destructive patterns (`rm -rf`, `DROP TABLE`, shell meta) → Critical
/// - Write ops without a path guard → High
/// - Recognized read-only patterns → Low
/// - Everything else → Medium
pub fn classify_command(cmd: &str) -> CommandRiskLevel {
    let lower = cmd.to_lowercase();

    let critical_patterns = [
        "rm -rf",
        "drop table",
        "drop database",
        "format ",
        "mkfs",
        "; rm ",
        "&& rm ",
        "| rm ",
        "`rm",
        "$(rm",
        "truncate /",
    ];
    for pat in &critical_patterns {
        if lower.contains(pat) {
            return CommandRiskLevel::Critical;
        }
    }

    let high_patterns = ["write", "delete", "remove", "overwrite", "execute", "chmod", "chown"];
    for pat in &high_patterns {
        if lower.contains(pat) {
            return CommandRiskLevel::High;
        }
    }

    let low_patterns = ["read", "list", "show", "get", "fetch", "search", "query", "cat ", "ls ", "find "];
    for pat in &low_patterns {
        if lower.contains(pat) {
            return CommandRiskLevel::Low;
        }
    }

    CommandRiskLevel::Medium
}

// ── SecurityPolicy gate ───────────────────────────────────────────────────────

/// Decision returned by the security policy gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateDecision {
    Allow,
    RequireApproval,
    Deny,
}

/// Evaluate whether `risk` is permitted at `level`.
///
/// Matrix (rows = AutonomyLevel, cols = CommandRiskLevel):
/// ```text
///                Low   Medium  High    Critical
/// Supervised:    Allow  Req    Req     Deny
/// Semi:          Allow  Allow  Req     Deny
/// Autonomous:    Allow  Allow  Allow   Req
/// ```
pub fn evaluate(level: AutonomyLevel, risk: CommandRiskLevel) -> GateDecision {
    match (level, risk) {
        // Supervised
        (AutonomyLevel::Supervised, CommandRiskLevel::Low) => GateDecision::Allow,
        (AutonomyLevel::Supervised, CommandRiskLevel::Medium) => GateDecision::RequireApproval,
        (AutonomyLevel::Supervised, CommandRiskLevel::High) => GateDecision::RequireApproval,
        (AutonomyLevel::Supervised, CommandRiskLevel::Critical) => GateDecision::Deny,
        // SemiAutonomous
        (AutonomyLevel::SemiAutonomous, CommandRiskLevel::Low) => GateDecision::Allow,
        (AutonomyLevel::SemiAutonomous, CommandRiskLevel::Medium) => GateDecision::Allow,
        (AutonomyLevel::SemiAutonomous, CommandRiskLevel::High) => GateDecision::RequireApproval,
        (AutonomyLevel::SemiAutonomous, CommandRiskLevel::Critical) => GateDecision::Deny,
        // Autonomous
        (AutonomyLevel::Autonomous, CommandRiskLevel::Low) => GateDecision::Allow,
        (AutonomyLevel::Autonomous, CommandRiskLevel::Medium) => GateDecision::Allow,
        (AutonomyLevel::Autonomous, CommandRiskLevel::High) => GateDecision::Allow,
        (AutonomyLevel::Autonomous, CommandRiskLevel::Critical) => GateDecision::RequireApproval,
    }
}

// ── ActionTracker — sliding-window rolling cap ────────────────────────────────

/// Rolling action rate limiter using a 1-hour sliding window.
pub struct ActionTracker {
    cap: usize,
    window: Duration,
    timestamps: Vec<Instant>,
}

impl ActionTracker {
    pub fn new(cap: usize) -> Self {
        ActionTracker {
            cap,
            window: Duration::from_secs(3600),
            timestamps: Vec::new(),
        }
    }

    pub fn new_with_window(cap: usize, window: Duration) -> Self {
        ActionTracker { cap, window, timestamps: Vec::new() }
    }

    /// Record an action. Returns `Err(AutonomyError::RateLimitExceeded)` when
    /// the rolling count is at or above `cap`.
    pub fn record(&mut self, now: Instant) -> Result<(), AutonomyError> {
        self.prune(now);
        if self.timestamps.len() >= self.cap {
            return Err(AutonomyError::RateLimitExceeded);
        }
        self.timestamps.push(now);
        Ok(())
    }

    /// Current count within the sliding window.
    pub fn count(&mut self, now: Instant) -> usize {
        self.prune(now);
        self.timestamps.len()
    }

    fn prune(&mut self, now: Instant) {
        self.timestamps.retain(|&t| now.duration_since(t) < self.window);
    }
}

// ── ApprovalGate ──────────────────────────────────────────────────────────────

/// Outcome of an approval decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalDecision {
    Approved,
    Denied,
}

/// State of a pending approval gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalState {
    Pending,
    Resolved(ApprovalDecision),
}

/// A pending approval request.
pub struct ApprovalGate {
    pub id: ApprovalId,
    pub command: String,
    pub risk: CommandRiskLevel,
    pub created_at: Instant,
    pub ttl: Duration,
    pub state: ApprovalState,
}

impl ApprovalGate {
    fn new(id: ApprovalId, command: String, risk: CommandRiskLevel, ttl_ms: u64) -> Self {
        ApprovalGate {
            id,
            command,
            risk,
            created_at: Instant::now(),
            ttl: Duration::from_millis(ttl_ms),
            state: ApprovalState::Pending,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.is_expired_at(Instant::now())
    }

    pub fn is_expired_at(&self, now: Instant) -> bool {
        now.duration_since(self.created_at) >= self.ttl
    }

    /// An expired pending gate auto-denies.
    pub fn effective_decision(&self, now: Instant) -> Option<ApprovalDecision> {
        match self.state {
            ApprovalState::Pending if self.is_expired_at(now) => Some(ApprovalDecision::Denied),
            ApprovalState::Pending => None,
            ApprovalState::Resolved(d) => Some(d),
        }
    }
}

// ── ApprovalId ────────────────────────────────────────────────────────────────

/// Unique identifier for an approval gate, derived from a counter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ApprovalId(u64);

// ── ApprovalManager ───────────────────────────────────────────────────────────

/// Grace period after resolution before a gate can be evicted.
pub const RESOLVED_ENTRY_GRACE_MS: u64 = 15_000;

/// Manages the lifecycle of approval gates.
pub struct ApprovalManager {
    next_id: u64,
    gates: HashMap<ApprovalId, ApprovalGate>,
}

impl Default for ApprovalManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ApprovalManager {
    pub fn new() -> Self {
        ApprovalManager {
            next_id: 1,
            gates: HashMap::new(),
        }
    }

    /// Create a pending gate and return its ID.
    pub fn create(&mut self, command: String, risk: CommandRiskLevel, ttl_ms: u64) -> ApprovalId {
        let id = ApprovalId(self.next_id);
        self.next_id += 1;
        self.gates.insert(id, ApprovalGate::new(id, command, risk, ttl_ms));
        id
    }

    /// Register an approval decision for an existing gate.
    ///
    /// Registering the same ID twice is idempotent — second call is ignored.
    pub fn register(&mut self, id: ApprovalId, decision: ApprovalDecision) -> Result<(), AutonomyError> {
        match self.gates.get_mut(&id) {
            Some(gate) => {
                if gate.state == ApprovalState::Pending {
                    gate.state = ApprovalState::Resolved(decision);
                }
                Ok(())
            }
            None => Err(AutonomyError::GateNotFound),
        }
    }

    /// Query the current decision for a gate.
    pub fn decision(&self, id: ApprovalId, now: Instant) -> Option<ApprovalDecision> {
        self.gates.get(&id)?.effective_decision(now)
    }

    /// Evict resolved gates older than the grace period.
    pub fn gc(&mut self, now: Instant) {
        self.gates.retain(|_, g| {
            if let ApprovalState::Resolved(_) = g.state {
                let age = now.duration_since(g.created_at);
                age < Duration::from_millis(RESOLVED_ENTRY_GRACE_MS)
            } else {
                true
            }
        });
    }
}

// ── Error ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutonomyError {
    RateLimitExceeded,
    GateNotFound,
}

impl std::fmt::Display for AutonomyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AutonomyError::RateLimitExceeded => write!(f, "autonomy: action rate limit exceeded"),
            AutonomyError::GateNotFound => write!(f, "autonomy: approval gate not found"),
        }
    }
}

impl std::error::Error for AutonomyError {}
