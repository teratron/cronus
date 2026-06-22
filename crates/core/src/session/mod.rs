//! Agent session loop — TurnContext, IterationBudget, InterruptFence,
//! text-loop detection, durable prompt admission, and session runner registry.

pub mod entry;
pub mod hooks;
pub mod interrupt;
pub mod migration;
pub mod turn;

pub use entry::SessionEntry;
pub use hooks::{HookOutcome, StopHook};
pub use interrupt::InterruptFence;
pub use turn::{IterationBudget, TurnContext};

use std::collections::HashMap;
use std::sync::Mutex;

use crate::context_router::SessionContext;

// ── SessionId ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SessionId(String);

impl SessionId {
    pub fn new(id: impl Into<String>) -> Self {
        SessionId(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

// ── RunnerState ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunnerStatus {
    Idle,
    Busy,
}

pub struct RunnerState {
    pub status: RunnerStatus,
    pub interrupt: InterruptFence,
}

// ── RunnerMap ─────────────────────────────────────────────────────────────────

/// Registry of active session runners.
///
/// Each session has at most one runner at a time. `assert_not_busy` is the
/// admission gate that prevents two callers from driving the same session
/// concurrently.
pub struct RunnerMap {
    inner: Mutex<HashMap<SessionId, RunnerState>>,
}

impl RunnerMap {
    pub fn new() -> Self {
        RunnerMap { inner: Mutex::new(HashMap::new()) }
    }

    /// Register a new session and return its fence.
    ///
    /// Initialises the session in `Idle` state.
    pub fn register(&self, id: SessionId) -> InterruptFence {
        let fence = InterruptFence::new();
        let mut map = self.inner.lock().expect("runner map lock poisoned");
        map.insert(
            id,
            RunnerState { status: RunnerStatus::Idle, interrupt: fence.clone() },
        );
        fence
    }

    /// Atomically assert that the session is not busy and mark it Busy.
    ///
    /// Returns `Err` if the session is already busy or not registered.
    pub fn assert_not_busy(&self, id: &SessionId) -> Result<(), SessionError> {
        let mut map = self.inner.lock().expect("runner map lock poisoned");
        let state = map.get_mut(id).ok_or(SessionError::NotRegistered)?;
        if state.status == RunnerStatus::Busy {
            return Err(SessionError::AlreadyBusy);
        }
        state.status = RunnerStatus::Busy;
        Ok(())
    }

    /// Mark a session as Idle again after a turn completes.
    pub fn mark_idle(&self, id: &SessionId) {
        let mut map = self.inner.lock().expect("runner map lock poisoned");
        if let Some(state) = map.get_mut(id) {
            state.status = RunnerStatus::Idle;
        }
    }

    /// Remove a session from the registry (retire).
    pub fn retire(&self, id: &SessionId) {
        let mut map = self.inner.lock().expect("runner map lock poisoned");
        map.remove(id);
    }

    pub fn is_registered(&self, id: &SessionId) -> bool {
        self.inner.lock().expect("runner map lock poisoned").contains_key(id)
    }
}

impl Default for RunnerMap {
    fn default() -> Self {
        Self::new()
    }
}

// ── SessionError ──────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq, Eq)]
pub enum SessionError {
    AlreadyBusy,
    NotRegistered,
    InterruptRequested,
    TextLoopDetected,
    GoalCapExceeded,
    Oversized { original_len: usize },
}

impl std::fmt::Display for SessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionError::AlreadyBusy => write!(f, "session is already running a turn"),
            SessionError::NotRegistered => write!(f, "session not registered"),
            SessionError::InterruptRequested => write!(f, "interrupted by fence"),
            SessionError::TextLoopDetected => write!(f, "text loop detected"),
            SessionError::GoalCapExceeded => write!(f, "goal re-entry cap exceeded"),
            SessionError::Oversized { original_len } => {
                write!(f, "output oversized ({original_len} chars), truncated")
            }
        }
    }
}

impl std::error::Error for SessionError {}

// ── Loop runner (seam) ────────────────────────────────────────────────────────

/// Seam: follow-up messages injected after each turn (wires in Phase 6).
pub fn get_follow_up_messages(_ctx: &SessionContext) -> Vec<String> {
    vec![]
}

/// Seam: steering messages injected on text-loop detection (wires in Phase 6).
pub fn get_steering_messages(_ctx: &SessionContext) -> Vec<String> {
    vec![]
}

// ── Output size guard ─────────────────────────────────────────────────────────

/// Maximum assistant output characters before truncation.
pub const MAX_OUTPUT_CHARS: usize = 15_000;

/// Truncate oversized output and annotate it.
pub fn guard_output_size(output: &str) -> (String, Option<SessionError>) {
    if output.len() <= MAX_OUTPUT_CHARS {
        return (output.to_owned(), None);
    }
    let truncated = format!(
        "{}\n[output truncated: {} chars]",
        &output[..MAX_OUTPUT_CHARS],
        output.len()
    );
    let err = SessionError::Oversized { original_len: output.len() };
    (truncated, Some(err))
}

// ── Goal cap ──────────────────────────────────────────────────────────────────

pub const MAX_GOAL_REACT: u32 = 12;
