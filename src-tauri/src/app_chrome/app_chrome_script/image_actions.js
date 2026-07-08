    const imageUrlFromCssValue = (value) => {
        const match = /url\((?:"([^"]+)"|'([^']+)'|([^)]*))\)/i.exec(value || "");
        const rawUrl = match?.[1] || match?.[2] || match?.[3];

        if (!rawUrl) {
            return null;
        }

        try {
            return new URL(rawUrl.trim(), window.location.href).href;
        } catch (_) {
            return null;
        }
    };

    const visibleImageCandidateFromElement = (element) => {
        if (!(element instanceof Element)) {
            return null;
        }

        const rect = element.getBoundingClientRect();

        if (rect.width < 24 || rect.height < 24) {
            return null;
        }

        const style = window.getComputedStyle(element);

        if (style.display === "none" || style.visibility === "hidden" || style.opacity === "0") {
            return null;
        }

        if (element instanceof HTMLImageElement) {
            const url = element.currentSrc || element.src;

            if (url) {
                return {
                    element,
                    url,
                    label: element.alt || "Messenger image"
                };
            }
        }

        if (style.backgroundImage && style.backgroundImage !== "none") {
            const url = imageUrlFromCssValue(style.backgroundImage);

            if (url) {
                return {
                    element,
                    url,
                    label: element.getAttribute("aria-label") || "Messenger image"
                };
            }
        }

        return null;
    };

    const imageCandidateFromEvent = (event) => {
        const path = event.composedPath ? event.composedPath() : [];

        for (const node of path) {
            const candidate = visibleImageCandidateFromElement(node);

            if (candidate) {
                return candidate;
            }
        }

        return visibleImageCandidateFromElement(event.target);
    };

    const fitMenuToViewport = (menu, x, y) => {
        const margin = 8;
        const rect = menu.getBoundingClientRect();
        const left = Math.min(Math.max(margin, x), window.innerWidth - rect.width - margin);
        const top = Math.min(Math.max(margin, y), window.innerHeight - rect.height - margin);

        menu.style.left = `${left}px`;
        menu.style.top = `${top}px`;
    };

    const hideImageContextMenu = () => {
        document.getElementById(imageContextMenuId)?.remove();
    };

    const setImageContextMenuStatus = (text) => {
        const menu = document.getElementById(imageContextMenuId);

        if (!menu) {
            return false;
        }

        menu.dataset.status = text;
        const status = menu.querySelector(".tauri-messenger-image-status");

        if (status) {
            status.textContent = text;
        }

        return true;
    };

    const setImageFeatureStatus = (text) => {
        const didSetMenuStatus = setImageContextMenuStatus(text);
        const viewer = document.getElementById(imageViewerId);
        const status = viewer?.querySelector(".tauri-messenger-image-viewer-status");

        if (viewer && status) {
            viewer.dataset.status = text;
            status.textContent = text;
            window.setTimeout(() => {
                if (viewer.dataset.status === text) {
                    delete viewer.dataset.status;
                    status.textContent = "";
                }
            }, 1600);
        }

        return didSetMenuStatus || Boolean(viewer && status);
    };

    const imageBlobAsPng = async (blob) => {
        const bitmap = await createImageBitmap(blob);
        const canvas = document.createElement("canvas");
        canvas.width = bitmap.width;
        canvas.height = bitmap.height;
        canvas.getContext("2d")?.drawImage(bitmap, 0, 0);

        if (typeof bitmap.close === "function") {
            bitmap.close();
        }

        return await new Promise((resolve, reject) => {
            canvas.toBlob((pngBlob) => {
                if (pngBlob) {
                    resolve(pngBlob);
                } else {
                    reject(new Error("Could not convert image for clipboard"));
                }
            }, "image/png");
        });
    };

    const blobToDataUrl = async (blob) => {
        return await new Promise((resolve, reject) => {
            const reader = new FileReader();
            reader.addEventListener("load", () => resolve(reader.result));
            reader.addEventListener("error", () => reject(new Error("Could not read image data")));
            reader.readAsDataURL(blob);
        });
    };

    const fetchImageBlob = async (url) => {
        const response = await fetch(url, {
            credentials: "include",
            mode: "cors"
        });

        if (!response.ok) {
            throw new Error("Could not download image");
        }

        const blob = await response.blob();

        if (!(blob.type || "image/png").startsWith("image/")) {
            throw new Error("Selected media is not an image");
        }

        return blob;
    };

    const canvasBlobFromImageElement = async (image) => {
        if (!(image instanceof HTMLImageElement)) {
            throw new Error("Rendered image element is not available");
        }

        if (!image.complete) {
            await image.decode();
        }

        const width = image.naturalWidth || Math.round(image.getBoundingClientRect().width);
        const height = image.naturalHeight || Math.round(image.getBoundingClientRect().height);

        if (!width || !height) {
            throw new Error("Rendered image size is not available");
        }

        const canvas = document.createElement("canvas");
        canvas.width = width;
        canvas.height = height;
        canvas.getContext("2d")?.drawImage(image, 0, 0, width, height);

        return await new Promise((resolve, reject) => {
            try {
                canvas.toBlob((blob) => {
                    if (blob) {
                        resolve(blob);
                    } else {
                        reject(new Error("Could not read rendered image"));
                    }
                }, "image/png");
            } catch (error) {
                reject(error instanceof Error ? error : new Error("Could not read rendered image"));
            }
        });
    };

    const blobFromImageCandidate = async (candidate) => {
        try {
            return await canvasBlobFromImageElement(candidate.element);
        } catch (canvasError) {
            try {
                return await fetchImageBlob(candidate.url);
            } catch (fetchError) {
                const canvasMessage = canvasError instanceof Error ? canvasError.message : String(canvasError);
                const fetchMessage = fetchError instanceof Error ? fetchError.message : String(fetchError);
                throw new Error(`Could not read image pixels. Canvas: ${canvasMessage}. Fetch: ${fetchMessage}.`);
            }
        }
    };

    const imageUrlNeedsBrowserFetch = (url) => {
        return /^blob:/i.test(url) || /^data:/i.test(url);
    };

    const tauriInvoke = () => {
        return window.__TAURI_INTERNALS__?.invoke;
    };

    const copyImageBlobWithNativeClipboard = async (invoke, blob) => {
        const pngBlob = await imageBlobAsPng(blob);
        const imageDataUrl = await blobToDataUrl(pngBlob);

        await invoke("copy_image_to_clipboard", {
            imageDataUrl,
            imageUrl: null
        });
    };

    const writeImageToNativeClipboard = async (candidate) => {
        if (!imageUrlNeedsBrowserFetch(candidate.url)) {
            const copyUrl = new URL(window.location.href);
            copyUrl.searchParams.set(copyImageNavigationParam, candidate.url);
            copyUrl.searchParams.set("_tauri_copy_image_ts", String(Date.now()));
            window.location.href = copyUrl.href;

            return "Copy requested";
        }

        const invoke = tauriInvoke();

        if (typeof invoke !== "function") {
            throw new Error("Native clipboard is not available");
        }

        try {
            const blob = await blobFromImageCandidate(candidate);
            await copyImageBlobWithNativeClipboard(invoke, blob);
        } catch (error) {
            throw error instanceof Error ? error : new Error("Could not copy image");
        }

        return "Copied image";
    };

    const writeImageToWebClipboard = async (candidate) => {
        if (!navigator.clipboard) {
            throw new Error("Clipboard is not available in this WebView");
        }

        if (!window.ClipboardItem || typeof navigator.clipboard.write !== "function") {
            throw new Error("Image clipboard is not available in this WebView");
        }

        let blob = await blobFromImageCandidate(candidate);
        let type = blob.type || "image/png";

        if (type !== "image/png" && typeof ClipboardItem.supports === "function" && !ClipboardItem.supports(type)) {
            blob = await imageBlobAsPng(blob);
            type = "image/png";
        }

        await navigator.clipboard.write([
            new ClipboardItem({
                [type]: blob
            })
        ]);

        return "Copied image";
    };

    const writeImageToClipboard = async (candidate) => {
        try {
            return await writeImageToNativeClipboard(candidate);
        } catch (nativeError) {
            try {
                return await writeImageToWebClipboard(candidate);
            } catch (webError) {
                const nativeMessage = nativeError instanceof Error ? nativeError.message : String(nativeError);
                const webMessage = webError instanceof Error ? webError.message : String(webError);
                const scheme = /^[a-z][a-z0-9+.-]*:/i.exec(candidate.url)?.[0] || "unknown:";
                throw new Error(`Could not copy ${scheme} image. Native: ${nativeMessage}. Web: ${webMessage}.`);
            }
        }
    };

    const copyImageCandidate = async (candidate) => {
        setImageFeatureStatus("Copying...");

        try {
            const status = await writeImageToClipboard(candidate);
            setImageFeatureStatus(status);
            window.setTimeout(hideImageContextMenu, 900);
        } catch (error) {
            setImageFeatureStatus(error instanceof Error ? error.message : "Could not copy image");
        }
    };

    const imageViewerState = {
        candidate: null,
        scale: 1,
        panX: 0,
        panY: 0,
        dragging: false,
        startX: 0,
        startY: 0,
        startPanX: 0,
        startPanY: 0
    };

    const updateImageViewerTransform = () => {
        const viewer = document.getElementById(imageViewerId);
        const image = viewer?.querySelector("img");

        if (!image) {
            return;
        }

        image.style.transform = `translate(calc(-50% + ${imageViewerState.panX}px), calc(-50% + ${imageViewerState.panY}px)) scale(${imageViewerState.scale})`;
    };

    const zoomImageViewer = (delta, centerX, centerY) => {
        const oldScale = imageViewerState.scale;
        const nextScale = Math.min(8, Math.max(0.4, oldScale * delta));

        if (nextScale === oldScale) {
            return;
        }

        const viewportX = centerX - window.innerWidth / 2;
        const viewportY = centerY - window.innerHeight / 2;
        imageViewerState.panX = viewportX - ((viewportX - imageViewerState.panX) / oldScale) * nextScale;
        imageViewerState.panY = viewportY - ((viewportY - imageViewerState.panY) / oldScale) * nextScale;
        imageViewerState.scale = nextScale;
        updateImageViewerTransform();
    };

    const resetImageViewer = () => {
        imageViewerState.scale = 1;
        imageViewerState.panX = 0;
        imageViewerState.panY = 0;
        updateImageViewerTransform();
    };

    const closeImageViewer = () => {
        document.getElementById(imageViewerId)?.remove();
        imageViewerState.candidate = null;
        window.removeEventListener("keydown", handleImageViewerKeydown, true);
    };

    function handleImageViewerKeydown(event) {
        if (!document.getElementById(imageViewerId)) {
            return;
        }

        if (event.key === "Escape") {
            event.preventDefault();
            closeImageViewer();
        } else if (event.key === "+" || event.key === "=") {
            event.preventDefault();
            zoomImageViewer(1.18, window.innerWidth / 2, window.innerHeight / 2);
        } else if (event.key === "-") {
            event.preventDefault();
            zoomImageViewer(1 / 1.18, window.innerWidth / 2, window.innerHeight / 2);
        } else if (event.key === "0") {
            event.preventDefault();
            resetImageViewer();
        }
    }

    const openImageViewer = (candidate) => {
        ensureImageFeatureStyles();
        hideImageContextMenu();
        closeImageViewer();
        imageViewerState.candidate = candidate;
        imageViewerState.scale = 1;
        imageViewerState.panX = 0;
        imageViewerState.panY = 0;

        const viewer = document.createElement("div");
        viewer.id = imageViewerId;
        viewer.innerHTML = `
            <div class="tauri-messenger-image-stage">
                <img alt="">
            </div>
            <div class="tauri-messenger-image-toolbar">
                <button type="button" data-action="zoom-out" title="Zoom out" aria-label="Zoom out">
                    <svg viewBox="0 0 24 24" aria-hidden="true">
                        <circle cx="11" cy="11" r="7"></circle>
                        <path d="M8 11h6"></path>
                        <path d="m16.5 16.5 4 4"></path>
                    </svg>
                </button>
                <button type="button" data-action="reset" title="Reset zoom">100%</button>
                <button type="button" data-action="zoom-in" title="Zoom in" aria-label="Zoom in">
                    <svg viewBox="0 0 24 24" aria-hidden="true">
                        <circle cx="11" cy="11" r="7"></circle>
                        <path d="M8 11h6"></path>
                        <path d="M11 8v6"></path>
                        <path d="m16.5 16.5 4 4"></path>
                    </svg>
                </button>
                <button type="button" data-action="copy" title="Copy image" aria-label="Copy image">
                    <svg viewBox="0 0 24 24" aria-hidden="true">
                        <rect x="9" y="9" width="10" height="10" rx="2"></rect>
                        <path d="M5 15V7a2 2 0 0 1 2-2h8"></path>
                    </svg>
                </button>
                <button type="button" data-action="close" title="Close" aria-label="Close">
                    <svg viewBox="0 0 24 24" aria-hidden="true">
                        <path d="M6 6l12 12"></path>
                        <path d="M18 6 6 18"></path>
                    </svg>
                </button>
                <span class="tauri-messenger-image-viewer-status"></span>
            </div>
        `;

        const image = viewer.querySelector("img");
        const stage = viewer.querySelector(".tauri-messenger-image-stage");
        image.src = candidate.url;
        image.alt = candidate.label || "";

        viewer.addEventListener("wheel", (event) => {
            event.preventDefault();
            zoomImageViewer(event.deltaY < 0 ? 1.12 : 1 / 1.12, event.clientX, event.clientY);
        }, { passive: false });

        stage?.addEventListener("pointerdown", (event) => {
            if (event.button !== 0) {
                return;
            }

            imageViewerState.dragging = true;
            imageViewerState.startX = event.clientX;
            imageViewerState.startY = event.clientY;
            imageViewerState.startPanX = imageViewerState.panX;
            imageViewerState.startPanY = imageViewerState.panY;
            viewer.dataset.dragging = "true";
            stage.setPointerCapture(event.pointerId);
        });

        stage?.addEventListener("pointermove", (event) => {
            if (!imageViewerState.dragging) {
                return;
            }

            imageViewerState.panX = imageViewerState.startPanX + event.clientX - imageViewerState.startX;
            imageViewerState.panY = imageViewerState.startPanY + event.clientY - imageViewerState.startY;
            updateImageViewerTransform();
        });

        const endDrag = (event) => {
            if (!imageViewerState.dragging) {
                return;
            }

            imageViewerState.dragging = false;
            viewer.dataset.dragging = "false";

            try {
                stage?.releasePointerCapture(event.pointerId);
            } catch (_) {}
        };

        stage?.addEventListener("pointerup", endDrag);
        stage?.addEventListener("pointercancel", endDrag);
        stage?.addEventListener("dblclick", resetImageViewer);

        viewer.addEventListener("click", (event) => {
            const button = event.target instanceof Element ? event.target.closest("button[data-action]") : null;

            if (!button) {
                return;
            }

            event.preventDefault();
            event.stopPropagation();

            switch (button.dataset.action) {
                case "zoom-out":
                    zoomImageViewer(1 / 1.18, window.innerWidth / 2, window.innerHeight / 2);
                    break;
                case "reset":
                    resetImageViewer();
                    break;
                case "zoom-in":
                    zoomImageViewer(1.18, window.innerWidth / 2, window.innerHeight / 2);
                    break;
                case "copy":
                    copyImageCandidate(candidate);
                    break;
                case "close":
                    closeImageViewer();
                    break;
            }
        }, true);

        document.body.appendChild(viewer);
        updateImageViewerTransform();
        window.addEventListener("keydown", handleImageViewerKeydown, true);
    };

    const showImageContextMenu = (candidate, x, y) => {
        ensureImageFeatureStyles();
        hideImageContextMenu();

        const menu = document.createElement("div");
        menu.id = imageContextMenuId;
        menu.innerHTML = `
            <button type="button" data-action="copy">Copy image</button>
            <button type="button" data-action="zoom">Zoom image</button>
            <div class="tauri-messenger-image-status"></div>
        `;

        menu.addEventListener("click", (event) => {
            const button = event.target instanceof Element ? event.target.closest("button[data-action]") : null;

            if (!button) {
                return;
            }

            event.preventDefault();
            event.stopPropagation();

            if (button.dataset.action === "copy") {
                copyImageCandidate(candidate);
            } else if (button.dataset.action === "zoom") {
                openImageViewer(candidate);
            }
        }, true);

        document.body.appendChild(menu);
        fitMenuToViewport(menu, x, y);
    };
