#!/usr/bin/env bash
set -euo pipefail

# Build script for Zettings on Kubuntu / Ubuntu-based Linux distributions.
# Targets:
# - Flutter Release desktop binary (zettings)
# - Rust System Daemon (zettings-backend)
# - Packaging distribution tree

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"
DIST_DIR="${ROOT_DIR}/dist"

echo "[zettings] Starting build process for Kubuntu target..."

# 1. Check prerequisite toolchains
command -v flutter >/dev/null 2>&1 || {
  echo "[error] flutter command not found. Please install Flutter SDK and add it to PATH." >&2
  exit 1
}

command -v cargo >/dev/null 2>&1 || {
  echo "[error] cargo command not found. Please install Rust toolchain and add it to PATH." >&2
  exit 1
}

# 2. Build Flutter Linux application
echo "[zettings] Building Flutter Linux Release..."
cd "${ROOT_DIR}"
flutter pub get
flutter build linux --release

# 3. Build Rust System Daemon
echo "[zettings] Building Rust Backend Daemon Release..."
cd "${ROOT_DIR}/rust"
cargo build --release

# 4. Assemble distribution staging folder
echo "[zettings] Assembling distribution bundle in ${DIST_DIR}..."
rm -rf "${DIST_DIR}"
mkdir -p "${DIST_DIR}/bin"
mkdir -p "${DIST_DIR}/libexec"
mkdir -p "${DIST_DIR}/share/applications"
mkdir -p "${DIST_DIR}/share/dbus-1/services"
mkdir -p "${DIST_DIR}/share/polkit-1/actions"
mkdir -p "${DIST_DIR}/share/icons/hicolor/512x512/apps"
mkdir -p "${DIST_DIR}/lib/systemd/user"
mkdir -p "${DIST_DIR}/opt/zettings"

# Copy Flutter bundle
cp -r "${ROOT_DIR}/build/linux/x64/release/bundle/"* "${DIST_DIR}/opt/zettings/"
ln -sf "/opt/zettings/zettings" "${DIST_DIR}/bin/zettings"

# Copy Rust backend binary
cp "${ROOT_DIR}/rust/target/release/zettings-backend" "${DIST_DIR}/libexec/zettings-backend"

# Copy packaging configuration files
cp "${SCRIPT_DIR}/org.zyntrix.zettings.desktop" "${DIST_DIR}/share/applications/"
cp "${SCRIPT_DIR}/org.zyntrix.Zettings.service" "${DIST_DIR}/share/dbus-1/services/"
cp "${SCRIPT_DIR}/org.zyntrix.zettings.policy" "${DIST_DIR}/share/polkit-1/actions/"
cp "${SCRIPT_DIR}/zsettings-backend.service" "${DIST_DIR}/lib/systemd/user/"

# Copy icon if available
if [ -f "${ROOT_DIR}/Zettings-icon.png" ]; then
  cp "${ROOT_DIR}/Zettings-icon.png" "${DIST_DIR}/share/icons/hicolor/512x512/apps/org.zyntrix.zettings.png"
fi

echo "[zettings] Build completed successfully. Output ready in: ${DIST_DIR}"
