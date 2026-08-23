/**
 * L2 page registry (spec §4 templates + §10 catalog): every category's
 * settings areas, mapped to the Zyntrix backend service that owns them
 * (spec §13 mapping — native Linux services, never faked).
 *
 * `live` areas have working pages today; the rest render as intentional,
 * explained "arriving" rows (spec §15 disabled-with-explanation), each
 * naming the exact integration that unlocks them. This is roadmap truth
 * from docs/research/backend-capability-matrix.md, not filler.
 */

export interface SettingArea {
  /** URL slug (L2 route segment). */
  slug: string;
  title: string;
  description: string;
  /** Backend integration this area ships with. */
  via: string;
  /** A working L2 page exists today. */
  live?: boolean;
}

/** Areas per category id (registry order = display order). */
export const CATEGORY_AREAS: Readonly<Record<string, readonly SettingArea[]>> = {
  system: [
    { slug: "power", title: "Power & battery", description: "Performance modes and battery behavior.", via: "power-profiles-daemon", live: true },
    { slug: "bluetooth", title: "Bluetooth", description: "Adapter power, pairing and devices.", via: "BlueZ", live: true },
    { slug: "sound", title: "Sound", description: "Output devices, volume and mute.", via: "PulseAudio/PipeWire", live: true },
    { slug: "display", title: "Display", description: "Connectors, modes and topology.", via: "KScreen/KWin", live: true },
    { slug: "storage", title: "Storage", description: "Disks, volumes and usage.", via: "udisks2" },
    { slug: "notifications", title: "Notifications", description: "Do not disturb and per-app alerts.", via: "KDE notifications" },
  ],
  devices: [
    { slug: "bluetooth", title: "Bluetooth", description: "Devices, pairing and adapter power.", via: "BlueZ", live: true },
    { slug: "printers", title: "Printers & scanners", description: "Print queues and scanner sources.", via: "CUPS" },
    { slug: "mouse-touchpad", title: "Mouse & touchpad", description: "Pointer speed, scrolling and gestures.", via: "libinput" },
    { slug: "usb", title: "USB", description: "Per-device power and autorun behavior.", via: "udev" },
    { slug: "autoplay", title: "AutoPlay", description: "Removable-media default actions.", via: "Solid/KDE" },
  ],
  network: [
    { slug: "status", title: "Internet status", description: "Connectivity, Wi-Fi and interface state.", via: "NetworkManager", live: true },
    { slug: "vpn", title: "VPN", description: "Connections and providers.", via: "NetworkManager VPN plugins" },
    { slug: "proxy", title: "Proxy", description: "System-wide proxy configuration.", via: "NetworkManager" },
    { slug: "firewall", title: "Firewall", description: "Zone and application rules.", via: "firewalld" },
  ],
  personalization: [
    { slug: "theme", title: "Theme & appearance", description: "Light, dark, OLED, high contrast; wallpaper motion.", via: "Zettings frontend", live: true },
    { slug: "colors", title: "Colors", description: "Accent color across the system.", via: "KDE color scheme" },
    { slug: "fonts", title: "Fonts", description: "Interface and monospace typefaces.", via: "fontconfig/KDE fonts" },
    { slug: "cursor", title: "Cursor", description: "Pointer theme and size.", via: "KDE cursor themes" },
  ],
  apps: [
    { slug: "installed", title: "Installed apps", description: "Installed applications and sizes.", via: "PackageKit/Flatpak/Snap" },
    { slug: "defaults", title: "Default apps", description: "Handlers for web, mail and media.", via: "xdg-mime" },
    { slug: "startup", title: "Startup", description: "Autostart entries and systemd user units.", via: "systemd" },
  ],
  accounts: [
    { slug: "users", title: "Your account", description: "Identity and avatar.", via: "AccountsService" },
    { slug: "sign-in", title: "Sign-in options", description: "Password, fingerprint, auto-login.", via: "SDDM/KDE" },
    { slug: "sync", title: "Sync", description: "Roaming preferences across devices.", via: "KDED sync agents" },
  ],
  "time-language": [
    { slug: "date-time", title: "Date & time", description: "Timezone and NTP synchronization.", via: "timedatectl", live: true },
    { slug: "region", title: "Region", description: "Formats for dates, numbers and units.", via: "localeconf" },
    { slug: "language", title: "Language", description: "UI language and input methods.", via: "KDE localization" },
  ],
  gaming: [
    { slug: "game-mode", title: "Game mode", description: "Performance bias while playing.", via: "power-profiles-daemon" },
    { slug: "captures", title: "Captures", description: "Screenshots and clip recording.", via: "Spectacle/KWin" },
  ],
  accessibility: [
    { slug: "vision", title: "Vision", description: "Display size, contrast and narration.", via: "AT-SPI2/KDE" },
    { slug: "hearing", title: "Hearing", description: "Sound visualization and captions.", via: "PulseAudio/KDE" },
    { slug: "interaction", title: "Interaction", description: "Keyboard, pointer and switch access.", via: "libinput/KDE" },
  ],
  "privacy-security": [
    { slug: "permissions", title: "Permissions", description: "Device and data access grants.", via: "polkit/portals" },
    { slug: "firewall", title: "Firewall & protection", description: "Network trust and exposure.", via: "firewalld" },
    { slug: "search-permissions", title: "Search permissions", description: "File indexing scope.", via: "Tracker/KDE" },
  ],
  updates: [
    { slug: "check", title: "Check for updates", description: "Package and system updates.", via: "PackageKit" },
    { slug: "history", title: "Update history", description: "Applied transactions.", via: "PackageKit/dnf history" },
  ],
  developer: [
    { slug: "services", title: "Services", description: "System units and their states.", via: "systemd" },
    { slug: "journal", title: "Journal", description: "Structured system logs.", via: "journalctl" },
    { slug: "environment", title: "Environment", description: "Session variables and overrides.", via: "systemd environment" },
  ],
};

/** Areas that currently resolve to a live L2 page. */
export function findLiveArea(category: string, sub: string): SettingArea | null {
  const area = CATEGORY_AREAS[category]?.find((a) => a.slug === sub);
  return area?.live === true ? area : null;
}
