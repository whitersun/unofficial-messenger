#[cfg(target_os = "linux")]
pub(crate) fn configure_call_media(window: &tauri::WebviewWindow) {
    let label = window.label().to_owned();

    if let Err(error) = window.with_webview(configure_linux_call_media) {
        eprintln!("failed to configure call media for webview {label}: {error}");
    }
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn configure_call_media(_window: &tauri::WebviewWindow) {}

#[cfg(target_os = "linux")]
fn configure_linux_call_media(webview: tauri::webview::PlatformWebview) {
    use webkit2gtk::{
        glib::prelude::Cast, DeviceInfoPermissionRequest, PermissionRequestExt, SettingsExt,
        UserMediaPermissionRequest, WebViewExt,
    };

    let webview = webview.inner();

    if let Some(settings) = webview.settings() {
        settings.set_enable_media_stream(true);
        settings.set_enable_webrtc(true);
    }

    webview.connect_permission_request(|webview, request| {
        let is_call_media_request = request
            .dynamic_cast_ref::<UserMediaPermissionRequest>()
            .is_some()
            || request
                .dynamic_cast_ref::<DeviceInfoPermissionRequest>()
                .is_some();

        if !is_call_media_request {
            return false;
        }

        let is_trusted_origin = webview
            .uri()
            .as_deref()
            .and_then(|uri| tauri::Url::parse(uri).ok())
            .is_some_and(|url| {
                url.scheme() == "https" && crate::navigation::should_open_in_app(&url)
            });

        if is_trusted_origin {
            request.allow();
        } else {
            request.deny();
        }

        true
    });
}
