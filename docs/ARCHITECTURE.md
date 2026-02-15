# Desktop Runtime — Architecture

## Constraints

| Constraint | Value |
|------------|--------|
| WebView | OS native (wry); no bundled Chromium |
| Stack | No Tauri / Electron / Node at runtime |
| Binary | Single native binary |
| UI assets | Embedded; no filesystem reads at runtime |
| IPC | Typed enum only; no eval, no dynamic commands |
| Idle RAM | < 70 MB |
| Binary size | See [BUILD.md](BUILD.md#binary-size) |
| Threads | No background threads without owners |

## Layout

- **core/** — Rust runtime (tao event loop, wry WebView, protocol, IPC).
  - `config` — Centralized constants (window dimensions, IPC limits, env vars, embedded UI path).
  - `event_loop` — User events, IPC queue drain, tray icon creation, window bounds save on close.
  - `ipc/` — Typed commands (mod, updates). Blocking commands run on a rayon worker pool.
  - `paths` — Platform-specific user data dir; cached via `OnceLock`.
  - `protocol` — `app://` serve, MIME, path normalization, CSP.
  - `storage` — Persistent `config.json` in user data dir (window bounds, theme, key-value).
  - `window` — Icon loading (window + tray), init script, `window.native` bridge.
- **ui/** — React (Vite, JS). See [REACT.md](REACT.md).
- **docs/** — Architecture and build.

## Security

- **Protocol:** `app://` only. Path traversal (`..`) rejected. HTTP status from protocol layer (no inference from body).
- **CSP:** `default-src 'self'; script-src 'self'; connect-src 'none';`
- **IPC:** Single entry point, typed commands. `OpenUrl` restricted to `http://` and `https://` only.
- **User data:** WebView data dir is always a user-writable path (platform app data or temp). Never the install directory. A `config.json` in that dir stores window bounds, theme, and generic key-value data (ReadConfig/WriteConfig IPC).
- No shell, plugins, or dynamic lib loading.

## Runtime behavior

- **Window:** Created hidden; shown after first page load (or after a short timeout if load never fires). Position and size persisted to `config.json` on close and restored on startup.
- **Tray icon:** System tray with Show/Quit menu (icon from same asset as window).
- **Context menu:** Default browser menu (Save, Print, etc.) disabled via page script.
- **DevTools:** Disabled unless `DESKTOP_RUNTIME_DEVTOOLS=1`.
- **IPC:** Blocking commands (file dialogs, update check, OpenUrl) run on a rayon worker pool (4 threads); non-blocking commands run inline. Backpressure: max 256 pending responses.
- **Accessibility:** OS a11y (UIA / VoiceOver / AT-SPI) via the WebView; no extra config.

## GPU / native rendering

- Video/WebGL/WebGPU run inside the WebView (hardware-accelerated where the OS supports it).
- For native GPU (e.g. custom engine), use a second tao window and wgpu; wry does not support embedding a native view in the same window as the WebView.
