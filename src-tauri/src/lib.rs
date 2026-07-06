mod app_chrome_script;
mod badge;
mod navigation;
mod settings;
mod startup;
mod tray;
mod window;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use app_chrome_script::APP_CHROME_SCRIPT;
use badge::{unread_count_from_title, update_taskbar_badge};
use navigation::{
    external_url_from_marked_navigation, is_auth_navigation, open_in_system_browser,
    should_keep_in_webview, should_open_in_app,
};
use settings::{is_hide_on_close_enabled, load_settings};
use startup::{
    normalize_startup_working_directory, was_launched_from_windows_startup, STARTUP_LOAD_DELAY_MS,
};
use tauri::{window::Color, WebviewUrl, WebviewWindowBuilder};
use tray::create_tray;
use window::{hide_main_window, navigate_main_window, title_bar_text};

static POPUP_WINDOW_COUNTER: AtomicUsize = AtomicUsize::new(1);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    normalize_startup_working_directory();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let mut window_config = app
                .config()
                .app
                .windows
                .first()
                .expect("missing main window config")
                .clone();
            let launched_from_windows_startup = was_launched_from_windows_startup();
            let startup_url = match &window_config.url {
                WebviewUrl::External(url) => Some(url.clone()),
                _ => None,
            };

            if launched_from_windows_startup {
                window_config.url = WebviewUrl::External(
                    "about:blank"
                        .parse()
                        .expect("about:blank should be a valid URL"),
                );
                window_config.visible = false;
                window_config.focus = false;
            }

            let app_handle = app.handle().clone();
            let app_handle_for_navigation = app_handle.clone();
            let app_handle_for_new_window = app_handle.clone();
            let settings = Arc::new(Mutex::new(load_settings(&app_handle)));
            let badge_state = Arc::new(Mutex::new(None));
            let badge_state_for_title = Arc::clone(&badge_state);

            let window = WebviewWindowBuilder::from_config(app, &window_config)?
                .initialization_script(APP_CHROME_SCRIPT)
                .on_navigation(move |url| {
                    handle_navigation_request(&app_handle_for_navigation, url)
                })
                .on_new_window(move |url, _features| {
                    handle_new_window_request(&app_handle_for_new_window, url, _features)
                })
                .on_document_title_changed(move |window, title| {
                    let title = title.trim();

                    if let Some(unread_count) = unread_count_from_title(title) {
                        update_taskbar_badge(&window, &badge_state_for_title, Some(unread_count));
                    }

                    let title = if title.is_empty() { "Messenger" } else { title };
                    let title = title_bar_text(title);

                    if let Err(error) = window.set_title(&title) {
                        eprintln!("failed to update window title: {error}");
                    }
                })
                .build()?;
            if let Err(error) = window.set_background_color(Some(Color(247, 248, 251, 255))) {
                eprintln!("failed to set startup background color: {error}");
            }
            if let Some(icon) = app.default_window_icon().cloned() {
                if let Err(error) = window.set_icon(icon) {
                    eprintln!("failed to set window icon: {error}");
                }
            }

            if launched_from_windows_startup {
                if let Some(startup_url) = startup_url {
                    let app_handle_for_startup_load = app_handle.clone();

                    std::thread::spawn(move || {
                        std::thread::sleep(Duration::from_millis(STARTUP_LOAD_DELAY_MS));
                        navigate_main_window(&app_handle_for_startup_load, startup_url);
                    });
                }
            }

            create_tray(app, Arc::clone(&settings))?;

            let window_for_events = window.clone();
            let app_handle_for_events = app_handle.clone();
            let settings_for_events = Arc::clone(&settings);
            let badge_state_for_events = Arc::clone(&badge_state);
            window.on_window_event(move |event| match event {
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    if is_hide_on_close_enabled(&settings_for_events) {
                        api.prevent_close();
                        hide_main_window(&app_handle_for_events);
                    }
                }
                tauri::WindowEvent::Focused(true) => {
                    update_taskbar_badge(&window_for_events, &badge_state_for_events, None);
                }
                _ => {}
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn handle_navigation_request(app: &tauri::AppHandle, url: &tauri::Url) -> bool {
    if let Some(external_url) = external_url_from_marked_navigation(url) {
        if is_auth_navigation(&external_url) {
            navigate_main_window(app, external_url);
            return false;
        }

        open_in_system_browser(app, &external_url);
        return false;
    }

    if should_open_in_app(url) || should_keep_in_webview(url) {
        return true;
    }

    open_in_system_browser(app, url);
    false
}

fn handle_new_window_request(
    app: &tauri::AppHandle,
    url: tauri::Url,
    features: tauri::webview::NewWindowFeatures,
) -> tauri::webview::NewWindowResponse<tauri::Wry> {
    if let Some(external_url) = external_url_from_marked_navigation(&url) {
        open_in_system_browser(app, &external_url);
        return tauri::webview::NewWindowResponse::Deny;
    }

    if is_auth_navigation(&url) {
        navigate_main_window(app, url);
        return tauri::webview::NewWindowResponse::Deny;
    }

    if should_open_in_app(&url) || should_keep_in_webview(&url) {
        return match create_popup_window(app, url, features) {
            Ok(window) => tauri::webview::NewWindowResponse::Create { window },
            Err(error) => {
                eprintln!("failed to create Messenger popup window: {error}");
                tauri::webview::NewWindowResponse::Deny
            }
        };
    }

    open_in_system_browser(app, &url);
    tauri::webview::NewWindowResponse::Deny
}

fn create_popup_window(
    app: &tauri::AppHandle,
    url: tauri::Url,
    features: tauri::webview::NewWindowFeatures,
) -> tauri::Result<tauri::WebviewWindow> {
    let label = format!(
        "messenger-popup-{}",
        POPUP_WINDOW_COUNTER.fetch_add(1, Ordering::Relaxed)
    );
    let app_handle_for_navigation = app.clone();
    let app_handle_for_new_window = app.clone();

    WebviewWindowBuilder::new(app, label, WebviewUrl::External(url.clone()))
        .title("Messenger")
        .inner_size(980.0, 720.0)
        .min_inner_size(420.0, 520.0)
        .window_features(features)
        .initialization_script(APP_CHROME_SCRIPT)
        .on_navigation(move |url| handle_navigation_request(&app_handle_for_navigation, url))
        .on_new_window(move |url, features| {
            handle_new_window_request(&app_handle_for_new_window, url, features)
        })
        .on_document_title_changed(|window, title| {
            let title = title.trim();
            let title = if title.is_empty() { "Messenger" } else { title };
            let title = title_bar_text(title);

            if let Err(error) = window.set_title(&title) {
                eprintln!("failed to update popup window title: {error}");
            }
        })
        .build()
}
