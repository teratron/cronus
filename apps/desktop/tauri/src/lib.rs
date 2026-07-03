//! Cronus desktop shell (Tauri v2).
//!
//! Presentation host only: it owns the window and the IPC bridge to the core.
//! No domain logic lives here — UI intents cross to the core over the typed
//! commands in [`bridge`].

pub mod bridge;
pub mod instance;
pub mod overlay;
pub mod settings;
pub mod shortcuts;
pub mod tray;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(bridge::core_bridge())
        .setup(|app| {
            // Settings load is fail-soft: a broken file must not stop the
            // shell — fall back to defaults and report on stderr.
            let loaded = app
                .path()
                .app_config_dir()
                .map_err(|e| e.to_string())
                .and_then(|dir| {
                    settings::load_or_create(&dir.join("settings.json")).map_err(|e| e.to_string())
                });
            let settings = match loaded {
                Ok(settings) => settings,
                Err(reason) => {
                    eprintln!("settings: falling back to defaults ({reason})");
                    settings::Settings::default()
                }
            };
            app.manage(settings);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            bridge::capability_version,
            bridge::capability_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running the Cronus desktop application");
}
