# Build and validation

## Linux system dependencies

All packages needed to build the Linux binary (wry/tao, webkit2gtk, soup, JavaScriptCore, GTK) are installed by one script so CI and local builds stay in sync. When a new Rust crate needs a system library, add the apt package to **`packaging/linux/install-build-deps.sh`** only; no workflow edits required.

**CI:** The release workflow runs `packaging/linux/install-build-deps.sh` on the Linux job.

**Local (Ubuntu/Debian):**
```bash
./packaging/linux/install-build-deps.sh
```
Then set `PKG_CONFIG_PATH` if needed (e.g. `export PKG_CONFIG_PATH=/usr/lib/x86_64-linux-gnu/pkgconfig`) and run `cargo build --release` from `core/`.

## Memory and leak validation

- **macOS:** Instruments (Leaks + VM)
- **Windows:** WPA + heap snapshot
- **Linux:** Valgrind / heaptrack

**Scenarios:** Open/close app 100×; idle 8 hours; heavy IPC; UI reload if supported.

**Tolerance:** < 1 MB growth over hours.

## Binary size

- Run `cargo bloat --release -n 30` from `core/`.
- Remove unused crates; replace heavy deps; strip symbols (release profile already has `strip = true`).

**Targets:**

| OS      | Size    |
| ------- | ------- |
| Windows | ≤ 15 MB |
| macOS   | ≤ 20 MB |
| Linux   | ≤ 12 MB |

## Packaging and signing

- **Windows:** MSI, signed.
- **macOS:** `.app`, hardened runtime, notarized.
- **Linux:** AppImage.
- No auto-updater initially.

## Long-term stability

- Dependency lockfile (`Cargo.lock`) committed.
- Update cadence defined; UI and core versioned independently.
- Backward-compatible IPC; crash-safe startup; deterministic builds where possible.
