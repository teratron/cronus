//! ModelProvider trait and supporting types.

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

/// Error returned by a provider when routing fails.
#[derive(Debug)]
pub enum ProviderError {
    Unavailable(String),
    ContextOverflow { required: u32, available: u32 },
    QuotaExhausted,
    RateLimit,
}

impl std::fmt::Display for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderError::Unavailable(r) => write!(f, "provider unavailable: {r}"),
            ProviderError::ContextOverflow { required, available } => {
                write!(f, "context overflow: need {required}, have {available}")
            }
            ProviderError::QuotaExhausted => write!(f, "quota exhausted"),
            ProviderError::RateLimit => write!(f, "rate limited"),
        }
    }
}

impl std::error::Error for ProviderError {}

/// A routing request passed to the pool.
#[derive(Debug, Clone)]
pub struct RoutingRequest {
    /// Hash of the prompt for semantic cache lookup.
    pub prompt_hash: u64,
    /// Number of tokens required in context.
    pub required_context: u32,
    /// Task category for scoring.
    pub task_type: TaskType,
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

/// The selected provider and routing metadata.
#[derive(Debug, Clone)]
pub struct RouteDecision {
    pub provider_id: String,
    pub score: f64,
    pub via_lkgp: bool,
    pub via_bandit: bool,
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

#[cfg(test)]
mod tests {
    use super::*;

    struct MockProvider {
        pub id: &'static str,
        pub health: ProviderHealth,
        pub ctx: u32,
        pub cost: f64,
        pub latency: u64,
        pub tier: ProviderTier,
    }

    impl ModelProvider for MockProvider {
        fn id(&self) -> &str { self.id }
        fn health(&self) -> ProviderHealth { self.health }
        fn context_window(&self) -> u32 { self.ctx }
        fn cost_per_1k_tokens(&self) -> f64 { self.cost }
        fn latency_p50_ms(&self) -> u64 { self.latency }
        fn tier(&self) -> ProviderTier { self.tier }
        fn task_fit(&self, _task: TaskType) -> f64 { 0.8 }
    }

    #[test]
    fn provider_tier_ordering() {
        assert!(ProviderTier::Local < ProviderTier::Economy);
        assert!(ProviderTier::Economy < ProviderTier::Standard);
        assert!(ProviderTier::Standard < ProviderTier::Premium);
    }

    #[test]
    fn mock_provider_contract() {
        let p = MockProvider {
            id: "test",
            health: ProviderHealth::Healthy,
            ctx: 128_000,
            cost: 0.01,
            latency: 300,
            tier: ProviderTier::Standard,
        };
        assert_eq!(p.id(), "test");
        assert_eq!(p.health(), ProviderHealth::Healthy);
        assert_eq!(p.context_window(), 128_000);
    }
}
