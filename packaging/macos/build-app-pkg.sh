#!/usr/bin/env bash
# Build macOS .app bundle, .tar.gz, and .pkg installer.
# Run from repo root. Requires: VERSION, ARCH, TARGET (e.g. x86_64-apple-darwin or aarch64-apple-darwin).
# Optional: source packaging/build-env.env for APP_NAME, BUNDLE_ID (defaults: Desktop Runtime, io.desktop-runtime.app).
set -e

REPO_ROOT="${REPO_ROOT:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$REPO_ROOT"

VERSION="${VERSION:?VERSION is required}"
ARCH="${ARCH:?ARCH is required}"
TARGET="${TARGET:?TARGET is required}"
APP_NAME="${APP_NAME:-Desktop Runtime}"
BUNDLE_ID="${BUNDLE_ID:-io.desktop-runtime.app}"

BINARY="${REPO_ROOT}/target/${TARGET}/release/desktop-runtime-core"
ICON_PNG="${REPO_ROOT}/packaging/icons/react.png"
MACOS_RESOURCES="${REPO_ROOT}/packaging/macos/resources"
INFO_IN="${REPO_ROOT}/packaging/macos/Info.plist.in"
DIST_IN="${REPO_ROOT}/packaging/macos/distribution.xml.in"

if [ -f "${REPO_ROOT}/packaging/build-env.env" ]; then
  set -a
  # shellcheck source=/dev/null
  . "${REPO_ROOT}/packaging/build-env.env"
  set +a
  APP_NAME="${APP_NAME:-Desktop Runtime}"
  BUNDLE_ID="${BUNDLE_ID:-io.desktop-runtime.app}"
fi

if [ ! -f "$BINARY" ]; then
  echo "Error: Binary not found at $BINARY" >&2
  exit 1
fi

if [ ! -f "$ICON_PNG" ]; then
  echo "Error: $ICON_PNG not found" >&2
  exit 1
fi

echo ">> Packaging macOS .app for version $VERSION ($ARCH)"

mkdir -p "Desktop Runtime.app/Contents/MacOS"
cp "$BINARY" "Desktop Runtime.app/Contents/MacOS/desktop-runtime-core"
chmod +x "Desktop Runtime.app/Contents/MacOS/desktop-runtime-core"

# Info.plist from template
sed -e "s|@VERSION@|$VERSION|g" \
    -e "s|@BUNDLE_ID@|$BUNDLE_ID|g" \
    -e "s|@APP_NAME@|$APP_NAME|g" \
  "$INFO_IN" > "Desktop Runtime.app/Contents/Info.plist"

# Generate .icns from PNG
mkdir -p "Desktop Runtime.app/Contents/Resources" App.icns.iconset
for size in 16 32 64 128 256 512; do
  sips -z "$size" "$size" "$ICON_PNG" --out "App.icns.iconset/icon_${size}x${size}.png"
  sips -z $((size*2)) $((size*2)) "$ICON_PNG" --out "App.icns.iconset/icon_${size}x${size}@2x.png"
done
sips -z 1024 1024 "$ICON_PNG" --out "App.icns.iconset/icon_512x512@2x.png"
iconutil -c icns App.icns.iconset -o "Desktop Runtime.app/Contents/Resources/App.icns"
rm -rf App.icns.iconset

tar czf "desktop-runtime-${VERSION}-${ARCH}-apple-darwin.tar.gz" "Desktop Runtime.app"

# .pkg installer: background + welcome + license
cp "$ICON_PNG" "${MACOS_RESOURCES}/background.png"
pkgbuild --root "Desktop Runtime.app" --identifier "$BUNDLE_ID" --install-location /Applications "desktop-runtime.pkg"

# installKBytes: approximate size of .app for installer UI
INSTALL_KBYTES="${INSTALL_KBYTES:-20000}"
if command -v du >/dev/null 2>&1; then
  INSTALL_KBYTES=$(du -sk "Desktop Runtime.app" | cut -f1)
fi

sed -e "s|@VERSION@|$VERSION|g" \
    -e "s|@INSTALL_KBYTES@|$INSTALL_KBYTES|g" \
    -e "s|@APP_NAME@|$APP_NAME|g" \
    -e "s|@BUNDLE_ID@|$BUNDLE_ID|g" \
  "$DIST_IN" > distribution.xml

productbuild --distribution distribution.xml --resources "$MACOS_RESOURCES" --package-path . "desktop-runtime-${VERSION}-${ARCH}-apple-darwin.pkg"

echo ">> Produced: desktop-runtime-${VERSION}-${ARCH}-apple-darwin.tar.gz, desktop-runtime-${VERSION}-${ARCH}-apple-darwin.pkg"
