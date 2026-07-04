use std::sync::{Arc, Mutex};

use tauri::{image::Image, Manager, Url, WebviewWindowBuilder};
use tauri_plugin_opener::OpenerExt;

const APP_CHROME_SCRIPT: &str = r##"
(function() {
    const allowedHosts = ["messenger.com", "facebook.com"];
    const host = window.location.hostname.toLowerCase();

    if (!allowedHosts.some((allowedHost) => host === allowedHost || host.endsWith(`.${allowedHost}`))) {
        return;
    }

    const buttonId = "tauri-messenger-reload-button";
    const styleId = "tauri-messenger-responsive-style";

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

    const ensureReloadButton = () => {
        if (!document.body || document.getElementById(buttonId)) {
            return;
        }

        const button = document.createElement("button");
        button.id = buttonId;
        button.type = "button";
        button.innerHTML = `
            <svg viewBox="0 0 24 24" width="17" height="17" aria-hidden="true">
                <path d="M20 11a8 8 0 1 0-2.34 5.66" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
                <path d="M20 4v7h-7" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
        `;
        button.title = "Reload page";
        button.setAttribute("aria-label", "Reload page");

        Object.assign(button.style, {
            position: "fixed",
            top: "6px",
            right: "1rem",
            zIndex: "2147483647",
            height: "28px",
            width: "34px",
            minWidth: "34px",
            display: "inline-flex",
            alignItems: "center",
            justifyContent: "center",
            border: "1px solid rgba(255, 255, 255, 0.22)",
            borderRadius: "6px",
            color: "#ffffff",
            background: "rgba(255, 255, 255, 0.08)",
            boxShadow: "none",
            cursor: "pointer",
            letterSpacing: "0",
            padding: "0",
            userSelect: "none",
        });

        button.addEventListener("mouseenter", () => {
            button.style.background = "rgba(255, 255, 255, 0.16)";
        });

        button.addEventListener("mouseleave", () => {
            button.style.background = "rgba(255, 255, 255, 0.08)";
        });

        button.addEventListener("click", (event) => {
            event.preventDefault();
            event.stopPropagation();
            window.location.reload();
        });

        document.body.appendChild(button);
    };

    ensureResponsiveStyles();
    ensureReloadButton();
    window.addEventListener("DOMContentLoaded", ensureResponsiveStyles);
    window.addEventListener("DOMContentLoaded", ensureReloadButton);
    window.addEventListener("load", ensureResponsiveStyles);
    window.addEventListener("load", ensureReloadButton);
    setInterval(() => {
        ensureResponsiveStyles();
        ensureReloadButton();
    }, 1000);
})();
"##;

fn should_open_in_app(url: &Url) -> bool {
    if url.scheme() != "http" && url.scheme() != "https" {
        return true;
    }

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

fn open_in_system_browser(app: &tauri::AppHandle, url: &Url) {
    if let Err(error) = app.opener().open_url(url.as_str(), None::<&str>) {
        eprintln!("failed to open external URL {url}: {error}");
    }
}

fn title_bar_text(title: &str) -> String {
    format!(" {}", title)
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
            let badge_state = Arc::new(Mutex::new(None));
            let badge_state_for_title = Arc::clone(&badge_state);

            let window = WebviewWindowBuilder::from_config(app, &window_config)?
                .initialization_script(APP_CHROME_SCRIPT)
                .on_navigation(move |url| {
                    if should_open_in_app(url) {
                        return true;
                    }

                    open_in_system_browser(&app_handle_for_navigation, url);
                    false
                })
                .on_new_window(move |url, _features| {
                    if should_open_in_app(&url) {
                        if let Some(window) = app_handle_for_new_window.get_webview_window("main") {
                            if let Err(error) = window.navigate(url.clone()) {
                                eprintln!("failed to navigate main window to {url}: {error}");
                            }
                        }
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

            let window_for_events = window.clone();
            let badge_state_for_events = Arc::clone(&badge_state);
            window.on_window_event(move |event| {
                if matches!(event, tauri::WindowEvent::Focused(true)) {
                    update_taskbar_badge(&window_for_events, &badge_state_for_events, None);
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
