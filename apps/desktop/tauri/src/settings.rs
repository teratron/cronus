//! Settings persistence for the desktop shell.
//!
//! Settings live as one JSON file in the platform config directory. Loading
//! follows the merge-don't-clobber pattern: absent fields are filled from
//! defaults, an additive migration inserts newly shipped entries without
//! touching user choices, and unknown keys round-trip untouched so a newer
//! file survives an older binary. Writes go through a temp-file rename so an
//! interrupted write never leaves a torn settings file.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Mutex, MutexGuard};

use serde::{Deserialize, Deserializer, Serialize};

/// Hot copy of the log level, readable on every log call without a lock.
/// Staleness is acceptable (Relaxed) — it avoids contention on the hot path.
static LOG_LEVEL_HOT: AtomicU8 = AtomicU8::new(LogLevel::Info as u8);

/// Log verbosity. Stored as a lowercase string; legacy files stored 0–5.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
    Off = 5,
}

impl LogLevel {
    fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Trace),
            1 => Some(Self::Debug),
            2 => Some(Self::Info),
            3 => Some(Self::Warn),
            4 => Some(Self::Error),
            5 => Some(Self::Off),
            _ => None,
        }
    }

    fn from_name(name: &str) -> Option<Self> {
        match name {
            "trace" => Some(Self::Trace),
            "debug" => Some(Self::Debug),
            "info" => Some(Self::Info),
            "warn" => Some(Self::Warn),
            "error" => Some(Self::Error),
            "off" => Some(Self::Off),
            _ => None,
        }
    }
}

/// Dual deserializer: current string form first, legacy integer fallback.
fn deserialize_log_level<'de, D>(deserializer: D) -> Result<LogLevel, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Wire {
        Name(String),
        Legacy(u8),
    }

    match Wire::deserialize(deserializer)? {
        Wire::Name(name) => LogLevel::from_name(&name)
            .ok_or_else(|| serde::de::Error::custom(format!("unknown log level `{name}`"))),
        Wire::Legacy(value) => LogLevel::from_u8(value)
            .ok_or_else(|| serde::de::Error::custom(format!("log level {value} out of range"))),
    }
}

/// Where the quick-access overlay window docks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OverlayPosition {
    None,
    Top,
    Bottom,
}

fn default_log_level() -> LogLevel {
    LogLevel::Info
}

fn default_overlay_position() -> OverlayPosition {
    // Linux compositors steal focus on docked overlays; default it off there.
    #[cfg(target_os = "linux")]
    {
        OverlayPosition::None
    }
    #[cfg(not(target_os = "linux"))]
    {
        OverlayPosition::Bottom
    }
}

fn default_theme() -> String {
    // OS-appearance axis: system follows the OS, light / dark force a variant.
    "system".to_string()
}

fn default_color_scheme() -> String {
    // Visual-language axis: a named design-identity token package. `default` is
    // the built-in scheme every install ships and every fallback lands on.
    "default".to_string()
}

fn default_shortcuts() -> BTreeMap<String, String> {
    BTreeMap::from([
        (
            "toggle-overlay".to_string(),
            "CmdOrCtrl+Shift+K".to_string(),
        ),
        (
            "show-main-window".to_string(),
            "CmdOrCtrl+Shift+C".to_string(),
        ),
    ])
}

/// Application settings. Every field carries a `serde(default)` so adding a
/// field never breaks an existing file; unknown keys are retained via the
/// flattened `extra` map (forward compatibility — never dropped on rewrite).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    #[serde(
        default = "default_log_level",
        deserialize_with = "deserialize_log_level"
    )]
    pub log_level: LogLevel,

    #[serde(default = "default_overlay_position")]
    pub overlay_position: OverlayPosition,

    /// Theming axis 1 — OS-appearance mode: `system` | `light` | `dark`.
    /// Cosmetic-only; the frontend resolver maps it against the OS preference.
    #[serde(default = "default_theme")]
    pub theme: String,

    /// Theming axis 2 — active colour scheme (a named design identity). The
    /// frontend resolves `(theme × color_scheme)` into the surface token set;
    /// an unknown id falls back to `default` there.
    #[serde(default = "default_color_scheme")]
    pub color_scheme: String,

    /// Named shortcut bindings; additive migration inserts newly shipped
    /// names and never touches an existing (possibly user-edited) binding.
    #[serde(default = "default_shortcuts")]
    pub shortcuts: BTreeMap<String, String>,

    /// Persisted workbench layout. Opaque here: the frontend owns the
    /// `LayoutRecord` schema and its field-wise restore; the host only stores
    /// and returns the blob. Absent on an older file — restored to defaults
    /// there, never a failed startup.
    #[serde(default)]
    pub layout: serde_json::Value,

    /// User keymap overrides: action id -> chord string (an empty string
    /// disables the binding). The frontend merges this as the top layer.
    #[serde(default)]
    pub keymap_user: BTreeMap<String, String>,

    /// Unknown fields from newer versions, preserved verbatim.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            log_level: default_log_level(),
            overlay_position: default_overlay_position(),
            theme: default_theme(),
            color_scheme: default_color_scheme(),
            shortcuts: default_shortcuts(),
            layout: serde_json::Value::Null,
            keymap_user: BTreeMap::new(),
            extra: serde_json::Map::new(),
        }
    }
}

/// The shell-facing slice of settings, marshalled over IPC. Host-window
/// concerns (log level, overlay dock, OS shortcuts) are not exposed here — the
/// bridge is presentation-only.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShellSettings {
    pub theme: String,
    pub color_scheme: String,
    pub layout: serde_json::Value,
    pub keymap_user: BTreeMap<String, String>,
}

/// A partial update from the shell. Only `Some` fields are written, so the
/// frontend can persist one axis (a layout change, a rebind) without echoing
/// the rest.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShellSettingsPatch {
    pub theme: Option<String>,
    pub color_scheme: Option<String>,
    pub layout: Option<serde_json::Value>,
    pub keymap_user: Option<BTreeMap<String, String>>,
}

/// Live settings plus the file they persist to. Managed as Tauri state so the
/// IPC bridge can read and write the shell-facing slice under a lock.
pub struct SettingsStore {
    path: PathBuf,
    current: Mutex<Settings>,
}

impl SettingsStore {
    /// Wrap already-loaded settings and the path future writes go to.
    pub fn new(path: PathBuf, initial: Settings) -> Self {
        Self {
            path,
            current: Mutex::new(initial),
        }
    }

    fn lock(&self) -> MutexGuard<'_, Settings> {
        self.current
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Snapshot the shell-facing slice.
    pub fn shell_settings(&self) -> ShellSettings {
        let settings = self.lock();
        ShellSettings {
            theme: settings.theme.clone(),
            color_scheme: settings.color_scheme.clone(),
            layout: settings.layout.clone(),
            keymap_user: settings.keymap_user.clone(),
        }
    }

    /// Apply a partial update and persist it atomically.
    pub fn update_shell(&self, patch: ShellSettingsPatch) -> io::Result<()> {
        let mut settings = self.lock();
        if let Some(theme) = patch.theme {
            settings.theme = theme;
        }
        if let Some(color_scheme) = patch.color_scheme {
            settings.color_scheme = color_scheme;
        }
        if let Some(layout) = patch.layout {
            settings.layout = layout;
        }
        if let Some(keymap_user) = patch.keymap_user {
            settings.keymap_user = keymap_user;
        }
        save(&self.path, &settings)
    }
}

impl Settings {
    /// Additive migration: insert newly shipped entries, never remove or
    /// rename existing keys. Returns whether anything was added.
    fn ensure_defaults(&mut self) -> bool {
        let mut changed = false;
        for (name, binding) in default_shortcuts() {
            self.shortcuts.entry(name).or_insert_with(|| {
                changed = true;
                binding
            });
        }
        changed
    }

    /// Publish hot settings to their lock-free copies.
    fn publish_hot(&self) {
        LOG_LEVEL_HOT.store(self.log_level as u8, Ordering::Relaxed);
    }
}

/// Read the hot log level without touching the settings store.
pub fn hot_log_level() -> LogLevel {
    LogLevel::from_u8(LOG_LEVEL_HOT.load(Ordering::Relaxed)).unwrap_or(LogLevel::Info)
}

/// Load settings from `path`, creating the file with defaults on first
/// launch, merging in new defaults, and running the additive migration.
pub fn load_or_create(path: &Path) -> io::Result<Settings> {
    let mut settings = if path.exists() {
        let raw = fs::read_to_string(path)?;
        serde_json::from_str::<Settings>(&raw).map_err(io::Error::other)?
    } else {
        Settings::default()
    };

    let migrated = settings.ensure_defaults();
    if migrated || !path.exists() {
        save(path, &settings)?;
    }

    settings.publish_hot();
    Ok(settings)
}

/// Persist settings atomically: write a sibling temp file, then rename over
/// the target so an interrupted write leaves the previous file intact.
pub fn save(path: &Path, settings: &Settings) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(settings).map_err(io::Error::other)?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, json)?;
    fs::rename(&tmp, path)?;
    settings.publish_hot();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::{Mutex, MutexGuard};

    /// Serializes tests that publish to the shared LOG_LEVEL_HOT atomic —
    /// parallel saves would otherwise race the hot-level assertions.
    static HOT_PUBLISH: Mutex<()> = Mutex::new(());

    fn hot_lock() -> MutexGuard<'static, ()> {
        HOT_PUBLISH
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Unique temp path per test; the directory is created by `save`.
    fn temp_settings_path(tag: &str) -> PathBuf {
        std::env::temp_dir()
            .join(format!(
                "cronus-desktop-settings-{tag}-{}",
                std::process::id()
            ))
            .join("settings.json")
    }

    fn cleanup(path: &Path) {
        if let Some(dir) = path.parent() {
            let _ = fs::remove_dir_all(dir);
        }
    }

    #[test]
    fn missing_file_yields_defaults_and_creates_the_file() {
        let _hot = hot_lock();
        let path = temp_settings_path("create");
        cleanup(&path);

        let settings = load_or_create(&path).expect("load_or_create");
        assert_eq!(settings, Settings::default());
        assert!(path.exists(), "first launch writes the defaults file");
        cleanup(&path);
    }

    #[test]
    fn saved_settings_round_trip() {
        let _hot = hot_lock();
        let path = temp_settings_path("roundtrip");
        cleanup(&path);

        let mut settings = Settings {
            log_level: LogLevel::Debug,
            ..Settings::default()
        };
        settings
            .shortcuts
            .insert("toggle-overlay".into(), "Alt+Space".into());
        save(&path, &settings).expect("save");

        let loaded = load_or_create(&path).expect("reload");
        assert_eq!(loaded, settings);
        cleanup(&path);
    }

    #[test]
    fn older_file_without_theming_fields_deserializes_with_defaults_filled() {
        let _hot = hot_lock();
        let path = temp_settings_path("theming-defaults");
        cleanup(&path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("mkdir");
        }
        // A file predating the two theming axes — neither key present.
        fs::write(&path, r#"{ "log_level": "info" }"#).expect("write older file");

        let settings = load_or_create(&path).expect("load older file");
        assert_eq!(settings.theme, "system");
        assert_eq!(settings.color_scheme, "default");
        cleanup(&path);
    }

    #[test]
    fn theming_axes_round_trip() {
        let _hot = hot_lock();
        let path = temp_settings_path("theming-roundtrip");
        cleanup(&path);

        let settings = Settings {
            theme: "dark".to_string(),
            color_scheme: "midnight".to_string(),
            ..Settings::default()
        };
        save(&path, &settings).expect("save");
        let loaded = load_or_create(&path).expect("reload");
        assert_eq!(loaded.theme, "dark");
        assert_eq!(loaded.color_scheme, "midnight");
        cleanup(&path);
    }

    #[test]
    fn dual_deserializer_reads_the_legacy_integer_shape() {
        let _hot = hot_lock();
        let path = temp_settings_path("legacy");
        cleanup(&path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("mkdir");
        }
        fs::write(&path, r#"{ "log_level": 3 }"#).expect("write legacy file");

        let settings = load_or_create(&path).expect("load legacy");
        assert_eq!(settings.log_level, LogLevel::Warn);
        cleanup(&path);
    }

    #[test]
    fn additive_migration_inserts_new_defaults_and_keeps_user_values() {
        let _hot = hot_lock();
        let path = temp_settings_path("migrate");
        cleanup(&path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("mkdir");
        }
        // A legacy file: one customized binding, one shipped binding missing,
        // plus a key this version does not know about.
        fs::write(
            &path,
            r#"{
                "log_level": "error",
                "shortcuts": { "toggle-overlay": "Alt+Space" },
                "from_the_future": { "keep": true }
            }"#,
        )
        .expect("write legacy file");

        let settings = load_or_create(&path).expect("load");
        assert_eq!(
            settings.shortcuts.get("toggle-overlay").map(String::as_str),
            Some("Alt+Space"),
            "user-edited binding untouched"
        );
        assert!(
            settings.shortcuts.contains_key("show-main-window"),
            "newly shipped binding inserted"
        );
        assert!(
            settings.extra.contains_key("from_the_future"),
            "unknown field preserved in memory"
        );

        // The migrated file was written back; unknown keys survive on disk.
        let raw = fs::read_to_string(&path).expect("read back");
        assert!(raw.contains("from_the_future"), "unknown field persisted");
        assert!(raw.contains("show-main-window"));
        cleanup(&path);
    }

    #[test]
    fn platform_default_overlay_position_resolves_per_os() {
        let expected = if cfg!(target_os = "linux") {
            OverlayPosition::None
        } else {
            OverlayPosition::Bottom
        };
        assert_eq!(default_overlay_position(), expected);
    }

    #[test]
    fn hot_log_level_tracks_the_loaded_settings() {
        let _hot = hot_lock();
        let path = temp_settings_path("hot");
        cleanup(&path);

        let mut settings = Settings {
            log_level: LogLevel::Trace,
            ..Settings::default()
        };
        save(&path, &settings).expect("save");
        assert_eq!(hot_log_level(), LogLevel::Trace);

        settings.log_level = LogLevel::Error;
        save(&path, &settings).expect("save again");
        assert_eq!(hot_log_level(), LogLevel::Error);
        cleanup(&path);
    }

    #[test]
    fn older_file_without_layout_or_keymap_user_deserializes_with_defaults() {
        let _hot = hot_lock();
        let path = temp_settings_path("layout-defaults");
        cleanup(&path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("mkdir");
        }
        fs::write(&path, r#"{ "log_level": "info", "theme": "dark" }"#).expect("write older file");

        let settings = load_or_create(&path).expect("load older file");
        assert_eq!(
            settings.layout,
            serde_json::Value::Null,
            "absent layout -> Null"
        );
        assert!(
            settings.keymap_user.is_empty(),
            "absent keymap_user -> empty"
        );
        cleanup(&path);
    }

    #[test]
    fn settings_store_get_returns_the_persisted_shell_slice() {
        let _hot = hot_lock();
        let path = temp_settings_path("store-get");
        cleanup(&path);

        let settings = Settings {
            theme: "light".into(),
            color_scheme: "midnight".into(),
            layout: serde_json::json!({ "version": 1, "activeSubsystem": "kanban" }),
            keymap_user: BTreeMap::from([("view.command-palette".into(), "Ctrl+P".into())]),
            ..Settings::default()
        };
        save(&path, &settings).expect("save");

        let store = SettingsStore::new(path.clone(), load_or_create(&path).expect("reload"));
        let shell = store.shell_settings();
        assert_eq!(shell.theme, "light");
        assert_eq!(shell.color_scheme, "midnight");
        assert_eq!(shell.layout["activeSubsystem"], "kanban");
        assert_eq!(
            shell
                .keymap_user
                .get("view.command-palette")
                .map(String::as_str),
            Some("Ctrl+P")
        );
        cleanup(&path);
    }

    #[test]
    fn settings_store_set_round_trips_a_partial_update() {
        let _hot = hot_lock();
        let path = temp_settings_path("store-set");
        cleanup(&path);

        let store = SettingsStore::new(path.clone(), {
            let base = Settings::default();
            save(&path, &base).expect("seed");
            base
        });

        // A layout-only patch must not disturb the theming axes.
        store
            .update_shell(ShellSettingsPatch {
                layout: Some(serde_json::json!({ "version": 1, "sidebarVisible": false })),
                ..ShellSettingsPatch::default()
            })
            .expect("update");

        let reloaded = load_or_create(&path).expect("reload after set");
        assert_eq!(reloaded.layout["sidebarVisible"], false);
        assert_eq!(
            reloaded.theme, "system",
            "theming axis untouched by a layout patch"
        );

        // A second store over the same file sees the persisted layout.
        let store2 = SettingsStore::new(path.clone(), reloaded);
        assert_eq!(store2.shell_settings().layout["sidebarVisible"], false);
        cleanup(&path);
    }
}
