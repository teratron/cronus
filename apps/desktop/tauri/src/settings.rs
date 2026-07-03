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
use std::path::Path;
use std::sync::atomic::{AtomicU8, Ordering};

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

    /// Named shortcut bindings; additive migration inserts newly shipped
    /// names and never touches an existing (possibly user-edited) binding.
    #[serde(default = "default_shortcuts")]
    pub shortcuts: BTreeMap<String, String>,

    /// Unknown fields from newer versions, preserved verbatim.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            log_level: default_log_level(),
            overlay_position: default_overlay_position(),
            shortcuts: default_shortcuts(),
            extra: serde_json::Map::new(),
        }
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
}
