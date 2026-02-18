#!/usr/bin/env bash
# Install all system dependencies required to build desktop-runtime-core on Linux
# (wry/tao, webkit2gtk, soup2/soup3, JavaScriptCore, GTK). Single source of truth:
# when a new crate needs a system lib, add it here so CI and local builds stay in sync.
# Supports: Debian/Ubuntu (apt), Fedora/RHEL (dnf), Arch (pacman).
set -e

echo "Installing Linux build dependencies..."

if command -v apt-get >/dev/null 2>&1; then
  sudo apt-get update
  sudo apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libgtk-3-dev \
    libgdk-pixbuf2.0-dev \
    librsvg2-dev \
    libwebkit2gtk-4.0-dev \
    libwebkit2gtk-4.1-dev \
    libjavascriptcoregtk-4.1-dev \
    libsoup-3.0-dev \
    libsoup2.4-dev \
    libayatana-appindicator3-dev \
    libxdo-dev \
    patchelf
elif command -v dnf >/dev/null 2>&1; then
  sudo dnf install -y \
    gcc gcc-c++ make pkg-config \
    openssl-devel \
    gtk3-devel \
    gdk-pixbuf2-devel \
    librsvg2-devel \
    webkit2gtk4.1-devel \
    libsoup3-devel \
    libsoup-devel \
    libappindicator-gtk3-devel \
    libxdo-devel \
    patchelf
elif command -v pacman >/dev/null 2>&1; then
  sudo pacman -S --noconfirm --needed \
    base-devel \
    pkg-config \
    openssl \
    gtk3 \
    gdk-pixbuf2 \
    librsvg \
    webkit2gtk-4.1 \
    libsoup3 \
    libsoup \
    libayatana-appindicator3 \
    libxdo \
    patchelf
else
  echo "Unsupported package manager. This script supports: apt-get (Debian/Ubuntu), dnf (Fedora/RHEL), pacman (Arch)." >&2
  echo "See packaging/README.md for manual dependency lists." >&2
  exit 1
fi

echo "Linux build dependencies installed."
