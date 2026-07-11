    let badgeMayBeVisible = /\(\s*\d+\+?\s*\)/.test(document.title);
    let badgeClearInFlight = false;

    const syncBadgeVisibilityFromTitle = () => {
        badgeMayBeVisible = /\(\s*\d+\+?\s*\)/.test(document.title);
    };

    const clearBadgeAfterInteraction = async () => {
        if (!badgeMayBeVisible || badgeClearInFlight) {
            return;
        }

        const invoke = window.__TAURI_INTERNALS__?.invoke;
        if (typeof invoke !== "function") {
            return;
        }

        badgeClearInFlight = true;
        try {
            await invoke("clear_app_badge");
            badgeMayBeVisible = false;
        } catch (error) {
            console.warn("Failed to clear the app badge", error);
        } finally {
            badgeClearInFlight = false;
        }
    };

    const titleObserver = new MutationObserver(syncBadgeVisibilityFromTitle);
    const observeDocumentTitle = () => {
        const title = document.querySelector("title");
        if (title) {
            titleObserver.observe(title, { childList: true, characterData: true, subtree: true });
        }
        syncBadgeVisibilityFromTitle();
    };

    observeDocumentTitle();
    window.addEventListener("DOMContentLoaded", observeDocumentTitle, { once: true });
    window.addEventListener("focus", clearBadgeAfterInteraction);
    document.addEventListener("focusin", clearBadgeAfterInteraction, true);
    document.addEventListener("pointerdown", clearBadgeAfterInteraction, true);
    document.addEventListener("keydown", clearBadgeAfterInteraction, true);
    document.addEventListener("visibilitychange", () => {
        if (document.visibilityState === "visible") {
            clearBadgeAfterInteraction();
        }
    });
