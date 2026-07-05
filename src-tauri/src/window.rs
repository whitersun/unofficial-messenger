use tauri::{Manager, Url};

const MAIN_WINDOW_LABEL: &str = "main";

pub(crate) fn navigate_main_window(app: &tauri::AppHandle, url: Url) {
    with_main_window(app, |window| {
        if let Err(error) = window.navigate(url.clone()) {
            eprintln!("failed to navigate main window to {url}: {error}");
        }
    });
}

pub(crate) fn title_bar_text(title: &str) -> String {
    format!(" {}", title)
}

fn with_main_window(app: &tauri::AppHandle, action: impl FnOnce(tauri::WebviewWindow)) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        action(window);
    }
}

pub(crate) fn show_main_window(app: &tauri::AppHandle) {
    with_main_window(app, |window| {
        if let Err(error) = window.show() {
            eprintln!("failed to show main window: {error}");
        }

        if let Err(error) = window.set_focus() {
            eprintln!("failed to focus main window: {error}");
        }
    });
}

pub(crate) fn hide_main_window(app: &tauri::AppHandle) {
    with_main_window(app, |window| {
        if let Err(error) = window.hide() {
            eprintln!("failed to hide main window: {error}");
        }
    });
}

pub(crate) fn reload_main_window(app: &tauri::AppHandle) {
    with_main_window(app, |window| {
        if let Err(error) = window.reload() {
            eprintln!("failed to reload main window: {error}");
        }
    });
}
