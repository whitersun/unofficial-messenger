    let downloadNoticeTimer = 0;

    window.addEventListener("tauri-messenger-download", (event) => {
        const { status, filename } = event.detail || {};
        const messages = {
            started: "Downloading file…",
            finished: "Download complete",
            failed: "Could not download file. Please try again."
        };

        if (!messages[status]) {
            return;
        }

        const parent = document.body || document.documentElement;
        if (!parent) {
            return;
        }

        const styleId = "tauri-messenger-download-style";
        if (!document.getElementById(styleId)) {
            const style = document.createElement("style");
            style.id = styleId;
            style.textContent = downloadStyleText;
            (document.head || parent).appendChild(style);
        }

        const noticeId = "tauri-messenger-download-notice";
        document.getElementById(noticeId)?.remove();
        window.clearTimeout(downloadNoticeTimer);

        const notice = document.createElement("div");
        notice.id = noticeId;
        notice.setAttribute("role", status === "failed" ? "alert" : "status");
        notice.textContent = messages[status];

        if (filename) {
            const name = document.createElement("span");
            name.textContent = filename;
            notice.appendChild(name);
        }

        parent.appendChild(notice);
        downloadNoticeTimer = window.setTimeout(() => notice.remove(), 6000);
    });
