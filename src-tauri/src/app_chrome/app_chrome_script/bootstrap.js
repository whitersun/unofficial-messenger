    const allowedHosts = ["messenger.com", "facebook.com"];
    const host = window.location.hostname.toLowerCase();

    if (!allowedHosts.some((allowedHost) => host === allowedHost || host.endsWith(`.${allowedHost}`))) {
        return;
    }

    const styleId = "tauri-messenger-responsive-style";
    const overlayId = "tauri-messenger-load-overlay";
    const imageFeatureStyleId = "tauri-messenger-image-feature-style";
    const imageContextMenuId = "tauri-messenger-image-context-menu";
    const imageViewerId = "tauri-messenger-image-viewer";
    const notificationPromptId = "tauri-messenger-notification-permission";
    const notificationPromptStyleId = "tauri-messenger-notification-permission-style";
    const notificationPromptDismissedKey = "tauri-messenger-notification-permission-dismissed";
    const notificationPromptStableMs = 1500;
    const updateModalId = "tauri-messenger-update-modal";
    const updateStyleId = "tauri-messenger-update-style";
    const updateCheckIntervalMs = 60 * 60 * 1000;
    const externalNavigationParam = "__tauri_external";
    const copyImageNavigationParam = "__tauri_copy_image";
    const loadTimeoutMs = 15000;
    let notificationPromptEligibleSince = 0;
    let notificationPromptSyncTimer = 0;
    let availableUpdate = null;
    let updateCheckStarted = false;
    let updateReadyPollTimer = 0;
    let updateHourlyTimer = 0;

    const isMessengerOrFacebookHost = (host) => {
        return host === "messenger.com"
            || host.endsWith(".messenger.com")
            || host === "facebook.com"
            || host.endsWith(".facebook.com");
    };

    const authNavigationPaths = [
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
        "/help/contact"
    ];

    const isPathOrChild = (path, candidate) => {
        return path === candidate || path.startsWith(`${candidate}/`);
    };

    const isAuthNavigationUrl = (url) => {
        if (!isMessengerOrFacebookHost(url.hostname.toLowerCase())) {
            return false;
        }

        const path = url.pathname.toLowerCase().replace(/\/+$/, "") || "/";
        return authNavigationPaths.some((candidate) => isPathOrChild(path, candidate));
    };

    const isCurrentAuthSurface = () => {
        try {
            return isAuthNavigationUrl(new URL(window.location.href));
        } catch (_) {
            return false;
        }
    };

    const isLandingNavigationUrl = (url) => {
        const host = url.hostname.toLowerCase();
        const path = url.pathname.toLowerCase().replace(/\/+$/, "") || "/";

        return isMessengerOrFacebookHost(host) && path === "/";
    };

    const shouldAllowBodyOverflow = () => {
        try {
            const url = new URL(window.location.href);
            return isAuthNavigationUrl(url) || isLandingNavigationUrl(url);
        } catch (_) {
            return false;
        }
    };

    const syncRootOverflow = () => {
        const root = document.documentElement;
        const body = document.body;

        if (shouldAllowBodyOverflow()) {
            root?.classList.remove("tauri-messenger-lock-body-overflow");
            root?.classList.add("tauri-messenger-allow-body-overflow");
            root?.style.removeProperty("overflow-y");
            body?.style.removeProperty("overflow-y");
            return;
        }

        root?.classList.remove("tauri-messenger-allow-body-overflow");
        root?.classList.add("tauri-messenger-lock-body-overflow");
        root?.style.setProperty("overflow-y", "hidden", "important");
        body?.style.setProperty("overflow-y", "hidden", "important");
    };
