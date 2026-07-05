use tauri::Url;
use tauri_plugin_opener::OpenerExt;

const EXTERNAL_NAVIGATION_PARAM: &str = "__tauri_external";

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

pub(crate) fn should_open_in_app(url: &Url) -> bool {
    if url.scheme() != "http" && url.scheme() != "https" {
        return false;
    }

    is_messenger_or_facebook_host(url)
}

pub(crate) fn should_keep_in_webview(url: &Url) -> bool {
    matches!(url.scheme(), "about" | "blob")
}

pub(crate) fn is_auth_navigation(url: &Url) -> bool {
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

pub(crate) fn external_url_from_marked_navigation(url: &Url) -> Option<Url> {
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

pub(crate) fn open_in_system_browser(app: &tauri::AppHandle, url: &Url) {
    if should_keep_in_webview(url) {
        return;
    }

    if let Err(error) = app.opener().open_url(url.as_str(), None::<&str>) {
        eprintln!("failed to open external URL {url}: {error}");
    }
}
