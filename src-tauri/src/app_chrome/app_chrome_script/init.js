    ensureResponsiveStyles();
    ensureImageFeatureStyles();
    syncMessengerThreadListCollapse();
    syncMessengerThreadListToggleButton();
    if (window.MutationObserver && document.documentElement) {
        new MutationObserver(() => {
            queueNotificationPermissionPromptSync();
            queueMessengerThreadListCollapseSync();
        }).observe(document.documentElement, {
            childList: true,
            subtree: true,
            attributes: true,
            attributeFilter: ["class", "style", "aria-hidden", "role", "aria-modal"]
        });
    }
    syncNotificationPermissionPrompt();
    window.addEventListener("DOMContentLoaded", () => {
        ensureResponsiveStyles();
        syncMessengerThreadListCollapse();
        syncMessengerThreadListToggleButton();
        syncNotificationPermissionPrompt();
    });
    window.addEventListener("load", () => {
        ensureResponsiveStyles();
        ensureImageFeatureStyles();
        syncMessengerThreadListCollapse();
        syncMessengerThreadListToggleButton();
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
        syncMessengerThreadListCollapse();
        syncMessengerThreadListToggleButton();
        syncNotificationPermissionPrompt();
        if (shouldSuppressLoadOverlay()) {
            removeLoadOverlay();
        }
    }, 1000);
