    const messengerThreadListSelector = [
        '[role="navigation"][aria-label="Thread list" i]',
        '[role="navigation"][aria-label*="thread list" i]',
        '[role="navigation"][aria-label*="chat list" i]',
        '[role="navigation"][aria-label*="conversation list" i]'
    ].join(",");

    const messengerHeaderActionSelector = [
        'a[aria-label]',
        'button[aria-label]',
        '[role="link"][aria-label]',
        '[role="button"][aria-label]'
    ].join(",");

    const isMessengerConversationHeaderActionLabel = (label) => {
        const normalizedLabel = (label || "").trim().toLowerCase();

        return [
            "call",
            "voice",
            "audio",
            "phone",
            "video",
            "info",
            "information",
            "details",
            "gọi",
            "thoại",
            "cuộc gọi",
            "thông tin",
            "chi tiết"
        ].some((token) => normalizedLabel.includes(token));
    };

    const messengerThreadListElement = () => {
        const candidates = Array.from(document.querySelectorAll(messengerThreadListSelector));

        return candidates.find((element) => {
            if (!(element instanceof HTMLElement) || element.closest('[role="main"]')) {
                return false;
            }

            const rect = element.getBoundingClientRect();
            const maxLeft = Math.max(220, Math.min(680, window.innerWidth * 0.5));

            return rect.left >= 0 && rect.left < maxLeft;
        }) || null;
    };

    const messengerConversationHeaderActionAnchor = () => {
        const reliableActions = Array.from(document.querySelectorAll(messengerHeaderActionSelector))
            .filter((element) => {
                if (
                    !(element instanceof HTMLElement)
                    || element.id === threadListToggleButtonId
                    || element.closest(`#${threadListToggleButtonId}`)
                    || element.closest('[role="navigation"]')
                    || !isMessengerConversationHeaderActionLabel(element.getAttribute("aria-label"))
                    || !isVisibleElement(element)
                ) {
                    return false;
                }

                const rect = element.getBoundingClientRect();

                return rect.top >= 0
                    && rect.top <= 96
                    && rect.right >= window.innerWidth - 280
                    && rect.width >= 16
                    && rect.width <= 72
                    && rect.height >= 16
                    && rect.height <= 72;
            })
            .sort((first, second) => {
                return first.getBoundingClientRect().left - second.getBoundingClientRect().left;
            });

        const colorElement = reliableActions[0] || null;

        if (!colorElement) {
            return null;
        }

        return {
            colorElement,
            positionElement: colorElement
        };
    };

    const nearbyHeaderActionBefore = (anchorElement, desiredLeft, desiredRight) => {
        if (!(anchorElement instanceof HTMLElement)) {
            return null;
        }

        const anchorRect = anchorElement.getBoundingClientRect();
        const anchorCenterY = anchorRect.top + anchorRect.height / 2;
        const blockers = Array.from(document.querySelectorAll(messengerHeaderActionSelector))
            .filter((element) => {
                if (
                    !(element instanceof HTMLElement)
                    || element === anchorElement
                    || element.id === threadListToggleButtonId
                    || element.closest(`#${threadListToggleButtonId}`)
                    || element.closest('[role="navigation"]')
                    || !isVisibleElement(element)
                ) {
                    return false;
                }

                const rect = element.getBoundingClientRect();
                const centerY = rect.top + rect.height / 2;

                return rect.top >= 0
                    && rect.top <= 96
                    && rect.right <= anchorRect.left + 8
                    && rect.right >= anchorRect.left - 128
                    && rect.width >= 16
                    && rect.width <= 120
                    && rect.height >= 16
                    && rect.height <= 72
                    && Math.abs(centerY - anchorCenterY) <= 16
                    && rect.left < desiredRight
                    && rect.right > desiredLeft;
            })
            .sort((first, second) => {
                return first.getBoundingClientRect().left - second.getBoundingClientRect().left;
            });

        return blockers[0] || null;
    };

    const isUsableCssColor = (value) => {
        const color = (value || "").trim();

        return Boolean(color)
            && color !== "none"
            && color !== "transparent"
            && !/^rgba?\(\s*0\s*,\s*0\s*,\s*0\s*(?:,\s*0\s*)?\)$/i.test(color);
    };

    const resolvedColorInScope = (scope, colorValue) => {
        if (!colorValue || !(scope instanceof HTMLElement)) {
            return null;
        }

        const probe = document.createElement("span");
        probe.style.position = "absolute";
        probe.style.width = "0";
        probe.style.height = "0";
        probe.style.overflow = "hidden";
        probe.style.pointerEvents = "none";
        probe.style.color = colorValue;

        try {
            scope.appendChild(probe);
            const resolvedColor = window.getComputedStyle(probe).color;
            return isUsableCssColor(resolvedColor) ? resolvedColor : null;
        } catch (_) {
            return null;
        } finally {
            probe.remove();
        }
    };

    const messengerHeaderActionIconColor = (headerAction) => {
        if (!(headerAction instanceof HTMLElement)) {
            return null;
        }

        const headerActionStyle = window.getComputedStyle(headerAction);
        const scopedVariableColor = resolvedColorInScope(headerAction, "var(--mwp-header-button-color)")
            || resolvedColorInScope(headerAction, "var(--primary-icon)");

        if (scopedVariableColor) {
            return scopedVariableColor;
        }

        const iconCandidates = [
            ...headerAction.querySelectorAll("svg, path, use, i, span, div"),
            headerAction
        ];

        for (const candidate of iconCandidates) {
            const style = window.getComputedStyle(candidate);
            const fill = style.fill;
            const stroke = style.stroke;
            const color = style.color;

            if (isUsableCssColor(fill)) {
                return fill;
            }

            if (isUsableCssColor(stroke)) {
                return stroke;
            }

            if (isUsableCssColor(color)) {
                return color;
            }
        }

        const variableColor = headerActionStyle.getPropertyValue("--mwp-header-button-color").trim()
            || headerActionStyle.getPropertyValue("--primary-icon").trim();

        return isUsableCssColor(variableColor) ? variableColor : null;
    };

    const isMessengerThreadListCollapsed = () => {
        const threadList = messengerThreadListElement();

        if (!threadList) {
            return messengerThreadListCollapsed;
        }

        const minWidth = threadList.style.getPropertyValue(messengerCardMinWidthProperty).trim();
        const maxWidth = threadList.style.getPropertyValue(messengerCardMaxWidthProperty).trim();

        return minWidth === "0" && maxWidth === "0";
    };

    const syncMessengerThreadListCollapse = () => {
        const threadList = messengerThreadListElement();

        if (!threadList) {
            return;
        }

        if (messengerThreadListCollapsed) {
            messengerThreadListHasManagedStyle = true;

            if (threadList.style.getPropertyValue(messengerCardMinWidthProperty).trim() !== "0") {
                threadList.style.setProperty(messengerCardMinWidthProperty, "0");
            }

            if (threadList.style.getPropertyValue(messengerCardMaxWidthProperty).trim() !== "0") {
                threadList.style.setProperty(messengerCardMaxWidthProperty, "0");
            }

            return;
        }

        if (!messengerThreadListHasManagedStyle) {
            return;
        }

        threadList.style.removeProperty(messengerCardMinWidthProperty);
        threadList.style.removeProperty(messengerCardMaxWidthProperty);
        messengerThreadListHasManagedStyle = false;
    };

    const setMessengerThreadListCollapsed = (collapsed) => {
        if (!collapsed && isMessengerThreadListCollapsed()) {
            messengerThreadListHasManagedStyle = true;
        }

        messengerThreadListCollapsed = collapsed;
        syncMessengerThreadListCollapse();
        syncMessengerThreadListToggleButton();
    };

    const queueMessengerThreadListCollapseSync = () => {
        window.clearTimeout(messengerThreadListCollapseSyncTimer);
        messengerThreadListCollapseSyncTimer = window.setTimeout(() => {
            syncMessengerThreadListCollapse();
            syncMessengerThreadListToggleButton();
        }, 100);
    };

    const toggleMessengerThreadList = () => {
        setMessengerThreadListCollapsed(!isMessengerThreadListCollapsed());
    };

    const ensureMessengerThreadListToggleButton = () => {
        const parent = document.body || document.documentElement;

        if (!parent) {
            return null;
        }

        let button = document.getElementById(threadListToggleButtonId);

        if (button) {
            return button;
        }

        button = document.createElement("button");
        button.id = threadListToggleButtonId;
        button.type = "button";
        button.innerHTML = `
            <svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24">
                <path d="M0 0h24v24H0z" fill="none" />
                <path fill="currentColor" d="M18 3a3 3 0 0 1 2.995 2.824L21 6v12a3 3 0 0 1-2.824 2.995L18 21H6a3 3 0 0 1-2.995-2.824L3 18V6a3 3 0 0 1 2.824-2.995L6 3zm0 2H9v14h9a1 1 0 0 0 .993-.883L19 18V6a1 1 0 0 0-.883-.993zm-2.293 4.293a1 1 0 0 1 .083 1.32l-.083.094L14.415 12l1.292 1.293a1 1 0 0 1 .083 1.32l-.083.094a1 1 0 0 1-1.32.083l-.094-.083l-2-2a1 1 0 0 1-.083-1.32l.083-.094l2-2a1 1 0 0 1 1.414 0" />
            </svg>
        `;
        button.addEventListener("click", (event) => {
            event.preventDefault();
            event.stopPropagation();
            toggleMessengerThreadList();
        }, true);
        parent.appendChild(button);

        return button;
    };

    const syncMessengerThreadListToggleButton = () => {
        const root = document.documentElement;
        const button = ensureMessengerThreadListToggleButton();
        const headerActionAnchor = messengerConversationHeaderActionAnchor();
        const shouldShow = hasAuthenticatedMessengerShell()
            && Boolean(messengerThreadListElement())
            && Boolean(headerActionAnchor);
        const collapsed = isMessengerThreadListCollapsed();

        if (!button || !root) {
            return;
        }

        root.classList.toggle("tauri-messenger-thread-list-toggle-visible", shouldShow);

        if (headerActionAnchor) {
            const headerActionRect = headerActionAnchor.positionElement.getBoundingClientRect();
            const buttonRect = button.getBoundingClientRect();
            const buttonSize = buttonRect.width || 28;
            const gap = 8;
            const headerActionColor = messengerHeaderActionIconColor(headerActionAnchor.colorElement);
            let nextButtonLeft = Math.round(headerActionRect.left - gap - buttonSize);
            const blocker = nearbyHeaderActionBefore(
                headerActionAnchor.positionElement,
                nextButtonLeft,
                nextButtonLeft + buttonSize
            );

            if (blocker) {
                nextButtonLeft = Math.round(blocker.getBoundingClientRect().left - gap - buttonSize);
            }

            button.style.top = `${Math.max(0, Math.round(headerActionRect.top + (headerActionRect.height - buttonSize) / 2))}px`;
            button.style.right = `${Math.max(8, Math.round(window.innerWidth - nextButtonLeft - buttonSize))}px`;

            if (headerActionColor) {
                button.style.setProperty("--mwp-header-button-color", headerActionColor);
                button.style.setProperty("color", headerActionColor, "important");
            }
        }

        button.dataset.collapsed = collapsed ? "true" : "false";
        button.setAttribute("aria-pressed", collapsed ? "true" : "false");
        button.setAttribute(
            "aria-label",
            collapsed ? "Show chat list" : "Hide chat list"
        );
        button.title = collapsed ? "Show chat list" : "Hide chat list";
    };
