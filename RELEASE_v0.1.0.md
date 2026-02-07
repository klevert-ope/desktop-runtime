# Desktop Runtime v0.1.0

**Release date:** 2025-02-06

First stable release of Desktop Runtime: a native desktop host (Rust, tao + wry) with an embedded React UI in a single binary. No Chromium and no Node at runtime; uses the OS WebView.

---

## Downloads

| Platform   | Artifact |
|-----------|----------|
| Windows   | `desktop-runtime-0.1.0-x86_64-pc-windows-msvc.msi` |
| macOS     | `desktop-runtime-0.1.0-{x86_64,aarch64}-apple-darwin.pkg` and `.tar.gz` |
| Linux     | `desktop-runtime-0.1.0-x86_64.AppImage` and `desktop-runtime-0.1.0-linux-installer.tar.gz` |

*(Exact filenames may vary by CI; see the [Releases](https://github.com/klevert-ope/desktop-runtime/releases) page.)*

---

## Highlights

- **Single binary**: React UI built to static assets and embedded at compile time; no runtime filesystem reads for UI.
- **Small footprint**: Release profile tuned for size (LTO, strip, opt-level z); target idle RAM &lt; 70 MB.
- **Installers**: Windows MSI (with EULA and app icon), macOS .pkg (welcome, license, destination choice), Linux AppImage + guided installer script with EULA.
- **IPC**: Typed commands over a custom `app://` protocol; backpressure and logging for robustness.
- **Update check**: Optional check for newer versions via GitHub Releases.

---

## Requirements

- **Windows**: WebView2 Runtime (usually preinstalled on Windows 10/11).
- **macOS**: 10.15+.
- **Linux**: WebKitGTK 4.1, GTK 3.

---

## License

Apache-2.0 OR MIT. See [LICENSE](LICENSE) and [packaging/LICENSE.txt](packaging/LICENSE.txt).
