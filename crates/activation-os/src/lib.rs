//! Per-OS activation-registration adapter (l2-service-activation §4.2-§4.4),
//! implementing `cronus_contract::ActivationRegistry`. Minted per
//! `l2-crate-topology` §4.4(a): OS registration opens registries, writes
//! plists/unit files, and shells out to service managers — infrastructure by
//! every definition, never domain logic.
//!
//! Testability discipline: raw OS primitives sit behind the [`SystemCalls`]
//! seam, so the adapter's LOGIC — weaker-wins `observe()` derivation, the
//! login-scoped registration sequence — is unit-tested against a scriptable
//! fake on any host, independent of which platform module backs it. This
//! crate covers only login-scoped registration (Track D01); system-scoped
//! elevation (D02) extends the same seam.
//!
//! **Verification scope, disclosed:** the Windows implementation
//! (`windows_calls`) is compiled and its adapter logic tested on the host
//! this was written on. The Linux (`linux_calls`) and macOS (`macos_calls`)
//! implementations are cross-compile type-checked (`cargo check --target
//! x86_64-unknown-linux-gnu` / `--target aarch64-apple-darwin`) but not run —
//! no Linux or macOS host was available. Real-world validation on those
//! platforms is deferred.

use cronus_contract::{
    ActivationCapabilities, ActivationMode, ActivationRegistry, ActivationState, ModeSupport,
};

#[cfg(target_os = "linux")]
mod linux_calls;
#[cfg(target_os = "macos")]
mod macos_calls;
#[cfg(windows)]
mod windows_calls;

#[cfg(target_os = "linux")]
pub use linux_calls::LinuxSystemCalls;
#[cfg(target_os = "macos")]
pub use macos_calls::MacosSystemCalls;
#[cfg(windows)]
pub use windows_calls::WindowsSystemCalls;

/// The raw OS primitives a login-scoped adapter needs, factored out so the
/// adapter's decision logic — what to read, in what order, how to reconcile
/// a raw entry against the platform's own veto — is testable without a real
/// OS call (BA-8's weaker-wins rule: where the OS distinguishes *registered*
/// from *effective*, the weaker one wins).
pub trait SystemCalls: Send + Sync {
    /// Whether the login-scoped facility's raw entry exists.
    fn login_entry_present(&self) -> Result<bool, String>;
    /// The platform's own veto over that raw entry, if the platform has one
    /// (Windows `StartupApproved`, the `.desktop` `Hidden`/
    /// `X-GNOME-Autostart-enabled` keys). `None` when there is no separate
    /// veto signal to check — never fabricated as "not vetoed".
    fn login_entry_vetoed(&self) -> Result<Option<bool>, String>;
    /// Write the login-scoped entry (BA-6: unelevated).
    fn write_login_entry(&self) -> Result<(), String>;
    /// Remove the login-scoped entry.
    fn remove_login_entry(&self) -> Result<(), String>;
}

/// `observe()`'s weaker-wins derivation (BA-8): an entry present but vetoed
/// is `RequiresApproval`, present and not (verifiably) vetoed is `Active`,
/// absent is `Inactive`. Pure function of the two facts — the same rule
/// every platform's real [`SystemCalls`] impl feeds into.
fn derive_login_state(present: bool, vetoed: Option<bool>) -> ActivationState {
    if !present {
        return ActivationState::Inactive;
    }
    match vetoed {
        Some(true) => ActivationState::RequiresApproval(ActivationMode::Login),
        _ => ActivationState::Active(ActivationMode::Login),
    }
}

/// The login-scoped activation adapter (T-18D01): wraps a [`SystemCalls`]
/// implementation as a full `ActivationRegistry`. `ActivationMode::System`
/// is `Unsupported` until Track D02 lands — this adapter never claims a
/// capability it cannot yet deliver.
pub struct LoginScopedAdapter<C: SystemCalls> {
    calls: C,
}

impl<C: SystemCalls> LoginScopedAdapter<C> {
    pub fn new(calls: C) -> Self {
        LoginScopedAdapter { calls }
    }
}

impl<C: SystemCalls> ActivationRegistry for LoginScopedAdapter<C> {
    fn capabilities(&self) -> ActivationCapabilities {
        ActivationCapabilities {
            login: ModeSupport::Supported,
            system: ModeSupport::Unsupported {
                reason: "system-scoped registration is not yet implemented (Track D02)".to_string(),
            },
        }
    }

    fn observe(&self) -> ActivationState {
        let present = match self.calls.login_entry_present() {
            Ok(p) => p,
            Err(reason) => return ActivationState::Unknown { reason },
        };
        let vetoed = match self.calls.login_entry_vetoed() {
            Ok(v) => v,
            Err(reason) => return ActivationState::Unknown { reason },
        };
        derive_login_state(present, vetoed)
    }

    fn enable(&self, mode: ActivationMode) -> Result<(), String> {
        match mode {
            ActivationMode::Login => self.calls.write_login_entry(),
            ActivationMode::System => {
                Err("system-scoped registration is not yet implemented (Track D02)".to_string())
            }
        }
    }

    fn disable(&self) -> Result<(), String> {
        self.calls.remove_login_entry()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// A scriptable `SystemCalls` for tests: presence/veto are set at
    /// construction and mutated through `write`/`remove`; `query_fails_with`
    /// forces `Err` from both reads to exercise the `Unknown` path — enough
    /// to drive the adapter's weaker-wins derivation with no real OS call.
    struct FakeSystemCalls {
        present: Mutex<bool>,
        vetoed: Mutex<Option<bool>>,
        query_fails_with: Mutex<Option<String>>,
    }

    impl FakeSystemCalls {
        fn new(present: bool, vetoed: Option<bool>) -> Self {
            FakeSystemCalls {
                present: Mutex::new(present),
                vetoed: Mutex::new(vetoed),
                query_fails_with: Mutex::new(None),
            }
        }
    }

    impl SystemCalls for FakeSystemCalls {
        fn login_entry_present(&self) -> Result<bool, String> {
            if let Some(reason) = self.query_fails_with.lock().unwrap().clone() {
                return Err(reason);
            }
            Ok(*self.present.lock().unwrap())
        }

        fn login_entry_vetoed(&self) -> Result<Option<bool>, String> {
            if let Some(reason) = self.query_fails_with.lock().unwrap().clone() {
                return Err(reason);
            }
            Ok(*self.vetoed.lock().unwrap())
        }

        fn write_login_entry(&self) -> Result<(), String> {
            *self.present.lock().unwrap() = true;
            *self.vetoed.lock().unwrap() = None;
            Ok(())
        }

        fn remove_login_entry(&self) -> Result<(), String> {
            *self.present.lock().unwrap() = false;
            Ok(())
        }
    }

    #[test]
    fn observe_derives_inactive_when_no_entry_present() {
        let adapter = LoginScopedAdapter::new(FakeSystemCalls::new(false, None));
        assert_eq!(adapter.observe(), ActivationState::Inactive);
    }

    #[test]
    fn observe_derives_active_when_present_and_not_vetoed() {
        let adapter = LoginScopedAdapter::new(FakeSystemCalls::new(true, Some(false)));
        assert_eq!(
            adapter.observe(),
            ActivationState::Active(ActivationMode::Login)
        );
    }

    #[test]
    fn observe_derives_requires_approval_when_present_and_vetoed() {
        // BA-8 weaker-wins: the veto beats the raw "present" entry.
        let adapter = LoginScopedAdapter::new(FakeSystemCalls::new(true, Some(true)));
        assert_eq!(
            adapter.observe(),
            ActivationState::RequiresApproval(ActivationMode::Login)
        );
    }

    #[test]
    fn observe_reports_unknown_when_the_facility_cannot_be_queried() {
        let fake = FakeSystemCalls::new(true, None);
        *fake.query_fails_with.lock().unwrap() = Some("registry unreadable".to_string());
        let adapter = LoginScopedAdapter::new(fake);
        assert!(matches!(adapter.observe(), ActivationState::Unknown { .. }));
    }

    #[test]
    fn enable_login_writes_the_entry_and_disable_removes_it() {
        let adapter = LoginScopedAdapter::new(FakeSystemCalls::new(false, None));
        assert_eq!(adapter.observe(), ActivationState::Inactive);

        adapter
            .enable(ActivationMode::Login)
            .expect("enable succeeds");
        assert_eq!(
            adapter.observe(),
            ActivationState::Active(ActivationMode::Login)
        );

        adapter.disable().expect("disable succeeds");
        assert_eq!(adapter.observe(), ActivationState::Inactive);
    }

    #[test]
    fn system_mode_is_unsupported_until_track_d02() {
        let adapter = LoginScopedAdapter::new(FakeSystemCalls::new(false, None));
        let caps = adapter.capabilities();
        assert_eq!(caps.login, ModeSupport::Supported);
        assert!(matches!(caps.system, ModeSupport::Unsupported { .. }));
        assert!(adapter.enable(ActivationMode::System).is_err());
    }
}
