    ensureResponsiveStyles();
    ensureImageFeatureStyles();
    if (window.MutationObserver && document.documentElement) {
        new MutationObserver(() => {
            queueNotificationPermissionPromptSync();
        }).observe(document.documentElement, {
            childList: true,
            subtree: true,
            attributes: true,
            attributeFilter: ["class", "style", "aria-hidden", "role", "aria-modal", "aria-expanded"]
        });
    }
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
        scheduleInitialUpdateCheck();
    });
    window.setTimeout(showLoadOverlay, loadTimeoutMs);
    setInterval(() => {
        ensureResponsiveStyles();
        syncNotificationPermissionPrompt();
        if (shouldSuppressLoadOverlay()) {
            removeLoadOverlay();
        }
    }, 1000);
