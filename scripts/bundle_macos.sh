#!/bin/bash
set -euo pipefail

APP_NAME="M Reader"
APP_BUNDLE_NAME="${APP_NAME}.app"
DIST_DIR="dist"

dx bundle --desktop --release --package-types macos

APP_PATH="$(find "${DIST_DIR}" -maxdepth 1 -type d -name "*.app" -print | head -n 1)"
if [[ -z "${APP_PATH}" ]]; then
  APP_PATH="$(find target/dx -type d -path "*/macos/*.app" -print | head -n 1)"
fi
if [[ -z "${APP_PATH}" ]]; then
  echo "No .app bundle found under target/dx" >&2
  exit 1
fi

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

VERSION="$(awk -F\" '/^version/ {print $2; exit}' Cargo.toml)"
mkdir -p "${DIST_DIR}"

TMP_DIR="$(mktemp -d)"
cp -R "${APP_PATH}" "${TMP_DIR}/${APP_BUNDLE_NAME}"
ln -s /Applications "${TMP_DIR}/Applications"

DMG_PATH="${DIST_DIR}/${APP_NAME}_${VERSION}_x64.dmg"
hdiutil create -volname "${APP_NAME}" -srcfolder "${TMP_DIR}" -ov -format UDZO "${DMG_PATH}" >/dev/null
rm -rf "${TMP_DIR}"

echo "Bundled app: ${APP_PATH}"
echo "DMG: ${DMG_PATH}"
