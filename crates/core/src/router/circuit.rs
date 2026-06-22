//! Circuit breaker for model providers.
//!
//! States:  Closed → (failures ≥ threshold) → Open
//!          Open   → (cooldown elapsed)      → HalfOpen
//!          HalfOpen → (success)             → Closed
//!          HalfOpen → (failure)             → Open

use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Degraded,
    Open,
    HalfOpen,
}

#[derive(Debug)]
pub struct CircuitBreaker {
    state: CircuitState,
    failure_count: u32,
    failure_threshold: u32,
    cooldown: Duration,
    opened_at: Option<Instant>,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, cooldown_secs: u64) -> Self {
        CircuitBreaker {
            state: CircuitState::Closed,
            failure_count: 0,
            failure_threshold,
            cooldown: Duration::from_secs(cooldown_secs),
            opened_at: None,
        }
    }

    pub fn state(&self) -> CircuitState {
        self.state
    }

    /// Returns true when the circuit is open and the provider must be skipped.
    pub fn is_open(&mut self) -> bool {
        if self.state == CircuitState::Open {
            if let Some(opened) = self.opened_at
                && opened.elapsed() >= self.cooldown
            {
                self.state = CircuitState::HalfOpen;
                return false;
            }
            return true;
        }
        false
    }

    pub fn record_success(&mut self) {
        self.failure_count = 0;
        self.opened_at = None;
        self.state = CircuitState::Closed;
    }

    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        if self.failure_count >= self.failure_threshold {
            self.state = CircuitState::Open;
            self.opened_at = Some(Instant::now());
        } else if self.failure_count > 0 {
            self.state = CircuitState::Degraded;
        }
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new(5, 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_closed() {
        let cb = CircuitBreaker::default();
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn opens_after_threshold_failures() {
        let mut cb = CircuitBreaker::new(3, 60);
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Degraded);
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
    }

    #[test]
    fn success_resets_to_closed() {
        let mut cb = CircuitBreaker::new(2, 60);
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(!cb.is_open(), "closed circuit must not be open");
    }

    #[test]
    fn transitions_to_half_open_after_cooldown() {
        let mut cb = CircuitBreaker::new(1, 0); // zero cooldown
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        // With zero cooldown, is_open() should transition to HalfOpen
        let open = cb.is_open();
        assert!(!open, "zero-cooldown circuit must transition to HalfOpen immediately");
        assert_eq!(cb.state(), CircuitState::HalfOpen);
    }
}
