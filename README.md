# Desktop Runtime

Native desktop runtime: Rust host (tao + wry), React UI, single binary. No Chromium, no Node at runtime. Uses the OS WebView and keeps the binary small.

## Quick Start

```bash
cd ui && npm install && npm run build && cd ..
cd core && cargo build --release
```

Binary: `core/target/release/desktop-runtime-core.exe` (Windows) or equivalent on macOS/Linux.

From repo root: `cargo run --manifest-path core/Cargo.toml --release`

## Architecture

| Layer | Tech |
|-------|------|
| Window/event loop | tao |
| WebView | wry (OS-provided) |
| UI | React + Vite, built to static assets |
| IPC | Typed commands over custom `app://` protocol |

UI assets are embedded at compile time via `include_dir` — no filesystem reads at runtime. IPC is a typed command enum; no `eval`, no dynamic dispatch.

See [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) for constraints and security. UI guidelines: [`docs/REACT.md`](docs/REACT.md).

## Design Constraints

- Idle RAM: &lt; 70 MB
- Binary size: Windows ≤ 15 MB, macOS ≤ 20 MB, Linux ≤ 12 MB
- No background threads without explicit owners
- `unsafe` forbidden in core (see `Cargo.toml` lints)

## Prerequisites

- **Rust** — stable (toolchain in `rust-toolchain.toml`)
- **Node.js + npm** — build-time only, for `ui/`

## Build Notes

1. **UI must be built first.** The core binary embeds `ui/dist/`. A minimal placeholder exists if you skip the UI build, but you’ll want a real build for development.
2. **Release profile** uses `opt-level = "z"`, LTO, and `strip = true` for size.
3. **Platforms:** Windows, macOS, Linux. wry picks the native WebView per OS.

## Tooling

| Task | Command |
|------|---------|
| Dependency audit | `cargo deny check` (from repo root) |
| Security audit | `cargo audit` |
| Size analysis | `cargo bloat --release -n 30` (from `core/`) |

## Installation (Release Builds)

Release artifacts include guided installers with license agreement:

| Platform | Format | Install flow |
|----------|--------|--------------|
| Windows | `.msi` | Wizard: Welcome → EULA → Install directory → Progress → Finish |
| macOS | `.pkg` | Installer: License → Destination → Install (`.app` tarball also available) |
| Linux | AppImage + installer | `tar xzf desktop-runtime-*-linux-installer.tar.gz` then `./install-desktop-runtime.sh ./desktop-runtime-*.AppImage` for guided install with EULA |

## Roadmap

Memory/leak validation, binary size review, packaging and signing (MSI/.app/AppImage), lockfile and versioning. See [`docs/BUILD.md`](docs/BUILD.md).

## License

Apache-2.0 OR MIT. See [LICENSE](LICENSE) for the Apache-2.0 text.
