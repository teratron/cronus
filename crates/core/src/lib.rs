//! Cronus core engine library.
//!
//! All domain logic lives here; frontends (CLI, TUI, app) are thin and call the
//! capability contract below. The core has no presentation dependencies.

pub mod agent_registry;
pub mod autonomy;
pub mod budget;
pub mod checkpoint;
pub mod constitution;
pub mod context_mgmt;
pub mod inbox;
pub mod context_router;
pub mod egress;
pub mod exec_workspace;
pub mod extensions;
pub mod hooks;
pub mod kanban;
pub mod learning;
pub mod memory;
pub mod quality;
pub mod roles;
pub mod session;
pub mod paths;
pub mod redact;
pub mod router;
pub mod secrets;
pub mod state;
pub mod store;
pub mod tool_security;
pub mod workspace;

/// The public capability contract that frontends invoke.
///
/// Frontends hold no domain logic — they map input to these calls and render
/// the results.
pub trait Capabilities {
    /// Engine/product version string.
    fn version(&self) -> &str;

    /// A human-readable status line (placeholder until subsystems land).
    fn status(&self) -> String;
}

/// The Cronus engine. Embeddable: links into a host without pulling any frontend.
#[derive(Debug, Default)]
pub struct Engine {
    _private: (),
}

impl Engine {
    /// Construct an engine instance.
    pub fn new() -> Self {
        Engine { _private: () }
    }
}

impl Capabilities for Engine {
    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn status(&self) -> String {
        format!("Cronus core {} — no subsystems loaded yet", self.version())
    }
}
