use std::sync::{Arc, Mutex};

use tauri::{image::Image, Manager};

use crate::tray::TRAY_ID;

const ENABLE_TASKBAR_BADGE: bool = true;

pub(crate) fn unread_count_from_title(title: &str) -> Option<i64> {
    let start = title.find('(')?;
    let end = title[start + 1..].find(')')? + start + 1;
    let value = title[start + 1..end]
        .trim()
        .trim_end_matches('+')
        .parse::<i64>()
        .ok()?;

    (value > 0).then_some(value)
}

pub(crate) fn update_taskbar_badge(
    window: &tauri::WebviewWindow,
    badge_state: &Arc<Mutex<Option<i64>>>,
    unread_count: Option<i64>,
) {
    apply_taskbar_badge(window, badge_state, BadgeUpdate::FromTitle(unread_count));
}

pub(crate) fn clear_taskbar_badge(
    window: &tauri::WebviewWindow,
    badge_state: &Arc<Mutex<Option<i64>>>,
) {
    apply_taskbar_badge(window, badge_state, BadgeUpdate::Clear);
}

enum BadgeUpdate {
    FromTitle(Option<i64>),
    Clear,
}

fn apply_taskbar_badge(
    window: &tauri::WebviewWindow,
    badge_state: &Arc<Mutex<Option<i64>>>,
    update: BadgeUpdate,
) {
    if !ENABLE_TASKBAR_BADGE {
        #[cfg(not(windows))]
        let _ = window.set_badge_count(None);
        #[cfg(windows)]
        let _ = window.set_overlay_icon(None);
        update_tray_unread_dot(window, None);
        return;
    }

    let unread_count = match update {
        BadgeUpdate::FromTitle(None) => return,
        BadgeUpdate::FromTitle(unread_count) => unread_count,
        BadgeUpdate::Clear => None,
    };

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

    #[cfg(not(windows))]
    if let Err(error) = window.set_badge_count(unread_count) {
        eprintln!("failed to update badge count: {error}");
    }

    #[cfg(windows)]
    if let Err(error) = window.set_overlay_icon(unread_count.map(overlay_icon_for_count)) {
        eprintln!("failed to update overlay icon: {error}");
    }

    update_tray_unread_dot(window, unread_count);
}

fn update_tray_unread_dot(window: &tauri::WebviewWindow, unread_count: Option<i64>) {
    let app = window.app_handle();
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return;
    };
    let Some(icon) = app.default_window_icon().cloned() else {
        return;
    };
    let next_icon = unread_count
        .filter(|count| *count > 0)
        .map(|_| tray_icon_with_unread_dot(&icon))
        .unwrap_or(icon);

    if let Err(error) = tray.set_icon(Some(next_icon)) {
        eprintln!("failed to update tray unread dot: {error}");
    }
}

fn tray_icon_with_unread_dot(icon: &Image<'_>) -> Image<'static> {
    let width = icon.width();
    let height = icon.height();
    let min_side = width.min(height) as f32;
    let dot_radius = (min_side * 0.16).max(2.5);
    let edge_padding = (min_side * 0.08).max(1.0);
    let center_x = width as f32 - dot_radius - edge_padding;
    let center_y = dot_radius + edge_padding;
    let mut rgba = icon.rgba().to_vec();

    draw_circle(
        &mut rgba,
        width,
        height,
        center_x,
        center_y,
        dot_radius,
        [255, 59, 48, 255],
    );

    Image::new_owned(rgba, width, height)
}

fn draw_circle(
    rgba: &mut [u8],
    width: u32,
    height: u32,
    center_x: f32,
    center_y: f32,
    radius: f32,
    color: [u8; 4],
) {
    let left = (center_x - radius).floor().max(0.0) as u32;
    let top = (center_y - radius).floor().max(0.0) as u32;
    let right = (center_x + radius).ceil().min(width as f32 - 1.0) as u32;
    let bottom = (center_y + radius).ceil().min(height as f32 - 1.0) as u32;

    for y in top..=bottom {
        for x in left..=right {
            let dx = x as f32 + 0.5 - center_x;
            let dy = y as f32 + 0.5 - center_y;

            if (dx * dx + dy * dy).sqrt() <= radius {
                let index = (y as usize * width as usize + x as usize) * 4;
                rgba[index..index + 4].copy_from_slice(&color);
            }
        }
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
const BADGE_SINGLE_DIGIT_SCALE: i32 = 3;
#[cfg(windows)]
const BADGE_MULTI_DIGIT_SCALE: i32 = 2;

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
            [false, true, false],
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
