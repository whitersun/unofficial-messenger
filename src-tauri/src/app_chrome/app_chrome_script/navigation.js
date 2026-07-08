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

    window.addEventListener("contextmenu", (event) => {
        const candidate = imageCandidateFromEvent(event);

        if (!candidate) {
            hideImageContextMenu();
            return;
        }

        event.preventDefault();
        event.stopPropagation();
        showImageContextMenu(candidate, event.clientX, event.clientY);
    }, true);

    window.addEventListener("pointerdown", (event) => {
        const menu = document.getElementById(imageContextMenuId);

        if (!menu || !(event.target instanceof Node) || menu.contains(event.target)) {
            return;
        }

        hideImageContextMenu();
    }, true);
