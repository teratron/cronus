//! TurnContext and IterationBudget.

use super::SessionId;
use crate::context_router::ContextBundle;

// ── IterationBudget ───────────────────────────────────────────────────────────

/// Per-turn iteration and token budget.
#[derive(Debug, Clone)]
pub struct IterationBudget {
    pub max_iterations: u32,
    pub spent_iterations: u32,
    pub max_tokens: u64,
    pub spent_tokens: u64,
}

impl IterationBudget {
    pub fn new(max_iterations: u32, max_tokens: u64) -> Self {
        IterationBudget {
            max_iterations,
            spent_iterations: 0,
            max_tokens,
            spent_tokens: 0,
        }
    }

    pub fn tick(&mut self) {
        self.spent_iterations += 1;
    }

    pub fn spend_tokens(&mut self, n: u64) {
        self.spent_tokens += n;
    }

    pub fn is_exhausted(&self) -> bool {
        self.spent_iterations >= self.max_iterations || self.spent_tokens >= self.max_tokens
    }

    /// Reset for a new turn (keeps limits, zeroes counters).
    pub fn reset(&mut self) {
        self.spent_iterations = 0;
        self.spent_tokens = 0;
    }
}

// ── TurnContext ───────────────────────────────────────────────────────────────

/// Context for a single agent turn.
pub struct TurnContext {
    pub session_id: SessionId,
    pub iteration: u32,
    pub budget: IterationBudget,
    pub context: ContextBundle,
}

impl TurnContext {
    pub fn new(
        session_id: SessionId,
        iteration: u32,
        budget: IterationBudget,
        context: ContextBundle,
    ) -> Self {
        TurnContext {
            session_id,
            iteration,
            budget,
            context,
        }
    }

    /// Advance to the next iteration within this turn.
    pub fn tick(&mut self) {
        self.iteration += 1;
        self.budget.tick();
    }
}
