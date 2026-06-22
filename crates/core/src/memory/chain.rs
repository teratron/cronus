//! Session chaining and Bellman trust propagation.

/// Automatically chain memories within a 2-hour session window.
pub const SESSION_CHAIN_WINDOW_SECS: u64 = 7200;

/// Bellman propagation hyperparameters.
pub const BELLMAN_GAMMA: f64 = 0.9;
pub const BELLMAN_ALPHA: f64 = 0.1;
pub const BELLMAN_MAX_DEPTH: usize = 2;
pub const BELLMAN_THRESHOLD: f64 = 0.5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChainKind {
    Continuation,
    Supersedes,
    RelatedTo,
}

impl ChainKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ChainKind::Continuation => "Continuation",
            ChainKind::Supersedes => "Supersedes",
            ChainKind::RelatedTo => "RelatedTo",
        }
    }

    pub fn from_db_str(s: &str) -> Option<Self> {
        match s {
            "Continuation" => Some(ChainKind::Continuation),
            "Supersedes" => Some(ChainKind::Supersedes),
            "RelatedTo" => Some(ChainKind::RelatedTo),
            _ => None,
        }
    }
}

/// Compute the propagated trust delta at a given graph depth.
///
/// Uses the Bellman credit equation:
///   `delta_d = base_delta * ALPHA * GAMMA^depth`
pub fn propagated_delta(base_delta: f64, depth: usize) -> f64 {
    base_delta * BELLMAN_ALPHA * BELLMAN_GAMMA.powi(depth as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chain_kind_roundtrip() {
        for kind in [ChainKind::Continuation, ChainKind::Supersedes, ChainKind::RelatedTo] {
            let s = kind.as_str();
            assert_eq!(ChainKind::from_db_str(s), Some(kind));
        }
    }

    #[test]
    fn propagated_delta_decays_with_depth() {
        let d0 = propagated_delta(1.0, 0);
        let d1 = propagated_delta(1.0, 1);
        let d2 = propagated_delta(1.0, 2);
        assert!(d0 > d1, "delta must decay with depth");
        assert!(d1 > d2, "delta must decay with depth");
    }

    #[test]
    fn propagated_delta_at_depth_0() {
        let d = propagated_delta(1.0, 0);
        assert!((d - BELLMAN_ALPHA).abs() < 1e-10);
    }

    #[test]
    fn session_window_is_two_hours() {
        assert_eq!(SESSION_CHAIN_WINDOW_SECS, 7200);
    }
}
