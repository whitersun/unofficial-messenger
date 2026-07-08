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
        style.textContent = responsiveStyleText;
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
        style.textContent = imageFeatureStyleText;
        parent.appendChild(style);
    };

    const ensureNotificationPromptStyles = () => {
        const parent = document.head || document.documentElement;

        if (!parent || document.getElementById(notificationPromptStyleId)) {
            return;
        }

        const style = document.createElement("style");
        style.id = notificationPromptStyleId;
        style.textContent = notificationPromptStyleText;
        parent.appendChild(style);
    };

