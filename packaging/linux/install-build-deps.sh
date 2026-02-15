#!/usr/bin/env bash
# Install all system dependencies required to build desktop-runtime-core on Linux
# (wry/tao, webkit2gtk, soup2/soup3, JavaScriptCore, GTK). Single source of truth:
# when a new crate needs a system lib, add it here so CI and local builds stay in sync.
set -e

echo "Installing Linux build dependencies..."

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

echo "Linux build dependencies installed."
