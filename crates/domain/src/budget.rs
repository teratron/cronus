//! Budget engine — hierarchical budget policies, cost events, hard-stop enforcement.
//!
//! Foundation: policy CRUD + cost ingestion + kanban seam.
//! Real wiring to kanban board's done-transition guard is deferred.

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetPeriod {
    Monthly,
    Weekly,
    Daily,
    Unlimited,
}

#[derive(Debug, Clone)]
pub struct BudgetPolicy {
    pub workspace_id: Option<String>,
    pub role_id: Option<String>,
    pub card_id: Option<String>,
    pub limit_usd: f64,
    pub period: BudgetPeriod,
}

impl BudgetPolicy {
    pub fn workspace(workspace_id: &str, limit_usd: f64, period: BudgetPeriod) -> Self {
        BudgetPolicy {
            workspace_id: Some(workspace_id.to_string()),
            role_id: None,
            card_id: None,
            limit_usd,
            period,
        }
    }

    pub fn for_card(card_id: &str, limit_usd: f64, period: BudgetPeriod) -> Self {
        BudgetPolicy {
            workspace_id: None,
            role_id: None,
            card_id: Some(card_id.to_string()),
            limit_usd,
            period,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CostEvent {
    pub session_id: String,
    pub card_id: Option<String>,
    pub role_id: Option<String>,
    pub amount_usd: f64,
    pub tokens_used: u32,
    pub model: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BudgetStatus {
    Ok { spent: f64, remaining: f64 },
    Exhausted { spent: f64, limit: f64 },
}

#[derive(Debug)]
pub enum BudgetError {
    Exhausted { spent: f64, limit: f64 },
    NoPolicyFound,
}

impl std::fmt::Display for BudgetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BudgetError::Exhausted { spent, limit } => {
                write!(f, "budget exhausted: spent={spent:.4} limit={limit:.4}")
            }
            BudgetError::NoPolicyFound => write!(f, "no budget policy found"),
        }
    }
}

impl std::error::Error for BudgetError {}

/// In-memory budget engine (SQLite-backed in production).
#[derive(Debug, Default)]
pub struct BudgetEngine {
    policies: Vec<BudgetPolicy>,
    spent: HashMap<String, f64>,
}

impl BudgetEngine {
    pub fn new() -> Self {
        BudgetEngine::default()
    }

    pub fn add_policy(&mut self, policy: BudgetPolicy) {
        self.policies.push(policy);
    }

    /// Resolve the most-specific policy for a cost event (card > role > workspace).
    fn resolve_policy(&self, event: &CostEvent) -> Option<&BudgetPolicy> {
        // card-specific (most specific)
        if let Some(card_id) = &event.card_id
            && let Some(p) = self
                .policies
                .iter()
                .find(|p| p.card_id.as_deref() == Some(card_id))
        {
            return Some(p);
        }
        // role-specific
        if let Some(role_id) = &event.role_id
            && let Some(p) = self
                .policies
                .iter()
                .find(|p| p.role_id.as_deref() == Some(role_id))
        {
            return Some(p);
        }
        // workspace fallback
        self.policies.iter().find(|p| p.workspace_id.is_some())
    }

    /// Ingest a cost event. Returns Err if the budget is exhausted.
    pub fn ingest_cost(&mut self, event: &CostEvent) -> Result<BudgetStatus, BudgetError> {
        let key = event
            .card_id
            .clone()
            .or_else(|| event.role_id.clone())
            .unwrap_or_else(|| event.session_id.clone());

        let limit = match self.resolve_policy(event) {
            Some(p) => p.limit_usd,
            None => f64::INFINITY,
        };

        let spent = self.spent.entry(key).or_default();
        *spent += event.amount_usd;

        if *spent > limit {
            Err(BudgetError::Exhausted {
                spent: *spent,
                limit,
            })
        } else {
            Ok(BudgetStatus::Ok {
                spent: *spent,
                remaining: limit - *spent,
            })
        }
    }

    /// Running total for a given key (card_id / role_id / session_id).
    pub fn spent_for(&self, key: &str) -> f64 {
        self.spent.get(key).copied().unwrap_or(0.0)
    }

    /// Reset all counters (monthly reset).
    pub fn reset(&mut self) {
        self.spent.clear();
    }
}
