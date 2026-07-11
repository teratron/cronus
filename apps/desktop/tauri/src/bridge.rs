//! Shell ↔ core IPC bridge.
//!
//! The typed command surface the UI invokes over Tauri IPC. Each command maps
//! one-to-one onto the core capability contract (`cronus_core::Capabilities`) — the
//! same surface the CLI and TUI bind — and returns already-masked output via
//! the core redaction path. The shell only marshals: no domain logic, no
//! re-implemented redaction.

use cronus_core::{Capabilities, Engine};

/// Bridge over a core handle plus the secret values to mask in any output
/// that crosses the IPC boundary.
pub struct Bridge<C: Capabilities> {
    core: C,
    secrets: Vec<String>,
}

impl<C: Capabilities> Bridge<C> {
    /// Wrap a core handle and the secret values to mask in bridged output.
    pub fn new(core: C, secrets: Vec<String>) -> Self {
        Self { core, secrets }
    }

    fn mask(&self, raw: &str) -> String {
        let secret_refs: Vec<&str> = self.secrets.iter().map(String::as_str).collect();
        cronus_core::redact::redact(raw, &secret_refs)
    }

    /// Core/product version, masked like every bridged value.
    pub fn version(&self) -> String {
        self.mask(self.core.version())
    }

    /// Core status line, masked before it crosses to the WebView.
    pub fn status(&self) -> String {
        self.mask(&self.core.status())
    }
}

/// The production bridge the shell manages as Tauri state.
pub type CoreBridge = Bridge<Engine>;

/// Construct the production bridge over an embedded engine.
///
/// The secret list starts empty; population from the core secret store is
/// deferred until the core exposes it (same pending note as the TUI).
pub fn core_bridge() -> CoreBridge {
    Bridge::new(Engine::new(), Vec::new())
}

/// IPC: `capability_version` — the core version string.
#[tauri::command]
pub fn capability_version(bridge: tauri::State<'_, CoreBridge>) -> String {
    bridge.version()
}

/// IPC: `capability_status` — the core status line.
#[tauri::command]
pub fn capability_status(bridge: tauri::State<'_, CoreBridge>) -> String {
    bridge.status()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Capability stub whose status embeds a known secret value.
    struct StubCore {
        status: String,
    }

    impl Capabilities for StubCore {
        fn version(&self) -> &str {
            "9.9.9-test"
        }

        fn status(&self) -> String {
            self.status.clone()
        }
    }

    #[test]
    fn version_passes_through_the_core_value() {
        let bridge = Bridge::new(
            StubCore {
                status: String::new(),
            },
            Vec::new(),
        );
        assert_eq!(bridge.version(), "9.9.9-test");
    }

    #[test]
    fn status_masks_known_secrets_via_core_redaction() {
        let bridge = Bridge::new(
            StubCore {
                status: "token=sk-LIVE-777 ready".into(),
            },
            vec!["sk-LIVE-777".into()],
        );
        let out = bridge.status();
        assert!(!out.contains("sk-LIVE-777"), "secret must not cross IPC");
        assert!(out.contains(cronus_core::redact::MASK));
        assert!(out.contains("ready"), "non-secret content preserved");
    }

    #[test]
    fn production_bridge_reports_the_embedded_core_status() {
        let bridge = core_bridge();
        assert!(bridge.status().contains(bridge.version().as_str()));
    }

    // Command *registration* is verified at compile time: `run()` passes both
    // commands through `tauri::generate_handler!`, so a renamed or mis-typed
    // command fails the build. A mock-runtime IPC round-trip (tauri "test"
    // feature) is not used here: on windows-gnu it makes the test binary fail
    // to load (STATUS_ENTRYPOINT_NOT_FOUND via WebView2Loader imports).
}
