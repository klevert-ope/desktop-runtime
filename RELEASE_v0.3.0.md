# Desktop Runtime v0.3.0

**Release date:** 2026-02-16

Third release: tray icon, config persistence, window bounds restore, rayon worker pool for blocking IPC.

---

## Downloads

| Platform   | Artifact |
|------------|----------|
| Windows   | `desktop-runtime-0.3.0-x86_64-pc-windows-msvc.msi` |
| macOS     | `desktop-runtime-0.3.0-{x86_64,aarch64}-apple-darwin.pkg` and `.tar.gz` |
| Linux     | `desktop-runtime-0.3.0-x86_64.AppImage` and `desktop-runtime-0.3.0-linux-installer.tar.gz` |

*(See the [Releases](https://github.com/klevert-ope/desktop-runtime/releases) page.)*

---

## Highlights

- **Tray icon:** System tray with Show and Quit menu items. Icon shared with window.
- **Config persistence:** `config.json` in user data dir stores window bounds, theme, and arbitrary key-value data. ReadConfig/WriteConfig IPC for UI access.
- **Window bounds:** Position and size saved on close, restored on startup.
- **Rayon worker pool:** Blocking commands (file dialogs, update check, OpenUrl) run on a 4-thread pool instead of blocking the main loop.
- **Module refactor:** `ipc` split into `ipc/` (mod, updates, tests). New modules: `config`, `event_loop`, `paths`, `storage`, `window`. Config constants centralized.

---

## Requirements

Unchanged from v0.2.0:

- **Windows:** WebView2 Runtime (typically preinstalled on Windows 10/11).
- **macOS:** 10.15+.
- **Linux:** WebKitGTK 4.1, GTK 3.

---

## License

Apache-2.0 OR MIT. See [LICENSE](LICENSE) and [packaging/LICENSE.txt](packaging/LICENSE.txt).
