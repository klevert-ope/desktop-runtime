# Desktop Runtime v0.4.0

**Release date:** 2026-02-18

Fourth release: modernized packaging with a single source of truth, platform build scripts, WiX 4 on Windows, and multi-distro Linux support.

---

## Downloads

| Platform   | Artifact |
|------------|----------|
| Windows   | `desktop-runtime-0.4.0-x86_64-pc-windows-msvc.msi` |
| macOS     | `desktop-runtime-0.4.0-{x86_64,aarch64}-apple-darwin.pkg` and `.tar.gz` |
| Linux     | `desktop-runtime-0.4.0-x86_64.AppImage` and `desktop-runtime-0.4.0-linux-installer.tar.gz` |

*(See the [Releases](https://github.com/klevert-ope/desktop-runtime/releases) page.)*

---

## Highlights

### Packaging overhaul

- **Single source of truth:** App name, bundle ID, description, and Windows upgrade code live in `packaging/metadata.toml`. Version comes from git tag or `core/Cargo.toml`. CI writes `packaging/build-env.env` so all packagers use the same values.
- **Platform build scripts:** Packaging logic moved out of the workflow into scripts you can run locally:
  - **Windows:** `packaging/windows/build-msi.ps1` (WiX 4, optional .ico generation from PNG).
  - **macOS:** `packaging/macos/build-app-pkg.sh` (templates: `distribution.xml.in`, `Info.plist.in`).
  - **Linux:** `packaging/linux/build-appimage.sh` (AppImage + installer tarball using `desktop-runtime.desktop.in`).
- **Windows: WiX 4:** MSI is built with WiX 4 (`dotnet tool install --global wix`) instead of WiX 3. Icon: `react.ico` is generated from `packaging/icons/react.png` when missing.
- **macOS: Templates:** `distribution.xml.in` and `Info.plist.in` use placeholders (`@VERSION@`, `@BUNDLE_ID@`, `@APP_NAME@`, `@INSTALL_KBYTES@`); the build script substitutes them.
- **Linux: Multi-distro deps:** `packaging/linux/install-build-deps.sh` now supports Debian/Ubuntu (apt), Fedora/RHEL (dnf), and Arch (pacman) with equivalent package sets.
- **Linux: Single .desktop template:** `packaging/desktop-runtime.desktop.in` with `Exec=@EXEC@ %U` is used by both the AppImage build and `install-desktop-runtime.sh` so name, comment, categories, and icon stay in one place.
- **Icon pipeline:** Only `packaging/icons/react.png` is committed; `.ico` (Windows) and `.icns` (macOS) are generated at pack time. Documented in `packaging/README.md` and `packaging/icons/README.md`.
- **Packaging docs:** `packaging/README.md` describes layout, tools per platform, how to run packaging locally, supported Linux distros (with package lists), and the icon pipeline.

### No functional changes to the runtime

Runtime behavior, APIs, and requirements are unchanged from v0.3.0. This release is focused on build and distribution maintainability.

---

## Requirements

Unchanged from v0.3.0:

- **Windows:** WebView2 Runtime (typically preinstalled on Windows 10/11).
- **macOS:** 10.15+.
- **Linux:** WebKitGTK 4.1, GTK 3.

---

## License

Apache-2.0. See [LICENSE](LICENSE) and [packaging/LICENSE.txt](packaging/LICENSE.txt).
