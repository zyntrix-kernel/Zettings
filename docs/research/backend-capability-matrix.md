# Backend Capability Matrix — Kubuntu 24.04 / KDE Plasma 5.27

> Purpose: state, per PLAN §13/§19 adapter boundary, what is technically available
> on the target platform, the integration transport, and the honest-unavailable
> strategy when a capability is absent. No fake integrations (PLAN §2).

Legend: **D-Bus** = zbus client · **Native** = C ABI / proc / sysfs ·
**CLI** = spawn documented binary via validated allowlist ·
Availability: ✔ standard · ◐ conditional (hardware/install) · ✘ not available.

## Tier 1 — Core desktop

| Adapter | Transport | Service/API | Availability | Notes |
|---|---|---|---|---|
| Display | D-Bus | `org.kde.KScreen` (/kscreen), KWin Wayland protocols | ✔ | scale/resolution/mode via KScreen2; HDR ◐ (driver-dependent); night light via KWin ColorScheme/`org.kde.KWin.Effect` ◐ |
| Audio | Native + D-Bus | PipeWire (`pipewire-rs`) / WirePlumber, PulseAudio compat (`libpulse-binding`) | ✔ | device enumeration, volumes, per-app routing; mic monitor |
| Network | D-Bus | `org.freedesktop.NetworkManager` | ✔ | WiFi/Ethernet/VPN/Airplane/Hotspot/known networks; secret prompts via NM agent API |
| Bluetooth | D-Bus | `org.bluez` | ◐ (adapter present) | pairing, connect, battery via `org.bluez.Battery1` |
| Power | D-Bus | `net.hadess.PowerProfiles` / `org.freedesktop.UPower` | ✔ PPD on 24.04 | profiles, battery %, energy estimates |
| Storage | D-Bus | `org.freedesktop.UDisks2` | ✔ | disks/volumes, smart ◐, mount ops require polkit |
| Appearance | D-Bus/Native | plasma-lookup, kdeglobals via KConfig D-Bus, `org.kde.plasmashell` ◐ | ✔ core | wallpaper/accent/theme/cursor/icons; live blur depends on KWin |
| Input | Native + D-Bus | libinput quirks, xinput ◐, KDE kcm conventions | ✔ basic | pointer speed/acceleration profile, natural scroll, per-device where exposed |
| Users | D-Bus | `org.freedesktop.Accounts` (+AccountsService) | ✔ | user list, avatar, admin flag changes → polkit |
| Accessibility | D-Bus/Native | AT-SPI2 Bus, `org.a11y.Bus`, gsettings bridges | ✔ | screen-reader presence detection (Orca), magnifier ◐ |

## Tier 2 — System management

| Adapter | Transport | Service/API | Availability |
|---|---|---|---|
| Apps/Packages | D-Bus | `org.freedesktop.PackageKit` | ✔ |
| Flatpak | CLI (validated) | `flatpak`-spawn or system D-Bus `org.freedesktop.Flatpak` | ◐ installed |
| Snap | D-Bus+CLI | `snapd` REST socket `/run/snapd.socket` | ◐ installed |
| Updates | D-Bus | PackageKit `GetUpdates/UpdatePackages`; unattended-upgrades config (polkit) | ✔ |
| Printers | D-Bus | CUPS via `org.cups.cupsd` / cups-pk-helper | ✔ |
| Privacy | D-Bus | XDG portals (`org.freedesktop.portal.*`) permission store | ✔ partial coverage |
| Security/Firewall | CLI/D-Bus | `ufw`/`firewalld` (`org.fedoraproject.FirewallD1`) | ◐ which one installed |
| Developer options | Native | journald namespaces, cgroups v2 | ✔ |
| Services | D-Bus | `org.freedesktop.systemd1` | ✔ |
| Startup apps | Native | XDG autostart dirs + systemd user units | ✔ |
| Processes | Native | /proc, systemd-cgroup mapping | ✔ |

## Tier 3 — KDE/Zyntrix

| Adapter | Transport | Service/API | Availability |
|---|---|---|---|
| KWin/window rules | D-Bus/scripting | `org.kde.KWin` Scripting, window rules config files | ✔ (script deploy needs care) |
| Virtual desktops | D-Bus | `org.kde.KWin` `VirtualDesktopManager` | ✔ |
| KDE effects | D-Bus | `org.kde.KWin.Effect` / effect config | ✔ |
| Plasma shell | D-Bus | `org.kde.plasmashell` | ◐ versioned API |
| Shell/Terminal | Registry | default-terminal via XDG + Konsole profile (polkit-free) | ✔ |
| Theme engine | Native/D-Bus | look-and-feel packages, color schemes | ✔ |

## Tier 4 — Advanced ecosystem

| Adapter | Transport | Availability |
|---|---|---|
| Containers (Docker/Podman) | socket REST (`/var/run/docker.sock`, podman socket) | ◐ |
| VMs (libvirt/qemu) | `org.libvirt` D-Bus ◐ / virsh CLI | ✘→◐ rare on desktop installs |
| SSH config | file-based (~/.ssh/config, validated write) | ✔ read; writes polkit-free but audited |
| Remote desktop | gnome-remote-desktop ✘ / krfb ◐ | ◐ |
| Network shares | mount.cifs/NFS via udisks2 + fstab helpers | ◐ |
| Diagnostics/logs/journal | `tracing-journald` + `journalctl` reader bindings (systemd journal native API) | ✔ |
| Sensors/fans/thermals | hwmon sysfs, `org.freedesktop.HardwareSensor` ✘ (not in stock KDE) | ◐ read-only |
| Thunderbolt/USB/DisplayLink | `org.rndb.gms` ✘, bolt daemon `org.gnome.bolt` ◐, udev enumerate | ◐ |
| BIOS/DMI info | /sys/class/dmi read-only | ✔ |

## Honest-unavailable contract

Every adapter implements:

```rust
enum CapabilityState { Available, Degraded { reason }, Unavailable { reason } }
```

- UI renders capability state truthfully (disabled-with-explanation, empty states).
- `Unavailable` is never rendered as success.
- Privileged mutations route through `zettings-polkit` with action IDs registered
  in `packaging/org.zyntrix.zettings.policy`.
- All CLI transports are allowlisted argv templates — no string interpolation of
  untrusted input into shells (see threat-model.md).

## Environment split

- **Windows host dev:** all adapters behind `zettings-mock` feature (state-machine mocks).
- **WSL2 Kubuntu:** real services; some desktop services (KWin/plasmashell D-Bus)
  require a running Plasma session — matrix marks those ◐ under WSL without GUI.
