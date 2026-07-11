    const messengerThreadListSelector = [
        '[role="navigation"][aria-label="Thread list" i]',
        '[role="navigation"][aria-label*="thread list" i]',
        '[role="navigation"][aria-label*="chat list" i]',
        '[role="navigation"][aria-label*="conversation list" i]'
    ].join(",");

    const messengerThreadListElement = () => {
        const candidates = [
            ...document.querySelectorAll(messengerThreadListSelector),
            ...document.querySelectorAll('[role="navigation"]')
        ];

        return candidates.find((element) => {
            if (!(element instanceof HTMLElement) || element.closest('[role="main"]')) {
                return false;
            }

            const rect = element.getBoundingClientRect();
            const maxLeft = Math.max(220, Math.min(680, window.innerWidth * 0.5));
            const isManagedCollapsed = element.style.getPropertyValue(messengerCardMinWidthProperty).trim() === "0"
                && element.style.getPropertyValue(messengerCardMaxWidthProperty).trim() === "0";

            return rect.left >= 0
                && rect.left < maxLeft
                && (rect.width >= 180 || isManagedCollapsed)
                && rect.height >= 240;
        }) || null;
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
        const shouldShow = hasAuthenticatedMessengerShell() && Boolean(messengerThreadListElement());
        const collapsed = isMessengerThreadListCollapsed();

        if (!button || !root) {
            return;
        }

        root.classList.toggle("tauri-messenger-thread-list-toggle-visible", shouldShow);
        button.dataset.collapsed = collapsed ? "true" : "false";
        button.setAttribute("aria-pressed", collapsed ? "true" : "false");
        button.setAttribute(
            "aria-label",
            collapsed ? "Show chat list" : "Hide chat list"
        );
        button.title = collapsed ? "Show chat list" : "Hide chat list";
    };
