use std::sync::atomic::{AtomicBool, Ordering};

use tauri::Manager;

#[cfg(windows)]
use webview2_com::{
    ClearBrowsingDataCompletedHandler,
    Microsoft::Web::WebView2::Win32::{
        ICoreWebView2Profile2, ICoreWebView2_13, COREWEBVIEW2_BROWSING_DATA_KINDS_CACHE_STORAGE,
        COREWEBVIEW2_BROWSING_DATA_KINDS_DISK_CACHE,
    },
};
#[cfg(windows)]
use windows_core::Interface;

const MAIN_WINDOW_LABEL: &str = "main";

static QUIT_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

pub(crate) fn quit_after_clearing_cache(app: &tauri::AppHandle) {
    if QUIT_IN_PROGRESS.swap(true, Ordering::AcqRel) {
        return;
    }

    let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) else {
        app.exit(0);
        return;
    };

    start_cache_cleanup(window, app);
}

fn start_cache_cleanup(window: tauri::WebviewWindow, app: &tauri::AppHandle) {
    let app_for_cache = app.clone();
    if window.with_webview(move |webview| {
        clear_cache_then_exit(webview, app_for_cache);
    }).is_err() {
        app.exit(0);
    }
}

#[cfg(windows)]
fn clear_cache_then_exit(webview: tauri::webview::PlatformWebview, app: tauri::AppHandle) {
    if clear_windows_cache(webview, app.clone()).is_err() {
        app.exit(0);
    }
}

#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
fn clear_cache_then_exit(_webview: tauri::webview::PlatformWebview, app: tauri::AppHandle) {
    app.exit(0);
}

#[cfg(windows)]
fn clear_windows_cache(
    webview: tauri::webview::PlatformWebview,
    app: tauri::AppHandle,
) -> windows_core::Result<()> {
    let profile = unsafe {
        webview
            .controller()
            .CoreWebView2()?
            .cast::<ICoreWebView2_13>()?
            .Profile()?
            .cast::<ICoreWebView2Profile2>()?
    };
    let handler = ClearBrowsingDataCompletedHandler::create(Box::new(move |_result| {
        app.exit(0);
        Ok(())
    }));
    let cache_types = COREWEBVIEW2_BROWSING_DATA_KINDS_DISK_CACHE
        | COREWEBVIEW2_BROWSING_DATA_KINDS_CACHE_STORAGE;

    unsafe { profile.ClearBrowsingData(cache_types, &handler) }
}

#[cfg(target_os = "linux")]
fn clear_cache_then_exit(webview: tauri::webview::PlatformWebview, app: tauri::AppHandle) {
    use webkit2gtk::{WebViewExt, WebsiteDataManagerExtManual, WebsiteDataTypes};

    let Some(manager) = webview.inner().website_data_manager() else {
        app.exit(0);
        return;
    };
    manager.clear(
        WebsiteDataTypes::DISK_CACHE | WebsiteDataTypes::DOM_CACHE,
        webkit2gtk::glib::TimeSpan::from_microseconds(0),
        None::<&webkit2gtk::gio::Cancellable>,
        move |_result| app.exit(0),
    );
}

#[cfg(target_os = "macos")]
fn clear_cache_then_exit(webview: tauri::webview::PlatformWebview, app: tauri::AppHandle) {
    use block2::RcBlock;
    use objc2_foundation::{NSDate, NSSet};
    use objc2_web_kit::{WKWebView, WKWebsiteDataTypeDiskCache};

    unsafe {
        let webview: &WKWebView = &*webview.inner().cast();
        let store = webview.configuration().websiteDataStore();
        let cache_types = NSSet::setWithObject(WKWebsiteDataTypeDiskCache);
        let beginning_of_time = NSDate::dateWithTimeIntervalSince1970(0.0);
        let handler = RcBlock::new(move || app.exit(0));

        store.removeDataOfTypes_modifiedSince_completionHandler(
            &cache_types,
            &beginning_of_time,
            &handler,
        );
    }
}
