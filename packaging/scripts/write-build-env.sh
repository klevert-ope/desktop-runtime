#!/usr/bin/env bash
# Writes packaging/build-env.env from VERSION (arg or env) and packaging/metadata.toml.
# Run from repo root. CI then appends this file to GITHUB_ENV so all packaging steps get the vars.
set -e

REPO_ROOT="${REPO_ROOT:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$REPO_ROOT"
METADATA="${REPO_ROOT}/packaging/metadata.toml"
OUTFILE="${REPO_ROOT}/packaging/build-env.env"

VERSION="${1:-$VERSION}"
if [ -z "$VERSION" ]; then
  echo "Usage: $0 VERSION" >&2
  echo "  or set VERSION in the environment" >&2
  exit 1
fi

# Simple extraction of key = "value" from flat TOML (no nested tables)
get_metadata() {
  local key="$1"
  grep -E "^${key}\s*=" "$METADATA" | sed -n 's/^[^=]*=\s*"\(.*\)"\s*$/\1/p' | head -1
}

APP_NAME="$(get_metadata name)"
BUNDLE_ID="$(get_metadata bundle_id)"
DESCRIPTION="$(get_metadata description)"
MANUFACTURER="$(get_metadata manufacturer)"
UPGRADE_CODE="$(get_metadata upgrade_code)"

{
  echo "VERSION=$VERSION"
  echo "APP_NAME=$APP_NAME"
  echo "BUNDLE_ID=$BUNDLE_ID"
  echo "DESCRIPTION=$DESCRIPTION"
  echo "MANUFACTURER=$MANUFACTURER"
  echo "UPGRADE_CODE=$UPGRADE_CODE"
} > "$OUTFILE"

echo ">> Wrote $OUTFILE (VERSION=$VERSION, APP_NAME=$APP_NAME, BUNDLE_ID=$BUNDLE_ID)"
