//! Trust scoring constants and update logic.

pub const TRUST_POSITIVE_DELTA: f64 = 0.05;
pub const TRUST_NEGATIVE_DELTA: f64 = 0.10;
pub const TRUST_MIN_SEARCH: f64 = 0.30;

/// Clamp a raw trust delta into [0.0, 1.0] range.
pub fn apply_delta(current: f64, positive: bool) -> f64 {
    let delta = if positive {
        TRUST_POSITIVE_DELTA
    } else {
        -TRUST_NEGATIVE_DELTA
    };
    (current + delta).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positive_delta_increases_trust() {
        let result = apply_delta(0.5, true);
        assert!((result - 0.55).abs() < 1e-10);
    }

    #[test]
    fn negative_delta_decreases_trust() {
        let result = apply_delta(0.5, false);
        assert!((result - 0.40).abs() < 1e-10);
    }

    #[test]
    fn trust_clamped_at_upper_bound() {
        assert!((apply_delta(0.99, true) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn trust_clamped_at_lower_bound() {
        assert!((apply_delta(0.02, false) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn negative_delta_is_larger_than_positive() {
        const { assert!(TRUST_NEGATIVE_DELTA > TRUST_POSITIVE_DELTA) }
    }
}
