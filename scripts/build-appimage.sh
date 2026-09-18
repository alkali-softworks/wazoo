#!/usr/bin/env bash
set -euo pipefail

# ---------------------------------------------------------------------------
# ALKALI SOFTWORKS - Wazoo AppImage Packaging Script
#
# Bundles Wazoo into a single self-contained executable (.AppImage) for Linux.
# ---------------------------------------------------------------------------

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
DIST_DIR="${ROOT_DIR}/dist"
APP_DIR="${ROOT_DIR}/target/AppDir"

echo "==> Building release binary..."
cargo build --release --manifest-path "${ROOT_DIR}/Cargo.toml"

echo "==> Preparing AppDir at ${APP_DIR}..."
rm -rf "${APP_DIR}"
mkdir -p "${APP_DIR}/usr/bin"
mkdir -p "${APP_DIR}/usr/share/icons/hicolor/256x256/apps"
mkdir -p "${DIST_DIR}"

# 1. Copy binary
cp "${ROOT_DIR}/target/release/wazoo" "${APP_DIR}/usr/bin/wazoo"
chmod +x "${APP_DIR}/usr/bin/wazoo"

# 2. Copy application icons
cp "${ROOT_DIR}/crates/wazoo-app/resources/icon.png" "${APP_DIR}/wazoo.png"
cp "${ROOT_DIR}/crates/wazoo-app/resources/icon.png" "${APP_DIR}/usr/share/icons/hicolor/256x256/apps/wazoo.png"

# 3. Create desktop entry
cat << 'EOF' > "${APP_DIR}/wazoo.desktop"
[Desktop Entry]
Name=Wazoo
Comment=Ambient media engine for non-stop viewing
Exec=wazoo %U
Icon=wazoo
Terminal=false
Type=Application
Categories=AudioVideo;Player;Video;
StartupWMClass=wazoo
MimeType=video/x-matroska;video/mp4;video/webm;video/x-msvideo;video/quicktime;
EOF

# 4. Create AppRun launcher
cat << 'EOF' > "${APP_DIR}/AppRun"
#!/bin/sh
HERE="$(dirname "$(readlink -f "${0}")")"
export PATH="${HERE}/usr/bin:${PATH}"
export LD_LIBRARY_PATH="${HERE}/usr/lib:${HERE}/usr/lib/x86_64-linux-gnu:${LD_LIBRARY_PATH:-}"
export WAZOO_APPIMAGE=1
exec "${HERE}/usr/bin/wazoo" "$@"
EOF
chmod +x "${APP_DIR}/AppRun"

# 5. Build AppImage using appimagetool
APPIMAGE_TOOL=""
if command -v appimagetool &> /dev/null; then
    APPIMAGE_TOOL="appimagetool"
elif [ -f "${ROOT_DIR}/target/appimagetool" ]; then
    APPIMAGE_TOOL="${ROOT_DIR}/target/appimagetool"
fi

if [ -z "${APPIMAGE_TOOL}" ]; then
    echo "==> Fetching appimagetool..."
    curl -sLo "${ROOT_DIR}/target/appimagetool" "https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage" || true
    if [ -f "${ROOT_DIR}/target/appimagetool" ]; then
        chmod +x "${ROOT_DIR}/target/appimagetool"
        APPIMAGE_TOOL="${ROOT_DIR}/target/appimagetool"
    fi
fi

OUTPUT_APPIMAGE="${DIST_DIR}/wazoo-x86_64.AppImage"
if [ -n "${APPIMAGE_TOOL}" ]; then
    echo "==> Packaging into ${OUTPUT_APPIMAGE}..."
    ARCH=x86_64 "${APPIMAGE_TOOL}" --appimage-extract-and-run "${APP_DIR}" "${OUTPUT_APPIMAGE}" || \
    ARCH=x86_64 "${APPIMAGE_TOOL}" "${APP_DIR}" "${OUTPUT_APPIMAGE}"
    chmod +x "${OUTPUT_APPIMAGE}"
    echo "==> Successfully created single executable AppImage: ${OUTPUT_APPIMAGE}"
else
    echo "==> Notice: appimagetool could not be run in this environment."
    echo "    AppDir structure generated at: ${APP_DIR}"
fi
