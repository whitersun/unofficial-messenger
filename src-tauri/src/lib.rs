use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};
use tauri::{
    image::Image,
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, Url, WebviewWindowBuilder,
};
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_updater::{Update, UpdaterExt};

const APP_CHROME_SCRIPT: &str = r##"
(function() {
    const allowedHosts = ["messenger.com", "facebook.com"];
    const host = window.location.hostname.toLowerCase();

    if (!allowedHosts.some((allowedHost) => host === allowedHost || host.endsWith(`.${allowedHost}`))) {
        return;
    }

    const styleId = "tauri-messenger-responsive-style";
    const overlayId = "tauri-messenger-load-overlay";
    const externalNavigationParam = "__tauri_external";
    const loadTimeoutMs = 15000;

    const ensureResponsiveStyles = () => {
        const parent = document.head || document.documentElement;

        if (!parent || document.getElementById(styleId)) {
            return;
        }

        const viewport = document.querySelector('meta[name="viewport"]') || document.createElement("meta");
        viewport.name = "viewport";
        viewport.content = "width=device-width, initial-scale=1, viewport-fit=cover";

        if (!viewport.parentNode) {
            parent.appendChild(viewport);
        }

        const style = document.createElement("style");
        style.id = styleId;
        style.textContent = `
            html, body {
                width: 100% !important;
                max-width: 100% !important;
                overflow-x: hidden !important;
            }

            body, #globalContainer, #pagelet_bluebar, #content, #contentArea, .fb_content, ._li, ._95k9, ._8esf, ._8esj, [role="main"] {
                min-width: 0 !important;
                max-width: 100vw !important;
                box-sizing: border-box !important;
            }

            img, video, canvas, iframe, table, form {
                max-width: 100% !important;
            }

            @media (max-width: 719.98px) {
                #globalContainer, #content, #contentArea, .fb_content, ._li, ._95k9, ._8esf, ._8esj {
                    width: 100% !important;
                    margin-left: 0 !important;
                    margin-right: 0 !important;
                }
            }
        `;
        parent.appendChild(style);
    };

    const hasPageContent = () => {
        const body = document.body;

        if (!body) {
            return false;
        }

        return body.childElementCount > 0 && body.getBoundingClientRect().height > 100;
    };

    const removeLoadOverlay = () => {
        document.getElementById(overlayId)?.remove();
    };

    const showLoadOverlay = () => {
        const parent = document.body || document.documentElement;

        if (!parent || document.getElementById(overlayId) || hasPageContent()) {
            return;
        }

        const overlay = document.createElement("div");
        overlay.id = overlayId;
        overlay.innerHTML = `
            <div class="tauri-messenger-load-card">
                <div class="tauri-messenger-load-title">Messenger is taking longer than expected.</div>
                <button type="button">Reload</button>
            </div>
        `;

        const style = document.createElement("style");
        style.textContent = `
            #${overlayId} {
                position: fixed;
                inset: 0;
                z-index: 2147483647;
                display: grid;
                place-items: center;
                background: rgba(10, 10, 14, 0.2);
                font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
            }

            #${overlayId} .tauri-messenger-load-card {
                display: flex;
                align-items: center;
                gap: 12px;
                padding: 12px;
                color: #f5f7fb;
                background: #1f2028;
                border: 1px solid rgba(255, 255, 255, 0.12);
                border-radius: 8px;
                box-shadow: 0 12px 40px rgba(0, 0, 0, 0.32);
            }

            #${overlayId} .tauri-messenger-load-title {
                font-size: 13px;
                line-height: 18px;
            }

            #${overlayId} button {
                min-height: 32px;
                padding: 0 12px;
                color: #ffffff;
                font: inherit;
                font-size: 13px;
                font-weight: 600;
                background: #0866ff;
                border: 0;
                border-radius: 6px;
                cursor: pointer;
            }
        `;

        overlay.appendChild(style);
        overlay.querySelector("button")?.addEventListener("click", () => {
            window.location.reload();
        });
        parent.appendChild(overlay);
    };

    window.addEventListener("keydown", (event) => {
        if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "r") {
            event.preventDefault();
            window.location.reload();
        }
    }, true);

    const shellNavigationSelector = [
        "nav",
        '[role="navigation"]',
        '[role="search"]',
        '[aria-label="Chats" i]',
        '[aria-label*="chat list" i]',
        '[aria-label*="conversation list" i]',
        '[aria-label*="cuộc trò chuyện" i]',
        '[aria-label*="danh sách đoạn chat" i]'
    ].join(",");

    const isMessengerChatRoute = (url) => {
        const host = url.hostname.toLowerCase();
        const path = url.pathname.replace(/\/+$/, "");

        if (host !== "messenger.com" && !host.endsWith(".messenger.com")) {
            return false;
        }

        return /^\/(?:u\/\d+\/)?(?:e2ee\/)?t(?:\/|$)/.test(path);
    };

    const isShellNavigationLink = (anchor, url) => {
        if (anchor.closest(shellNavigationSelector)) {
            return true;
        }

        return isMessengerChatRoute(url);
    };

    const isLikelyConversationArea = (anchor) => {
        const rect = anchor.getBoundingClientRect();
        const conversationLeft = Math.min(560, window.innerWidth * 0.34);

        if (rect.width <= 0 || rect.height <= 0 || rect.left < conversationLeft) {
            return false;
        }

        return Boolean(anchor.closest('[role="main"], [aria-label*="messages" i], [aria-label*="tin nhắn" i]'));
    };

    const looksLikeSharedUrl = (anchor, url) => {
        const host = url.hostname.toLowerCase();
        const text = anchor.textContent?.trim() || "";

        if (host && host !== "messenger.com" && !host.endsWith(".messenger.com")) {
            return true;
        }

        return /^(https?:\/\/|www\.)/i.test(text);
    };

    const shouldOpenClickExternally = (anchor, url) => {
        if (!["http:", "https:"].includes(url.protocol) || isShellNavigationLink(anchor, url)) {
            return false;
        }

        return isLikelyConversationArea(anchor) || looksLikeSharedUrl(anchor, url);
    };

    window.addEventListener("click", (event) => {
        if (event.defaultPrevented || event.button !== 0) {
            return;
        }

        const anchor = event.composedPath()
            .find((node) => node instanceof HTMLAnchorElement && node.href);

        if (!anchor) {
            return;
        }

        const url = new URL(anchor.href, window.location.href);

        if (!shouldOpenClickExternally(anchor, url)) {
            return;
        }

        url.searchParams.set(externalNavigationParam, "1");
        event.preventDefault();
        event.stopPropagation();
        window.location.href = url.href;
    }, true);

    ensureResponsiveStyles();
    window.addEventListener("DOMContentLoaded", ensureResponsiveStyles);
    window.addEventListener("load", () => {
        ensureResponsiveStyles();
        window.setTimeout(() => {
            if (hasPageContent()) {
                removeLoadOverlay();
            }
        }, 500);
    });
    window.setTimeout(showLoadOverlay, loadTimeoutMs);
    setInterval(() => {
        ensureResponsiveStyles();
    }, 1000);
})();
"##;

const MAIN_WINDOW_LABEL: &str = "main";
const TRAY_MENU_SHOW: &str = "tray-show";
const TRAY_MENU_HIDE: &str = "tray-hide";
const TRAY_MENU_RELOAD: &str = "tray-reload";
const TRAY_MENU_UPDATE: &str = "tray-update";
const TRAY_MENU_START_WITH_WINDOWS: &str = "tray-start-with-windows";
const TRAY_MENU_HIDE_ON_CLOSE: &str = "tray-hide-on-close";
const TRAY_MENU_QUIT: &str = "tray-quit";
const EXTERNAL_NAVIGATION_PARAM: &str = "__tauri_external";
const SETTINGS_FILE_NAME: &str = "settings.json";
const STARTUP_REGISTRY_VALUE: &str = "UnofficialMessenger";

#[derive(Debug, Clone, Deserialize, Serialize)]
struct AppSettings {
    hide_on_close: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            hide_on_close: true,
        }
    }
}

type SharedAppSettings = Arc<Mutex<AppSettings>>;
type SharedPendingUpdate = Arc<Mutex<Option<Update>>>;

fn is_messenger_or_facebook_host(url: &Url) -> bool {
    url.host_str()
        .map(|host| {
            let host = host.to_ascii_lowercase();
            host == "messenger.com"
                || host.ends_with(".messenger.com")
                || host == "facebook.com"
                || host.ends_with(".facebook.com")
        })
        .unwrap_or(false)
}

fn should_open_in_app(url: &Url) -> bool {
    if url.scheme() != "http" && url.scheme() != "https" {
        return false;
    }

    is_messenger_or_facebook_host(url)
}

fn is_auth_navigation(url: &Url) -> bool {
    if !should_open_in_app(url) {
        return false;
    }

    let path = url.path().to_ascii_lowercase();
    let auth_paths = [
        "/login",
        "/login.php",
        "/checkpoint",
        "/recover",
        "/two_factor",
        "/confirm",
        "/confirmemail",
        "/security",
        "/auth",
        "/dialog/oauth",
        "/privacy/consent",
        "/cookie/consent",
        "/help/contact",
    ];

    auth_paths.iter().any(|auth_path| {
        path == *auth_path
            || path
                .strip_prefix(auth_path)
                .is_some_and(|suffix| suffix.starts_with('/'))
    })
}

fn external_url_from_marked_navigation(url: &Url) -> Option<Url> {
    let has_marker = url
        .query_pairs()
        .any(|(key, value)| key == EXTERNAL_NAVIGATION_PARAM && value == "1");

    if !has_marker {
        return None;
    }

    let query_pairs = url
        .query_pairs()
        .filter(|(key, _)| key != EXTERNAL_NAVIGATION_PARAM)
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect::<Vec<_>>();

    let mut external_url = url.clone();
    external_url.set_query(None);

    if !query_pairs.is_empty() {
        external_url
            .query_pairs_mut()
            .extend_pairs(query_pairs.iter().map(|(key, value)| (&**key, &**value)));
    }

    Some(external_url)
}

fn open_in_system_browser(app: &tauri::AppHandle, url: &Url) {
    if let Err(error) = app.opener().open_url(url.as_str(), None::<&str>) {
        eprintln!("failed to open external URL {url}: {error}");
    }
}

fn navigate_main_window(app: &tauri::AppHandle, url: Url) {
    with_main_window(app, |window| {
        if let Err(error) = window.navigate(url.clone()) {
            eprintln!("failed to navigate main window to {url}: {error}");
        }
    });
}

fn title_bar_text(title: &str) -> String {
    format!(" {}", title)
}

fn with_main_window(app: &tauri::AppHandle, action: impl FnOnce(tauri::WebviewWindow)) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        action(window);
    }
}

fn show_main_window(app: &tauri::AppHandle) {
    with_main_window(app, |window| {
        if let Err(error) = window.show() {
            eprintln!("failed to show main window: {error}");
        }

        if let Err(error) = window.set_focus() {
            eprintln!("failed to focus main window: {error}");
        }
    });
}

fn hide_main_window(app: &tauri::AppHandle) {
    with_main_window(app, |window| {
        if let Err(error) = window.hide() {
            eprintln!("failed to hide main window: {error}");
        }
    });
}

fn reload_main_window(app: &tauri::AppHandle) {
    with_main_window(app, |window| {
        if let Err(error) = window.reload() {
            eprintln!("failed to reload main window: {error}");
        }
    });
}

fn check_for_updates(
    app: tauri::AppHandle,
    update_menu_item: MenuItem<tauri::Wry>,
    pending_update: SharedPendingUpdate,
    manual: bool,
) {
    tauri::async_runtime::spawn(async move {
        let _ = update_menu_item.set_enabled(false);
        let _ = update_menu_item.set_text("Checking for Updates...");

        let check_result = match app.updater() {
            Ok(updater) => updater.check().await,
            Err(error) => {
                eprintln!("failed to initialize updater: {error}");
                let _ = update_menu_item.set_text("Update Check Failed");
                let _ = update_menu_item.set_enabled(true);
                return;
            }
        };

        match check_result {
            Ok(Some(update)) => {
                let version = update.version.clone();

                match pending_update.lock() {
                    Ok(mut pending_update) => {
                        pending_update.replace(update);
                    }
                    Err(error) => {
                        eprintln!("failed to lock pending update state: {error}");
                    }
                }

                let _ = update_menu_item.set_text(format!("Update Available: v{version}"));
                let _ = update_menu_item.set_enabled(true);
            }
            Ok(None) => {
                if let Ok(mut pending_update) = pending_update.lock() {
                    pending_update.take();
                }

                let _ = update_menu_item.set_text(if manual {
                    "No Updates Found"
                } else {
                    "Check for Updates"
                });
                let _ = update_menu_item.set_enabled(true);
            }
            Err(error) => {
                eprintln!("failed to check for updates: {error}");
                let _ = update_menu_item.set_text("Update Check Failed");
                let _ = update_menu_item.set_enabled(true);
            }
        }
    });
}

fn install_pending_update(
    update_menu_item: MenuItem<tauri::Wry>,
    pending_update: SharedPendingUpdate,
) {
    let update = match pending_update.lock() {
        Ok(mut pending_update) => pending_update.take(),
        Err(error) => {
            eprintln!("failed to lock pending update state: {error}");
            None
        }
    };

    let Some(update) = update else {
        let _ = update_menu_item.set_text("Check for Updates");
        let _ = update_menu_item.set_enabled(true);
        return;
    };

    tauri::async_runtime::spawn(async move {
        let _ = update_menu_item.set_enabled(false);
        let _ = update_menu_item.set_text("Downloading Update...");

        let mut downloaded = 0usize;
        let update_result = update
            .download_and_install(
                |chunk_length, content_length| {
                    downloaded += chunk_length;

                    if let Some(content_length) = content_length {
                        let percent = ((downloaded as f64 / content_length as f64) * 100.0)
                            .round()
                            .clamp(0.0, 100.0) as u8;
                        let _ =
                            update_menu_item.set_text(format!("Downloading Update... {percent}%"));
                    }
                },
                || {
                    let _ = update_menu_item.set_text("Installing Update...");
                },
            )
            .await;

        if let Err(error) = update_result {
            eprintln!("failed to install update: {error}");
            let _ = update_menu_item.set_text("Update Install Failed");
            let _ = update_menu_item.set_enabled(true);
        }
    });
}

fn settings_path(app: &tauri::AppHandle) -> tauri::Result<PathBuf> {
    app.path()
        .app_config_dir()
        .map(|dir| dir.join(SETTINGS_FILE_NAME))
}

fn load_settings(app: &tauri::AppHandle) -> AppSettings {
    let Ok(path) = settings_path(app) else {
        return AppSettings::default();
    };

    fs::read_to_string(path)
        .ok()
        .and_then(|settings| serde_json::from_str(&settings).ok())
        .unwrap_or_default()
}

fn save_settings(app: &tauri::AppHandle, settings: &AppSettings) {
    let Ok(path) = settings_path(app) else {
        return;
    };

    if let Some(parent) = path.parent() {
        if let Err(error) = fs::create_dir_all(parent) {
            eprintln!("failed to create settings directory: {error}");
            return;
        }
    }

    match serde_json::to_string_pretty(settings) {
        Ok(settings) => {
            if let Err(error) = fs::write(path, settings) {
                eprintln!("failed to write settings: {error}");
            }
        }
        Err(error) => {
            eprintln!("failed to serialize settings: {error}");
        }
    }
}

#[cfg(windows)]
fn is_start_with_windows_enabled() -> bool {
    Command::new("reg")
        .args([
            "query",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
            "/v",
            STARTUP_REGISTRY_VALUE,
        ])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg(not(windows))]
fn is_start_with_windows_enabled() -> bool {
    false
}

#[cfg(windows)]
fn set_start_with_windows(enabled: bool) -> std::io::Result<()> {
    let status = if enabled {
        let executable = std::env::current_exe()?;
        let executable = format!("\"{}\"", executable.display());

        Command::new("reg")
            .arg("add")
            .arg(r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run")
            .arg("/v")
            .arg(STARTUP_REGISTRY_VALUE)
            .arg("/t")
            .arg("REG_SZ")
            .arg("/d")
            .arg(executable)
            .arg("/f")
            .status()?
    } else {
        Command::new("reg")
            .arg("delete")
            .arg(r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run")
            .arg("/v")
            .arg(STARTUP_REGISTRY_VALUE)
            .arg("/f")
            .status()?
    };

    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other(
            "failed to update Windows startup registry entry",
        ))
    }
}

#[cfg(not(windows))]
fn set_start_with_windows(_enabled: bool) -> std::io::Result<()> {
    Err(std::io::Error::other(
        "start with Windows is only supported on Windows",
    ))
}

fn is_hide_on_close_enabled(settings: &SharedAppSettings) -> bool {
    settings
        .lock()
        .map(|settings| settings.hide_on_close)
        .unwrap_or(true)
}

fn set_hide_on_close_enabled(app: &tauri::AppHandle, settings: &SharedAppSettings, enabled: bool) {
    let snapshot = match settings.lock() {
        Ok(mut settings) => {
            settings.hide_on_close = enabled;
            settings.clone()
        }
        Err(error) => {
            eprintln!("failed to lock app settings: {error}");
            return;
        }
    };

    save_settings(app, &snapshot);
}

fn create_tray(app: &tauri::App, settings: SharedAppSettings) -> tauri::Result<()> {
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
        "Check for Updates",
        true,
        None::<&str>,
    )?;
    let start_with_windows = CheckMenuItem::with_id(
        app,
        TRAY_MENU_START_WITH_WINDOWS,
        "Start with Windows",
        cfg!(windows),
        is_start_with_windows_enabled(),
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
            &update,
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
    let hide_on_close_for_menu = hide_on_close.clone();
    let pending_update = Arc::new(Mutex::new(None));
    let pending_update_for_menu = Arc::clone(&pending_update);
    let update_for_menu = update.clone();
    let settings_for_menu = Arc::clone(&settings);

    TrayIconBuilder::with_id("messenger-tray")
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
                let has_pending_update = pending_update_for_menu
                    .lock()
                    .map(|pending_update| pending_update.is_some())
                    .unwrap_or(false);

                if has_pending_update {
                    install_pending_update(
                        update_for_menu.clone(),
                        Arc::clone(&pending_update_for_menu),
                    );
                } else {
                    check_for_updates(
                        app.clone(),
                        update_for_menu.clone(),
                        Arc::clone(&pending_update_for_menu),
                        true,
                    );
                }
            }
            TRAY_MENU_START_WITH_WINDOWS => {
                let enabled = start_with_windows_for_menu.is_checked().unwrap_or(false);

                if let Err(error) = set_start_with_windows(enabled) {
                    eprintln!("failed to toggle start with Windows: {error}");
                    let _ = start_with_windows_for_menu.set_checked(!enabled);
                }
            }
            TRAY_MENU_HIDE_ON_CLOSE => {
                let enabled = hide_on_close_for_menu.is_checked().unwrap_or(true);
                set_hide_on_close_enabled(app, &settings_for_menu, enabled);
            }
            TRAY_MENU_QUIT => app.exit(0),
            _ => {}
        })
        .build(app)?;

    #[cfg(not(debug_assertions))]
    check_for_updates(
        app.handle().clone(),
        update,
        Arc::clone(&pending_update),
        false,
    );

    Ok(())
}

const ENABLE_TASKBAR_BADGE: bool = true;

fn unread_count_from_title(title: &str) -> Option<i64> {
    let start = title.find('(')?;
    let end = title[start + 1..].find(')')? + start + 1;
    let value = title[start + 1..end]
        .trim()
        .trim_end_matches('+')
        .parse::<i64>()
        .ok()?;

    (value > 0).then_some(value)
}

fn update_taskbar_badge(
    window: &tauri::WebviewWindow,
    badge_state: &Arc<Mutex<Option<i64>>>,
    unread_count: Option<i64>,
) {
    if !ENABLE_TASKBAR_BADGE {
        let _ = window.set_badge_count(None);
        #[cfg(windows)]
        let _ = window.set_overlay_icon(None);
        return;
    }

    let mut current_badge = match badge_state.lock() {
        Ok(current_badge) => current_badge,
        Err(error) => {
            eprintln!("failed to lock badge state: {error}");
            return;
        }
    };

    if *current_badge == unread_count {
        return;
    }

    *current_badge = unread_count;
    drop(current_badge);

    if let Err(error) = window.set_badge_count(unread_count) {
        eprintln!("failed to update badge count: {error}");
    }

    #[cfg(windows)]
    if let Err(error) = window.set_overlay_icon(unread_count.map(overlay_icon_for_count)) {
        eprintln!("failed to update overlay icon: {error}");
    }
}

#[cfg(windows)]
const BADGE_BACKGROUND_RGBA: [u8; 4] = [41, 43, 51, 245];
#[cfg(windows)]
const BADGE_BORDER_RGBA: [u8; 4] = [41, 43, 51, 245];
#[cfg(windows)]
const BADGE_TEXT_RGBA: [u8; 4] = [236, 238, 245, 255];
#[cfg(windows)]
const BADGE_RADIUS: f32 = 15.0;
#[cfg(windows)]
const BADGE_BORDER_RADIUS: f32 = 16.0;
#[cfg(windows)]
const BADGE_SINGLE_DIGIT_SCALE: i32 = 4;
#[cfg(windows)]
const BADGE_MULTI_DIGIT_SCALE: i32 = 3;

#[cfg(windows)]
fn overlay_icon_for_count(count: i64) -> Image<'static> {
    const SIZE: u32 = 32;
    const SIZE_USIZE: usize = SIZE as usize;

    let count = count.clamp(1, 99);
    let label = count.to_string();
    let scale = if label.len() == 1 {
        BADGE_SINGLE_DIGIT_SCALE
    } else {
        BADGE_MULTI_DIGIT_SCALE
    };
    let digit_width = 3 * scale;
    let digit_height = 5 * scale;
    let gap = scale;
    let text_width = label.len() as i32 * digit_width + (label.len() as i32 - 1) * gap;
    let start_x = (SIZE as i32 - text_width) / 2;
    let start_y = (SIZE as i32 - digit_height) / 2;

    let mut rgba = vec![0; SIZE_USIZE * SIZE_USIZE * 4];

    for y in 0..SIZE_USIZE {
        for x in 0..SIZE_USIZE {
            let dx = x as f32 + 0.5 - SIZE as f32 / 2.0;
            let dy = y as f32 + 0.5 - SIZE as f32 / 2.0;
            let distance = (dx * dx + dy * dy).sqrt();
            let index = (y * SIZE_USIZE + x) * 4;

            if distance <= BADGE_RADIUS {
                rgba[index..index + 4].copy_from_slice(&BADGE_BACKGROUND_RGBA);
            }

            if distance > BADGE_RADIUS && distance <= BADGE_BORDER_RADIUS {
                rgba[index..index + 4].copy_from_slice(&BADGE_BORDER_RGBA);
            }
        }
    }

    for (digit_index, digit) in label.chars().enumerate() {
        let pattern = digit_pattern(digit);
        let offset_x = start_x + digit_index as i32 * (digit_width + gap);

        for (row_index, row) in pattern.iter().enumerate() {
            for (column_index, is_on) in row.iter().enumerate() {
                if !is_on {
                    continue;
                }

                for y in 0..scale {
                    for x in 0..scale {
                        let pixel_x = offset_x + column_index as i32 * scale + x;
                        let pixel_y = start_y + row_index as i32 * scale + y;

                        if pixel_x < 0
                            || pixel_y < 0
                            || pixel_x >= SIZE as i32
                            || pixel_y >= SIZE as i32
                        {
                            continue;
                        }

                        let index = (pixel_y as usize * SIZE_USIZE + pixel_x as usize) * 4;
                        rgba[index..index + 4].copy_from_slice(&BADGE_TEXT_RGBA);
                    }
                }
            }
        }
    }

    Image::new_owned(rgba, SIZE, SIZE)
}

#[cfg(windows)]
fn digit_pattern(digit: char) -> [[bool; 3]; 5] {
    match digit {
        '0' => [
            [true, true, true],
            [true, false, true],
            [true, false, true],
            [true, false, true],
            [true, true, true],
        ],
        '1' => [
            [false, true, false],
            [true, true, false],
            [false, true, false],
            [false, true, false],
            [true, true, true],
        ],
        '2' => [
            [true, true, true],
            [false, false, true],
            [true, true, true],
            [true, false, false],
            [true, true, true],
        ],
        '3' => [
            [true, true, true],
            [false, false, true],
            [true, true, true],
            [false, false, true],
            [true, true, true],
        ],
        '4' => [
            [true, false, true],
            [true, false, true],
            [true, true, true],
            [false, false, true],
            [false, false, true],
        ],
        '5' => [
            [true, true, true],
            [true, false, false],
            [true, true, true],
            [false, false, true],
            [true, true, true],
        ],
        '6' => [
            [true, true, true],
            [true, false, false],
            [true, true, true],
            [true, false, true],
            [true, true, true],
        ],
        '7' => [
            [true, true, true],
            [false, false, true],
            [false, true, false],
            [false, true, false],
            [false, true, false],
        ],
        '8' => [
            [true, true, true],
            [true, false, true],
            [true, true, true],
            [true, false, true],
            [true, true, true],
        ],
        '9' => [
            [true, true, true],
            [true, false, true],
            [true, true, true],
            [false, false, true],
            [true, true, true],
        ],
        _ => [[false; 3]; 5],
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let window_config = app
                .config()
                .app
                .windows
                .first()
                .expect("missing main window config")
                .clone();
            let app_handle = app.handle().clone();
            let app_handle_for_navigation = app_handle.clone();
            let app_handle_for_new_window = app_handle.clone();
            let settings = Arc::new(Mutex::new(load_settings(&app_handle)));
            let badge_state = Arc::new(Mutex::new(None));
            let badge_state_for_title = Arc::clone(&badge_state);

            let window = WebviewWindowBuilder::from_config(app, &window_config)?
                .initialization_script(APP_CHROME_SCRIPT)
                .on_navigation(move |url| {
                    if let Some(external_url) = external_url_from_marked_navigation(url) {
                        open_in_system_browser(&app_handle_for_navigation, &external_url);
                        return false;
                    }

                    if should_open_in_app(url) {
                        return true;
                    }

                    open_in_system_browser(&app_handle_for_navigation, url);
                    false
                })
                .on_new_window(move |url, _features| {
                    if let Some(external_url) = external_url_from_marked_navigation(&url) {
                        open_in_system_browser(&app_handle_for_new_window, &external_url);
                    } else if is_auth_navigation(&url) {
                        navigate_main_window(&app_handle_for_new_window, url);
                    } else {
                        open_in_system_browser(&app_handle_for_new_window, &url);
                    }

                    tauri::webview::NewWindowResponse::Deny
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
