# ZETTINGS — Phase 0 Research

**Date:** 2026-09-26
**Status:** Complete — findings below are the authoritative input to Phase 1 (Architecture).
**Rule:** Every integration below links its upstream source. Nothing here is implemented yet;
an API is only "adopted" after runtime verification on Kubuntu/Zyntrix (see §9).

---

## 1. Existing settings applications — information architecture

| App | Top-level structure observed | Source |
|---|---|---|
| Windows 11 Settings | Home + System, Bluetooth & devices, Network & internet, Personalization, Apps, Accounts, Time & language, Gaming, Accessibility, Privacy & security, Windows Update. `ms-settings:` URI catalog = stable deep links. Shell = NavigationView + content viewport with cards/rows/expanders. | In-repo `ZETTINGS_Windows_11_Settings_Deep_UI_Spec.md`; https://support.microsoft.com/en-us/windows/experience/exploring-windows-settings |
| macOS System Settings | Sidebar hierarchy, search-first discovery | Qualitative reference (prompt.txt §33) |
| KDE System Settings / GNOME Settings / COSMIC Settings | Platform-native grouping + search | Qualitative reference; verify on Kubuntu during Phase 7 |

**Takeaways adopted for ZETTINGS (original design, not a clone):**
centralized settings registry with keywords/synonyms → powers nav + search + deep links;
stable route per page (`zettings://category/page`); card/row/expander composition;
search must be local, indexed, keyboard-navigable, instant (no per-keystroke backend calls).

---

## 2. Flutter desktop — architecture constraints (authoritative: docs.flutter.dev)

- Desktop support is stable: `flutter create`, `flutter run -d linux`, `flutter build linux`.
  Docs: https://docs.flutter.dev/platform-integration/desktop,
  https://docs.flutter.dev/platform-integration/linux/setup.md
- **Cross-compilation is NOT supported.** Linux builds must run on Linux.
  → Windows host = editing + Dart analysis (after SDK install); Linux builds/tests run in
  WSL Ubuntu; runtime integration tests require real Kubuntu/Zyntrix hardware.
- Linux toolchain prerequisites (Debian/Ubuntu):
  `clang cmake ninja-build pkg-config libgtk-3-dev libstdc++-12-dev`
  (+ `curl git unzip xz-utils zip libglu1-mesa`). Validate with `flutter doctor -v`
  and `flutter devices` (must list a `linux` device).
- Enablement: `flutter config --enable-linux-desktop` (also `--enable-windows-desktop`
  for the Windows runner). Disable unneeded platforms to silence `flutter doctor` noise.

---

## 3. Material 3 desktop patterns (authoritative: m3.material.io, M3 Expressive 2025)

- **NavigationDrawer is deprecated** in the M3 Expressive update; use the **expanded
  NavigationRail** instead (same function, adapts across breakpoints).
  https://m3.material.io/components/navigation-drawer/overview,
  https://m3.material.io/components/navigation-rail/guidelines
- Window size classes: compact <600dp · medium 600–839 · expanded 840–1199 ·
  large 1200–1599 · extra-large 1600+. Rail = 3–7 destinations + optional FAB.
- ZETTINGS has 10+ categories → collapsed rail cannot hold them all: use expanded rail
  on large windows, modal rail/drawer on small windows, preserve selection + scroll state.
- Keyboard contract: Tab lands on first destination, Space/Enter selects, Arrows move
  between destinations. Every control needs a semantic label; never color-only state.

---

## 4. Flutter ↔ Rust IPC — decision

Two canonical options evaluated:

1. **Rust D-Bus daemon + Dart `dbus` client (CHOSEN).**
   `dbus` package (canonical.com, verified publisher, 160/160 pub points, v0.7.15):
   native Dart D-Bus client, `DBusClient.session()/system()`, `dart-dbus
   generate-remote-object` generates typed clients from introspection XML.
   https://pub.dev/packages/dbus
2. `flutter_rust_bridge` codegen — tighter coupling, heavier build; rejected for now
   (revisit only with measured justification).

Resulting boundary (detail in Phase 1 `ARCHITECTURE.md`):

```text
Flutter UI → Dart service interfaces → dbus-generated typed clients
    → session-bus daemon org.zyntrix.Zettings (Rust) → system D-Bus / Linux APIs
```

No generic `execute(command)` channel. Every operation is a typed method with
typed errors (Unavailable/Unsupported/PermissionDenied/InvalidInput/Timeout/…).

---

## 5. Rust + D-Bus (authoritative: docs.rs/zbus v5.19)

- `zbus` crate: `#[proxy]` macro builds typed async clients from a trait;
  `#[interface]` macro exposes a Rust struct as a D-Bus service;
  `connection::Builder::session()/system()` + `.serve_at(path, impl)` + `.build()`.
  Runtime-agnostic; tokio is the documented choice. https://docs.rs/zbus/latest/zbus/
- Backend depends on: `zbus`, `tokio`, `serde`/`serde_json`, `zvariant`,
  `sysinfo` (system info), plus small typed wrappers per service.
- Property-change and signal streams (`receive_property_changed`, `receive_signal`)
  give us external-state updates (Phase: subscribe, don't poll).

---

## 6. System API map — adopted interfaces and their verification status

| Feature | Service / bus | Key interface / ops | Upstream ref | Status |
|---|---|---|---|---|
| System info | local sysfs/procfs + `sysinfo` crate; `/etc/os-release`; `lspci` as fallback | typed structs, no D-Bus needed | https://docs.rs/sysinfo | implement first (Phase 9 plan) |
| Display | session bus `org.kde.KScreen` (`applyLayoutPreset`, `outputConnected`); `kscreen-doctor` (mode/position/scale/primary/enable/rotation, atomic multi-change); `xrandr` X11 fallback | wrap `kscreen-doctor` as **typed ops with validated args** (not shell passthrough) | KDE/kscreen repo; `kscreen-console` man page | VERIFY on Kubuntu Wayland |
| Network | system bus `org.freedesktop.NetworkManager` @ `/org/freedesktop/NetworkManager`: `GetDevices`, `ActivateConnection`, `DeactivateConnection`; profiles @ `…/Settings/` | read-only first; secrets via Secret Agent pattern | https://networkmanager.dev/docs/api/latest | VERIFY on Kubuntu |
| Audio | **PipeWire via pipewire-pulse compat**: PulseAudio client protocol (`pactl`/`pavucontrol` work unchanged). Rust: `pulseaudio` protocol crate / `libpulse-binding` | default sink/source, volume, mute, device list | https://docs.pipewire.org/page_module_protocol_pulse.html; https://docs.rs/pulseaudio | VERIFY `Server Name: PulseAudio (on PipeWire …)` on target |
| Power | system bus `org.freedesktop.UPower` (battery devices) + `org.freedesktop.UPower.PowerProfiles` @ `/org/freedesktop/UPower/PowerProfiles`: `Profiles`, `ActiveProfile` (rw), `HoldProfile`/`ReleaseProfile` | old `net.hadess.PowerProfiles` name is obsolete — do not use | https://upower.pages.freedesktop.org/power-profiles-daemon/ | VERIFY on Kubuntu |
| Bluetooth | system bus `org.bluez`: `org.bluez.Adapter1` (`StartDiscovery`/`StopDiscovery`), `org.bluez.Device1` (`Connect`/`Disconnect`/`Pair`) | pairing relies on default agent; no-adapter state is normal | bluez `doc/org.bluez.{Adapter,Device}.rst` | VERIFY on hardware |
| Accounts | system bus `org.freedesktop.Accounts` + `…Accounts.User`; polkit actions `org.freedesktop.accounts.user-administration`, `….change-own-user-data` | read current user first; mutations need auth | cgit.freedesktop.org/accountsservice | VERIFY on Kubuntu |
| Privilege | system bus `org.freedesktop.PolicyKit1.Authority` `CheckAuthorization`; auth agents come from the desktop session (KDE provides one) — never run the app as root | request auth per-operation, least privilege | https://www.freedesktop.org/software/polkit/docs/latest/ | VERIFY agent presence |
| Storage | system bus `org.freedesktop.UDisks2`: Manager/Drive/Block/Filesystem/Partition; `udisksctl` for manual probing only | overview + mount info first; destructive ops gated + confirmed | https://storaged.org/udisks/docs/ref-dbus.html | VERIFY on Kubuntu |
| Updates | system bus `org.freedesktop.PackageKit`: `GetTid` → one-shot `…Transaction` path → `GetUpdates`/`InstallPackages`; one transaction = one action | **never** `apt`-from-UI; explicit user confirmation; note CVE-2026-41651 class (fixed 1.3.5): single-use transactions, no re-invocation | https://packagekit.freedesktop.org/gtk-doc/ | VERIFY backend + version |
| Personalization | KDE config mechanisms (`kwriteconfig6`, `plasma-apply-wallpaperimage`, look-and-feel packages) | **unverified** — enumerate real knobs on target before exposing any control | target-system probing in Phase 7 | NOT ADOPTED until probed |

---

## 7. Security model (from polkit + PackageKit + AGENTS.md)

- Least privilege; normal settings never need admin. Elevated ops: explain why →
  polkit `CheckAuthorization` (interactive only on explicit user action) → perform one
  typed op → verify result → return actual result.
- Never: arbitrary shell execution, string-interpolated commands, full-app root,
  silent elevation, UI-state-as-authorization.
- PackageKit transactions are single-use by design; our IPC mirrors that (one typed
  call = one verified operation).

---

## 8. Skill provenance (AGENTS.md §§39–47 compliance)

- Installed skills searched: 11 flutter/flutter contributor skills — all cover
  upstream repo maintenance (flake analysis, cherry-picks, device-lab), none covers
  app architecture / Material 3 / Rust / D-Bus / KDE. No overlap; no duplication.
- `npx skills find "flutter desktop material3"`: 12 hits, all <200 installs from
  unknown authors → **rejected** (below 1K-install trust threshold, unreviewed code).
- `npx skills find "rust dbus linux system"`: zero hits.
- **Decision (user-approved):** proceed on authoritative upstream docs listed above
  (§§2–6) + installed contributor skills where relevant (e.g. `dart-log-failure-parser`
  for test logs). No third-party skill installed, no installer scripts executed.

---

## 9. Verification environments

| Environment | Role | State |
|---|---|---|
| Windows host (this machine) | editing, Dart analysis, Windows runner builds | Flutter SDK install approved — in progress |
| WSL Ubuntu 26.04 (passwordless sudo, 947G free) | Linux builds, `flutter test`, Rust `cargo test`, D-Bus API-shape tests | Flutter SDK + toolchain install approved — in progress |
| Real Kubuntu / Zyntrix OS machine | runtime integration (NM, UPower, BlueZ, KScreen, polkit agent) | NOT AVAILABLE in this session — every hardware claim stays `UNVERIFIED` until run there |

`Compiled ≠ Analyzed ≠ Unit-tested ≠ Integration-tested ≠ Runtime-tested` (AGENTS.md §34).
