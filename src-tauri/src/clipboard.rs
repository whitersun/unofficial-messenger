use std::borrow::Cow;
use std::time::Duration;

use arboard::{Clipboard, ImageData};
use base64::Engine;

const MAX_IMAGE_BYTES: usize = 32 * 1024 * 1024;

#[tauri::command]
pub(crate) fn copy_image_to_clipboard(
    image_data_url: Option<String>,
    image_url: Option<String>,
) -> Result<(), String> {
    let bytes = match image_data_url {
        Some(data_url) => image_bytes_from_data_url(&data_url)?,
        None => image_bytes_from_url(
            image_url
                .as_deref()
                .ok_or_else(|| "Missing image data".to_string())?,
        )?,
    };

    if bytes.len() > MAX_IMAGE_BYTES {
        return Err("Image is too large to copy".to_string());
    }

    let image = image::load_from_memory(&bytes)
        .map_err(|error| format!("Could not decode image: {error}"))?
        .to_rgba8();
    let (width, height) = image.dimensions();
    let mut clipboard =
        Clipboard::new().map_err(|error| format!("Could not open clipboard: {error}"))?;

    clipboard
        .set_image(ImageData {
            width: width as usize,
            height: height as usize,
            bytes: Cow::Owned(image.into_raw()),
        })
        .map_err(|error| format!("Could not copy image: {error}"))
}

fn image_bytes_from_data_url(data_url: &str) -> Result<Vec<u8>, String> {
    let (metadata, payload) = data_url
        .split_once(',')
        .ok_or_else(|| "Invalid image data".to_string())?;
    let metadata = metadata.to_ascii_lowercase();

    if !metadata.starts_with("data:image/") || !metadata.contains(";base64") {
        return Err("Invalid image data".to_string());
    }

    let bytes = base64::engine::general_purpose::STANDARD
        .decode(payload)
        .map_err(|error| format!("Could not read image data: {error}"))?;

    if bytes.is_empty() {
        return Err("Image data is empty".to_string());
    }

    Ok(bytes)
}

fn image_bytes_from_url(image_url: &str) -> Result<Vec<u8>, String> {
    let url = tauri::Url::parse(image_url).map_err(|_| "Invalid image URL".to_string())?;

    if !is_allowed_image_url(&url) {
        return Err("Unsupported image URL".to_string());
    }

    let response = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(20))
        .user_agent("unofficial-messenger-next")
        .build()
        .map_err(|error| format!("Could not prepare image download: {error}"))?
        .get(url.as_str())
        .send()
        .map_err(|error| format!("Could not download image: {error}"))?;

    if !response.status().is_success() {
        return Err(format!("Could not download image: HTTP {}", response.status()));
    }

    let bytes = response
        .bytes()
        .map_err(|error| format!("Could not read image: {error}"))?;

    if bytes.is_empty() {
        return Err("Downloaded image is empty".to_string());
    }

    Ok(bytes.to_vec())
}

fn is_allowed_image_url(url: &tauri::Url) -> bool {
    if url.scheme() != "https" {
        return false;
    }

    let Some(host) = url.host_str().map(str::to_ascii_lowercase) else {
        return false;
    };
    let path = url.path().to_ascii_lowercase();

    is_image_asset_host(&host)
        || (is_messenger_or_facebook_host(&host)
            && (has_image_extension(&path)
                || path_contains_media_route(&path)
                || path.contains("/ajax/mercury/attachments/")))
}

fn is_image_asset_host(host: &str) -> bool {
    host.ends_with(".fbcdn.net")
        || host.contains(".xx.fbcdn.net")
        || host.starts_with("scontent.")
        || host.starts_with("lookaside.")
}

fn is_messenger_or_facebook_host(host: &str) -> bool {
    host == "messenger.com"
        || host.ends_with(".messenger.com")
        || host == "facebook.com"
        || host.ends_with(".facebook.com")
}

fn has_image_extension(path: &str) -> bool {
    [".avif", ".gif", ".jpg", ".jpeg", ".png", ".webp"]
        .iter()
        .any(|extension| path.ends_with(extension))
}

fn path_contains_media_route(path: &str) -> bool {
    ["/photo", "/photos", "/media", "/image", "/attachment", "/attachments"]
        .iter()
        .any(|route| {
            path.contains(&format!("{route}/")) || path.contains(&format!("{route}.php"))
        })
}
