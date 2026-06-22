//! Memory consolidation — merge redundant entries and evict stale ones.
//!
//! Consolidation runs at startup (bounded by MAX_ROLLOUTS_PER_STARTUP) and
//! is gated on idle time. The LLM-assisted dedup and scoring pipeline is a
//! seam; this module exposes the configuration contract and a no-op runner
//! that the real implementation will replace.

pub const MAX_ROLLOUTS_PER_STARTUP: usize = 2;
pub const MAX_ROLLOUT_AGE_DAYS: u64 = 10;
pub const MIN_ROLLOUT_IDLE_HOURS: u64 = 6;
pub const MAX_RAW_MEMORIES: usize = 256;
pub const MAX_UNUSED_DAYS: u64 = 30;

/// Configuration for a consolidation pass.
#[derive(Debug, Clone)]
pub struct ConsolidationConfig {
    pub max_rollouts: usize,
    pub max_age_days: u64,
    pub min_idle_hours: u64,
    pub max_raw: usize,
    pub max_unused_days: u64,
}

impl Default for ConsolidationConfig {
    fn default() -> Self {
        ConsolidationConfig {
            max_rollouts: MAX_ROLLOUTS_PER_STARTUP,
            max_age_days: MAX_ROLLOUT_AGE_DAYS,
            min_idle_hours: MIN_ROLLOUT_IDLE_HOURS,
            max_raw: MAX_RAW_MEMORIES,
            max_unused_days: MAX_UNUSED_DAYS,
        }
    }
}

/// Summary produced by a consolidation pass.
#[derive(Debug, Default)]
pub struct ConsolidationResult {
    pub merged: usize,
    pub evicted: usize,
    pub skipped: bool,
}

/// Consolidation runner seam.
///
/// The real implementation requires an LLM scoring pipeline; at Phase 4
/// this is a no-op that always reports `skipped = true`.
pub trait Consolidator: Send + Sync {
    fn run(&self, config: &ConsolidationConfig) -> ConsolidationResult;
}

/// No-op consolidator used at Phase 4.
pub struct NoOpConsolidator;

impl Consolidator for NoOpConsolidator {
    fn run(&self, _config: &ConsolidationConfig) -> ConsolidationResult {
        ConsolidationResult { merged: 0, evicted: 0, skipped: true }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_matches_constants() {
        let cfg = ConsolidationConfig::default();
        assert_eq!(cfg.max_rollouts, MAX_ROLLOUTS_PER_STARTUP);
        assert_eq!(cfg.max_age_days, MAX_ROLLOUT_AGE_DAYS);
        assert_eq!(cfg.max_raw, MAX_RAW_MEMORIES);
    }

    #[test]
    fn noop_consolidator_skips() {
        let c = NoOpConsolidator;
        let result = c.run(&ConsolidationConfig::default());
        assert!(result.skipped);
        assert_eq!(result.merged, 0);
        assert_eq!(result.evicted, 0);
    }
}
