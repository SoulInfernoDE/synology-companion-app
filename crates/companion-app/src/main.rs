//! Syno Companion: a desktop companion for Synology NAS.

// No console window next to the app on Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod devices;
mod i18n;
mod monitor;
mod preferences;
mod secrets;
mod setup;
mod ssh;
mod tray;

use tauri::{Manager, WindowEvent};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // A second start brings the running window to the front instead.
            tray::show_window(app);
        }))
        .setup(|app| {
            let state = commands::AppState::new(app.path().app_config_dir()?);
            // Without a tray (some Linux desktops have none) the app works as before.
            match tray::create(app.handle(), &state.texts()) {
                Ok(items) => {
                    app.manage(items);
                }
                Err(error) => eprintln!("no tray icon: {error}"),
            }
            app.manage(state);
            app.manage(ssh::SshState::default());
            tauri::async_runtime::spawn(monitor::run(app.handle().clone()));
            Ok(())
        })
        .on_window_event(|window, event| {
            // With "keep running in the tray", closing only hides the window.
            if let WindowEvent::CloseRequested { api, .. } = event {
                let app = window.app_handle();
                let keep = app
                    .state::<commands::AppState>()
                    .preferences()
                    .close_to_tray;
                if keep && app.try_state::<tray::TrayItems>().is_some() {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::discover,
            commands::known_devices,
            commands::connect,
            commands::overview,
            commands::disconnect,
            commands::connected,
            commands::switch_to,
            commands::forget_device,
            commands::updates,
            commands::shares,
            commands::stop_beep,
            commands::power,
            commands::open_share,
            commands::wake,
            commands::get_preferences,
            commands::set_preferences,
            setup::probe,
            setup::begin_setup,
            setup::complete_setup,
            commands::ssh_service,
            commands::enable_ssh,
            ssh::ssh_link,
            ssh::ssh_unlink,
            ssh::diagnostics,
            ssh::memory_modules,
            ssh::terminal_open,
            ssh::terminal_write,
            ssh::terminal_resize,
            ssh::terminal_close,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Syno Companion");
}
