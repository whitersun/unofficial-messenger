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

    const isVisibleElement = (element) => {
        if (!(element instanceof Element)) {
            return false;
        }

        const rect = element.getBoundingClientRect();

        if (rect.width < 16 || rect.height < 16) {
            return false;
        }

        const style = window.getComputedStyle(element);

        return style.display !== "none"
            && style.visibility !== "hidden"
            && style.opacity !== "0"
            && style.pointerEvents !== "none";
    };

    const hasVisibleMessengerBlockingDialog = () => {
        const body = document.body;

        if (!body) {
            return false;
        }

        return Array.from(body.querySelectorAll([
            '[role="dialog"]',
            '[aria-modal="true"]',
            '[data-visualcompletion="ignore-dynamic"] [role="dialog"]'
        ].join(","))).some((element) => {
            if (element.id === notificationPromptId || element.closest(`#${notificationPromptId}`)) {
                return false;
            }

            return isVisibleElement(element);
        });
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

        if (hasVisibleMessengerBlockingDialog()) {
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

    const queueNotificationPermissionPromptSync = () => {
        window.clearTimeout(notificationPromptSyncTimer);
        notificationPromptSyncTimer = window.setTimeout(syncNotificationPermissionPrompt, 250);
    };

