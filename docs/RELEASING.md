# Releasing unofficial-messenger-next

This document is for project maintainers. User-facing installation instructions remain in the main README.

## Local builds

The default Tauri configuration does not create updater artifacts, so a normal local installer build does not require the updater private key:

```powershell
bun install --frozen-lockfile
bun run tauri build
```

Build a specific Windows installer format with:

```powershell
bun run tauri build --bundles msi
```

If Tauri reports a JavaScript/Rust plugin version mismatch after dependencies were removed or upgraded, delete only the stale package from `node_modules` and run `bun install --frozen-lockfile` again. A clean checkout used by GitHub Actions does not retain undeclared packages.

## GitHub releases

The release workflow uses `src-tauri/tauri.release.conf.json` to enable signed updater artifacts. Configure these repository secrets before creating a release:

- `TAURI_SIGNING_PRIVATE_KEY`: the private updater key matching the public key configured in `src-tauri/tauri.conf.json`.
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`: the private key password, or an empty value when the key has no password.

Never commit the private updater key or its password.

Keep the version in these files synchronized before creating a tag:

- `package.json`
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`

Push a `v*` tag to start the release workflow. It builds platform installers, creates updater signatures, generates `latest.json`, and publishes the files to GitHub Releases.

The macOS job must build both the `app` and `dmg` targets. The `app` target produces the `.app.tar.gz` updater artifact, while the `dmg` target produces the user-facing installer.
