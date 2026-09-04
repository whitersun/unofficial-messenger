# unofficial-messenger-next

[![Release](https://img.shields.io/github/v/release/whitersun/messenger?label=release)](https://github.com/whitersun/messenger/releases/latest)
[![License](https://img.shields.io/github/license/whitersun/messenger?label=license)](LICENSE)
[![GitHub](https://img.shields.io/badge/GitHub-Repository-181717?logo=github)](https://github.com/whitersun/messenger)
[![Tauri](https://img.shields.io/badge/Tauri-2-24C8DB?logo=tauri&logoColor=white)](https://tauri.app/)
[![Vue](https://img.shields.io/badge/Vue-3-42B883?logo=vuedotjs&logoColor=white)](https://vuejs.org/)
[![Rust](https://img.shields.io/badge/Rust-Desktop-000000?logo=rust&logoColor=white)](https://www.rust-lang.org/)

An unofficial desktop webview wrapper for Messenger, built with Tauri for low CPU and RAM usage.

## Downloads

Install the latest build directly from the `v0.1.5` release:

| Platform | Download |
| --- | --- |
| Windows | [Download for Windows](https://github.com/whitersun/unofficial-messenger/releases/download/v0.1.5/unofficial-messenger-next-windows.msi) |
| macOS | [Download for macOS](https://github.com/whitersun/unofficial-messenger/releases/download/v0.1.5/unofficial-messenger-next-macos.dmg) |
| Linux | [Download for Linux](https://github.com/whitersun/unofficial-messenger/releases/download/v0.1.5/unofficial-messenger-next-linux.deb) |

You can also open the [v0.1.5 release page](https://github.com/whitersun/unofficial-messenger/releases/tag/v0.1.5) and download the installer for your operating system from the assets.

The Windows installer is about 4,516 KB. After installation, the app uses about 12.3 MB of disk space.

## Features

- Built with Tauri, so it uses the system WebView instead of bundling a full browser runtime.
- Built with Rust for low CPU and RAM usage.
- High-performance desktop wrapper for Messenger Web.
- Supports Messenger calls and video calls through the loaded Messenger web app.
- Supports image copy.
- Supports image zoom.
- Supports toggle users.
- Checks for signed GitHub updates after Messenger finishes initializing and then once per hour, with update progress and a tray reminder.

## Publishing updates

The release workflow publishes the platform installers, signed updater artifacts, and `latest.json` used by the in-app updater. Add the private key that matches `src-tauri/.updater-private/updater.key.pub` to the repository secret `TAURI_SIGNING_PRIVATE_KEY`. If the key has a password, add it as `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.

Keep the version in `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json` in sync before pushing a `v*` tag. The updater private key must remain private and must not be committed.

## Disclaimer

This project is an unofficial, non-commercial desktop wrapper for Messenger Web.
It is not affiliated with, endorsed by, sponsored by, or connected to Meta Platforms, Inc.

Messenger, Facebook, Meta, and related names, logos, and trademarks are the property of Meta Platforms, Inc. They are referenced only to identify the web service this app opens.

This project does not collect, store, proxy, or modify login credentials. Authentication happens directly inside the Messenger/Facebook web page loaded in the system WebView.
