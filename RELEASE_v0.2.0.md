# Desktop Runtime v0.2.0

**Release date:** 2026-02-08

Second release: production-hardened runtime, no white splash, stricter security and build behavior.

---

## Downloads

| Platform   | Artifact |
|------------|----------|
| Windows   | `desktop-runtime-0.2.0-x86_64-pc-windows-msvc.msi` |
| macOS     | `desktop-runtime-0.2.0-{x86_64,aarch64}-apple-darwin.pkg` and `.tar.gz` |
| Linux     | `desktop-runtime-0.2.0-x86_64.AppImage` and `desktop-runtime-0.2.0-linux-installer.tar.gz` |

*(See the [Releases](https://github.com/klevert-ope/desktop-runtime/releases) page.)*

---

## Highlights

- **No white splash:** Window stays hidden until first page load (or 3s fallback) so the first frame is your UI.
- **User data never on install path:** WebView data dir is always user-writable (platform app data or temp); fallback to temp if preferred dir can’t be created.
- **Stricter protocol & IPC:** Protocol returns explicit HTTP status (no inference from body). `OpenUrl` accepts only `http://` and `https://`.
- **Context menu disabled:** Right-click no longer shows Save/Print/Inspect; opt-in DevTools via `DESKTOP_RUNTIME_DEVTOOLS=1`.
- **Build script:** Runs `npm install` and `npm run build` in `ui/` when `ui/dist/` is missing; build fails on npm failure. `DESKTOP_RUNTIME_GITHUB_REPO` for update-check repo.
- **Production refactor:** Main, protocol, and IPC rewritten for clarity, tests, and single source of truth for config and errors.

---

## Requirements

Unchanged from v0.1.0:

- **Windows:** WebView2 Runtime (typically preinstalled on Windows 10/11).
- **macOS:** 10.15+.
- **Linux:** WebKitGTK 4.1, GTK 3.

---

## License

Apache-2.0 OR MIT. See [LICENSE](LICENSE) and [packaging/LICENSE.txt](packaging/LICENSE.txt).
