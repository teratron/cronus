//! Model router — selects the best provider for a request.
//!
//! Routing applies a 9-factor score, circuit breakers, LKGP fallback,
//! and a deterministic 5% bandit exploration policy.

pub mod cache;
pub mod circuit;
pub mod hardware;
pub mod provider;
pub mod recovery;
pub mod scoring;

use std::sync::{
    Mutex,
    atomic::{AtomicU64, Ordering},
};

use cache::SemanticCache;
use circuit::{CircuitBreaker, CircuitState};
use provider::{ModelProvider, ProviderHealth, RoutingRequest, RouteDecision};
use scoring::{ModePack, score};

pub use hardware::FitLevel;
pub use provider::{ProviderError, ProviderTier, TaskType};
pub use scoring::ModePack as RouterMode;

// ── Error ─────────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum RouterError {
    NoProvidersAvailable,
    AllCircuitsBroken { lkgp_id: Option<String> },
}

impl std::fmt::Display for RouterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RouterError::NoProvidersAvailable => write!(f, "no model providers registered"),
            RouterError::AllCircuitsBroken { lkgp_id } => {
                if let Some(id) = lkgp_id {
                    write!(f, "all provider circuits open; LKGP fallback: {id}")
                } else {
                    write!(f, "all provider circuits open; no LKGP available")
                }
            }
        }
    }
}

impl std::error::Error for RouterError {}

// ── RouterPool ────────────────────────────────────────────────────────────────

/// Deterministic exploration counter — every 20th call uses bandit selection.
static BANDIT_COUNTER: AtomicU64 = AtomicU64::new(0);

struct ProviderEntry {
    provider: Box<dyn ModelProvider>,
    circuit: CircuitBreaker,
}

struct PoolInner {
    entries: Vec<ProviderEntry>,
    mode: ModePack,
    lkgp: Option<String>,
    max_cost: f64,
    max_latency_ms: u64,
}

pub struct RouterPool {
    inner: Mutex<PoolInner>,
    cache: SemanticCache,
}

impl RouterPool {
    pub fn new(mode: ModePack) -> Self {
        RouterPool {
            inner: Mutex::new(PoolInner {
                entries: Vec::new(),
                mode,
                lkgp: None,
                max_cost: 1.0,
                max_latency_ms: 5000,
            }),
            cache: SemanticCache::new(),
        }
    }

    /// Register a provider. Uses default circuit breaker (threshold=5, cooldown=60s).
    pub fn register(&self, provider: Box<dyn ModelProvider>) {
        let mut inner = self.inner.lock().expect("router lock poisoned");
        inner.entries.push(ProviderEntry {
            provider,
            circuit: CircuitBreaker::default(),
        });
    }

    /// Register a provider with a custom circuit breaker.
    pub fn register_with_circuit(
        &self,
        provider: Box<dyn ModelProvider>,
        circuit: CircuitBreaker,
    ) {
        let mut inner = self.inner.lock().expect("router lock poisoned");
        inner.entries.push(ProviderEntry { provider, circuit });
    }

    pub fn set_mode(&self, mode: ModePack) {
        self.inner.lock().expect("router lock poisoned").mode = mode;
    }

    /// Record a successful call to a provider — resets its circuit.
    pub fn record_success(&self, provider_id: &str) {
        let mut inner = self.inner.lock().expect("router lock poisoned");
        for entry in &mut inner.entries {
            if entry.provider.id() == provider_id {
                entry.circuit.record_success();
                inner.lkgp = Some(provider_id.to_owned());
                return;
            }
        }
    }

    /// Record a failed call to a provider — may open its circuit.
    pub fn record_failure(&self, provider_id: &str) {
        let mut inner = self.inner.lock().expect("router lock poisoned");
        for entry in &mut inner.entries {
            if entry.provider.id() == provider_id {
                entry.circuit.record_failure();
                return;
            }
        }
    }

    /// Return the current LKGP (Last Known Good Provider) ID, if any.
    pub fn lkgp(&self) -> Option<String> {
        self.inner.lock().expect("router lock poisoned").lkgp.clone()
    }

    /// Return the current circuit state for a registered provider.
    pub fn circuit_state(&self, provider_id: &str) -> Option<circuit::CircuitState> {
        let inner = self.inner.lock().expect("router lock poisoned");
        inner
            .entries
            .iter()
            .find(|e| e.provider.id() == provider_id)
            .map(|e| e.circuit.state())
    }

    /// Select the best provider for a request.
    ///
    /// Returns an error if all circuits are open (includes LKGP ID if known).
    pub fn route(&self, req: &RoutingRequest) -> Result<RouteDecision, RouterError> {
        // Semantic cache lookup (always misses at Phase 4)
        if let Some(cached) = self.cache.lookup(req.prompt_hash) {
            return Ok(cached);
        }

        let mut inner = self.inner.lock().expect("router lock poisoned");

        if inner.entries.is_empty() {
            return Err(RouterError::NoProvidersAvailable);
        }

        // 5% bandit exploration — pick a random healthy provider.
        // Use `state()` (&self) rather than `is_open()` (&mut self) because the
        // `find` predicate only gets a shared reference to each entry.
        let use_bandit = BANDIT_COUNTER.fetch_add(1, Ordering::Relaxed).is_multiple_of(20);
        if use_bandit
            && let Some(entry) = inner.entries.iter_mut().find(|e| {
                e.circuit.state() != CircuitState::Open
                    && e.provider.health() != ProviderHealth::Unavailable
            })
        {
            return Ok(RouteDecision {
                provider_id: entry.provider.id().to_owned(),
                score: 0.0,
                via_lkgp: false,
                via_bandit: true,
            });
        }

        // Score all healthy providers
        let mode = inner.mode;
        let max_cost = inner.max_cost;
        let max_latency = inner.max_latency_ms;

        let mut best_id: Option<String> = None;
        let mut best_score = f64::NEG_INFINITY;

        for entry in inner.entries.iter_mut() {
            if entry.circuit.is_open() {
                continue;
            }
            let s = score(
                entry.provider.as_ref(),
                req,
                mode,
                max_cost,
                max_latency,
            );
            if s > best_score {
                best_score = s;
                best_id = Some(entry.provider.id().to_owned());
            }
        }

        if let Some(id) = best_id {
            return Ok(RouteDecision {
                provider_id: id,
                score: best_score,
                via_lkgp: false,
                via_bandit: false,
            });
        }

        // All circuits open — LKGP fallback
        let lkgp = inner.lkgp.clone();
        Err(RouterError::AllCircuitsBroken { lkgp_id: lkgp })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use provider::{ProviderHealth, ProviderTier, TaskType};

    struct TestProvider {
        id: &'static str,
        health: ProviderHealth,
    }

    impl ModelProvider for TestProvider {
        fn id(&self) -> &str { self.id }
        fn health(&self) -> ProviderHealth { self.health }
        fn context_window(&self) -> u32 { 128_000 }
        fn cost_per_1k_tokens(&self) -> f64 { 0.01 }
        fn latency_p50_ms(&self) -> u64 { 300 }
        fn tier(&self) -> ProviderTier { ProviderTier::Standard }
        fn task_fit(&self, _: TaskType) -> f64 { 0.8 }
    }

    fn req() -> RoutingRequest {
        RoutingRequest {
            prompt_hash: 42,
            required_context: 1000,
            task_type: TaskType::Chat,
        }
    }

    #[test]
    fn routes_to_healthy_provider() {
        let pool = RouterPool::new(ModePack::Quality);
        pool.register(Box::new(TestProvider { id: "p1", health: ProviderHealth::Healthy }));

        let decision = pool.route(&req()).expect("must route");
        assert_eq!(decision.provider_id, "p1");
    }

    #[test]
    fn no_providers_returns_error() {
        let pool = RouterPool::new(ModePack::Quality);
        assert!(matches!(pool.route(&req()), Err(RouterError::NoProvidersAvailable)));
    }

    #[test]
    fn records_success_updates_lkgp() {
        let pool = RouterPool::new(ModePack::Quality);
        pool.register(Box::new(TestProvider { id: "p1", health: ProviderHealth::Healthy }));
        pool.route(&req()).unwrap();
        pool.record_success("p1");
        let inner = pool.inner.lock().unwrap();
        assert_eq!(inner.lkgp.as_deref(), Some("p1"));
    }

    #[test]
    fn open_circuit_skips_provider() {
        let pool = RouterPool::new(ModePack::Quality);
        let cb = CircuitBreaker::new(1, 3600); // 1-hour cooldown — stays Open
        let mut cb_open = cb;
        cb_open.record_failure(); // opens it
        pool.register_with_circuit(
            Box::new(TestProvider { id: "broken", health: ProviderHealth::Healthy }),
            cb_open,
        );
        pool.register(Box::new(TestProvider { id: "ok", health: ProviderHealth::Healthy }));

        let decision = pool.route(&req()).expect("must route via fallback");
        assert_eq!(decision.provider_id, "ok");
    }
}
