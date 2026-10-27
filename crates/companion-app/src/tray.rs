//! The tray icon: the state of the active NAS at a glance, the window and quitting.

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Wry};

use crate::i18n::Texts;

const TRAY_ID: &str = "main";

/// Menu entries whose text changes with state or language.
pub struct TrayItems {
    status: MenuItem<Wry>,
    open: MenuItem<Wry>,
    quit: MenuItem<Wry>,
}

pub fn show_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

pub fn create(app: &AppHandle, texts: &Texts) -> tauri::Result<TrayItems> {
    let status = MenuItem::with_id(
        app,
        "status",
        texts.get("tray-not-connected", None),
        false,
        None::<&str>,
    )?;
    let open = MenuItem::with_id(
        app,
        "open",
        texts.get("tray-open", None),
        true,
        None::<&str>,
    )?;
    let quit = MenuItem::with_id(
        app,
        "quit",
        texts.get("tray-quit", None),
        true,
        None::<&str>,
    )?;
    let menu = Menu::with_items(
        app,
        &[&status, &PredefinedMenuItem::separator(app)?, &open, &quit],
    )?;
    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("Syno Companion")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => show_window(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_window(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(TrayItems { status, open, quit })
}

impl TrayItems {
    /// Shows `text` as the first menu line and as the tooltip.
    pub fn set_status(&self, app: &AppHandle, text: &str) {
        let _ = self.status.set_text(text);
        if let Some(tray) = app.tray_by_id(TRAY_ID) {
            let _ = tray.set_tooltip(Some(format!("Syno Companion – {text}")));
        }
    }

    /// Re-labels the fixed entries after a change of language.
    pub fn relabel(&self, texts: &Texts) {
        let _ = self.open.set_text(texts.get("tray-open", None));
        let _ = self.quit.set_text(texts.get("tray-quit", None));
    }
}
