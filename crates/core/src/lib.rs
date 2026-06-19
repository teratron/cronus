//! Cronus core engine library.
//!
//! All domain logic lives here; frontends (CLI, TUI, app) are thin and call the
//! capability contract below. The core has no presentation dependencies.

pub mod paths;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_exposes_version_and_status() {
        let engine = Engine::new();
        assert!(!engine.version().is_empty());
        assert!(engine.status().contains("Cronus core"));
    }
}
