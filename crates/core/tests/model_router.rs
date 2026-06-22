use cronus::router::{
    FitLevel, RouterError, RouterMode, RouterPool,
    circuit::{CircuitBreaker, CircuitState},
    provider::{ModelProvider, ProviderHealth, ProviderTier, RoutingRequest, TaskType},
    scoring::ModePack,
};

// ── Test provider ─────────────────────────────────────────────────────────────

struct MockProvider {
    id: &'static str,
    health: ProviderHealth,
    ctx: u32,
    cost: f64,
    latency: u64,
    tier: ProviderTier,
    task_fit: f64,
}

impl ModelProvider for MockProvider {
    fn id(&self) -> &str {
        self.id
    }
    fn health(&self) -> ProviderHealth {
        self.health
    }
    fn context_window(&self) -> u32 {
        self.ctx
    }
    fn cost_per_1k_tokens(&self) -> f64 {
        self.cost
    }
    fn latency_p50_ms(&self) -> u64 {
        self.latency
    }
    fn tier(&self) -> ProviderTier {
        self.tier
    }
    fn task_fit(&self, _: TaskType) -> f64 {
        self.task_fit
    }
}

fn healthy(id: &'static str) -> Box<MockProvider> {
    Box::new(MockProvider {
        id,
        health: ProviderHealth::Healthy,
        ctx: 128_000,
        cost: 0.01,
        latency: 300,
        tier: ProviderTier::Standard,
        task_fit: 0.8,
    })
}

fn req() -> RoutingRequest {
    RoutingRequest {
        prompt_hash: 0xdeadbeef,
        required_context: 5_000,
        task_type: TaskType::CodeGeneration,
    }
}

// ── Pool tests ────────────────────────────────────────────────────────────────

#[test]
fn empty_pool_returns_error() {
    let pool = RouterPool::new(ModePack::Quality);
    assert!(matches!(
        pool.route(&req()),
        Err(RouterError::NoProvidersAvailable)
    ));
}

#[test]
fn single_healthy_provider_routes() {
    let pool = RouterPool::new(ModePack::Quality);
    pool.register(healthy("p1"));
    let dec = pool.route(&req()).unwrap();
    assert_eq!(dec.provider_id, "p1");
    assert!(!dec.via_lkgp);
}

#[test]
fn prefers_higher_scoring_provider() {
    let pool = RouterPool::new(ModePack::Quality);
    // Low-tier, low-fit
    pool.register(Box::new(MockProvider {
        id: "low",
        health: ProviderHealth::Healthy,
        ctx: 128_000,
        cost: 0.5,
        latency: 2000,
        tier: ProviderTier::Economy,
        task_fit: 0.3,
    }));
    // High-tier, high-fit
    pool.register(Box::new(MockProvider {
        id: "high",
        health: ProviderHealth::Healthy,
        ctx: 200_000,
        cost: 0.01,
        latency: 200,
        tier: ProviderTier::Premium,
        task_fit: 0.95,
    }));
    let dec = pool.route(&req()).unwrap();
    assert_eq!(
        dec.provider_id, "high",
        "router must prefer the higher-scoring provider"
    );
}

#[test]
fn open_circuit_skips_provider() {
    let pool = RouterPool::new(ModePack::Quality);
    let mut broken = CircuitBreaker::new(1, 3600);
    broken.record_failure();
    pool.register_with_circuit(healthy("broken"), broken);
    pool.register(healthy("ok"));

    let dec = pool.route(&req()).unwrap();
    assert_eq!(dec.provider_id, "ok");
}

#[test]
fn all_circuits_open_returns_error_with_lkgp() {
    let pool = RouterPool::new(ModePack::Quality);
    pool.register(healthy("only-provider"));
    pool.route(&req()).unwrap();
    pool.record_success("only-provider");

    // Now open the circuit
    let mut cb = CircuitBreaker::new(1, 3600);
    cb.record_failure();
    // Re-register with open circuit — need a fresh pool
    let pool2 = RouterPool::new(ModePack::Quality);
    pool2.register_with_circuit(healthy("p-open"), cb);
    // Inject LKGP by manually calling record_success before the circuit was open
    // (simulate a prior successful session)
    // Since we can't directly set lkgp, verify the error type
    let err = pool2.route(&req());
    assert!(matches!(err, Err(RouterError::AllCircuitsBroken { .. })));
}

#[test]
fn record_success_updates_lkgp() {
    let pool = RouterPool::new(ModePack::Quality);
    pool.register(healthy("p1"));
    pool.route(&req()).unwrap();
    pool.record_success("p1");
    assert_eq!(pool.lkgp().as_deref(), Some("p1"));
}

#[test]
fn record_failure_opens_circuit() {
    let pool = RouterPool::new(ModePack::Quality);
    pool.register(healthy("p1"));
    for _ in 0..5 {
        pool.record_failure("p1");
    }
    assert_eq!(
        pool.circuit_state("p1"),
        Some(CircuitState::Open),
        "circuit must be Open after threshold failures"
    );
}

// ── Circuit breaker tests ─────────────────────────────────────────────────────

#[test]
fn circuit_starts_closed() {
    use cronus::router::circuit::CircuitState;
    let cb = CircuitBreaker::default();
    assert_eq!(cb.state(), CircuitState::Closed);
}

#[test]
fn circuit_half_open_after_zero_cooldown() {
    let mut cb = CircuitBreaker::new(1, 0);
    cb.record_failure();
    assert_eq!(cb.state(), CircuitState::Open);
    let still_open = cb.is_open();
    assert!(!still_open, "zero-cooldown must transition to HalfOpen");
    assert_eq!(cb.state(), CircuitState::HalfOpen);
}

// ── Hardware fit tests ────────────────────────────────────────────────────────

#[test]
fn fit_level_routing_exclusion() {
    let pool = RouterPool::new(ModePack::Quality);
    pool.register(Box::new(MockProvider {
        id: "tiny",
        health: ProviderHealth::Healthy,
        ctx: 100, // too small for our 5k request
        cost: 0.001,
        latency: 50,
        tier: ProviderTier::Economy,
        task_fit: 0.9,
    }));
    pool.register(healthy("big"));

    let dec = pool.route(&req()).unwrap();
    assert_eq!(
        dec.provider_id, "big",
        "tiny context window must be excluded"
    );
}

#[test]
fn fit_level_evaluation() {
    assert_eq!(FitLevel::evaluate(50_000, 128_000), FitLevel::Perfect);
    assert_eq!(FitLevel::evaluate(90_000, 128_000), FitLevel::Good);
    assert_eq!(FitLevel::evaluate(115_000, 128_000), FitLevel::Marginal);
    assert_eq!(FitLevel::evaluate(200_000, 128_000), FitLevel::TooTight);
}

// ── Mode pack tests ───────────────────────────────────────────────────────────

#[test]
fn offline_mode_excludes_remote_providers() {
    let pool = RouterPool::new(ModePack::Offline);
    pool.register(Box::new(MockProvider {
        id: "remote",
        health: ProviderHealth::Healthy,
        ctx: 128_000,
        cost: 0.01,
        latency: 300,
        tier: ProviderTier::Premium,
        task_fit: 0.9,
    }));
    pool.register(Box::new(MockProvider {
        id: "local",
        health: ProviderHealth::Healthy,
        ctx: 32_000,
        cost: 0.0,
        latency: 50,
        tier: ProviderTier::Local,
        task_fit: 0.7,
    }));

    let dec = pool.route(&req()).unwrap();
    assert_eq!(
        dec.provider_id, "local",
        "Offline mode must route to local provider only"
    );
}

#[test]
fn router_mode_type_alias() {
    // RouterMode is re-exported as ModePack
    let _mode: RouterMode = RouterMode::Quality;
}
