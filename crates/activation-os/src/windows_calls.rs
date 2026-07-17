//! Real Windows login-scoped registration (`l2-service-activation` §4.2):
//! the `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` value, plus the
//! `StartupApproved\Run` veto read (BA-8: the raw `Run` value alone is not
//! the state — Windows records a user's Task-Manager disablement separately,
//! leaving `Run` intact).
//!
//! Raw `windows-sys` FFI, not a wrapper crate — matches this project's
//! std-first / hand-rolled-primitive precedent (e.g. `crates/model-local`'s
//! hand-rolled HTTP/1.1 rather than pulling in `ureq`).
//!
//! **Disclosed limitation:** the `StartupApproved\Run` binary value's format
//! is undocumented by Microsoft; the disabled-marker byte (`0x02`/`0x03` in
//! the first byte) is based on widely-cited community reverse-engineering,
//! not an official contract. An absent value, an unexpected length, or any
//! read failure reports `None` (no veto signal) — never guessed.

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

use windows_sys::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
use windows_sys::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE, REG_SZ, RegCloseKey, RegDeleteValueW,
    RegOpenKeyExW, RegQueryValueExW, RegSetValueExW,
};

use crate::SystemCalls;

const RUN_KEY_PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const STARTUP_APPROVED_KEY_PATH: &str =
    r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run";
const VALUE_NAME: &str = "Cronus";

fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

/// An open registry key handle, closed on drop.
struct OpenKey(HKEY);

impl OpenKey {
    /// Open `subkey` under `HKEY_CURRENT_USER`. `Ok(None)` means the key
    /// itself does not exist (distinct from a real failure) — `CurrentVersion
    /// \Run` always exists on a real Windows install, but `StartupApproved
    /// \Run` may not if the user has never touched Task Manager's Startup
    /// tab, which is the expected common case, not an error.
    fn open(subkey: &str, access: u32) -> Result<Option<Self>, String> {
        let wide = to_wide(subkey);
        let mut handle: HKEY = std::ptr::null_mut();
        // SAFETY: `wide` is a valid null-terminated UTF-16 string alive for
        // the duration of the call; `handle` is a valid out-pointer sized
        // for a single HKEY.
        let status =
            unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, wide.as_ptr(), 0, access, &mut handle) };
        match status as u32 {
            ERROR_SUCCESS => Ok(Some(OpenKey(handle))),
            ERROR_FILE_NOT_FOUND => Ok(None),
            other => Err(format!("RegOpenKeyExW({subkey}) failed: error {other}")),
        }
    }
}

impl Drop for OpenKey {
    fn drop(&mut self) {
        // SAFETY: `self.0` is a valid HKEY opened by `RegOpenKeyExW` above,
        // not yet closed.
        unsafe {
            RegCloseKey(self.0);
        }
    }
}

pub struct WindowsSystemCalls;

impl SystemCalls for WindowsSystemCalls {
    fn login_entry_present(&self) -> Result<bool, String> {
        let Some(key) = OpenKey::open(RUN_KEY_PATH, KEY_READ)? else {
            return Ok(false);
        };
        let wide_name = to_wide(VALUE_NAME);
        let mut value_type: u32 = 0;
        let mut data_len: u32 = 0;
        // A null data buffer just measures/checks presence; absence of the
        // named value (not the key) also reports ERROR_FILE_NOT_FOUND.
        // SAFETY: `key.0` is a valid open handle; `wide_name` is a valid
        // null-terminated UTF-16 string; the data pointer is null (measuring
        // only), and `value_type`/`data_len` are valid out-pointers.
        let status = unsafe {
            RegQueryValueExW(
                key.0,
                wide_name.as_ptr(),
                std::ptr::null_mut(),
                &mut value_type,
                std::ptr::null_mut(),
                &mut data_len,
            )
        };
        match status as u32 {
            ERROR_SUCCESS => Ok(true),
            ERROR_FILE_NOT_FOUND => Ok(false),
            other => Err(format!(
                "RegQueryValueExW({VALUE_NAME}) failed: error {other}"
            )),
        }
    }

    fn login_entry_vetoed(&self) -> Result<Option<bool>, String> {
        let Some(key) = OpenKey::open(STARTUP_APPROVED_KEY_PATH, KEY_READ)? else {
            return Ok(None); // no StartupApproved key at all — no veto signal
        };
        let wide_name = to_wide(VALUE_NAME);
        let mut value_type: u32 = 0;
        let mut buf = [0u8; 12]; // the community-documented value is 12 bytes
        let mut data_len: u32 = buf.len() as u32;
        // SAFETY: `key.0` is a valid open handle; `wide_name` is a valid
        // null-terminated UTF-16 string; `buf` is a 12-byte buffer and
        // `data_len` correctly reports its capacity as the in/out length.
        let status = unsafe {
            RegQueryValueExW(
                key.0,
                wide_name.as_ptr(),
                std::ptr::null_mut(),
                &mut value_type,
                buf.as_mut_ptr(),
                &mut data_len,
            )
        };
        match status as u32 {
            ERROR_SUCCESS if data_len >= 1 => {
                // Disclosed heuristic (see module doc): 0x02/0x03 in the
                // first byte denotes a user- or policy-disabled entry.
                Ok(Some(buf[0] == 0x02 || buf[0] == 0x03))
            }
            // Unexpected zero-length value, absence, or any other read
            // failure: no veto signal, never guessed as vetoed.
            _ => Ok(None),
        }
    }

    fn write_login_entry(&self) -> Result<(), String> {
        // `CurrentVersion\Run` always exists on a real Windows install.
        let key = OpenKey::open(RUN_KEY_PATH, KEY_SET_VALUE)?
            .ok_or_else(|| "Run key not found".to_string())?;
        let exe_path = std::env::current_exe()
            .map_err(|e| format!("could not resolve the current executable path: {e}"))?;
        let wide_name = to_wide(VALUE_NAME);
        let wide_value = to_wide(&exe_path.display().to_string());
        // SAFETY: reinterpreting a `Vec<u16>` (UTF-16 code units) as its raw
        // little-endian byte representation for the REG_SZ write — valid for
        // the buffer's own lifetime, which outlives this call.
        let data_bytes: &[u8] = unsafe {
            std::slice::from_raw_parts(wide_value.as_ptr() as *const u8, wide_value.len() * 2)
        };
        // SAFETY: `key.0` is a valid open handle with `KEY_SET_VALUE` access;
        // `wide_name` is a valid null-terminated UTF-16 string; `data_bytes`
        // is a valid, correctly-sized REG_SZ payload (UTF-16, null-terminated).
        let status = unsafe {
            RegSetValueExW(
                key.0,
                wide_name.as_ptr(),
                0,
                REG_SZ,
                data_bytes.as_ptr(),
                data_bytes.len() as u32,
            )
        };
        match status as u32 {
            ERROR_SUCCESS => Ok(()),
            other => Err(format!("RegSetValueExW failed: error {other}")),
        }
    }

    fn remove_login_entry(&self) -> Result<(), String> {
        let Some(key) = OpenKey::open(RUN_KEY_PATH, KEY_SET_VALUE)? else {
            return Ok(()); // key absent — nothing to remove, already inactive
        };
        let wide_name = to_wide(VALUE_NAME);
        // SAFETY: `key.0` is a valid open handle with `KEY_SET_VALUE` access;
        // `wide_name` is a valid null-terminated UTF-16 string.
        let status = unsafe { RegDeleteValueW(key.0, wide_name.as_ptr()) };
        match status as u32 {
            ERROR_SUCCESS | ERROR_FILE_NOT_FOUND => Ok(()),
            other => Err(format!("RegDeleteValueW failed: error {other}")),
        }
    }
}
