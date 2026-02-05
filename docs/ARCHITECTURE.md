# Desktop Runtime — Architecture

## Engineering Constraints

| Constraint | Value |
|------------|--------|
| WebView | Native OS (wry); no Chromium |
| Stack | No Tauri / Electron / Node / JS runtime at run time |
| Binary | Single native binary |
| UI assets | Embedded in binary (zero IO at runtime) |
| IPC | Typed only; no eval, no dynamic commands |
| Idle RAM | < 70 MB |
| Binary size | < 15 MB (Windows); see BUILD.md for per-OS targets |
| Threads | Zero background threads without owners |

## Layout

- `core/` — Rust native runtime (tao event loop + wry WebView)
- `ui/` — React (Vite, production-only, JavaScript). See [REACT.md](REACT.md) for best practices (e.g. avoiding memory leaks).
- `docs/` — Architecture and process docs

Licensing: see root [LICENSE](../LICENSE) and [README](../README.md#license).

## Security

- Custom protocol `app://` only; no filesystem access at runtime
- CSP: `default-src 'self'; script-src 'self'; connect-src 'none';`
- No shell execution, plugin loading, or dynamic libraries
- Single IPC entry point with typed command enum and timeouts
