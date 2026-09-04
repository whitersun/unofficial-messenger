# unofficial-messenger-next

[![Release](https://img.shields.io/github/v/release/whitersun/messenger?label=release)](https://github.com/whitersun/messenger/releases/latest)
[![License](https://img.shields.io/github/license/whitersun/messenger?label=license)](LICENSE)
[![GitHub](https://img.shields.io/badge/GitHub-Repository-181717?logo=github)](https://github.com/whitersun/messenger)
[![Tauri](https://img.shields.io/badge/Tauri-2-24C8DB?logo=tauri&logoColor=white)](https://tauri.app/)
[![Vue](https://img.shields.io/badge/Vue-3-42B883?logo=vuedotjs&logoColor=white)](https://vuejs.org/)
[![Rust](https://img.shields.io/badge/Rust-Desktop-000000?logo=rust&logoColor=white)](https://www.rust-lang.org/)

An unofficial desktop webview wrapper for Messenger, built with Tauri for low CPU and RAM usage.

## Why this exists

Messenger works in a browser, but using it there is not always convenient:

- You have to open the browser and navigate back to Messenger whenever you want to check a conversation.
- When many tabs are open, the Messenger tab can be difficult to find or easy to close by accident.
- Messenger has to compete with other tabs, extensions, and browser processes, which can make the experience feel heavier or less responsive.
- A browser tab does not feel as immediate as a dedicated app that can be launched from the desktop, taskbar, dock, or app menu.
- Keeping Messenger in its own window separates conversations from everyday browsing and makes switching between work and messages easier.
- Desktop-oriented controls such as the system tray, update reminders, image tools, and account switching make Messenger more convenient to use throughout the day.

This project exists to provide a focused, lightweight desktop home for Messenger without attempting to replace or reimplement the service itself.

## What makes it different

- **A dedicated Messenger window:** No need to search through browser tabs whenever a message arrives.
- **Lightweight by design:** Built with Tauri and the system WebView instead of bundling a complete Electron browser runtime.
- **The familiar Messenger experience:** The app loads Messenger directly, so conversations and supported web features remain familiar.
- **Useful desktop integration:** Includes system tray support, update notifications, calls, image copy and zoom, and user switching.
- **Cross-platform:** Release builds are available for Windows, macOS, and Linux.
- **Open source and transparent:** The source code and release workflow are available for inspection, and the project is licensed under MIT.

## Quick start

1. Open the [latest release](https://github.com/whitersun/messenger/releases/latest), or use one of the direct downloads below.
2. Download the installer for your operating system.
3. Install and launch the app.
4. Sign in on the Facebook/Messenger page displayed inside the app.
5. Optionally pin the app to your taskbar or dock for quick access.

> [!NOTE]
> This is an unofficial project. Authentication happens directly on Facebook/Meta pages; the app does not receive or store your password.

## Downloads

Install the latest available build:

| Platform | Download |
| --- | --- |
| Windows | [Download MSI](https://github.com/whitersun/messenger/releases/latest/download/unofficial-messenger-next-windows.msi) |
| macOS | [Download DMG](https://github.com/whitersun/messenger/releases/latest/download/unofficial-messenger-next-macos.dmg) |
| Linux (Debian/Ubuntu) | [Download DEB](https://github.com/whitersun/messenger/releases/latest/download/unofficial-messenger-next-linux.deb) |
| Linux (other distributions) | [Download AppImage](https://github.com/whitersun/messenger/releases/latest/download/unofficial-messenger-next-linux-x86_64.AppImage) |

You can also open the [latest release page](https://github.com/whitersun/messenger/releases/latest) and choose an installer from the assets.

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

## Disclaimer

This project is an unofficial, non-commercial desktop wrapper for Messenger Web.
It is not affiliated with, endorsed by, sponsored by, or connected to Meta Platforms, Inc.

Messenger, Facebook, Meta, and related names, logos, and trademarks are the property of Meta Platforms, Inc. They are referenced only to identify the web service this app opens.

This project does not collect, store, proxy, or modify login credentials. Authentication happens directly inside the Messenger/Facebook web page loaded in the system WebView.
