pub(crate) const APP_CHROME_SCRIPT: &str = r##"
(function() {
    const allowedHosts = ["messenger.com", "facebook.com"];
    const host = window.location.hostname.toLowerCase();

    if (!allowedHosts.some((allowedHost) => host === allowedHost || host.endsWith(`.${allowedHost}`))) {
        return;
    }

    const styleId = "tauri-messenger-responsive-style";
    const overlayId = "tauri-messenger-load-overlay";
    const externalNavigationParam = "__tauri_external";
    const loadTimeoutMs = 15000;

    const forceRootOverflow = () => {
        document.documentElement?.style.setProperty("overflow-y", "hidden", "important");
        document.body?.style.setProperty("overflow-y", "hidden", "important");
    };

    const ensureResponsiveStyles = () => {
        const parent = document.head || document.documentElement;

        if (!parent || document.getElementById(styleId)) {
            forceRootOverflow();
            return;
        }

        const viewport = document.querySelector('meta[name="viewport"]') || document.createElement("meta");
        viewport.name = "viewport";
        viewport.content = "width=device-width, initial-scale=1, viewport-fit=cover";

        if (!viewport.parentNode) {
            parent.appendChild(viewport);
        }

        const style = document.createElement("style");
        style.id = styleId;
        style.textContent = `
            html, body {
                width: 100% !important;
                max-width: 100% !important;
                overflow-x: hidden !important;
                overflow-y: hidden !important;
            }

            html#facebook,
            html#facebook body,
            html._9dls,
            html._9t1d {
                overflow-y: hidden !important;
            }

            body, #globalContainer, #pagelet_bluebar, #content, #contentArea, .fb_content, ._li, ._95k9, ._8esf, ._8esj, [role="main"] {
                min-width: 0 !important;
                max-width: 100vw !important;
                box-sizing: border-box !important;
            }

            img, video, canvas, iframe, table, form {
                max-width: 100% !important;
            }

            @media (max-width: 719.98px) {
                #globalContainer, #content, #contentArea, .fb_content, ._li, ._95k9, ._8esf, ._8esj {
                    width: 100% !important;
                    margin-left: 0 !important;
                    margin-right: 0 !important;
                }
            }
        `;
        parent.appendChild(style);
        forceRootOverflow();
    };

    const hasPageContent = () => {
        const body = document.body;

        if (!body) {
            return false;
        }

        return body.childElementCount > 0 && body.getBoundingClientRect().height > 100;
    };

    const removeLoadOverlay = () => {
        document.getElementById(overlayId)?.remove();
    };

    const showLoadOverlay = () => {
        const parent = document.body || document.documentElement;

        if (!parent || document.getElementById(overlayId) || hasPageContent()) {
            return;
        }

        const overlay = document.createElement("div");
        overlay.id = overlayId;
        overlay.innerHTML = `
            <div class="tauri-messenger-load-card">
                <div class="tauri-messenger-load-title">Messenger is taking longer than expected.</div>
                <button type="button">Reload</button>
            </div>
        `;

        const style = document.createElement("style");
        style.textContent = `
            #${overlayId} {
                position: fixed;
                inset: 0;
                z-index: 2147483647;
                display: grid;
                place-items: center;
                background: rgba(10, 10, 14, 0.2);
                font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
            }

            #${overlayId} .tauri-messenger-load-card {
                display: flex;
                align-items: center;
                gap: 12px;
                padding: 12px;
                color: #f5f7fb;
                background: #1f2028;
                border: 1px solid rgba(255, 255, 255, 0.12);
                border-radius: 8px;
                box-shadow: 0 12px 40px rgba(0, 0, 0, 0.32);
            }

            #${overlayId} .tauri-messenger-load-title {
                font-size: 13px;
                line-height: 18px;
            }

            #${overlayId} button {
                min-height: 32px;
                padding: 0 12px;
                color: #ffffff;
                font: inherit;
                font-size: 13px;
                font-weight: 600;
                background: #0866ff;
                border: 0;
                border-radius: 6px;
                cursor: pointer;
            }
        `;

        overlay.appendChild(style);
        overlay.querySelector("button")?.addEventListener("click", () => {
            window.location.reload();
        });
        parent.appendChild(overlay);
    };

    window.addEventListener("keydown", (event) => {
        if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "r") {
            event.preventDefault();
            window.location.reload();
        }
    }, true);

    const shellNavigationSelector = [
        "nav",
        '[role="navigation"]',
        '[role="search"]',
        '[aria-label="Chats" i]',
        '[aria-label*="chat list" i]',
        '[aria-label*="conversation list" i]',
        '[aria-label*="cuộc trò chuyện" i]',
        '[aria-label*="danh sách đoạn chat" i]'
    ].join(",");

    const isMessengerChatRoute = (url) => {
        const host = url.hostname.toLowerCase();
        const path = url.pathname.replace(/\/+$/, "");

        if (host !== "messenger.com" && !host.endsWith(".messenger.com")) {
            return false;
        }

        return /^\/(?:u\/\d+\/)?(?:e2ee\/)?t(?:\/|$)/.test(path);
    };

    const isShellNavigationLink = (anchor, url) => {
        if (anchor.closest(shellNavigationSelector)) {
            return true;
        }

        return isMessengerChatRoute(url);
    };

    const isLikelyConversationArea = (anchor) => {
        const rect = anchor.getBoundingClientRect();
        const conversationLeft = Math.min(560, window.innerWidth * 0.34);

        if (rect.width <= 0 || rect.height <= 0 || rect.left < conversationLeft) {
            return false;
        }

        return Boolean(anchor.closest('[role="main"], [aria-label*="messages" i], [aria-label*="tin nhắn" i]'));
    };

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

    const isImageAssetHost = (host) => {
        return host.endsWith(".fbcdn.net")
            || host.includes(".xx.fbcdn.net")
            || host.startsWith("scontent.")
            || host.startsWith("lookaside.");
    };

    const hasInlineMedia = (anchor) => {
        return Boolean(anchor.querySelector("img, video, canvas, [role='img']"));
    };

    const isMessengerMediaUrl = (url) => {
        const host = url.hostname.toLowerCase();
        const path = url.pathname.toLowerCase();

        return isImageAssetHost(host)
            || (isMessengerOrFacebookHost(host) && (
                /\.(?:avif|gif|jpe?g|png|webp)$/i.test(path)
                || /\/(?:photo|photos|media|image|attachment|attachments)(?:\.php|\/|$)/i.test(path)
                || path.includes("/ajax/mercury/attachments/")
            ));
    };

    const visibleTextFor = (root) => {
        const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
        const parts = [];

        while (walker.nextNode()) {
            const node = walker.currentNode;
            const text = node.nodeValue?.replace(/\s+/g, " ").trim();
            const parent = node.parentElement;

            if (!text || !parent) {
                continue;
            }

            const style = window.getComputedStyle(parent);

            if (style.display === "none" || style.visibility === "hidden" || style.opacity === "0") {
                continue;
            }

            if (parent.getClientRects().length === 0) {
                continue;
            }

            parts.push(text);
        }

        return parts.join(" ").trim();
    };

    const isLikelyLinkPreview = (anchor) => {
        const text = visibleTextFor(anchor);

        if (text.length >= 3) {
            return true;
        }

        return Boolean(anchor.querySelector(
            [
                '[aria-label*="play" i]',
                '[aria-label*="watch" i]'
            ].join(",")
        ));
    };

    const externalUrlForClick = (anchor, url) => {
        if (!["http:", "https:"].includes(url.protocol) || isShellNavigationLink(anchor, url)) {
            return null;
        }

        if (isAuthNavigationUrl(url)) {
            return null;
        }

        if (isCurrentAuthSurface() && isMessengerOrFacebookHost(url.hostname.toLowerCase())) {
            return null;
        }

        if (hasInlineMedia(anchor) && isMessengerMediaUrl(url)) {
            return null;
        }

        if (hasInlineMedia(anchor) && !isLikelyLinkPreview(anchor) && isMessengerOrFacebookHost(url.hostname.toLowerCase())) {
            return null;
        }

        if (isMessengerOrFacebookHost(url.hostname.toLowerCase())) {
            const linkShimTarget = url.searchParams.get("u");

            if (linkShimTarget) {
                try {
                    return new URL(linkShimTarget);
                } catch (_) {
                    return url;
                }
            }
        }

        return url;
    };

    window.addEventListener("click", (event) => {
        if (event.defaultPrevented || event.button !== 0) {
            return;
        }

        const anchor = event.composedPath()
            .find((node) => node instanceof HTMLAnchorElement && node.href);

        if (!anchor) {
            return;
        }

        const url = new URL(anchor.href, window.location.href);

        const externalUrl = externalUrlForClick(anchor, url);

        if (!externalUrl) {
            return;
        }

        externalUrl.searchParams.set(externalNavigationParam, "1");
        event.preventDefault();
        event.stopPropagation();
        window.location.href = externalUrl.href;
    }, true);

    ensureResponsiveStyles();
    window.addEventListener("DOMContentLoaded", ensureResponsiveStyles);
    window.addEventListener("load", () => {
        ensureResponsiveStyles();
        window.setTimeout(() => {
            if (hasPageContent()) {
                removeLoadOverlay();
            }
        }, 500);
    });
    window.setTimeout(showLoadOverlay, loadTimeoutMs);
    setInterval(() => {
        ensureResponsiveStyles();
    }, 1000);
})();
"##;
