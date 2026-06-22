//! Hardware-fit evaluation — checks whether a provider can serve the request
//! given the local context budget.

/// Fit level of a provider relative to the requested context size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FitLevel {
    /// Context comfortably fits (< 60 % of window).
    Perfect,
    /// Context fits with room (60–85 % of window).
    Good,
    /// Context fits but barely (85–100 % of window).
    Marginal,
    /// Context exceeds the provider's window.
    TooTight,
}

impl FitLevel {
    /// Evaluate fit given required token count and the provider's context window.
    pub fn evaluate(required: u32, context_window: u32) -> Self {
        if context_window == 0 || required > context_window {
            return FitLevel::TooTight;
        }
        let ratio = required as f64 / context_window as f64;
        if ratio < 0.60 {
            FitLevel::Perfect
        } else if ratio < 0.85 {
            FitLevel::Good
        } else {
            FitLevel::Marginal
        }
    }

    /// Score contribution from fit level (0.0–1.0).
    pub fn score(self) -> f64 {
        match self {
            FitLevel::Perfect => 1.0,
            FitLevel::Good => 0.75,
            FitLevel::Marginal => 0.40,
            FitLevel::TooTight => 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_levels_thresholds() {
        assert_eq!(FitLevel::evaluate(50_000, 128_000), FitLevel::Perfect);
        assert_eq!(FitLevel::evaluate(90_000, 128_000), FitLevel::Good);
        assert_eq!(FitLevel::evaluate(115_000, 128_000), FitLevel::Marginal);
        assert_eq!(FitLevel::evaluate(130_000, 128_000), FitLevel::TooTight);
    }

    #[test]
    fn too_tight_when_window_is_zero() {
        assert_eq!(FitLevel::evaluate(1, 0), FitLevel::TooTight);
    }

    #[test]
    fn fit_score_ordering() {
        assert!(FitLevel::Perfect.score() > FitLevel::Good.score());
        assert!(FitLevel::Good.score() > FitLevel::Marginal.score());
        assert_eq!(FitLevel::TooTight.score(), 0.0);
    }
}
