# Desktop Runtime

Native desktop runtime: Rust host (tao + wry), React UI, single binary. OS WebView only — no Chromium, no Node at runtime.

## Quick Start

```bash
cd core && cargo build --release
```

The build script runs `npm install` and `npm run build` in `ui/` if `ui/dist/index.html` is missing. Ensure Node.js and npm are installed.

Binary: `core/target/release/desktop-runtime-core.exe` (Windows) or equivalent on macOS/Linux.

From repo root: `cargo run --manifest-path core/Cargo.toml --release`

## Architecture

| Layer | Tech |
|-------|------|
| Window/event loop | tao |
| WebView | wry (OS-provided) |
| UI | React + Vite, built to static assets |
| IPC | Typed commands over `app://` protocol |

UI assets are embedded at compile time (`include_dir`). IPC is a typed command enum; no eval, no dynamic dispatch. Blocking commands (file dialogs, updates, OpenUrl) run on a rayon worker pool; backpressure capped at 256 pending responses. Window is shown after first page load (with a short timeout fallback); position and size persist to `config.json` on close. System tray icon with Show/Quit menu. Right-click context menu (Save/Print) is disabled.

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) and [docs/BUILD.md](docs/BUILD.md).

## Environment

| Variable | Effect |
|----------|--------|
| `DESKTOP_RUNTIME_DEVTOOLS=1` | Enable WebView DevTools (off by default to avoid event-loop warnings). |
| `DESKTOP_RUNTIME_GITHUB_REPO` | Build-time: `owner/repo` for update checks. Defaults from `CARGO_PKG_REPOSITORY` or `klevert-ope/desktop-runtime`. |

## Design Constraints

- Idle RAM: < 70 MB
- Binary size: Windows ≤ 15 MB, macOS ≤ 20 MB, Linux ≤ 12 MB
- No background threads without explicit owners
- `unsafe` forbidden in core

## Prerequisites

- **Rust** — stable (see `rust-toolchain.toml`)
- **Node.js + npm** — build-time only; required if `ui/dist/` is missing (build script will run npm).

## Build Notes

1. **UI:** Build script ensures `ui/dist/` exists before linking. If missing, it runs `npm install` and `npm run build` in `ui/`; failures fail the build.
2. **Release:** `opt-level = "z"`, LTO, `strip = true`.
3. **Platforms:** Windows, macOS, Linux. wry uses the OS WebView (WebView2, WKWebView, WebKitGTK).

## Tooling

| Task | Command |
|------|---------|
| Dependency audit | `cargo deny check` (repo root) |
| Security audit | `cargo audit` |
| Size analysis | `cargo bloat --release -n 30` (from `core/`) |

## Installation (Release Builds)

| Platform | Format |
|----------|--------|
| Windows | `.msi` wizard |
| macOS | `.pkg` or `.app` tarball |
| Linux | AppImage + `install-desktop-runtime.sh` |

## License

Apache-2.0 . See [LICENSE](LICENSE).
