#!/bin/bash
set -euo pipefail

APP_NAME="M Reader"
APP_BUNDLE_NAME="${APP_NAME}.app"
DIST_DIR="dist"

# Verify dx CLI is available
command -v dx >/dev/null 2>&1 || { echo "dx (Dioxus CLI) not found in PATH" >&2; exit 1; }

dx bundle --desktop --release --package-types macos

APP_PATH="$(find "${DIST_DIR}" -maxdepth 1 -type d -name "*.app" -print | head -n 1)"
if [[ -z "${APP_PATH}" ]]; then
  APP_PATH="$(find target/dx -type d -path "*/macos/*.app" -print | head -n 1)"
fi
if [[ -z "${APP_PATH}" ]]; then
  echo "No .app bundle found under dist/ or target/dx" >&2
  exit 1
fi

# Warn if multiple bundles exist — we'll use the first one found
COUNT="$(find "${DIST_DIR}" -maxdepth 1 -type d -name "*.app" | wc -l | tr -d ' ')"
[[ "${COUNT}" -gt 1 ]] && echo "Warning: multiple .app bundles found in ${DIST_DIR}/, using first" >&2

APP_DIR="$(dirname "${APP_PATH}")"
if [[ "$(basename "${APP_PATH}")" != "${APP_BUNDLE_NAME}" ]]; then
  rm -rf "${APP_DIR:?}/${APP_BUNDLE_NAME}"
  mv "${APP_PATH}" "${APP_DIR}/${APP_BUNDLE_NAME}"
  APP_PATH="${APP_DIR}/${APP_BUNDLE_NAME}"
fi

PLIST="${APP_PATH}/Contents/Info.plist"
if [[ -f "${PLIST}" ]]; then
  /usr/libexec/PlistBuddy -c "Set :CFBundleDisplayName ${APP_NAME}" "${PLIST}" 2>/dev/null \
    || /usr/libexec/PlistBuddy -c "Add :CFBundleDisplayName string ${APP_NAME}" "${PLIST}"
  /usr/libexec/PlistBuddy -c "Set :CFBundleName ${APP_NAME}" "${PLIST}" 2>/dev/null \
    || /usr/libexec/PlistBuddy -c "Add :CFBundleName string ${APP_NAME}" "${PLIST}"
fi

# Scope version extraction to the [package] section only
VERSION="$(awk -F'"' '/^\[package\]/,/^\[/ { if (/^version/) { print $2; exit } }' Cargo.toml)"
[[ -z "${VERSION}" ]] && { echo "Could not read version from Cargo.toml" >&2; exit 1; }
SHORT_VERSION="$(echo "${VERSION}" | cut -d. -f1,2)"

ARCH="$(uname -m)"
if [[ "${ARCH}" == "x86_64" ]]; then
  ARCH_SUFFIX="intel"
elif [[ "${ARCH}" == "arm64" ]]; then
  ARCH_SUFFIX="arm"
else
  ARCH_SUFFIX="${ARCH}"
fi

mkdir -p "${DIST_DIR}"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "${TMP_DIR}"' EXIT

cp -R "${APP_PATH}" "${TMP_DIR}/${APP_BUNDLE_NAME}"
ln -s /Applications "${TMP_DIR}/Applications"

# Use underscores in filename to avoid quoting issues in shells and CI
DMG_FILENAME="${APP_NAME// /_}_${SHORT_VERSION}_${ARCH_SUFFIX}.dmg"
DMG_PATH="${DIST_DIR}/${DMG_FILENAME}"
hdiutil create -volname "${APP_NAME}" -srcfolder "${TMP_DIR}" -ov -format UDZO "${DMG_PATH}" >/dev/null

echo "Bundled app: ${APP_PATH}"
echo "DMG: ${DMG_PATH}"
