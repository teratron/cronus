//! Locating the real product binary a run drives.
//!
//! `CARGO_BIN_EXE_<name>` — the mechanism `crates/cli/tests/cli_smoke.rs`
//! already uses — is scoped to a binary's *own* package's test/bench
//! targets; it is not set for `cronus-simulation`'s tests, and it does not
//! exist at all for `cronus-sim` once built and run outside `cargo test`.
//! This resolves the same binary the other way: from the current
//! executable's own location, walking up to the shared `target/<profile>/`
//! directory every workspace binary is built into, so the same lookup
//! works whether the caller is a `cargo test` binary (in `target/<profile>/
//! deps/`) or `cronus-sim` itself once installed there (in
//! `target/<profile>/` directly).

use std::path::PathBuf;

/// Resolve the path to a workspace binary named `bin_name`.
///
/// Resolution order: an explicit environment override
/// (`CRONUS_SIM_BIN_<BIN_NAME>`, e.g. `CRONUS_SIM_BIN_CRONUS` for
/// `bin_name = "cronus"`) always wins; otherwise, search the current
/// executable's parent and grandparent directories for a sibling binary of
/// that name. `None` means neither found anything — callers must refuse to
/// run rather than fall back to whatever `bin_name` resolves to on `PATH`,
/// since that could be an unrelated, already-installed copy of the product.
pub fn resolve_binary(bin_name: &str) -> Option<PathBuf> {
    let override_var = format!(
        "CRONUS_SIM_BIN_{}",
        bin_name.to_uppercase().replace('-', "_")
    );
    if let Ok(path) = std::env::var(&override_var)
        && !path.is_empty()
    {
        return Some(PathBuf::from(path));
    }

    let exe_name = if cfg!(windows) {
        format!("{bin_name}.exe")
    } else {
        bin_name.to_string()
    };

    let current = std::env::current_exe().ok()?;
    let mut dir = current.parent()?.to_path_buf();
    // One hop covers `cronus-sim` running from `target/<profile>/` directly;
    // a second covers a `cargo test` binary running from
    // `target/<profile>/deps/`.
    for _ in 0..2 {
        let candidate = dir.join(&exe_name);
        if candidate.is_file() {
            return Some(candidate);
        }
        dir = dir.parent()?.to_path_buf();
    }
    None
}

/// Test-only: guarantee `cronus` is built and return its path, cached for
/// the lifetime of this test binary's process.
///
/// A bare workspace-level `cargo test` builds every member's targets, but
/// gives no ordering guarantee between two packages that do not depend on
/// each other — `cronus-simulation` and `cronus-cli` are exactly such a
/// pair. Without this, [`resolve_binary`] alone could race a `cronus`
/// build that has not finished yet and fail with a confusing "not found"
/// rather than a clear one-time build. Invoking `cargo build --bin cronus`
/// directly removes the race and costs almost nothing once the binary is
/// already fresh, which `cargo` itself decides.
#[cfg(test)]
pub(crate) fn test_cronus_binary() -> PathBuf {
    use std::sync::OnceLock;
    static RESOLVED: OnceLock<PathBuf> = OnceLock::new();
    RESOLVED
        .get_or_init(|| {
            let status = std::process::Command::new(env!("CARGO"))
                .args(["build", "-p", "cronus-cli", "--bin", "cronus"])
                .status()
                .expect("failed to invoke `cargo build --bin cronus`");
            assert!(status.success(), "`cargo build --bin cronus` must succeed");
            resolve_binary("cronus").expect("cronus must resolve immediately after building it")
        })
        .clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_the_real_cronus_binary_built_by_this_workspace() {
        let resolved = test_cronus_binary();
        assert!(
            resolved.is_file(),
            "resolved path {resolved:?} must point at an existing file"
        );
        // Must not be a PATH-resolved fallback: it must sit under this
        // process's own build output tree, not some unrelated installed copy.
        let current = std::env::current_exe().expect("current_exe");
        let build_root = current
            .parent()
            .and_then(|p| p.parent())
            .expect("target/<profile>/deps has a target/<profile> parent");
        assert!(
            resolved.starts_with(build_root),
            "resolved {resolved:?} must sit under this build's own {build_root:?}"
        );
    }

    #[test]
    fn an_explicit_override_wins_over_discovery() {
        let key = "CRONUS_SIM_BIN_CRONUS";
        // SAFETY: single-threaded within this test's own scope; no other
        // test reads or writes this specific key.
        unsafe {
            std::env::set_var(key, "explicit-override-path");
        }
        let resolved = resolve_binary("cronus");
        unsafe {
            std::env::remove_var(key);
        }
        assert_eq!(
            resolved,
            Some(PathBuf::from("explicit-override-path")),
            "an explicit override must be returned verbatim, without existence-checking it"
        );
    }
}
