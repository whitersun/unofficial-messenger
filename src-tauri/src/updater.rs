use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Mutex,
};
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{Manager, WebviewWindow};
use tauri_plugin_updater::{Update, UpdaterExt};

use crate::tray::show_update_in_tray;

const UPDATE_CHECK_INTERVAL: Duration = Duration::from_secs(60 * 60);

#[derive(Default)]
pub(crate) struct UpdateState {
    available: Mutex<Option<Update>>,
    last_check_started: Mutex<Option<Instant>>,
    checking: AtomicBool,
    deferred: AtomicBool,
    installing: AtomicBool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UpdateInfo {
    available: bool,
    checking: bool,
    deferred: bool,
    current_version: String,
    version: Option<String>,
    notes: Option<String>,
}

impl UpdateInfo {
    fn unavailable(current_version: String, checking: bool) -> Self {
        Self {
            available: false,
            checking,
            deferred: false,
            current_version,
            version: None,
            notes: None,
        }
    }

    fn available(update: &Update, deferred: bool) -> Self {
        Self {
            available: true,
            checking: false,
            deferred,
            current_version: update.current_version.clone(),
            version: Some(update.version.clone()),
            notes: update.body.clone(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateProgress<'a> {
    stage: &'a str,
    downloaded: u64,
    total: Option<u64>,
}

#[tauri::command]
pub(crate) async fn check_for_update(
    app: tauri::AppHandle,
    window: WebviewWindow,
    state: tauri::State<'_, UpdateState>,
) -> Result<UpdateInfo, String> {
    if window.label() != "main" {
        return Ok(UpdateInfo::unavailable(
            app.package_info().version.to_string(),
            false,
        ));
    }

    if let Some(update) = state.available.lock().unwrap().as_ref() {
        return Ok(UpdateInfo::available(
            update,
            state.deferred.load(Ordering::Acquire),
        ));
    }

    if state
        .checking
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return Ok(UpdateInfo::unavailable(
            app.package_info().version.to_string(),
            true,
        ));
    }

    {
        let mut last_check_started = state.last_check_started.lock().unwrap();

        if last_check_started
            .as_ref()
            .is_some_and(|started| started.elapsed() < UPDATE_CHECK_INTERVAL)
        {
            state.checking.store(false, Ordering::Release);
            return Ok(UpdateInfo::unavailable(
                app.package_info().version.to_string(),
                false,
            ));
        }

        *last_check_started = Some(Instant::now());
    }

    let result = match app.updater() {
        Ok(updater) => updater.check().await.map_err(|error| error.to_string()),
        Err(error) => Err(error.to_string()),
    };
    state.checking.store(false, Ordering::Release);

    match result {
        Ok(Some(update)) => {
            let info = UpdateInfo::available(&update, false);
            *state.available.lock().unwrap() = Some(update);
            state.deferred.store(false, Ordering::Release);
            Ok(info)
        }
        Ok(None) => Ok(UpdateInfo::unavailable(
            app.package_info().version.to_string(),
            false,
        )),
        Err(error) => Err(format!("Could not check for updates: {error}")),
    }
}

#[tauri::command]
pub(crate) fn defer_update(
    app: tauri::AppHandle,
    state: tauri::State<'_, UpdateState>,
) -> Result<(), String> {
    let version = state
        .available
        .lock()
        .unwrap()
        .as_ref()
        .map(|update| update.version.clone())
        .ok_or_else(|| "No update is available".to_string())?;

    state.deferred.store(true, Ordering::Release);
    show_update_in_tray(&app, &version);
    Ok(())
}

#[tauri::command]
pub(crate) async fn install_update(
    app: tauri::AppHandle,
    state: tauri::State<'_, UpdateState>,
) -> Result<(), String> {
    if state
        .installing
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return Err("An update is already being installed".to_string());
    }

    let update = state
        .available
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| "No update is available".to_string());

    let update = match update {
        Ok(update) => update,
        Err(error) => {
            state.installing.store(false, Ordering::Release);
            return Err(error);
        }
    };

    let app_for_download = app.clone();
    let app_for_download_finish = app.clone();
    let downloaded = Arc::new(AtomicU64::new(0));
    let downloaded_for_chunk = Arc::clone(&downloaded);
    let downloaded_for_finish = Arc::clone(&downloaded);
    notify_progress(&app, "downloading", 0, None);

    let result = update
        .download_and_install(
            move |chunk_length, total| {
                let downloaded = downloaded_for_chunk
                    .fetch_add(chunk_length as u64, Ordering::AcqRel)
                    .saturating_add(chunk_length as u64);
                notify_progress(&app_for_download, "downloading", downloaded, total);
            },
            move || {
                notify_progress(
                    &app_for_download_finish,
                    "installing",
                    downloaded_for_finish.load(Ordering::Acquire),
                    None,
                );
            },
        )
        .await;

    if let Err(error) = result {
        state.installing.store(false, Ordering::Release);
        return Err(format!("Could not install the update: {error}"));
    }

    notify_progress(&app, "restarting", downloaded.load(Ordering::Acquire), None);
    app.restart();
}

fn notify_progress(app: &tauri::AppHandle, stage: &str, downloaded: u64, total: Option<u64>) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    let progress = UpdateProgress {
        stage,
        downloaded,
        total,
    };
    let Ok(payload) = serde_json::to_string(&progress) else {
        return;
    };

    if let Err(error) = window.eval(format!(
        "window.__TAURI_MESSENGER_UPDATE_PROGRESS__?.({payload})"
    )) {
        eprintln!("failed to update updater progress UI: {error}");
    }
}
