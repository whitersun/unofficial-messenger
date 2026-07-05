use std::sync::{Arc, Mutex};

#[cfg(windows)]
use tauri::image::Image;

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
