#!/usr/bin/env bash
set -euo pipefail

# Installation script for Zettings on Kubuntu / Ubuntu Linux.
# Must be executed with root/sudo privileges to install to /usr and /opt.

if [ "$(id -u)" -ne 0 ]; then
  echo "[error] This script must be run as root or with sudo." >&2
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"
DIST_DIR="${ROOT_DIR}/dist"

if [ ! -d "${DIST_DIR}" ]; then
  echo "[info] Distribution bundle not found in ${DIST_DIR}. Running build script first..."
  "${SCRIPT_DIR}/build_kubuntu.sh"
fi

echo "[zettings] Installing Zettings to system locations..."

# 1. Install Flutter App to /opt/zettings and symlink to /usr/bin/zettings
mkdir -p /opt/zettings
cp -r "${DIST_DIR}/opt/zettings/"* /opt/zettings/
ln -sf /opt/zettings/zettings /usr/bin/zettings

# 2. Install Rust Backend Daemon to /usr/libexec/zettings-backend
mkdir -p /usr/libexec
cp "${DIST_DIR}/libexec/zettings-backend" /usr/libexec/zettings-backend
chmod 755 /usr/libexec/zettings-backend

# 3. Install D-Bus session service
mkdir -p /usr/share/dbus-1/services
cp "${DIST_DIR}/share/dbus-1/services/org.zyntrix.Zettings.service" /usr/share/dbus-1/services/

# 4. Install Systemd user service unit
mkdir -p /usr/lib/systemd/user
cp "${DIST_DIR}/lib/systemd/user/zsettings-backend.service" /usr/lib/systemd/user/

# 5. Install Polkit actions
mkdir -p /usr/share/polkit-1/actions
cp "${DIST_DIR}/share/polkit-1/actions/org.zyntrix.zettings.policy" /usr/share/polkit-1/actions/

# 6. Install Desktop file and icon
mkdir -p /usr/share/applications
cp "${DIST_DIR}/share/applications/org.zyntrix.zettings.desktop" /usr/share/applications/

if [ -f "${DIST_DIR}/share/icons/hicolor/512x512/apps/org.zyntrix.zettings.png" ]; then
  mkdir -p /usr/share/icons/hicolor/512x512/apps
  cp "${DIST_DIR}/share/icons/hicolor/512x512/apps/org.zyntrix.zettings.png" /usr/share/icons/hicolor/512x512/apps/
  command -v gtk-update-icon-cache >/dev/null 2>&1 && gtk-update-icon-cache -f -t /usr/share/icons/hicolor || true
fi

# Update desktop database
command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database || true

echo "[zettings] Installation finished successfully."
echo "[zettings] You can now launch Zettings from the KDE Plasma Application Launcher or by running 'zettings' in terminal."
