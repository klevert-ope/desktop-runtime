#!/bin/bash
# Desktop Runtime Installer - Guided install with EULA acceptance
# Usage: ./install-desktop-runtime.sh [path-to-AppImage]
# If no path given, looks for desktop-runtime*.AppImage in current directory.

set -e

APP_NAME="Desktop Runtime"
DEFAULT_INSTALL_DIR="${HOME}/.local/bin"
LICENSE_FILE="$(dirname "$0")/LICENSE.txt"
if [ ! -f "$LICENSE_FILE" ]; then
  LICENSE_FILE="$(dirname "$0")/../LICENSE.txt"
fi

# Detect GUI support
has_zenity() { command -v zenity >/dev/null 2>&1; }
has_dialog() { command -v dialog >/dev/null 2>&1; }
use_gui() { has_zenity || has_dialog; }

show_license_text() {
  if [ -f "$LICENSE_FILE" ]; then
    cat "$LICENSE_FILE"
  else
    echo "Desktop Runtime - Apache License, Version 2.0"
    echo ""
    echo "Copyright 2025 The Desktop Runtime contributors"
    echo ""
    echo "Licensed under the Apache License, Version 2.0."
    echo "See http://www.apache.org/licenses/LICENSE-2.0"
    echo ""
    echo "By installing, you agree to the terms of the Apache License 2.0."
  fi
}

prompt_accept_license() {
  if has_zenity; then
    show_license_text | zenity --text-info --title="$APP_NAME - License Agreement" \
      --width=600 --height=400 --checkbox="I have read and accept the license agreement"
    return $?
  elif has_dialog; then
    show_license_text > /tmp/desktop-runtime-license.txt
    dialog --title "$APP_NAME - License Agreement" --textbox /tmp/desktop-runtime-license.txt 22 76
    dialog --title "Accept License" --yesno "Do you accept the license agreement?" 6 50
    local ret=$?
    rm -f /tmp/desktop-runtime-license.txt
    return $ret
  else
    echo "=== $APP_NAME - License Agreement ==="
    echo ""
    show_license_text
    echo ""
    printf "Do you accept the license agreement? (y/n): "
    read -r answer
    case "$answer" in
      [yY]|[yY][eE][sS]) return 0 ;;
      *) return 1 ;;
    esac
  fi
}

prompt_install_dir() {
  if has_zenity; then
    zenity --entry --title="$APP_NAME - Install Location" \
      --text="Enter install directory:" \
      --entry-text="$DEFAULT_INSTALL_DIR"
  elif has_dialog; then
    dialog --stdout --title "$APP_NAME - Install Location" \
      --inputbox "Enter install directory:" 8 60 "$DEFAULT_INSTALL_DIR" 2>/dev/null
  else
    printf "Install directory [%s]: " "$DEFAULT_INSTALL_DIR"
    read -r answer
    echo "${answer:-$DEFAULT_INSTALL_DIR}"
    return 0
  fi
}

prompt_desktop_shortcut() {
  if has_zenity; then
    zenity --question --title="$APP_NAME" --text="Create desktop shortcut?" --default-cancel
    return $?
  elif has_dialog; then
    dialog --title "$APP_NAME" --yesno "Create desktop shortcut?" 6 40
    return $?
  else
    printf "Create desktop shortcut? (y/n) [y]: "
    read -r answer
    case "${answer:-y}" in
      [yY]|[yY][eE][sS]) return 0 ;;
      *) return 1 ;;
    esac
  fi
}

show_message() {
  local msg="$1"
  if has_zenity; then
    zenity --info --title="$APP_NAME" --text="$msg"
  elif has_dialog; then
    dialog --title "$APP_NAME" --msgbox "$msg" 8 50
  else
    echo "$msg"
  fi
}

# Find AppImage
APPIMAGE="$1"
if [ -z "$APPIMAGE" ]; then
  APPIMAGE=$(find . -maxdepth 1 -name 'desktop-runtime*.AppImage' -type f 2>/dev/null | head -1)
fi
if [ -z "$APPIMAGE" ] || [ ! -f "$APPIMAGE" ]; then
  echo "Error: No AppImage found. Usage: $0 <path-to-desktop-runtime.AppImage>" >&2
  exit 1
fi
APPIMAGE="$(realpath "$APPIMAGE")"

# EULA
if ! prompt_accept_license; then
  echo "Installation cancelled."
  exit 1
fi

# Install directory
INSTALL_DIR=$(prompt_install_dir | tr -d '\n' | xargs)
if [ -z "$INSTALL_DIR" ]; then
  echo "Installation cancelled."
  exit 1
fi

# Create directory
mkdir -p "$INSTALL_DIR"
INSTALL_PATH="$INSTALL_DIR/$(basename "$APPIMAGE")"
cp "$APPIMAGE" "$INSTALL_PATH"
chmod +x "$INSTALL_PATH"

# Desktop shortcut
if prompt_desktop_shortcut; then
  SHORTCUT_DIR="${HOME}/.local/share/applications"
  mkdir -p "$SHORTCUT_DIR"
  DESKTOP_IN="$(dirname "$0")/desktop-runtime.desktop.in"
  if [ -f "$DESKTOP_IN" ]; then
    sed "s|@EXEC@|$INSTALL_PATH|g" "$DESKTOP_IN" > "$SHORTCUT_DIR/desktop-runtime.desktop"
  else
    cat > "$SHORTCUT_DIR/desktop-runtime.desktop" << EOF
[Desktop Entry]
Type=Application
Name=$APP_NAME
Comment=Native desktop runtime with embedded UI
Exec=$INSTALL_PATH %U
Icon=react
Categories=Utility;
Terminal=false
EOF
  fi
  if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$SHORTCUT_DIR" 2>/dev/null || true
  fi
fi

show_message "Desktop Runtime has been installed to $INSTALL_PATH"
echo "Installation complete: $INSTALL_PATH"
