//! Syno Companion: a desktop companion for Synology NAS.

// No console window next to the app on Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod devices;
mod secrets;

use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // A second start brings the running window to the front instead.
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .setup(|app| {
            let config = app.path().app_config_dir()?;
            app.manage(commands::AppState::new(devices::DeviceStore::load(&config)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::discover,
            commands::known_devices,
            commands::connect,
            commands::overview,
            commands::disconnect,
            commands::forget_device,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Syno Companion");
}
