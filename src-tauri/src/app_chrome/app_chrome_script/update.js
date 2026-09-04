    const updaterInvoke = () => window.__TAURI_INTERNALS__?.invoke;

    const formatUpdateBytes = (value) => {
        if (!Number.isFinite(value) || value <= 0) {
            return "0 MB";
        }

        return `${(value / 1024 / 1024).toFixed(value >= 10 * 1024 * 1024 ? 1 : 2)} MB`;
    };

    const removeUpdateModal = () => {
        document.getElementById(updateModalId)?.remove();
    };

    const createUpdateModal = () => {
        const parent = document.body || document.documentElement;

        if (!parent) {
            return null;
        }

        removeUpdateModal();
        ensureUpdateStyles();

        const overlay = document.createElement("div");
        overlay.id = updateModalId;
        overlay.setAttribute("role", "dialog");
        overlay.setAttribute("aria-modal", "true");
        overlay.setAttribute("aria-labelledby", "tauri-messenger-update-title");

        const card = document.createElement("div");
        card.className = "tauri-messenger-update-card";
        overlay.addEventListener("click", (event) => event.stopPropagation());
        overlay.appendChild(card);
        parent.appendChild(overlay);
        return card;
    };

    const showUpdateChoice = (update) => {
        if (!update?.available || !update.version) {
            return;
        }

        availableUpdate = update;
        const card = createUpdateModal();

        if (!card) {
            return;
        }

        const badge = document.createElement("div");
        badge.className = "tauri-messenger-update-badge";
        badge.setAttribute("aria-hidden", "true");
        badge.textContent = "↓";

        const title = document.createElement("div");
        title.id = "tauri-messenger-update-title";
        title.className = "tauri-messenger-update-title";
        title.textContent = "Messenger update available";

        const message = document.createElement("div");
        message.className = "tauri-messenger-update-message";
        message.textContent = `Version ${update.version} is ready. You are using ${update.currentVersion}.`;

        card.append(badge, title, message);

        if (update.notes?.trim()) {
            const notes = document.createElement("div");
            notes.className = "tauri-messenger-update-notes";
            notes.textContent = update.notes.trim();
            card.appendChild(notes);
        }

        const actions = document.createElement("div");
        actions.className = "tauri-messenger-update-actions";

        const later = document.createElement("button");
        later.type = "button";
        later.dataset.action = "later";
        later.textContent = "Later";

        const install = document.createElement("button");
        install.type = "button";
        install.dataset.action = "install";
        install.textContent = "Update now";

        actions.append(later, install);
        card.appendChild(actions);

        later.addEventListener("click", async () => {
            removeUpdateModal();

            try {
                await updaterInvoke()?.("defer_update");
            } catch (_) {}
        });

        install.addEventListener("click", async () => {
            showUpdateProgress(update);

            try {
                await updaterInvoke()?.("install_update");
            } catch (error) {
                try {
                    await updaterInvoke()?.("defer_update");
                } catch (_) {}
                showUpdateError(error);
            }
        });
    };

    const showUpdateProgress = (update) => {
        const card = createUpdateModal();

        if (!card) {
            return;
        }

        const title = document.createElement("div");
        title.id = "tauri-messenger-update-title";
        title.className = "tauri-messenger-update-title";
        title.textContent = `Updating Messenger to ${update.version}`;

        const message = document.createElement("div");
        message.className = "tauri-messenger-update-message";
        message.dataset.updateStatus = "true";
        message.textContent = "Preparing download...";

        const track = document.createElement("div");
        track.className = "tauri-messenger-update-progress";
        track.setAttribute("role", "progressbar");
        track.setAttribute("aria-label", "Update download progress");

        const bar = document.createElement("div");
        bar.className = "tauri-messenger-update-progress-bar";
        track.appendChild(bar);

        const detail = document.createElement("div");
        detail.className = "tauri-messenger-update-detail";
        detail.dataset.updateDetail = "true";
        detail.textContent = "This window will restart automatically when the update is ready.";

        card.append(title, message, track, detail);
    };

    const showUpdateError = (error) => {
        const modal = document.getElementById(updateModalId);
        const status = modal?.querySelector('[data-update-status="true"]');
        const detail = modal?.querySelector('[data-update-detail="true"]');

        if (status) {
            status.textContent = "The update could not be installed.";
        }

        if (detail) {
            detail.textContent = String(error || "Please try again from the tray.");
            detail.classList.add("tauri-messenger-update-error");
        }

        const card = modal?.querySelector(".tauri-messenger-update-card");

        if (card && !card.querySelector('button[data-action="close-error"]')) {
            const close = document.createElement("button");
            close.type = "button";
            close.dataset.action = "close-error";
            close.className = "tauri-messenger-update-close";
            close.textContent = "Close";
            close.addEventListener("click", removeUpdateModal);
            card.appendChild(close);
        }
    };

    const updateProgress = ({ stage, downloaded = 0, total = null } = {}) => {
        const modal = document.getElementById(updateModalId);

        if (!modal) {
            return;
        }

        const status = modal.querySelector('[data-update-status="true"]');
        const detail = modal.querySelector('[data-update-detail="true"]');
        const track = modal.querySelector(".tauri-messenger-update-progress");
        const bar = modal.querySelector(".tauri-messenger-update-progress-bar");

        if (!status || !detail) {
            return;
        }

        if (stage === "installing") {
            status.textContent = "Download complete. Installing...";
            detail.textContent = "Keep Messenger open while the update is installed.";
            track?.classList.add("is-complete");
            bar?.style.setProperty("width", "100%");
            return;
        }

        if (stage === "restarting") {
            status.textContent = "Update installed. Restarting Messenger...";
            detail.textContent = "";
            track?.classList.add("is-complete");
            bar?.style.setProperty("width", "100%");
            return;
        }

        status.textContent = "Downloading update...";

        if (total > 0) {
            const percent = Math.min(100, Math.round(downloaded / total * 100));
            track?.classList.remove("is-indeterminate");
            track?.setAttribute("aria-valuenow", String(percent));
            bar?.style.setProperty("width", `${percent}%`);
            detail.textContent = `${formatUpdateBytes(downloaded)} of ${formatUpdateBytes(total)} · ${percent}%`;
        } else {
            track?.classList.add("is-indeterminate");
            track?.removeAttribute("aria-valuenow");
            detail.textContent = downloaded > 0 ? `${formatUpdateBytes(downloaded)} downloaded` : "Connecting...";
        }
    };

    const requestAvailableUpdate = async () => {
        const invoke = updaterInvoke();

        if (!invoke) {
            return null;
        }

        for (let attempt = 0; attempt < 20; attempt += 1) {
            const result = await invoke("check_for_update");

            if (!result?.checking) {
                return result;
            }

            await new Promise((resolve) => window.setTimeout(resolve, 500));
        }

        return null;
    };

    const showAvailableUpdate = async () => {
        try {
            const update = availableUpdate?.available ? availableUpdate : await requestAvailableUpdate();

            if (update?.available) {
                showUpdateChoice(update);
            }
        } catch (_) {}
    };

    const performAutomaticUpdateCheck = async () => {
        try {
            const update = await requestAvailableUpdate();

            if (update?.available) {
                availableUpdate = update;

                if (!update.deferred && !document.getElementById(updateModalId)) {
                    showUpdateChoice(update);
                }
            }
        } catch (_) {}
    };

    const hasMessengerInitialized = () => {
        return document.readyState === "complete" && hasPageContent();
    };

    const beginHourlyUpdateChecks = () => {
        if (updateHourlyTimer) {
            return;
        }

        updateHourlyTimer = window.setInterval(
            performAutomaticUpdateCheck,
            updateCheckIntervalMs
        );
    };

    const runInitialUpdateCheckWhenReady = () => {
        if (!hasMessengerInitialized()) {
            return;
        }

        window.clearInterval(updateReadyPollTimer);
        updateReadyPollTimer = 0;
        void performAutomaticUpdateCheck();
        beginHourlyUpdateChecks();
    };

    const scheduleInitialUpdateCheck = () => {
        if (updateCheckStarted) {
            return;
        }

        updateCheckStarted = true;
        runInitialUpdateCheckWhenReady();

        if (!updateHourlyTimer) {
            updateReadyPollTimer = window.setInterval(
                runInitialUpdateCheckWhenReady,
                500
            );
        }
    };

    window.__TAURI_MESSENGER_SHOW_UPDATE__ = showAvailableUpdate;
    window.__TAURI_MESSENGER_UPDATE_PROGRESS__ = updateProgress;
