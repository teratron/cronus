//! Cronus desktop shell (Tauri v2).
//!
//! Presentation host only: it owns the window and (later) the IPC bridge to the
//! core. No domain logic lives here — UI intents cross to the core over typed
//! commands wired in a subsequent task.

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running the Cronus desktop application");
}
