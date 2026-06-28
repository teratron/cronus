//! Cronus TUI library — the interactive terminal frontend over the core.
//!
//! This crate is pure presentation: it renders core state and maps key/slash
//! input to core capability calls. It holds only view state, never domain state,
//! and links the core library directly (never the CLI frontend).
//!
//! Layer breakdown:
//! - [`terminal`] — raw-mode lifecycle + event polling (the RAII shell).
//! - [`app`] — the event-driven render loop and the view-model it drives.

pub mod app;
pub mod terminal;

pub use app::{
    App, CapabilitySource, CoreSnapshot, PlainRenderer, Renderer, SnapshotSource, TickResult,
    ViewModel, run, run_with,
};
pub use terminal::{CrosstermBackend, Key, TermEvent, TerminalBackend, Tui};
