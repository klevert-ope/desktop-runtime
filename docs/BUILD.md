# Build and validation

## Build script (core/build.rs)

- **GitHub repo:** Sets `GITHUB_REPO_FOR_UPDATES` for the crate. Override with `DESKTOP_RUNTIME_GITHUB_REPO` (e.g. `owner/repo`); else derived from `CARGO_PKG_REPOSITORY` or default.
- **UI:** If `ui/dist/index.html` is missing, runs `npm install` then `npm run build` in `ui/`. Non-zero exit or missing `npm` fails the build.
- **Rerun:** Script reruns when `../ui` sources, `package.json`, lockfile, or `../packaging/icons/react.png` change.

## Linux system dependencies

Install WebKit/GTK deps so wry/tao build. CI and local use the same script.

```bash
./packaging/linux/install-build-deps.sh
```

Set `PKG_CONFIG_PATH` if needed, then `cargo build --release` from `core/`.

## Memory and leak validation

- **macOS:** Instruments (Leaks + VM)
- **Windows:** WPA + heap snapshot
- **Linux:** Valgrind / heaptrack

Scenarios: open/close 100×; idle 8 h; heavy IPC. Target: < 1 MB growth over hours.

## Binary size

`cargo bloat --release -n 30` from `core/`. Release profile uses `strip = true`.

| OS      | Target  |
|---------|---------|
| Windows | ≤ 15 MB |
| macOS   | ≤ 20 MB |
| Linux   | ≤ 12 MB |

## Packaging and signing

- **Windows:** MSI, signed.
- **macOS:** `.app`, hardened runtime, notarized.
- **Linux:** AppImage.

## Stability

- `Cargo.lock` committed. IPC backward-compatible; crash-safe startup.
