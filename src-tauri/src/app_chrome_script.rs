pub(crate) const APP_CHROME_SCRIPT: &str = r##"
(function() {
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
    const externalNavigationParam = "__tauri_external";
    const copyImageNavigationParam = "__tauri_copy_image";
    const loadTimeoutMs = 15000;
    let notificationPromptEligibleSince = 0;

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

    const ensureResponsiveStyles = () => {
        const parent = document.head || document.documentElement;

        if (!parent || document.getElementById(styleId)) {
            syncRootOverflow();
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
            }

            html.tauri-messenger-lock-body-overflow,
            html.tauri-messenger-lock-body-overflow body,
            html#facebook.tauri-messenger-lock-body-overflow,
            html#facebook.tauri-messenger-lock-body-overflow body,
            html._9dls.tauri-messenger-lock-body-overflow,
            html._9t1d.tauri-messenger-lock-body-overflow {
                overflow-y: hidden !important;
            }

            html.tauri-messenger-allow-body-overflow,
            html.tauri-messenger-allow-body-overflow body {
                overflow-y: auto !important;
            }

            body, #globalContainer, #pagelet_bluebar, #content, #contentArea, .fb_content, ._li, ._95k9, ._8esf, ._8esj, [role="main"] {
                min-width: 0 !important;
                max-width: 100vw !important;
                box-sizing: border-box !important;
            }

            table, form {
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
        syncRootOverflow();
    };

    const hasPageContent = () => {
        const body = document.body;

        if (!body) {
            return false;
        }

        if (body.childElementCount > 0 && body.getBoundingClientRect().height > 100) {
            return true;
        }

        return Boolean(body.querySelector([
            "input",
            "textarea",
            "select",
            "button",
            "form",
            "main",
            '[role="button"]',
            '[role="main"]'
        ].join(",")));
    };

    const shouldSuppressLoadOverlay = () => {
        return isCurrentAuthSurface() || hasPageContent();
    };

    const removeLoadOverlay = () => {
        document.getElementById(overlayId)?.remove();
    };

    const ensureImageFeatureStyles = () => {
        const parent = document.head || document.documentElement;

        if (!parent || document.getElementById(imageFeatureStyleId)) {
            return;
        }

        const style = document.createElement("style");
        style.id = imageFeatureStyleId;
        style.textContent = `
            #${imageContextMenuId} {
                position: fixed;
                z-index: 2147483647;
                min-width: 156px;
                padding: 6px;
                color: #f5f7fb;
                background: rgba(28, 30, 36, 0.96);
                border: 1px solid rgba(255, 255, 255, 0.14);
                border-radius: 8px;
                box-shadow: 0 14px 42px rgba(0, 0, 0, 0.34);
                font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
                font-size: 13px;
                line-height: 18px;
                user-select: none;
            }

            #${imageContextMenuId} button,
            #${imageViewerId} button {
                width: 100%;
                min-height: 32px;
                padding: 0 10px;
                color: inherit;
                font: inherit;
                text-align: left;
                background: transparent;
                border: 0;
                border-radius: 6px;
                cursor: pointer;
            }

            #${imageContextMenuId} button:hover,
            #${imageViewerId} button:hover {
                background: rgba(255, 255, 255, 0.12);
            }

            #${imageContextMenuId} .tauri-messenger-image-status {
                display: none;
                padding: 6px 10px 2px;
                color: rgba(245, 247, 251, 0.72);
                font-size: 12px;
            }

            #${imageContextMenuId}[data-status] .tauri-messenger-image-status {
                display: block;
            }

            #${imageViewerId} {
                position: fixed;
                inset: 0;
                z-index: 2147483646;
                overflow: hidden;
                background: rgba(8, 9, 12, 0.88);
                font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
                touch-action: none;
            }

            #${imageViewerId} .tauri-messenger-image-stage {
                position: absolute;
                inset: 0;
                cursor: grab;
            }

            #${imageViewerId}[data-dragging="true"] .tauri-messenger-image-stage {
                cursor: grabbing;
            }

            #${imageViewerId} img {
                position: absolute;
                top: 50%;
                left: 50%;
                max-width: 92vw;
                max-height: 88vh;
                object-fit: contain;
                transform-origin: center center;
                user-select: none;
                -webkit-user-drag: none;
                box-shadow: 0 18px 70px rgba(0, 0, 0, 0.45);
            }

            #${imageViewerId} .tauri-messenger-image-toolbar {
                position: absolute;
                top: max(12px, env(safe-area-inset-top));
                right: max(12px, env(safe-area-inset-right));
                display: flex;
                align-items: center;
                gap: 6px;
                padding: 6px;
                color: #f5f7fb;
                background: rgba(28, 30, 36, 0.9);
                border: 1px solid rgba(255, 255, 255, 0.14);
                border-radius: 8px;
                box-shadow: 0 14px 42px rgba(0, 0, 0, 0.34);
            }

            #${imageViewerId} .tauri-messenger-image-toolbar button {
                width: auto;
                min-width: 40px;
                height: 36px;
                display: grid;
                place-items: center;
                padding: 0 10px;
                text-align: center;
            }

            #${imageViewerId} .tauri-messenger-image-toolbar button svg {
                width: 20px;
                height: 20px;
                display: block;
                fill: none;
                stroke: currentColor;
                stroke-width: 2.25;
                stroke-linecap: round;
                stroke-linejoin: round;
                pointer-events: none;
            }

            #${imageViewerId} .tauri-messenger-image-toolbar button[data-action="reset"] {
                min-width: 56px;
                font-weight: 600;
            }

            #${imageViewerId} .tauri-messenger-image-viewer-status {
                display: none;
                align-self: center;
                min-width: 96px;
                padding: 0 8px;
                color: rgba(245, 247, 251, 0.72);
                font-size: 12px;
                text-align: center;
            }

            #${imageViewerId}[data-status] .tauri-messenger-image-viewer-status {
                display: block;
            }
        `;
        parent.appendChild(style);
    };

    const ensureNotificationPromptStyles = () => {
        const parent = document.head || document.documentElement;

        if (!parent || document.getElementById(notificationPromptStyleId)) {
            return;
        }

        const style = document.createElement("style");
        style.id = notificationPromptStyleId;
        style.textContent = `
            #${notificationPromptId} {
                position: fixed;
                top: max(16px, env(safe-area-inset-top));
                left: 50%;
                z-index: 2147483647;
                display: flex;
                align-items: center;
                gap: 12px;
                width: min(520px, calc(100vw - 24px));
                padding: 12px;
                color: #f5f7fb;
                background: rgba(28, 30, 36, 0.97);
                border: 1px solid rgba(255, 255, 255, 0.14);
                border-radius: 8px;
                box-shadow: 0 14px 42px rgba(0, 0, 0, 0.34);
                box-sizing: border-box;
                font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
                transform: translateX(-50%);
            }

            #${notificationPromptId} .tauri-messenger-notification-copy {
                flex: 1 1 auto;
                min-width: 0;
            }

            #${notificationPromptId} .tauri-messenger-notification-title {
                color: #ffffff;
                font-size: 13px;
                font-weight: 700;
                line-height: 18px;
            }

            #${notificationPromptId} .tauri-messenger-notification-message {
                margin-top: 2px;
                color: rgba(245, 247, 251, 0.74);
                font-size: 12px;
                line-height: 17px;
            }

            #${notificationPromptId} button {
                min-height: 32px;
                color: #ffffff;
                font: inherit;
                font-size: 13px;
                border: 0;
                cursor: pointer;
            }

            #${notificationPromptId} button[data-action="allow"] {
                flex: 0 0 auto;
                padding: 0 12px;
                font-weight: 700;
                background: #0866ff;
                border-radius: 6px;
            }

            #${notificationPromptId} button[data-action="dismiss"] {
                flex: 0 0 32px;
                width: 32px;
                padding: 0;
                color: rgba(245, 247, 251, 0.72);
                background: transparent;
                border-radius: 6px;
                text-align: center;
            }

            #${notificationPromptId} button:hover {
                filter: brightness(1.08);
            }

            #${notificationPromptId} button[data-action="dismiss"]:hover {
                background: rgba(255, 255, 255, 0.12);
                filter: none;
            }

            @media (max-width: 520px) {
                #${notificationPromptId} {
                    align-items: stretch;
                    flex-wrap: wrap;
                    gap: 10px;
                }

                #${notificationPromptId} .tauri-messenger-notification-copy {
                    flex-basis: calc(100% - 42px);
                    order: 1;
                }

                #${notificationPromptId} button[data-action="dismiss"] {
                    order: 2;
                    margin-left: auto;
                }

                #${notificationPromptId} button[data-action="allow"] {
                    order: 3;
                    flex-basis: 100%;
                }
            }
        `;
        parent.appendChild(style);
    };

    const notificationPermission = () => {
        if (!("Notification" in window)) {
            return "unsupported";
        }

        return Notification.permission;
    };

    const requestNotificationPermission = async () => {
        if (!("Notification" in window) || typeof Notification.requestPermission !== "function") {
            return "unsupported";
        }

        return await new Promise((resolve) => {
            let didResolve = false;
            const finish = (permission) => {
                if (didResolve) {
                    return;
                }

                didResolve = true;
                resolve(permission || Notification.permission);
            };
            const result = Notification.requestPermission(finish);

            if (result && typeof result.then === "function") {
                result.then(finish, () => finish(Notification.permission));
            }
        });
    };

    const removeNotificationPrompt = () => {
        document.getElementById(notificationPromptId)?.remove();
    };

    const dismissNotificationPromptForSession = () => {
        try {
            sessionStorage.setItem(notificationPromptDismissedKey, "1");
        } catch (_) {}

        removeNotificationPrompt();
    };

    const wasNotificationPromptDismissed = () => {
        try {
            return sessionStorage.getItem(notificationPromptDismissedKey) === "1";
        } catch (_) {
            return false;
        }
    };

    const currentUrl = () => {
        try {
            return new URL(window.location.href);
        } catch (_) {
            return null;
        }
    };

    const isMessengerHost = (host) => {
        return host === "messenger.com" || host.endsWith(".messenger.com");
    };

    const isLikelyLoggedOutMessengerSurface = () => {
        const body = document.body;

        if (!body) {
            return true;
        }

        return Boolean(body.querySelector([
            'input[type="password"]',
            'form[action*="login" i]',
            'a[href*="/login" i]',
            'a[href*="recover" i]',
            'button[name="login" i]',
            '[data-testid*="login" i]'
        ].join(",")));
    };

    const hasAuthenticatedMessengerShell = () => {
        const body = document.body;
        const url = currentUrl();

        if (!body || !url || !isMessengerHost(url.hostname.toLowerCase()) || isCurrentAuthSurface()) {
            return false;
        }

        if (isLikelyLoggedOutMessengerSurface()) {
            return false;
        }

        const hasShellMarker = Boolean(body.querySelector([
            '[aria-label="Chats" i]',
            '[aria-label*="chat list" i]',
            '[aria-label*="conversation list" i]',
            '[aria-label*="cuoc tro chuyen" i]',
            '[aria-label*="danh sach doan chat" i]',
            'a[href*="/t/"]',
            'a[href*="/e2ee/t/"]',
            '[role="navigation"] [href*="/t/"]',
            '[role="main"] [contenteditable="true"]',
            '[role="textbox"][contenteditable="true"]'
        ].join(",")));

        if (hasShellMarker) {
            return true;
        }

        const path = url.pathname.replace(/\/+$/, "");
        const isChatPath = /^\/(?:u\/\d+\/)?(?:e2ee\/)?t(?:\/|$)/.test(path);

        return isChatPath && Boolean(body.querySelector([
            '[role="main"]',
            '[role="navigation"]',
            '[data-testid*="mw" i]',
            '[aria-label*="Messenger" i]'
        ].join(",")));
    };

    const isNotificationPromptEligible = () => {
        if (notificationPermission() !== "default" || wasNotificationPromptDismissed()) {
            notificationPromptEligibleSince = 0;
            return false;
        }

        if (!hasAuthenticatedMessengerShell()) {
            notificationPromptEligibleSince = 0;
            return false;
        }

        if (!notificationPromptEligibleSince) {
            notificationPromptEligibleSince = Date.now();
            return false;
        }

        return Date.now() - notificationPromptEligibleSince >= notificationPromptStableMs;
    };

    const syncNotificationPermissionPrompt = () => {
        const prompt = document.getElementById(notificationPromptId);

        if (
            prompt
            && (wasNotificationPromptDismissed() || notificationPermission() !== "default")
        ) {
            removeNotificationPrompt();
            return;
        }

        if (prompt || !isNotificationPromptEligible()) {
            return;
        }

        const parent = document.documentElement || document.body;

        if (!parent) {
            return;
        }

        ensureNotificationPromptStyles();

        const nextPrompt = document.createElement("div");
        nextPrompt.id = notificationPromptId;
        nextPrompt.setAttribute("role", "dialog");
        nextPrompt.setAttribute("aria-live", "polite");
        nextPrompt.innerHTML = `
            <div class="tauri-messenger-notification-copy">
                <div class="tauri-messenger-notification-title">Turn on Messenger notifications</div>
                <div class="tauri-messenger-notification-message">Allow notifications so new messages still arrive while the app is in the tray.</div>
            </div>
            <button type="button" data-action="allow">Allow</button>
            <button type="button" data-action="dismiss" aria-label="Dismiss notification prompt">x</button>
        `;

        nextPrompt.addEventListener("click", async (event) => {
            const button = event.target instanceof Element ? event.target.closest("button[data-action]") : null;

            if (!button) {
                return;
            }

            event.preventDefault();
            event.stopPropagation();

            if (button.dataset.action === "dismiss") {
                dismissNotificationPromptForSession();
                return;
            }

            button.disabled = true;
            button.textContent = "Opening...";

            try {
                await requestNotificationPermission();
            } catch (_) {}

            removeNotificationPrompt();
        }, true);

        parent.appendChild(nextPrompt);
    };

    const imageUrlFromCssValue = (value) => {
        const match = /url\((?:"([^"]+)"|'([^']+)'|([^)]*))\)/i.exec(value || "");
        const rawUrl = match?.[1] || match?.[2] || match?.[3];

        if (!rawUrl) {
            return null;
        }

        try {
            return new URL(rawUrl.trim(), window.location.href).href;
        } catch (_) {
            return null;
        }
    };

    const visibleImageCandidateFromElement = (element) => {
        if (!(element instanceof Element)) {
            return null;
        }

        const rect = element.getBoundingClientRect();

        if (rect.width < 24 || rect.height < 24) {
            return null;
        }

        const style = window.getComputedStyle(element);

        if (style.display === "none" || style.visibility === "hidden" || style.opacity === "0") {
            return null;
        }

        if (element instanceof HTMLImageElement) {
            const url = element.currentSrc || element.src;

            if (url) {
                return {
                    element,
                    url,
                    label: element.alt || "Messenger image"
                };
            }
        }

        if (style.backgroundImage && style.backgroundImage !== "none") {
            const url = imageUrlFromCssValue(style.backgroundImage);

            if (url) {
                return {
                    element,
                    url,
                    label: element.getAttribute("aria-label") || "Messenger image"
                };
            }
        }

        return null;
    };

    const imageCandidateFromEvent = (event) => {
        const path = event.composedPath ? event.composedPath() : [];

        for (const node of path) {
            const candidate = visibleImageCandidateFromElement(node);

            if (candidate) {
                return candidate;
            }
        }

        return visibleImageCandidateFromElement(event.target);
    };

    const fitMenuToViewport = (menu, x, y) => {
        const margin = 8;
        const rect = menu.getBoundingClientRect();
        const left = Math.min(Math.max(margin, x), window.innerWidth - rect.width - margin);
        const top = Math.min(Math.max(margin, y), window.innerHeight - rect.height - margin);

        menu.style.left = `${left}px`;
        menu.style.top = `${top}px`;
    };

    const hideImageContextMenu = () => {
        document.getElementById(imageContextMenuId)?.remove();
    };

    const setImageContextMenuStatus = (text) => {
        const menu = document.getElementById(imageContextMenuId);

        if (!menu) {
            return false;
        }

        menu.dataset.status = text;
        const status = menu.querySelector(".tauri-messenger-image-status");

        if (status) {
            status.textContent = text;
        }

        return true;
    };

    const setImageFeatureStatus = (text) => {
        const didSetMenuStatus = setImageContextMenuStatus(text);
        const viewer = document.getElementById(imageViewerId);
        const status = viewer?.querySelector(".tauri-messenger-image-viewer-status");

        if (viewer && status) {
            viewer.dataset.status = text;
            status.textContent = text;
            window.setTimeout(() => {
                if (viewer.dataset.status === text) {
                    delete viewer.dataset.status;
                    status.textContent = "";
                }
            }, 1600);
        }

        return didSetMenuStatus || Boolean(viewer && status);
    };

    const imageBlobAsPng = async (blob) => {
        const bitmap = await createImageBitmap(blob);
        const canvas = document.createElement("canvas");
        canvas.width = bitmap.width;
        canvas.height = bitmap.height;
        canvas.getContext("2d")?.drawImage(bitmap, 0, 0);

        if (typeof bitmap.close === "function") {
            bitmap.close();
        }

        return await new Promise((resolve, reject) => {
            canvas.toBlob((pngBlob) => {
                if (pngBlob) {
                    resolve(pngBlob);
                } else {
                    reject(new Error("Could not convert image for clipboard"));
                }
            }, "image/png");
        });
    };

    const blobToDataUrl = async (blob) => {
        return await new Promise((resolve, reject) => {
            const reader = new FileReader();
            reader.addEventListener("load", () => resolve(reader.result));
            reader.addEventListener("error", () => reject(new Error("Could not read image data")));
            reader.readAsDataURL(blob);
        });
    };

    const fetchImageBlob = async (url) => {
        const response = await fetch(url, {
            credentials: "include",
            mode: "cors"
        });

        if (!response.ok) {
            throw new Error("Could not download image");
        }

        const blob = await response.blob();

        if (!(blob.type || "image/png").startsWith("image/")) {
            throw new Error("Selected media is not an image");
        }

        return blob;
    };

    const canvasBlobFromImageElement = async (image) => {
        if (!(image instanceof HTMLImageElement)) {
            throw new Error("Rendered image element is not available");
        }

        if (!image.complete) {
            await image.decode();
        }

        const width = image.naturalWidth || Math.round(image.getBoundingClientRect().width);
        const height = image.naturalHeight || Math.round(image.getBoundingClientRect().height);

        if (!width || !height) {
            throw new Error("Rendered image size is not available");
        }

        const canvas = document.createElement("canvas");
        canvas.width = width;
        canvas.height = height;
        canvas.getContext("2d")?.drawImage(image, 0, 0, width, height);

        return await new Promise((resolve, reject) => {
            try {
                canvas.toBlob((blob) => {
                    if (blob) {
                        resolve(blob);
                    } else {
                        reject(new Error("Could not read rendered image"));
                    }
                }, "image/png");
            } catch (error) {
                reject(error instanceof Error ? error : new Error("Could not read rendered image"));
            }
        });
    };

    const blobFromImageCandidate = async (candidate) => {
        try {
            return await canvasBlobFromImageElement(candidate.element);
        } catch (canvasError) {
            try {
                return await fetchImageBlob(candidate.url);
            } catch (fetchError) {
                const canvasMessage = canvasError instanceof Error ? canvasError.message : String(canvasError);
                const fetchMessage = fetchError instanceof Error ? fetchError.message : String(fetchError);
                throw new Error(`Could not read image pixels. Canvas: ${canvasMessage}. Fetch: ${fetchMessage}.`);
            }
        }
    };

    const imageUrlNeedsBrowserFetch = (url) => {
        return /^blob:/i.test(url) || /^data:/i.test(url);
    };

    const tauriInvoke = () => {
        return window.__TAURI_INTERNALS__?.invoke;
    };

    const copyImageBlobWithNativeClipboard = async (invoke, blob) => {
        const pngBlob = await imageBlobAsPng(blob);
        const imageDataUrl = await blobToDataUrl(pngBlob);

        await invoke("copy_image_to_clipboard", {
            imageDataUrl,
            imageUrl: null
        });
    };

    const writeImageToNativeClipboard = async (candidate) => {
        if (!imageUrlNeedsBrowserFetch(candidate.url)) {
            const copyUrl = new URL(window.location.href);
            copyUrl.searchParams.set(copyImageNavigationParam, candidate.url);
            copyUrl.searchParams.set("_tauri_copy_image_ts", String(Date.now()));
            window.location.href = copyUrl.href;

            return "Copy requested";
        }

        const invoke = tauriInvoke();

        if (typeof invoke !== "function") {
            throw new Error("Native clipboard is not available");
        }

        try {
            const blob = await blobFromImageCandidate(candidate);
            await copyImageBlobWithNativeClipboard(invoke, blob);
        } catch (error) {
            throw error instanceof Error ? error : new Error("Could not copy image");
        }

        return "Copied image";
    };

    const writeImageToWebClipboard = async (candidate) => {
        if (!navigator.clipboard) {
            throw new Error("Clipboard is not available in this WebView");
        }

        if (!window.ClipboardItem || typeof navigator.clipboard.write !== "function") {
            throw new Error("Image clipboard is not available in this WebView");
        }

        let blob = await blobFromImageCandidate(candidate);
        let type = blob.type || "image/png";

        if (type !== "image/png" && typeof ClipboardItem.supports === "function" && !ClipboardItem.supports(type)) {
            blob = await imageBlobAsPng(blob);
            type = "image/png";
        }

        await navigator.clipboard.write([
            new ClipboardItem({
                [type]: blob
            })
        ]);

        return "Copied image";
    };

    const writeImageToClipboard = async (candidate) => {
        try {
            return await writeImageToNativeClipboard(candidate);
        } catch (nativeError) {
            try {
                return await writeImageToWebClipboard(candidate);
            } catch (webError) {
                const nativeMessage = nativeError instanceof Error ? nativeError.message : String(nativeError);
                const webMessage = webError instanceof Error ? webError.message : String(webError);
                const scheme = /^[a-z][a-z0-9+.-]*:/i.exec(candidate.url)?.[0] || "unknown:";
                throw new Error(`Could not copy ${scheme} image. Native: ${nativeMessage}. Web: ${webMessage}.`);
            }
        }
    };

    const copyImageCandidate = async (candidate) => {
        setImageFeatureStatus("Copying...");

        try {
            const status = await writeImageToClipboard(candidate);
            setImageFeatureStatus(status);
            window.setTimeout(hideImageContextMenu, 900);
        } catch (error) {
            setImageFeatureStatus(error instanceof Error ? error.message : "Could not copy image");
        }
    };

    const imageViewerState = {
        candidate: null,
        scale: 1,
        panX: 0,
        panY: 0,
        dragging: false,
        startX: 0,
        startY: 0,
        startPanX: 0,
        startPanY: 0
    };

    const updateImageViewerTransform = () => {
        const viewer = document.getElementById(imageViewerId);
        const image = viewer?.querySelector("img");

        if (!image) {
            return;
        }

        image.style.transform = `translate(calc(-50% + ${imageViewerState.panX}px), calc(-50% + ${imageViewerState.panY}px)) scale(${imageViewerState.scale})`;
    };

    const zoomImageViewer = (delta, centerX, centerY) => {
        const oldScale = imageViewerState.scale;
        const nextScale = Math.min(8, Math.max(0.4, oldScale * delta));

        if (nextScale === oldScale) {
            return;
        }

        const viewportX = centerX - window.innerWidth / 2;
        const viewportY = centerY - window.innerHeight / 2;
        imageViewerState.panX = viewportX - ((viewportX - imageViewerState.panX) / oldScale) * nextScale;
        imageViewerState.panY = viewportY - ((viewportY - imageViewerState.panY) / oldScale) * nextScale;
        imageViewerState.scale = nextScale;
        updateImageViewerTransform();
    };

    const resetImageViewer = () => {
        imageViewerState.scale = 1;
        imageViewerState.panX = 0;
        imageViewerState.panY = 0;
        updateImageViewerTransform();
    };

    const closeImageViewer = () => {
        document.getElementById(imageViewerId)?.remove();
        imageViewerState.candidate = null;
        window.removeEventListener("keydown", handleImageViewerKeydown, true);
    };

    function handleImageViewerKeydown(event) {
        if (!document.getElementById(imageViewerId)) {
            return;
        }

        if (event.key === "Escape") {
            event.preventDefault();
            closeImageViewer();
        } else if (event.key === "+" || event.key === "=") {
            event.preventDefault();
            zoomImageViewer(1.18, window.innerWidth / 2, window.innerHeight / 2);
        } else if (event.key === "-") {
            event.preventDefault();
            zoomImageViewer(1 / 1.18, window.innerWidth / 2, window.innerHeight / 2);
        } else if (event.key === "0") {
            event.preventDefault();
            resetImageViewer();
        }
    }

    const openImageViewer = (candidate) => {
        ensureImageFeatureStyles();
        hideImageContextMenu();
        closeImageViewer();
        imageViewerState.candidate = candidate;
        imageViewerState.scale = 1;
        imageViewerState.panX = 0;
        imageViewerState.panY = 0;

        const viewer = document.createElement("div");
        viewer.id = imageViewerId;
        viewer.innerHTML = `
            <div class="tauri-messenger-image-stage">
                <img alt="">
            </div>
            <div class="tauri-messenger-image-toolbar">
                <button type="button" data-action="zoom-out" title="Zoom out" aria-label="Zoom out">
                    <svg viewBox="0 0 24 24" aria-hidden="true">
                        <circle cx="11" cy="11" r="7"></circle>
                        <path d="M8 11h6"></path>
                        <path d="m16.5 16.5 4 4"></path>
                    </svg>
                </button>
                <button type="button" data-action="reset" title="Reset zoom">100%</button>
                <button type="button" data-action="zoom-in" title="Zoom in" aria-label="Zoom in">
                    <svg viewBox="0 0 24 24" aria-hidden="true">
                        <circle cx="11" cy="11" r="7"></circle>
                        <path d="M8 11h6"></path>
                        <path d="M11 8v6"></path>
                        <path d="m16.5 16.5 4 4"></path>
                    </svg>
                </button>
                <button type="button" data-action="copy" title="Copy image" aria-label="Copy image">
                    <svg viewBox="0 0 24 24" aria-hidden="true">
                        <rect x="9" y="9" width="10" height="10" rx="2"></rect>
                        <path d="M5 15V7a2 2 0 0 1 2-2h8"></path>
                    </svg>
                </button>
                <button type="button" data-action="close" title="Close" aria-label="Close">
                    <svg viewBox="0 0 24 24" aria-hidden="true">
                        <path d="M6 6l12 12"></path>
                        <path d="M18 6 6 18"></path>
                    </svg>
                </button>
                <span class="tauri-messenger-image-viewer-status"></span>
            </div>
        `;

        const image = viewer.querySelector("img");
        const stage = viewer.querySelector(".tauri-messenger-image-stage");
        image.src = candidate.url;
        image.alt = candidate.label || "";

        viewer.addEventListener("wheel", (event) => {
            event.preventDefault();
            zoomImageViewer(event.deltaY < 0 ? 1.12 : 1 / 1.12, event.clientX, event.clientY);
        }, { passive: false });

        stage?.addEventListener("pointerdown", (event) => {
            if (event.button !== 0) {
                return;
            }

            imageViewerState.dragging = true;
            imageViewerState.startX = event.clientX;
            imageViewerState.startY = event.clientY;
            imageViewerState.startPanX = imageViewerState.panX;
            imageViewerState.startPanY = imageViewerState.panY;
            viewer.dataset.dragging = "true";
            stage.setPointerCapture(event.pointerId);
        });

        stage?.addEventListener("pointermove", (event) => {
            if (!imageViewerState.dragging) {
                return;
            }

            imageViewerState.panX = imageViewerState.startPanX + event.clientX - imageViewerState.startX;
            imageViewerState.panY = imageViewerState.startPanY + event.clientY - imageViewerState.startY;
            updateImageViewerTransform();
        });

        const endDrag = (event) => {
            if (!imageViewerState.dragging) {
                return;
            }

            imageViewerState.dragging = false;
            viewer.dataset.dragging = "false";

            try {
                stage?.releasePointerCapture(event.pointerId);
            } catch (_) {}
        };

        stage?.addEventListener("pointerup", endDrag);
        stage?.addEventListener("pointercancel", endDrag);
        stage?.addEventListener("dblclick", resetImageViewer);

        viewer.addEventListener("click", (event) => {
            const button = event.target instanceof Element ? event.target.closest("button[data-action]") : null;

            if (!button) {
                return;
            }

            event.preventDefault();
            event.stopPropagation();

            switch (button.dataset.action) {
                case "zoom-out":
                    zoomImageViewer(1 / 1.18, window.innerWidth / 2, window.innerHeight / 2);
                    break;
                case "reset":
                    resetImageViewer();
                    break;
                case "zoom-in":
                    zoomImageViewer(1.18, window.innerWidth / 2, window.innerHeight / 2);
                    break;
                case "copy":
                    copyImageCandidate(candidate);
                    break;
                case "close":
                    closeImageViewer();
                    break;
            }
        }, true);

        document.body.appendChild(viewer);
        updateImageViewerTransform();
        window.addEventListener("keydown", handleImageViewerKeydown, true);
    };

    const showImageContextMenu = (candidate, x, y) => {
        ensureImageFeatureStyles();
        hideImageContextMenu();

        const menu = document.createElement("div");
        menu.id = imageContextMenuId;
        menu.innerHTML = `
            <button type="button" data-action="copy">Copy image</button>
            <button type="button" data-action="zoom">Zoom image</button>
            <div class="tauri-messenger-image-status"></div>
        `;

        menu.addEventListener("click", (event) => {
            const button = event.target instanceof Element ? event.target.closest("button[data-action]") : null;

            if (!button) {
                return;
            }

            event.preventDefault();
            event.stopPropagation();

            if (button.dataset.action === "copy") {
                copyImageCandidate(candidate);
            } else if (button.dataset.action === "zoom") {
                openImageViewer(candidate);
            }
        }, true);

        document.body.appendChild(menu);
        fitMenuToViewport(menu, x, y);
    };

    const showLoadOverlay = () => {
        const parent = document.body || document.documentElement;

        if (!parent || document.getElementById(overlayId) || shouldSuppressLoadOverlay()) {
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

    ensureResponsiveStyles();
    ensureImageFeatureStyles();
    syncNotificationPermissionPrompt();
    window.addEventListener("DOMContentLoaded", () => {
        ensureResponsiveStyles();
        syncNotificationPermissionPrompt();
    });
    window.addEventListener("load", () => {
        ensureResponsiveStyles();
        ensureImageFeatureStyles();
        syncNotificationPermissionPrompt();
        window.setTimeout(() => {
            if (hasPageContent()) {
                removeLoadOverlay();
            }
        }, 500);
    });
    window.setTimeout(showLoadOverlay, loadTimeoutMs);
    setInterval(() => {
        ensureResponsiveStyles();
        syncNotificationPermissionPrompt();
        if (shouldSuppressLoadOverlay()) {
            removeLoadOverlay();
        }
    }, 1000);
})();
"##;
