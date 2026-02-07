# Release checklist

1. **Version**
   - Set `version` in `core/Cargo.toml` (e.g. `0.1.0`).
   - Update `CHANGELOG.md`: add/complete the `[X.Y.Z]` section and release link at the bottom.

2. **Commit**
   - Commit all changes (including `CHANGELOG.md`).
   - Push to `origin/master`.

3. **Create the release (Option 1 – recommended)**
   - Go to **Actions** → **Create release tag** → **Run workflow**.
   - Enter the version (e.g. `0.1.0`) → **Run workflow**.
   - The workflow creates tag `v0.1.0` and pushes it. That triggers **Build and Release**, which builds all artifacts and creates the GitHub Release with `CHANGELOG.md` as the release notes.

   **Or (Option 2 – tag from your machine)**  
   - Create and push an annotated tag:
   ```bash
   git tag -a v0.1.0 -m "Desktop Runtime v0.1.0"
   git push origin v0.1.0
   ```

4. **What the CI does**
   - On any push to tag `v*`, the **Build and Release** workflow:
     - Builds Windows MSI, macOS .pkg/.tar.gz, Linux AppImage + installer.
     - Creates the GitHub Release for that tag and uploads all artifacts.
     - Uses `CHANGELOG.md` as the release body. Prerelease tags (e.g. `v0.2.0-beta.1`) are marked as prereleases.
