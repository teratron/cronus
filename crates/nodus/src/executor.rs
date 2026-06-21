//! Executor — step dispatch, control flow, and bounded execution.
//!
//! Forthcoming: walks the [`crate::ast`] following the boot sequence (load
//! schema → read hard rules → read preferences → register inputs/context →
//! match trigger → run steps), dispatches each command through a subsystem
//! seam, enforces hard constraints and iteration limits, and returns the
//! structured result. Subsystem handlers (memory / HITL / orchestration /
//! quality / model router) bind through traits; concrete wiring lands with
//! those subsystems.
