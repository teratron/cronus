//! Stop hook seam — fires after each turn and may override the result.

/// Outcome returned by a stop hook.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HookOutcome {
    /// Continue normally with this output.
    Continue(String),
    /// Replace the output with this value.
    Replace(String),
    /// Halt the session loop.
    Halt,
}

/// A stop hook that fires after an assistant turn.
pub trait StopHook: Send + Sync {
    fn on_turn_end(&self, output: &str) -> HookOutcome;
}

/// No-op hook — used when no stop hook is configured.
pub struct NoOpHook;

impl StopHook for NoOpHook {
    fn on_turn_end(&self, output: &str) -> HookOutcome {
        HookOutcome::Continue(output.to_owned())
    }
}
