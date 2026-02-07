# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-02-06

### Added

- **Desktop runtime**: Native host (Rust, tao + wry), embedded React UI, single binary. No Chromium, no Node at runtime.
- **Windows**
  - MSI installer with EULA, app icon (Add/Remove Programs + Start Menu), elevated install.
  - GUI-only process (no console window in release).
  - WebView2 user data in `%LOCALAPPDATA%\Desktop Runtime\WebView2` (no writes under Program Files).
- **macOS**
  - .pkg installer with welcome, license, background image, and install destination choice.
  - .app bundle with icon generated from packaging assets; .tar.gz also provided.
  - WebView/user data in `~/Library/Application Support/Desktop Runtime`.
- **Linux**
  - AppImage and guided installer script with EULA (zenity/dialog/CLI).
  - Dual-license text (Apache 2.0 + MIT) in installer and packaging.
- **IPC**
  - Typed command enum over custom `app://` protocol (no eval, no string dispatch).
  - Backpressure: cap of 256 pending IPC responses to avoid flooding the event loop.
- **Logging**
  - `log` + `env_logger`; default level `info` (debug) / `warn` (release). Configurable via `RUST_LOG`.
- **Update check**
  - Check for updates via GitHub Releases API (`klevert-ope/desktop-runtime`).

### Security / Reliability

- Path traversal rejected in `app://` protocol (paths containing `..` return 404).
- Graceful exit with message on window/webview creation failure (no panic).
- Protocol handler fallback to 500 response if HTTP response build fails.
- User data directories only used when `create_dir_all` succeeds.

### Fixed

- GitHub update-check URL: fallback repo corrected from `example/desktop-runtime` to `klevert-ope/desktop-runtime`.

---

[0.1.0]: https://github.com/klevert-ope/desktop-runtime/releases/tag/v0.1.0
