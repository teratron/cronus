//! Global shortcut binding system.
//!
//! Named bindings separate the factory default (in code) from the user's
//! override (in settings); materialized bindings feed a pluggable backend.
//! Switching backends re-validates every binding for the target backend and
//! auto-resets invalid ones to their defaults, reporting what was reset. A
//! failed registration likewise rolls the binding back to its default.

use std::collections::BTreeMap;

/// A named shortcut: stable id, labels, factory default, effective binding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortcutBinding {
    pub id: String,
    pub name: String,
    pub description: String,
    pub default_binding: String,
    pub current_binding: String,
}

/// Factory catalog of built-in bindings (the reset targets).
pub fn factory_bindings() -> Vec<ShortcutBinding> {
    let entry = |id: &str, name: &str, description: &str, default: &str| ShortcutBinding {
        id: id.into(),
        name: name.into(),
        description: description.into(),
        default_binding: default.into(),
        current_binding: default.into(),
    };
    vec![
        entry(
            "toggle-overlay",
            "Toggle overlay",
            "Show or hide the quick-access overlay",
            "CmdOrCtrl+Shift+K",
        ),
        entry(
            "show-main-window",
            "Show main window",
            "Bring the Cronus window to the front",
            "CmdOrCtrl+Shift+C",
        ),
    ]
}

/// Materialize bindings: factory defaults overlaid with user overrides from
/// settings. Unknown override keys are ignored; missing ones fall back to the
/// default (the additive-migration counterpart lives in settings).
pub fn materialize(overrides: &BTreeMap<String, String>) -> Vec<ShortcutBinding> {
    factory_bindings()
        .into_iter()
        .map(|mut binding| {
            if let Some(user) = overrides.get(&binding.id) {
                binding.current_binding = user.clone();
            }
            binding
        })
        .collect()
}

/// A shortcut backend: validates key strings and (un)registers bindings.
/// Production backends adapt the platform plugin / extended hook library.
pub trait ShortcutBackend {
    /// Backend identifier persisted to settings when a fallback is chosen.
    fn id(&self) -> &'static str;
    /// Whether this backend accepts the key-string format.
    fn validate(&self, key: &str) -> bool;
    /// Register one binding; `Err` means a conflict or platform refusal.
    fn register(&mut self, id: &str, key: &str) -> Result<(), String>;
    /// Unregister one binding (idempotent).
    fn unregister(&mut self, id: &str);
}

/// Outcome of registering a full binding set or switching backends.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct BindingReport {
    /// Bindings that were reset to their default (invalid or conflicting),
    /// reported to the frontend.
    pub reset_to_default: Vec<String>,
    /// Bindings that failed even at their default and stay unregistered.
    pub failed: Vec<String>,
}

/// Register every binding with auto-rollback: an invalid or conflicting
/// current binding is reset to its default and retried; a binding that fails
/// even at its default is reported and skipped.
pub fn register_all(
    backend: &mut dyn ShortcutBackend,
    bindings: &mut [ShortcutBinding],
) -> BindingReport {
    let mut report = BindingReport::default();
    for binding in bindings.iter_mut() {
        let valid = backend.validate(&binding.current_binding);
        let registered = valid
            && backend
                .register(&binding.id, &binding.current_binding)
                .is_ok();
        if registered {
            continue;
        }
        // Roll back to the factory default and retry once.
        if binding.current_binding != binding.default_binding {
            binding.current_binding = binding.default_binding.clone();
            report.reset_to_default.push(binding.id.clone());
            if backend.validate(&binding.current_binding)
                && backend
                    .register(&binding.id, &binding.current_binding)
                    .is_ok()
            {
                continue;
            }
        }
        report.failed.push(binding.id.clone());
    }
    report
}

/// Switch backends: unregister everything from the old one, re-validate for
/// the new one (formats differ per backend), reset invalid bindings to their
/// defaults, and register the set with the new backend.
pub fn switch_backend(
    old: &mut dyn ShortcutBackend,
    new: &mut dyn ShortcutBackend,
    bindings: &mut [ShortcutBinding],
) -> BindingReport {
    for binding in bindings.iter() {
        old.unregister(&binding.id);
    }
    register_all(new, bindings)
}

/// Runtime lifecycle over a registered set: suspend a binding while the user
/// edits it, resume it afterwards, and keep the dynamic `cancel` shortcut
/// registered only while an operation is active.
pub struct ShortcutManager<B: ShortcutBackend> {
    backend: B,
    cancel_binding: String,
    cancel_active: bool,
}

impl<B: ShortcutBackend> ShortcutManager<B> {
    pub fn new(backend: B, cancel_binding: impl Into<String>) -> Self {
        Self {
            backend,
            cancel_binding: cancel_binding.into(),
            cancel_active: false,
        }
    }

    /// Unregister while the user edits — prevents firing during capture.
    pub fn suspend_binding(&mut self, id: &str) {
        self.backend.unregister(id);
    }

    /// Re-register after editing completes.
    pub fn resume_binding(&mut self, id: &str, key: &str) -> Result<(), String> {
        self.backend.register(id, key)
    }

    /// The cancel shortcut exists only while an operation runs.
    pub fn set_operation_active(&mut self, active: bool) {
        if active && !self.cancel_active {
            self.cancel_active = self
                .backend
                .register("cancel", &self.cancel_binding.clone())
                .is_ok();
        } else if !active && self.cancel_active {
            self.backend.unregister("cancel");
            self.cancel_active = false;
        }
    }

    pub fn backend(&self) -> &B {
        &self.backend
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// Mock backend with a configurable set of key formats it accepts and
    /// key strings that conflict at registration time.
    struct MockBackend {
        id: &'static str,
        rejects_format: fn(&str) -> bool,
        conflicts: BTreeSet<String>,
        registered: BTreeMap<String, String>,
    }

    impl MockBackend {
        fn new(id: &'static str) -> Self {
            Self {
                id,
                rejects_format: |_| false,
                conflicts: BTreeSet::new(),
                registered: BTreeMap::new(),
            }
        }
    }

    impl ShortcutBackend for MockBackend {
        fn id(&self) -> &'static str {
            self.id
        }
        fn validate(&self, key: &str) -> bool {
            !(self.rejects_format)(key)
        }
        fn register(&mut self, id: &str, key: &str) -> Result<(), String> {
            if self.conflicts.contains(key) {
                return Err(format!("`{key}` already taken"));
            }
            self.registered.insert(id.into(), key.into());
            Ok(())
        }
        fn unregister(&mut self, id: &str) {
            self.registered.remove(id);
        }
    }

    #[test]
    fn materialize_overlays_user_overrides_on_factory_defaults() {
        let overrides = BTreeMap::from([("toggle-overlay".to_string(), "Alt+Space".to_string())]);
        let bindings = materialize(&overrides);
        let toggle = bindings
            .iter()
            .find(|b| b.id == "toggle-overlay")
            .expect("exists");
        assert_eq!(toggle.current_binding, "Alt+Space");
        assert_eq!(toggle.default_binding, "CmdOrCtrl+Shift+K");
        let show = bindings
            .iter()
            .find(|b| b.id == "show-main-window")
            .expect("exists");
        assert_eq!(show.current_binding, show.default_binding);
    }

    #[test]
    fn conflicting_binding_rolls_back_to_its_default_and_is_reported() {
        let mut backend = MockBackend::new("plugin");
        backend.conflicts.insert("Alt+Space".into());
        let overrides = BTreeMap::from([("toggle-overlay".to_string(), "Alt+Space".to_string())]);
        let mut bindings = materialize(&overrides);

        let report = register_all(&mut backend, &mut bindings);

        assert_eq!(report.reset_to_default, vec!["toggle-overlay".to_string()]);
        assert!(report.failed.is_empty());
        let toggle = bindings
            .iter()
            .find(|b| b.id == "toggle-overlay")
            .expect("exists");
        assert_eq!(toggle.current_binding, "CmdOrCtrl+Shift+K", "reset applied");
        assert_eq!(
            backend.registered.get("toggle-overlay").map(String::as_str),
            Some("CmdOrCtrl+Shift+K")
        );
    }

    #[test]
    fn backend_switch_revalidates_formats_and_resets_invalid_bindings() {
        let mut old = MockBackend::new("plugin");
        let overrides = BTreeMap::from([("toggle-overlay".to_string(), "Hyper+K".to_string())]);
        let mut bindings = materialize(&overrides);
        register_all(&mut old, &mut bindings);

        // The new backend rejects the `Hyper+` format entirely.
        let mut new = MockBackend::new("hook-lib");
        new.rejects_format = |key| key.starts_with("Hyper+");

        let report = switch_backend(&mut old, &mut new, &mut bindings);

        assert!(old.registered.is_empty(), "old backend fully unregistered");
        assert_eq!(report.reset_to_default, vec!["toggle-overlay".to_string()]);
        assert_eq!(
            new.registered.get("toggle-overlay").map(String::as_str),
            Some("CmdOrCtrl+Shift+K"),
            "re-registered at the default on the new backend"
        );
    }

    #[test]
    fn suspend_resume_and_dynamic_cancel_lifecycle() {
        let mut manager = ShortcutManager::new(MockBackend::new("plugin"), "Escape");

        assert!(!manager.backend().registered.contains_key("cancel"));
        manager.set_operation_active(true);
        assert!(manager.backend().registered.contains_key("cancel"));
        manager.set_operation_active(false);
        assert!(!manager.backend().registered.contains_key("cancel"));

        manager
            .resume_binding("toggle-overlay", "CmdOrCtrl+Shift+K")
            .expect("register");
        assert!(manager.backend().registered.contains_key("toggle-overlay"));
        manager.suspend_binding("toggle-overlay");
        assert!(!manager.backend().registered.contains_key("toggle-overlay"));
    }
}
