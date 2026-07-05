use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};
use tauri::Manager;

const SETTINGS_FILE_NAME: &str = "settings.json";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct AppSettings {
    hide_on_close: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            hide_on_close: true,
        }
    }
}

pub(crate) type SharedAppSettings = Arc<Mutex<AppSettings>>;

fn settings_path(app: &tauri::AppHandle) -> tauri::Result<PathBuf> {
    app.path()
        .app_config_dir()
        .map(|dir| dir.join(SETTINGS_FILE_NAME))
}

pub(crate) fn load_settings(app: &tauri::AppHandle) -> AppSettings {
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

pub(crate) fn is_hide_on_close_enabled(settings: &SharedAppSettings) -> bool {
    settings
        .lock()
        .map(|settings| settings.hide_on_close)
        .unwrap_or(true)
}

pub(crate) fn set_hide_on_close_enabled(
    app: &tauri::AppHandle,
    settings: &SharedAppSettings,
    enabled: bool,
) {
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
