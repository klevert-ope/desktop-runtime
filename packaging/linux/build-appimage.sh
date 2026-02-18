#!/usr/bin/env bash
# Build Linux AppImage and linux-installer tarball.
# Run from repo root. Requires: VERSION, and optionally TARGET (default x86_64-unknown-linux-gnu).
# Binary must exist at target/$TARGET/release/desktop-runtime-core.
set -e

REPO_ROOT="${REPO_ROOT:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$REPO_ROOT"

VERSION="${VERSION:?VERSION is required}"
TARGET="${TARGET:-x86_64-unknown-linux-gnu}"
BINARY="${REPO_ROOT}/target/${TARGET}/release/desktop-runtime-core"
APPIMAGETOOL_VERSION="1.9.1"
CACHE_DIR="${REPO_ROOT}/.cache/appimagetool"
APPIMAGETOOL="${CACHE_DIR}/appimagetool-x86_64.AppImage"

if [ ! -f "$BINARY" ]; then
  echo "Error: Binary not found at $BINARY" >&2
  exit 1
fi

if [ ! -f "${REPO_ROOT}/packaging/icons/react.png" ]; then
  echo "Error: packaging/icons/react.png not found (required for AppImage icon)" >&2
  exit 1
fi

echo ">> Packaging Linux AppImage for version $VERSION (target $TARGET)"

mkdir -p AppDir/usr/bin "$CACHE_DIR"
cp "$BINARY" AppDir/usr/bin/
chmod +x AppDir/usr/bin/desktop-runtime-core

# Use desktop template: for AppDir, Exec is the path inside the AppDir
sed "s|@EXEC@|usr/bin/desktop-runtime-core|g" \
  "${REPO_ROOT}/packaging/desktop-runtime.desktop.in" > AppDir/desktop-runtime.desktop
cp "${REPO_ROOT}/packaging/icons/react.png" AppDir/react.png

if [ ! -f "$APPIMAGETOOL" ]; then
  echo ">> Downloading appimagetool $APPIMAGETOOL_VERSION"
  curl -fsSL -o "$APPIMAGETOOL" \
    "https://github.com/AppImage/appimagetool/releases/download/${APPIMAGETOOL_VERSION}/appimagetool-x86_64.AppImage"
  chmod +x "$APPIMAGETOOL"
fi

ARCH=x86_64 "$APPIMAGETOOL" --no-appstream AppDir "desktop-runtime-${VERSION}-x86_64.AppImage"

# Bundle installer script and license for guided install flow
mkdir -p linux-installer
cp "${REPO_ROOT}/packaging/linux/install-desktop-runtime.sh" linux-installer/
cp "${REPO_ROOT}/packaging/LICENSE.txt" linux-installer/
cp "${REPO_ROOT}/packaging/desktop-runtime.desktop.in" linux-installer/
chmod +x linux-installer/install-desktop-runtime.sh
tar czf "desktop-runtime-${VERSION}-linux-installer.tar.gz" -C linux-installer \
  install-desktop-runtime.sh LICENSE.txt desktop-runtime.desktop.in

echo ">> Produced: desktop-runtime-${VERSION}-x86_64.AppImage, desktop-runtime-${VERSION}-linux-installer.tar.gz"
