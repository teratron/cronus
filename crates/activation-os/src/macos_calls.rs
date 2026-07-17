//! Real macOS login-scoped registration (`l2-service-activation` §4.3): a
//! `LaunchAgent` plist in `~/Library/LaunchAgents/`, loaded via `launchctl`.
//!
//! **Disclosed gap vs. the Stable spec:** §4.3 describes `SMAppService`
//! (`.loginItem`), whose `status` distinguishes `requiresApproval` — a
//! signal this LaunchAgent+launchctl implementation cannot produce (that
//! approval gate is specific to the modern `ServiceManagement` framework
//! API, callable only through Objective-C/Swift interop — e.g. `objc2` — a
//! meaningfully larger dependency undertaking deferred to its own follow-up,
//! and not evaluable at all on the Windows host this was written on, which
//! has no macOS SDK). This implementation still delivers genuine
//! login-scoped activation (BA-2); `login_entry_vetoed` always reports
//! `None` (no veto signal available), which is honest per BA-8 — never a
//! fabricated "not vetoed". Pure `std` — no crate: a plist is XML text
//! (`std::fs`), `launchctl` is an ordinary subprocess
//! (`std::process::Command`).
//!
//! **Verification scope, disclosed:** type-checked via cross-compilation
//! (`cargo check --target aarch64-apple-darwin`) from a Windows host; no
//! macOS host was available to run it. Real-world validation is deferred.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::SystemCalls;

const LABEL: &str = "com.cronus.engine";

fn home_dir() -> Result<PathBuf, String> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| "HOME is not set".to_string())
}

fn plist_path() -> Result<PathBuf, String> {
    Ok(home_dir()?
        .join("Library/LaunchAgents")
        .join(format!("{LABEL}.plist")))
}

fn plist_content(exe_path: &str) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
         <plist version=\"1.0\">\n\
         <dict>\n\
         \x20\x20\x20\x20<key>Label</key>\n\
         \x20\x20\x20\x20<string>{LABEL}</string>\n\
         \x20\x20\x20\x20<key>ProgramArguments</key>\n\
         \x20\x20\x20\x20<array>\n\
         \x20\x20\x20\x20\x20\x20\x20\x20<string>{exe_path}</string>\n\
         \x20\x20\x20\x20</array>\n\
         \x20\x20\x20\x20<key>RunAtLoad</key>\n\
         \x20\x20\x20\x20<true/>\n\
         </dict>\n\
         </plist>\n"
    )
}

pub struct MacosSystemCalls;

impl SystemCalls for MacosSystemCalls {
    fn login_entry_present(&self) -> Result<bool, String> {
        Ok(plist_path()?.exists())
    }

    fn login_entry_vetoed(&self) -> Result<Option<bool>, String> {
        // No veto signal available without SMAppService (see module doc) —
        // honestly reported as "no signal", never fabricated as "not vetoed".
        Ok(None)
    }

    fn write_login_entry(&self) -> Result<(), String> {
        let exe_path = std::env::current_exe()
            .map_err(|e| format!("could not resolve the current executable path: {e}"))?
            .display()
            .to_string();
        let path = plist_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("could not create {}: {e}", parent.display()))?;
        }
        fs::write(&path, plist_content(&exe_path))
            .map_err(|e| format!("could not write {}: {e}", path.display()))?;

        let status = Command::new("launchctl")
            .args(["load", "-w"])
            .arg(&path)
            .status()
            .map_err(|e| format!("launchctl load failed to run: {e}"))?;
        if !status.success() {
            return Err(format!("launchctl load exited with {status}"));
        }
        Ok(())
    }

    fn remove_login_entry(&self) -> Result<(), String> {
        let path = plist_path()?;
        if !path.exists() {
            return Ok(());
        }
        let _ = Command::new("launchctl")
            .args(["unload", "-w"])
            .arg(&path)
            .status();
        fs::remove_file(&path).map_err(|e| format!("could not remove {}: {e}", path.display()))
    }
}
