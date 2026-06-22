//! 9-factor routing score and mode packs.
//!
//! Factors: health, quota, cost_inv, latency_inv, task_fit, specificity,
//! stability, tier (double-weighted) — total max score = 9.0.

use crate::router::{
    hardware::FitLevel,
    provider::{ModelProvider, ProviderHealth, ProviderTier, RoutingRequest},
};

/// Routing mode pack — adjusts factor weights.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModePack {
    /// Prefer lowest latency.
    ShipFast,
    /// Prefer lowest cost.
    CostSaver,
    /// Prefer task-fit and stability.
    #[default]
    Quality,
    /// Local providers only — ignores cloud tier preference.
    Offline,
}

/// Weights for the 9 scoring factors.
#[derive(Debug, Clone)]
pub struct ScoringWeights {
    pub health: f64,
    pub quota: f64,
    pub cost_inv: f64,
    pub latency_inv: f64,
    pub task_fit: f64,
    pub specificity: f64,
    pub stability: f64,
    /// Tier factor weight is applied twice (×2 multiplier).
    pub tier: f64,
}

impl ScoringWeights {
    fn for_mode(mode: ModePack) -> Self {
        match mode {
            ModePack::ShipFast => ScoringWeights {
                health: 1.0,
                quota: 1.0,
                cost_inv: 0.5,
                latency_inv: 2.0,
                task_fit: 0.8,
                specificity: 0.5,
                stability: 0.8,
                tier: 1.0,
            },
            ModePack::CostSaver => ScoringWeights {
                health: 1.0,
                quota: 1.0,
                cost_inv: 2.0,
                latency_inv: 0.5,
                task_fit: 0.8,
                specificity: 0.5,
                stability: 0.8,
                tier: 0.5,
            },
            ModePack::Quality => ScoringWeights {
                health: 1.0,
                quota: 1.0,
                cost_inv: 0.8,
                latency_inv: 0.8,
                task_fit: 1.5,
                specificity: 1.0,
                stability: 1.5,
                tier: 1.0,
            },
            ModePack::Offline => ScoringWeights {
                health: 1.0,
                quota: 1.0,
                cost_inv: 1.0,
                latency_inv: 1.0,
                task_fit: 1.0,
                specificity: 1.0,
                stability: 1.0,
                tier: 0.0, // tier irrelevant for local-only
            },
        }
    }
}

/// Compute the routing score for a provider against a request.
///
/// Returns 0.0 if the provider is unavailable or the context is too tight.
pub fn score(
    provider: &dyn ModelProvider,
    req: &RoutingRequest,
    mode: ModePack,
    max_cost: f64,
    max_latency_ms: u64,
) -> f64 {
    if provider.health() == ProviderHealth::Unavailable {
        return 0.0;
    }
    let fit = FitLevel::evaluate(req.required_context, provider.context_window());
    if fit == FitLevel::TooTight {
        return 0.0;
    }

    // Offline mode excludes non-local providers entirely
    if mode == ModePack::Offline && provider.tier() != ProviderTier::Local {
        return 0.0;
    }

    let w = ScoringWeights::for_mode(mode);

    let health_score = match provider.health() {
        ProviderHealth::Healthy => 1.0,
        ProviderHealth::Degraded => 0.5,
        ProviderHealth::Unavailable => 0.0,
    };

    // quota: 1.0 = unlimited (Phase 4 stub — assume full quota)
    let quota_score = 1.0_f64;

    // cost_inv: 0.0 when cost == max_cost, 1.0 when cost is free
    let cost_inv = if max_cost <= 0.0 {
        1.0
    } else {
        (1.0 - provider.cost_per_1k_tokens() / max_cost).clamp(0.0, 1.0)
    };

    // latency_inv: 0.0 when latency == max_latency, 1.0 when latency is 0
    let latency_inv = if max_latency_ms == 0 {
        1.0
    } else {
        (1.0 - provider.latency_p50_ms() as f64 / max_latency_ms as f64).clamp(0.0, 1.0)
    };

    let task_fit = provider.task_fit(req.task_type);
    let specificity = fit.score();

    // stability: proxy by tier ordering (higher tier = more stable)
    let stability = provider.tier() as u8 as f64 / ProviderTier::Premium as u8 as f64;

    // tier contributes with double weight
    let tier_score = provider.tier() as u8 as f64 / ProviderTier::Premium as u8 as f64;

    w.health * health_score
        + w.quota * quota_score
        + w.cost_inv * cost_inv
        + w.latency_inv * latency_inv
        + w.task_fit * task_fit
        + w.specificity * specificity
        + w.stability * stability
        + w.tier * tier_score   // weight ×2 is built into w.tier for Quality mode
        + w.tier * tier_score // second application of tier weight
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::router::provider::{ProviderHealth, ProviderTier, TaskType};

    struct P {
        id: &'static str,
        health: ProviderHealth,
        ctx: u32,
        cost: f64,
        latency: u64,
        tier: ProviderTier,
        fit: f64,
    }
    impl ModelProvider for P {
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
            self.fit
        }
    }

    fn req() -> RoutingRequest {
        RoutingRequest {
            prompt_hash: 0,
            required_context: 10_000,
            task_type: TaskType::Chat,
        }
    }

    #[test]
    fn unavailable_provider_scores_zero() {
        let p = P {
            id: "p",
            health: ProviderHealth::Unavailable,
            ctx: 128_000,
            cost: 0.01,
            latency: 200,
            tier: ProviderTier::Standard,
            fit: 0.8,
        };
        assert_eq!(score(&p, &req(), ModePack::Quality, 1.0, 1000), 0.0);
    }

    #[test]
    fn context_overflow_scores_zero() {
        let p = P {
            id: "p",
            health: ProviderHealth::Healthy,
            ctx: 1_000,
            cost: 0.01,
            latency: 200,
            tier: ProviderTier::Standard,
            fit: 0.8,
        };
        assert_eq!(score(&p, &req(), ModePack::Quality, 1.0, 1000), 0.0);
    }

    #[test]
    fn healthy_provider_scores_positive() {
        let p = P {
            id: "p",
            health: ProviderHealth::Healthy,
            ctx: 128_000,
            cost: 0.01,
            latency: 200,
            tier: ProviderTier::Standard,
            fit: 0.9,
        };
        let s = score(&p, &req(), ModePack::Quality, 1.0, 1000);
        assert!(s > 0.0, "healthy provider must score positive");
    }

    #[test]
    fn offline_mode_excludes_non_local() {
        let remote = P {
            id: "r",
            health: ProviderHealth::Healthy,
            ctx: 128_000,
            cost: 0.01,
            latency: 200,
            tier: ProviderTier::Standard,
            fit: 0.9,
        };
        let local = P {
            id: "l",
            health: ProviderHealth::Healthy,
            ctx: 32_000,
            cost: 0.0,
            latency: 50,
            tier: ProviderTier::Local,
            fit: 0.7,
        };
        let remote_score = score(&remote, &req(), ModePack::Offline, 1.0, 1000);
        let local_score = score(&local, &req(), ModePack::Offline, 1.0, 1000);
        assert_eq!(
            remote_score, 0.0,
            "remote provider must be excluded in Offline mode"
        );
        assert!(
            local_score > 0.0,
            "local provider must score positive in Offline mode"
        );
    }
}
