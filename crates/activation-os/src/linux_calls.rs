//! Real Linux login-scoped registration (`l2-service-activation` §4.4): a
//! systemd **user** unit (`~/.config/systemd/user/cronus.service`) enabled
//! via `systemctl --user`, with an **XDG autostart** fallback
//! (`~/.config/autostart/cronus.desktop`) on non-systemd hosts (§2, §4.4 —
//! this spec's own RFC→Stable resolution of the non-systemd TBD).
//!
//! Pure `std` — no crate beyond `cronus-contract`: unit/desktop-entry
//! content is a plain text file (`std::fs`), and `systemctl` is invoked as
//! an ordinary subprocess (`std::process::Command`), matching the project's
//! dependency policy (prefer std; justify every third-party crate) — there
//! is nothing here a crate would meaningfully abstract.
//!
//! **Verification scope, disclosed:** type-checked via cross-compilation
//! (`cargo check --target x86_64-unknown-linux-gnu`) from a Windows host; no
//! Linux host was available to run it. Real-world validation is deferred.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::SystemCalls;

const UNIT_NAME: &str = "cronus.service";
const DESKTOP_NAME: &str = "cronus.desktop";

fn home_dir() -> Result<PathBuf, String> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| "HOME is not set".to_string())
}

fn systemd_user_unit_path() -> Result<PathBuf, String> {
    Ok(home_dir()?.join(".config/systemd/user").join(UNIT_NAME))
}

fn xdg_autostart_path() -> Result<PathBuf, String> {
    Ok(home_dir()?.join(".config/autostart").join(DESKTOP_NAME))
}

/// The conventional presence check: `systemctl --user` succeeds (or reports
/// the well-defined "degraded" exit code) only under an active user systemd
/// instance — absent on a non-systemd host (§2's resolved TBD).
fn has_systemd_user() -> bool {
    Command::new("systemctl")
        .args(["--user", "is-system-running"])
        .output()
        .map(|o| o.status.success() || o.status.code() == Some(1))
        .unwrap_or(false)
}

fn unit_file_content(exe_path: &str) -> String {
    format!(
        "[Unit]\nDescription=Cronus background engine\n\n\
         [Service]\nExecStart={exe_path}\nRestart=on-failure\n\n\
         [Install]\nWantedBy=default.target\n"
    )
}

fn desktop_entry_content(exe_path: &str) -> String {
    format!(
        "[Desktop Entry]\nType=Application\nName=Cronus\nExec={exe_path}\n\
         X-GNOME-Autostart-enabled=true\nHidden=false\n"
    )
}

pub struct LinuxSystemCalls;

impl SystemCalls for LinuxSystemCalls {
    fn login_entry_present(&self) -> Result<bool, String> {
        if has_systemd_user() {
            Ok(systemd_user_unit_path()?.exists())
        } else {
            Ok(xdg_autostart_path()?.exists())
        }
    }

    fn login_entry_vetoed(&self) -> Result<Option<bool>, String> {
        if has_systemd_user() {
            let path = systemd_user_unit_path()?;
            if !path.exists() {
                return Ok(None);
            }
            // `systemctl --user is-enabled` distinguishes "enabled" from a
            // unit file that exists but was disabled — the weaker-wins
            // signal BA-8 asks for.
            let output = Command::new("systemctl")
                .args(["--user", "is-enabled", UNIT_NAME])
                .output()
                .map_err(|e| format!("systemctl --user is-enabled failed to run: {e}"))?;
            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(Some(stdout.trim() == "disabled"))
        } else {
            let path = xdg_autostart_path()?;
            if !path.exists() {
                return Ok(None);
            }
            let content = fs::read_to_string(&path)
                .map_err(|e| format!("could not read {}: {e}", path.display()))?;
            let vetoed = content.lines().any(|line| {
                let line = line.trim();
                line.eq_ignore_ascii_case("Hidden=true")
                    || line.eq_ignore_ascii_case("X-GNOME-Autostart-enabled=false")
            });
            Ok(Some(vetoed))
        }
    }

    fn write_login_entry(&self) -> Result<(), String> {
        let exe_path = std::env::current_exe()
            .map_err(|e| format!("could not resolve the current executable path: {e}"))?
            .display()
            .to_string();

        if has_systemd_user() {
            let path = systemd_user_unit_path()?;
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("could not create {}: {e}", parent.display()))?;
            }
            fs::write(&path, unit_file_content(&exe_path))
                .map_err(|e| format!("could not write {}: {e}", path.display()))?;
            let status = Command::new("systemctl")
                .args(["--user", "enable", UNIT_NAME])
                .status()
                .map_err(|e| format!("systemctl --user enable failed to run: {e}"))?;
            if !status.success() {
                return Err(format!("systemctl --user enable exited with {status}"));
            }
        } else {
            let path = xdg_autostart_path()?;
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("could not create {}: {e}", parent.display()))?;
            }
            fs::write(&path, desktop_entry_content(&exe_path))
                .map_err(|e| format!("could not write {}: {e}", path.display()))?;
        }
        Ok(())
    }

    fn remove_login_entry(&self) -> Result<(), String> {
        if has_systemd_user() {
            let path = systemd_user_unit_path()?;
            if path.exists() {
                let _ = Command::new("systemctl")
                    .args(["--user", "disable", UNIT_NAME])
                    .status();
                fs::remove_file(&path)
                    .map_err(|e| format!("could not remove {}: {e}", path.display()))?;
            }
        } else {
            let path = xdg_autostart_path()?;
            if path.exists() {
                fs::remove_file(&path)
                    .map_err(|e| format!("could not remove {}: {e}", path.display()))?;
            }
        }
        Ok(())
    }
}
