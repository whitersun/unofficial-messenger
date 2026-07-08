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
        style.textContent = loadOverlayStyleText;

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
