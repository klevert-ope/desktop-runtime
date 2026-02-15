# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2026-02-16

### Added

- **Tray icon:** System tray with Show and Quit menu items (icon shared with window).
- **Config persistence:** `config.json` in user data dir stores window bounds, theme, and generic key-value data.
- **Window bounds persistence:** Position and size saved on close, restored on startup.
- **ReadConfig / WriteConfig IPC:** UI can read and write arbitrary config keys.
- **Rayon worker pool:** Blocking IPC commands (file dialogs, update check, OpenUrl) run on a 4-thread pool instead of blocking the main loop.

### Changed

- **Module refactor:** `ipc.rs` split into `ipc/` (mod, updates, tests). New modules: `config`, `event_loop`, `paths`, `storage`, `window`.
- **Config centralization:** Window dimensions, IPC limits, env vars, embedded UI path moved to `config` module.

---

## [0.2.0] - 2026-02-08

### Added

- Window shown only after first page load (no white splash); 3s timeout fallback if load never fires.
- User data dir fallback to temp when platform dir unavailable; never pass `None` to WebView (avoids install-path fallback).
- Protocol: explicit `ServeResult` (Found/NotFound); HTTP status from result, not body.
- IPC: `OpenUrl` restricted to `http://` and `https://` only.
- DevTools opt-in: set `DESKTOP_RUNTIME_DEVTOOLS=1` to enable (off by default to avoid event-loop warnings).
- Build script: runs `npm install` and `npm run build` when `ui/dist/index.html` missing; build fails on npm failure. `DESKTOP_RUNTIME_GITHUB_REPO` for update-check repo.
- Unit tests for protocol (normalize_path, serve, MIME) and IPC (parse_message, semver_compare, OpenUrl validation).

### Changed

- Right-click context menu (Save, Print, etc.) disabled via page script.
- Main/protocol/IPC refactored: config constants, `exit_fatal`, `escape_json_for_js`, `user_data_dir()`, `run_event_loop`; protocol and IPC documented and tested.
- Event loop: explicit handling of `MainEventsCleared` and `RedrawEventsCleared`.
- Docs and README updated (env vars, build script, security, runtime behavior).

### Fixed

- Window could stay hidden if first page load failed; fallback timeout ensures it appears.
- Protocol 404 inferred from body bytes; now uses explicit `ServeResult::NotFound`.

---

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

[0.3.0]: https://github.com/klevert-ope/desktop-runtime/releases/tag/v0.3.0
[0.2.0]: https://github.com/klevert-ope/desktop-runtime/releases/tag/v0.2.0
[0.1.0]: https://github.com/klevert-ope/desktop-runtime/releases/tag/v0.1.0
