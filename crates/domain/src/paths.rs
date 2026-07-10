//! OS-native path resolution for the two tiers plus cache and logs.
//!
//! The immutable program tier is install-located; the mutable state tier and the
//! cache/logs roots resolve to per-OS user directories. A portable mode groups
//! every root under a single base directory.

use std::path::PathBuf;

/// The resolvable root locations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Root {
    /// Immutable program tier (compiled binaries, templates, role catalog).
    Program,
    /// Mutable state tier (config, memory, offices, hired employees).
    State,
    /// Regenerable cache.
    Cache,
    /// Runtime logs.
    Logs,
}

/// Resolves Cronus roots to OS-native paths, or under a portable base when set.
#[derive(Debug, Clone)]
pub struct Paths {
    portable_base: Option<PathBuf>,
}

impl Paths {
    /// OS-native resolution (the default deployment mode).
    pub fn os_native() -> Self {
        Self {
            portable_base: None,
        }
    }

    /// Portable mode: all roots live under `base`.
    pub fn portable(base: impl Into<PathBuf>) -> Self {
        Self {
            portable_base: Some(base.into()),
        }
    }

    /// Resolve a root to its directory path.
    pub fn resolve(&self, root: Root) -> PathBuf {
        if let Some(base) = &self.portable_base {
            let sub = match root {
                Root::Program => "program",
                Root::State => "state",
                Root::Cache => "cache",
                Root::Logs => "logs",
            };
            return base.join(sub);
        }
        self.os_native_path(root)
    }

    #[cfg(target_os = "windows")]
    fn os_native_path(&self, root: Root) -> PathBuf {
        let env = |key: &str, fallback: &str| {
            std::env::var(key)
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from(fallback))
        };
        match root {
            Root::Program => env("ProgramFiles", "C:/Program Files").join("Cronus"),
            Root::State => env("APPDATA", ".").join("Cronus"),
            Root::Cache => env("LOCALAPPDATA", ".").join("Cronus").join("Cache"),
            Root::Logs => env("LOCALAPPDATA", ".").join("Cronus").join("Logs"),
        }
    }

    #[cfg(target_os = "macos")]
    fn os_native_path(&self, root: Root) -> PathBuf {
        let home = std::env::var("HOME").map(PathBuf::from).unwrap_or_default();
        match root {
            Root::Program => PathBuf::from("/Applications/Cronus.app/Contents/Resources"),
            Root::State => home.join("Library/Application Support/Cronus"),
            Root::Cache => home.join("Library/Caches/Cronus"),
            Root::Logs => home.join("Library/Logs/Cronus"),
        }
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    fn os_native_path(&self, root: Root) -> PathBuf {
        let home = std::env::var("HOME").map(PathBuf::from).unwrap_or_default();
        let xdg =
            |key: &str, default: PathBuf| std::env::var(key).map(PathBuf::from).unwrap_or(default);
        match root {
            Root::Program => PathBuf::from("/opt/cronus"),
            Root::State => xdg("XDG_DATA_HOME", home.join(".local/share")).join("cronus"),
            Root::Cache => xdg("XDG_CACHE_HOME", home.join(".cache")).join("cronus"),
            Root::Logs => xdg("XDG_STATE_HOME", home.join(".local/state")).join("cronus"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portable_groups_all_roots_under_base() {
        let paths = Paths::portable("/tmp/cronus-portable");
        for root in [Root::Program, Root::State, Root::Cache, Root::Logs] {
            assert!(paths.resolve(root).starts_with("/tmp/cronus-portable"));
        }
    }

    #[test]
    fn os_native_roots_are_nonempty_and_distinct() {
        let paths = Paths::os_native();
        let state = paths.resolve(Root::State);
        let cache = paths.resolve(Root::Cache);
        assert!(!state.as_os_str().is_empty());
        assert!(!cache.as_os_str().is_empty());
        assert_ne!(state, cache);
    }
}
