# App icon

Place **`react.png`** here (512×512 or 1024×1024 PNG). This is the single source for all platform icons.

- **Windows:** `.ico` is generated from this file during `build-msi.ps1`.
- **macOS:** `.icns` is generated from this file during `build-app-pkg.sh`.
- **Linux:** This file is copied into the AppImage and used by the `.desktop` entry.

If this file is missing, packaging and the core app build will fail until it is added.
