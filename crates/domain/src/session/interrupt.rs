//! Interrupt fence — a thread-safe boolean flag for session cancellation.

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

/// Shared cancellation signal.
///
/// Cheaply cloneable; the clone shares the same underlying flag.
/// Set to `true` to request interruption; the session loop polls `is_set()`
/// at iteration boundaries.
#[derive(Debug, Clone)]
pub struct InterruptFence(Arc<AtomicBool>);

impl InterruptFence {
    pub fn new() -> Self {
        InterruptFence(Arc::new(AtomicBool::new(false)))
    }

    pub fn set(&self) {
        self.0.store(true, Ordering::Release);
    }

    pub fn is_set(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }

    pub fn reset(&self) {
        self.0.store(false, Ordering::Release);
    }
}

impl Default for InterruptFence {
    fn default() -> Self {
        Self::new()
    }
}
