use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

use crate::{
    cache::quit_after_clearing_cache,
    settings::{is_hide_on_close_enabled, set_hide_on_close_enabled, SharedAppSettings},
    startup::{is_start_with_windows_enabled, set_start_with_windows},
    window::{hide_main_window, reload_main_window, show_main_window},
};

const TRAY_MENU_SHOW: &str = "tray-show";
const TRAY_MENU_HIDE: &str = "tray-hide";
const TRAY_MENU_RELOAD: &str = "tray-reload";
const TRAY_MENU_UPDATE: &str = "tray-update";
const TRAY_MENU_START_WITH_WINDOWS: &str = "tray-start-with-windows";
const TRAY_MENU_HIDE_ON_CLOSE: &str = "tray-hide-on-close";
const TRAY_MENU_QUIT: &str = "tray-quit";
pub(crate) const TRAY_ID: &str = "messenger-tray";

struct UpdateTrayMenu {
    menu: Menu<tauri::Wry>,
    item: MenuItem<tauri::Wry>,
    separator: PredefinedMenuItem<tauri::Wry>,
    is_visible: AtomicBool,
}

pub(crate) fn create_tray(app: &tauri::App, settings: SharedAppSettings) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, TRAY_MENU_SHOW, "Show", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, TRAY_MENU_HIDE, "Hide", true, None::<&str>)?;
    let reload = MenuItem::with_id(
        app,
        TRAY_MENU_RELOAD,
        "Reload Messenger",
        true,
        None::<&str>,
    )?;
    let update = MenuItem::with_id(
        app,
        TRAY_MENU_UPDATE,
        "Update Messenger",
        true,
        None::<&str>,
    )?;
    let update_separator = PredefinedMenuItem::separator(app)?;
    let start_with_windows = CheckMenuItem::with_id(
        app,
        TRAY_MENU_START_WITH_WINDOWS,
        "Start with Windows",
        cfg!(windows),
        false,
        None::<&str>,
    )?;
    let hide_on_close = CheckMenuItem::with_id(
        app,
        TRAY_MENU_HIDE_ON_CLOSE,
        "Hide on Close",
        true,
        is_hide_on_close_enabled(&settings),
        None::<&str>,
    )?;
    let quit = MenuItem::with_id(app, TRAY_MENU_QUIT, "Quit", true, None::<&str>)?;
    let options_separator = PredefinedMenuItem::separator(app)?;
    let quit_separator = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(
        app,
        &[
            &show,
            &hide,
            &reload,
            &options_separator,
            &start_with_windows,
            &hide_on_close,
            &quit_separator,
            &quit,
        ],
    )?;
    let icon = app
        .default_window_icon()
        .cloned()
        .expect("missing default window icon");
    let start_with_windows_for_menu = start_with_windows.clone();
    let start_with_windows_for_startup_check = start_with_windows.clone();
    let hide_on_close_for_menu = hide_on_close.clone();
    let settings_for_menu = Arc::clone(&settings);
    let update_for_state = update.clone();
    let update_separator_for_state = update_separator.clone();
    let menu_for_state = menu.clone();

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .tooltip("Messenger")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } | TrayIconEvent::DoubleClick {
                    button: MouseButton::Left,
                    ..
                }
            ) {
                show_main_window(tray.app_handle());
            }
        })
        .on_menu_event(move |app, event| match event.id().as_ref() {
            TRAY_MENU_SHOW => show_main_window(app),
            TRAY_MENU_HIDE => hide_main_window(app),
            TRAY_MENU_RELOAD => reload_main_window(app),
            TRAY_MENU_UPDATE => {
                show_main_window(app);

                if let Some(window) = app.get_webview_window("main") {
                    if let Err(error) = window.eval("window.__TAURI_MESSENGER_SHOW_UPDATE__?.()") {
                        eprintln!("failed to show update prompt from tray: {error}");
                    }
                }
            }
            TRAY_MENU_START_WITH_WINDOWS => {
                let enabled = start_with_windows_for_menu.is_checked().unwrap_or(false);
                let start_with_windows = start_with_windows_for_menu.clone();
                let _ = start_with_windows.set_enabled(false);

                std::thread::spawn(move || {
                    if let Err(error) = set_start_with_windows(enabled) {
                        eprintln!("failed to toggle start with Windows: {error}");
                        let _ = start_with_windows.set_checked(!enabled);
                    }

                    let _ = start_with_windows.set_enabled(true);
                });
            }
            TRAY_MENU_HIDE_ON_CLOSE => {
                let enabled = hide_on_close_for_menu.is_checked().unwrap_or(true);
                set_hide_on_close_enabled(app, &settings_for_menu, enabled);
            }
            TRAY_MENU_QUIT => quit_after_clearing_cache(app),
            _ => {}
        })
        .build(app)?;

    app.manage(UpdateTrayMenu {
        menu: menu_for_state,
        item: update_for_state,
        separator: update_separator_for_state,
        is_visible: AtomicBool::new(false),
    });

    std::thread::spawn(move || {
        let enabled = is_start_with_windows_enabled();

        if enabled {
            if let Err(error) = set_start_with_windows(true) {
                eprintln!("failed to refresh start with Windows entry: {error}");
            }
        }

        let _ = start_with_windows_for_startup_check.set_checked(enabled);
    });

    Ok(())
}

pub(crate) fn show_update_in_tray(app: &tauri::AppHandle, version: &str) {
    let state = app.state::<UpdateTrayMenu>();

    if let Err(error) = state.item.set_text(format!("Update to v{version}...")) {
        eprintln!("failed to update tray update label: {error}");
    }

    if state.is_visible.swap(true, Ordering::AcqRel) {
        return;
    }

    if let Err(error) = state.menu.insert_items(&[&state.separator, &state.item], 3) {
        state.is_visible.store(false, Ordering::Release);
        eprintln!("failed to add update action to tray: {error}");
    }
}
