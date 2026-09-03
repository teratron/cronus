//! Cronus desktop shell (Tauri v2).
//!
//! Presentation host only: it owns the window and the IPC bridge to the core.
//! No domain logic lives here — UI intents cross to the core over the typed
//! commands in [`bridge`].

pub mod bridge;
pub mod instance;
pub mod mcp;
pub mod overlay;
pub mod prompts;
pub mod settings;
pub mod shortcuts;
pub mod tray;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(bridge::core_bridge())
        .setup(|app| {
            // Settings load is fail-soft: a broken file (or no config dir) must
            // not stop the shell — fall back to defaults and report on stderr.
            // The resolved path is kept so IPC writes persist to the same file.
            let path = app
                .path()
                .app_config_dir()
                .map(|dir| dir.join("settings.json"));
            let settings = match &path {
                Ok(file) => settings::load_or_create(file).unwrap_or_else(|reason| {
                    eprintln!("settings: falling back to defaults ({reason})");
                    settings::Settings::default()
                }),
                Err(reason) => {
                    eprintln!("settings: no config dir ({reason})");
                    settings::Settings::default()
                }
            };
            let store_path = path.unwrap_or_else(|_| std::path::PathBuf::from("settings.json"));
            app.manage(settings::SettingsStore::new(store_path, settings));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            bridge::capability_version,
            bridge::capability_status,
            bridge::capability_settings_get,
            bridge::capability_settings_set
        ])
        .run(tauri::generate_context!())
        .expect("error while running the Cronus desktop application");
}
