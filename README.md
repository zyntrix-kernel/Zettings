# ZETTINGS — Zyntrix OS System Settings

[![CI](https://github.com/zyntrix-kernel/Zettings/actions/workflows/ci.yml/badge.svg)](https://github.com/zyntrix-kernel/Zettings/actions/workflows/ci.yml)
[![License: GPL-3.0-or-later](https://img.shields.io/badge/License-GPL--3.0--or--later-blue.svg)](LICENSE)
[![Flutter Tests](https://img.shields.io/badge/Flutter%20Tests-107%20passing-brightgreen.svg)](#verification-status)
[![Rust Tests](https://img.shields.io/badge/Rust%20Tests-42%20passing-brightgreen.svg)](#verification-status)
[![Platform](https://img.shields.io/badge/Platform-Linux%20%7C%20KDE%20Plasma-orange.svg)](#architecture-overview)

ZETTINGS is the next-generation system settings application for **Zyntrix OS**, built for Kubuntu and KDE Plasma. It pairs the elegance and accessibility of Flutter and Material 3 with the performance, safety, and least-privilege security of a native Rust backend communicating over D-Bus.

---

## Architecture Overview

```text
Flutter UI (Material 3 + ZDL)
    ↓ Typed calls (no shell execution from UI)
Application Layer (Registry, Search, Router, AppState)
    ↓ Typed D-Bus IPC client (org.zyntrix.Zettings)
Rust Backend Daemon (tokio + zbus)
    ↓ Typed system services & PolicyKit authorization
Linux / KDE Subsystems (NetworkManager, PipeWire, BlueZ, UPower, systemd, AccountsService)
```

- **Frontend:** Flutter Desktop, Dart, Material 3 (`useMaterial3: true`), Zyntrix Design Language (ZDL) tokens, full screen-reader and keyboard semantics.
- **Backend:** Rust, `tokio` async runtime, `zbus` D-Bus interface, sysfs/procfs inspection, PolicyKit integration.
- **Isolation:** Zero arbitrary command execution. Strict typed requests and typed error models (`Unavailable`, `Unsupported`, `PermissionDenied`, `InvalidInput`, `Timeout`, `ServiceUnavailable`, `HardwareUnavailable`, `SystemError`).

---

## Settings Surface

1. **System:** Operating system snapshot, kernel, uptime, CPU, RAM utilization, primary display resolution & scaling ($100\%-200\%$), Night light, master sound volume & mute, PipeWire sinks, battery status, power profiles (`Performance`, `Balanced`, `Power saver`), storage mounts, window snapping, active corners, and Do Not Disturb.
2. **Network:** Wi-Fi scanning & connect, Ethernet interface status, WireGuard VPN toggles, and firewall overview.
3. **Bluetooth & Devices:** Adapter power toggle, paired peripherals (headsets, mice, keyboards) with live battery indicators.
4. **Personalization:** Light, Dark, and true OLED black modes, 7 dynamic accent color seeds, typography scaling, cursor themes, and wallpaper selection.
5. **Accessibility:** Vision adjustments, high contrast mode, screen reader support, mouse cursor size, hearing mono audio, alert flashing, mobility sticky/slow keys, and global **reduced motion** synchronization.
6. **Accounts:** Local human accounts parsing, profile details, password change workflows, and admin privilege controls.
7. **Privacy & Security:** XDG portal permissions (location, camera, microphone, screencast), PolicyKit rule inspection, and firewall status.
8. **Applications:** Default handlers (browser, email, terminal, file manager), autostart desktop entries, Flatpak / Snap runtime diagnostics.
9. **Updates:** System package status, APT repository source management, firmware (fwupd / LVFS) updates.
10. **Developer:** Live Rust IPC roundtrip latency diagnostics, D-Bus session bus monitor, systemd journal viewer, performance overlays, and semantics inspector.

---

## Building and Testing

### Prerequisites
- Flutter SDK (≥ 3.24)
- Rust toolchain (`cargo`, `rustc` ≥ 1.80)
- Linux desktop dependencies (or web/desktop development tooling)

### Flutter UI

```bash
# Run static analysis
flutter analyze

# Verify code formatting
dart format --output=none --set-exit-if-changed .

# Run the full test suite (99 tests)
flutter test

# Build for Web
flutter build web

# Build for Linux Desktop
flutter build linux
```

### Rust Daemon

```bash
cd rust

# Check formatting
cargo fmt --all --check

# Run linter
cargo clippy --workspace --all-targets -- -D warnings

# Run all unit and integration tests (37 tests)
cargo test --workspace

### Linux Desktop & Kubuntu Deployment

To build and install Zettings and its system daemon on Kubuntu / Ubuntu:

```bash
# 1. Install system prerequisites on Kubuntu
sudo apt update
sudo apt install clang cmake ninja-build pkg-config libgtk-3-dev liblzma-dev libdbus-1-dev

# 2. Build both Flutter app and Rust daemon
./linux/packaging/build_kubuntu.sh

# 3. Install to system locations (/opt/zettings, /usr/libexec, D-Bus, systemd, Polkit)
sudo ./linux/packaging/install.sh
```

---

## Verification Status

- Rust Backend: 42 / 42 tests passing. 0 clippy warnings. Clean formatting (`cargo fmt`).
- Flutter UI: 107 / 107 tests passing. 0 analyzer issues (`flutter analyze`). Clean formatting (`dart format`).
- Architecture: Zero files exceeding 10KB. Decomposed into modular Rust and Dart components.
- Input Validation: Full typed range, length, and format validation on both Rust IPC daemon boundary and Flutter UI presentation layer.
- Community Standards: GPL-3.0 License, Security Policy (`SECURITY.md`), Contribution Guidelines (`CONTRIBUTING.md`), and GitHub Actions CI workflow (`.github/workflows/ci.yml`).
- Windows Release: Compiled and verified (`build/windows/x64/runner/Release/zettings.exe`).
- Kubuntu Integration: D-Bus auto-activation, systemd user service, PolicyKit policy, and desktop launcher packaged.

