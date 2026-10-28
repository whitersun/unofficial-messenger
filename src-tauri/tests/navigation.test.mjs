import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import vm from "node:vm";

const scriptRoot = new URL("../src/app_chrome/app_chrome_script/", import.meta.url);
const bootstrap = readFileSync(new URL("bootstrap.js", scriptRoot), "utf8");
const navigation = readFileSync(new URL("navigation.js", scriptRoot), "utf8");

class Anchor {
    constructor(href, { download = false, media = false, text = "", shell = false } = {}) {
        this.rawHref = href;
        this.href = new URL(href, "https://www.facebook.com/messages/t/123").href;
        this.download = download;
        this.media = media;
        this.text = text;
        this.shell = shell;
    }

    getAttribute(name) { return name === "href" ? this.rawHref : null; }
    hasAttribute(name) { return name === "download" && this.download; }
    closest() { return this.shell; }
    querySelector(selector) { return selector.startsWith("img,") && this.media; }
}

function setup(location = "https://www.facebook.com/messages/t/123") {
    const listeners = new Map();
    const context = vm.createContext({
        URL,
        HTMLAnchorElement: Anchor,
        NodeFilter: { SHOW_TEXT: 4 },
        document: {
            createTreeWalker(anchor) {
                let read = false;
                return {
                    currentNode: {
                        nodeValue: anchor.text,
                        parentElement: { getClientRects: () => [{}] }
                    },
                    nextNode() {
                        if (read) return false;
                        read = true;
                        return true;
                    }
                };
            }
        },
        window: {
            location: { href: location, hostname: new URL(location).hostname },
            addEventListener: (name, callback) => listeners.set(name, callback),
            getComputedStyle: () => ({ display: "block", visibility: "visible", opacity: "1" })
        }
    });
    vm.runInContext(`(function () {\n${bootstrap}\n${navigation}\n})();`, context);

    return (href, options) => {
        const anchor = new Anchor(href, options);
        const event = {
            button: 0,
            defaultPrevented: false,
            stopped: false,
            composedPath: () => [anchor],
            preventDefault() { this.defaultPrevented = true; },
            stopPropagation() { this.stopped = true; }
        };
        listeners.get("click")(event);
        return { event, url: new URL(context.window.location.href) };
    };
}

test("Messenger receives placeholder clicks so it can resolve encrypted attachments", () => {
    for (const href of ["", "#", "#attachment", "https://www.facebook.com/messages/t/123#attachment"]) {
        const { event } = setup()(href, { media: true, text: "Document.pdf" });
        assert.equal(event.defaultPrevented, false, href);
        assert.equal(event.stopped, false, href);
    }
});

test("WebView receives download anchors including remote URLs and local blobs", () => {
    for (const href of ["https://cdn.example.com/report.pdf", "blob:https://www.facebook.com/file-id"]) {
        const { event } = setup()(href, { download: true });
        assert.equal(event.defaultPrevented, false, href);
        assert.equal(event.stopped, false, href);
    }
});

test("PDF, Word and other documents open externally even with file icons", () => {
    for (const name of ["report.pdf", "Report.DOCX", "table.xlsx", "slides.pptx", "archive.zip"]) {
        const { event, url } = setup()(`https://scontent.xx.fbcdn.net/files/${name}?token=a%2Bb`, { media: true });
        assert.equal(event.defaultPrevented, true, name);
        assert.equal(event.stopped, true, name);
        assert.equal(url.searchParams.get("__tauri_external"), "1");
        assert.equal(url.searchParams.get("token"), "a+b");
    }
});

test("a document named in query parameters is not treated as an image", () => {
    const { event } = setup()("https://www.facebook.com/attachment.php?filename=report.pdf", { media: true });
    assert.equal(event.defaultPrevented, true);
});

test("opaque attachment URLs can use the displayed filename", () => {
    const { event } = setup()("https://scontent.xx.fbcdn.net/v/file-id", { media: true, text: "Report.pdf 15 KB" });
    assert.equal(event.defaultPrevented, true);
});

test("photo previews still use Messenger's own viewer", () => {
    for (const href of ["https://scontent.xx.fbcdn.net/photo.jpg", "https://www.facebook.com/photo/?fbid=123"]) {
        const { event } = setup()(href, { media: true });
        assert.equal(event.defaultPrevented, false, href);
    }
});

test("blob previews stay inside the WebView", () => {
    const { event } = setup()("blob:https://www.facebook.com/file-id", { media: true });
    assert.equal(event.defaultPrevented, false);
});

test("chat navigation and authentication are preserved", () => {
    for (const href of ["https://www.messenger.com/e2ee/t/123", "https://www.facebook.com/login"]) {
        const { event } = setup()(href);
        assert.equal(event.defaultPrevented, false, href);
    }
    assert.equal(setup()("https://www.facebook.com/messages/t/456", { shell: true }).event.defaultPrevented, false);
});

test("regular links and Facebook link shims still open externally", () => {
    for (const href of ["https://example.com/page", "https://l.facebook.com/l.php?u=https%3A%2F%2Fexample.com%2Fpage"]) {
        const { event, url } = setup()(href);
        assert.equal(event.defaultPrevented, true, href);
        assert.equal(url.hostname, "example.com");
        assert.equal(url.pathname, "/page");
    }
});
