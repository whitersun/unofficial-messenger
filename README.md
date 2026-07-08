# unofficial-messenger-next

An unofficial desktop webview wrapper for Messenger, built with Tauri for low CPU and RAM usage.

## Downloads

Install the latest build directly from the `v0.1.3` release:

| Platform | Download |
| --- | --- |
| Windows | [Download for Windows](https://github.com/whitersun/unofficial-messenger/releases/download/v0.1.3/unofficial-messenger-next-windows.msi) |
| macOS | [Download for macOS](https://github.com/whitersun/unofficial-messenger/releases/download/v0.1.3/unofficial-messenger-next-macos.dmg) |
| Linux | [Download for Linux](https://github.com/whitersun/unofficial-messenger/releases/download/v0.1.3/unofficial-messenger-next-linux.deb) |

You can also open the [v0.1.3 release page](https://github.com/whitersun/unofficial-messenger/releases/tag/v0.1.3) and download the installer for your operating system from the assets.

Release installers are built by GitHub Actions for Windows, macOS, and Linux when a version tag like `v0.1.0` is pushed.

## Features

- Built with Tauri, so it uses the system WebView instead of bundling a full browser runtime.
- Supports Messenger calls and video calls through the loaded Messenger web app.
- Supports image copy and image magnification when the underlying Edge/WebView runtime supports them. If the runtime does not provide those features, they will be added in an app release.

## Disclaimer

This project is an unofficial, non-commercial desktop wrapper for Messenger Web.
It is not affiliated with, endorsed by, sponsored by, or connected to Meta Platforms, Inc.

Messenger, Facebook, Meta, and related names, logos, and trademarks are the property of Meta Platforms, Inc. They are referenced only to identify the web service this app opens.

This project does not collect, store, proxy, or modify login credentials. Authentication happens directly inside the Messenger/Facebook web page loaded in the system WebView.
