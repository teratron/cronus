//! Semantic cache seam.
//!
//! This is a stub that always misses. The real implementation
//! requires an embedding encoder (sqlite-vec) which is deferred.

use crate::router::provider::RouteDecision;

pub struct SemanticCache;

impl SemanticCache {
    pub fn new() -> Self {
        SemanticCache
    }

    /// Look up a cached routing decision for a prompt hash.
    ///
    /// Always returns `None` for now — no embedding encoder is available.
    pub fn lookup(&self, _prompt_hash: u64) -> Option<RouteDecision> {
        None
    }

    /// Store a routing decision for future lookups.
    ///
    /// No-op (stub).
    pub fn store(&self, _prompt_hash: u64, _decision: &RouteDecision) {}
}

impl Default for SemanticCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_always_misses_at_phase4() {
        let cache = SemanticCache::new();
        assert!(cache.lookup(0xdeadbeef).is_none());
    }
}
