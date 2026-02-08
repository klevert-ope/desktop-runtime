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
- **ui/** — React (Vite, JS). See [REACT.md](REACT.md).
- **docs/** — Architecture and build.

## Security

- **Protocol:** `app://` only. Path traversal (`..`) rejected. HTTP status from protocol layer (no inference from body).
- **CSP:** `default-src 'self'; script-src 'self'; connect-src 'none';`
- **IPC:** Single entry point, typed commands. `OpenUrl` restricted to `http://` and `https://` only.
- **User data:** WebView data dir is always a user-writable path (platform app data or temp). Never the install directory.
- No shell, plugins, or dynamic lib loading.

## Runtime behavior

- **Window:** Created hidden; shown after first page load (or after a short timeout if load never fires).
- **Context menu:** Default browser menu (Save, Print, etc.) disabled via page script.
- **DevTools:** Disabled unless `DESKTOP_RUNTIME_DEVTOOLS=1`.
- **Accessibility:** OS a11y (UIA / VoiceOver / AT-SPI) via the WebView; no extra config.

## GPU / native rendering

- Video/WebGL/WebGPU run inside the WebView (hardware-accelerated where the OS supports it).
- For native GPU (e.g. custom engine), use a second tao window and wgpu; wry does not support embedding a native view in the same window as the WebView.
