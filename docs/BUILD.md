# Build and validation

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
