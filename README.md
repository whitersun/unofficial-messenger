# unofficial-messenger

An unofficial desktop webview wrapper for Messenger, built with Tauri.

## Downloads

Installers are published on the [latest release page](https://github.com/whitersun/unofficial-messenger/releases/latest).

| Platform | Download |
| --- | --- |
| Windows | [Download for Windows](https://github.com/whitersun/unofficial-messenger/releases/latest) |
| macOS | [Download for macOS](https://github.com/whitersun/unofficial-messenger/releases/latest) |
| Linux | [Download for Linux](https://github.com/whitersun/unofficial-messenger/releases/latest) |

Release assets are built by GitHub Actions for Windows, macOS, and Linux when a version tag like `v0.1.0` is pushed.

## Disclaimer

This project is an unofficial, non-commercial desktop wrapper for Messenger Web.
It is not affiliated with, endorsed by, sponsored by, or connected to Meta Platforms, Inc.

Messenger, Facebook, Meta, and related names, logos, and trademarks are the property of Meta Platforms, Inc. They are referenced only to identify the web service this app opens.

This project does not collect, store, proxy, or modify login credentials. Authentication happens directly inside the Messenger/Facebook web page loaded in the system WebView.

## Development

```bash
bun run tauri dev
```

The main window opens `https://www.messenger.com/`. App/system navigation such as login, checkpoint, 2FA, and session redirects stays inside the WebView. User-clicked links from conversations open in the system browser.

## Release

Create a release build by pushing a version tag:

```bash
git tag v0.1.0
git push origin v0.1.0
```

The release workflow creates a draft GitHub Release with installers for Windows, macOS, and Linux.

## Updates

The app includes a manual updater in the tray menu:

- `Check for Updates` checks GitHub Releases.
- If a new version is available, the menu changes to `Update Available: v...`.
- Clicking that menu item downloads and installs the update.

The updater reads:

```text
https://github.com/whitersun/unofficial-messenger/releases/latest/download/latest.json
```

Before publishing releases, add these GitHub repository secrets:

```text
TAURI_SIGNING_PRIVATE_KEY
TAURI_SIGNING_PRIVATE_KEY_PASSWORD
```

The private key was generated locally at:

```text
src-tauri/.updater-private/updater.key
```

That folder is ignored by Git and must not be committed. Copy the private key content into `TAURI_SIGNING_PRIVATE_KEY`. The password secret can be left empty if the key was generated without a password.

Updates only work after the GitHub Release is published, because draft releases are not visible to the updater endpoint.
