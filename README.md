# unofficial-messenger-webview

An unofficial desktop webview wrapper for Messenger, built with Tauri.

## Disclaimer

This project is an unofficial, non-commercial desktop wrapper for Messenger Web.
It is not affiliated with, endorsed by, sponsored by, or connected to Meta Platforms, Inc.

Messenger, Facebook, Meta, and related names, logos, and trademarks are the property of Meta Platforms, Inc. They are referenced only to identify the web service this app opens.

This project does not collect, store, proxy, or modify login credentials. Authentication happens directly inside the Messenger/Facebook web page loaded in the system WebView.

## Development

```bash
bun run tauri dev
```

The main window opens `https://www.messenger.com/`. Messenger and Facebook links stay inside the app; unrelated external links are opened in the system browser.
