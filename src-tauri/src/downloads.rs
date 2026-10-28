use tauri::{webview::DownloadEvent, Manager, Webview};
use tauri_plugin_opener::OpenerExt;

pub(crate) fn handle_download(webview: Webview, event: DownloadEvent<'_>) -> bool {
    match event {
        DownloadEvent::Requested { destination, .. } => {
            notify_download(&webview, "started", destination.file_name());
        }
        DownloadEvent::Finished { path, success, .. } => {
            let filename = path.as_deref().and_then(std::path::Path::file_name);
            notify_download(
                &webview,
                if success { "finished" } else { "failed" },
                filename,
            );

            if success {
                if let Some(path) = path {
                    if let Err(error) = webview.app_handle().opener().reveal_item_in_dir(path) {
                        eprintln!("failed to show downloaded file: {error}");
                    }
                }
            }
        }
        _ => {}
    }

    true
}

fn notify_download(webview: &Webview, status: &str, filename: Option<&std::ffi::OsStr>) {
    let detail = serde_json::json!({
        "status": status,
        "filename": filename.map(|name| name.to_string_lossy().into_owned()),
    });
    let script = format!(
        "window.dispatchEvent(new CustomEvent('tauri-messenger-download', {{ detail: {detail} }}));"
    );

    // Report popup downloads in the conversation window too.
    let result = if let Some(window) = webview.app_handle().get_webview_window("main") {
        window.eval(script)
    } else {
        webview.eval(script)
    };
    if let Err(error) = result {
        eprintln!("failed to report download status: {error}");
    }
}
