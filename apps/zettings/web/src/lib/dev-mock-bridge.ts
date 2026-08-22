/**
 * DEV-ONLY IPC bridge fixtures.
 *
 * Mirrors the Rust mock adapters (BackendSet::mocks) so plain-browser
 * `pnpm dev` sessions render the real UI for visual/a11y verification.
 * This module is dynamically imported only when
 * `import.meta.env.DEV && !isZettingsRuntime()` — production builds never
 * load it, and shipped settings always come from real adapters.
 */

import type { RegistrySnapshotDto, SystemSnapshotDto } from "@zettings/bindings";

const registry: RegistrySnapshotDto = {
  categories: (
    [
      ["system", "System", "Display, sound, power, storage and notifications."],
      ["devices", "Bluetooth & Devices", "Bluetooth, printers, mice, keyboards and cameras."],
      ["network", "Network & Internet", "Wi-Fi, ethernet, VPN and firewall."],
      ["personalization", "Personalization", "Background, colors, themes, fonts and cursor."],
      ["apps", "Apps", "Installed apps, defaults, startup and packages."],
      ["accounts", "Accounts", "Users, sign-in and synchronization."],
      ["time-language", "Time & Language", "Date and time, region, language and formats."],
      ["gaming", "Gaming", "Game mode, captures and performance profiles."],
      ["accessibility", "Accessibility", "Vision, hearing, interaction and narration."],
      ["privacy-security", "Privacy & Security", "Permissions, firewall and system security."],
      ["updates", "Updates", "System updates and package maintenance."],
      ["developer", "Developer", "Developer options, services and diagnostics."],
    ] as Array<[string, string, string]>
  ).map(([id, title, description]) => ({
    id,
    title,
    description,
    icon: "circle",
    route: `zettings://${id}`,
  })),
};

const searchIndex = registry.categories.map((c) => ({
  setting_id: `${c.id}.overview`,
  title: c.title,
  description: c.description,
  category: c.id,
  route: c.route,
  score: 50,
}));

/** In-memory state mirroring the Rust mocks' set→get semantics. */
const state = {
  powerProfile: "balanced",
  wirelessEnabled: true,
  sinks: [
    {
      name: "mock-output.analog-stereo",
      description: "Built-in Audio Analog Stereo",
      muted: false,
      volume_percent: 80,
      is_default: true,
    },
  ],
  bluetoothPowered: true,
};

function snapshot(): SystemSnapshotDto {
  return {
    power: {
      capability: { state: "available" },
      available: ["power-saver", "balanced", "performance"],
      active: state.powerProfile,
    },
    network: {
      capability: { state: "available" },
      networking_enabled: true,
      wireless_enabled: state.wirelessEnabled,
      devices: [
        { interface: "wlan0", kind: "wifi", state: 100 },
        { interface: "eth0", kind: "ethernet", state: 100 },
      ],
    },
    session: {
      capability: { state: "available" },
      can_power_off: "yes",
      can_reboot: "yes",
      can_suspend: "yes",
    },
    audio: {
      capability: { state: "available" },
      sinks: state.sinks.map((s) => ({ ...s })),
    },
    bluetooth: {
      capability: { state: "available" },
      powered: state.bluetoothPowered,
      devices: state.bluetoothPowered
        ? [{ alias: "Mock Mouse", paired: true, connected: true }]
        : [],
    },
    display: {
      capability: { state: "available" },
      outputs: [
        {
          name: "card0-eDP-1",
          connected: true,
          modes: ["1920x1080", "1600x900"],
          current_mode: "1920x1080",
        },
      ],
    },
  };
}

/**
 * Serves fixture responses for the commands the UI exercises.
 * Unknown commands reject honestly — fixtures never invent behavior.
 */
export function devInvoke<T>(command: string, args?: Record<string, unknown>): T {
  switch (command) {
    case "registry_snapshot":
      return registry as T;
    case "search_registry": {
      const query = String((args as { query?: string })?.query ?? "").toLowerCase();
      const hits =
        query === ""
          ? []
          : searchIndex.filter(
              (h) =>
                h.title.toLowerCase().includes(query) ||
                h.description.toLowerCase().includes(query) ||
                h.category.includes(query),
            );
      return { query, hits } as T;
    }
    case "system_snapshot":
      return snapshot() as T;
    case "set_power_profile":
      state.powerProfile = String((args as { profile?: string }).profile);
      return undefined as T;
    case "set_wireless_enabled":
      state.wirelessEnabled = Boolean((args as { enabled?: boolean }).enabled);
      return undefined as T;
    case "set_audio_sink": {
      const a = args as { sink?: string; volume_percent?: number; muted?: boolean };
      const sink = state.sinks.find((s) => s.name === a.sink);
      if (sink === undefined) throw new Error(`unknown sink ${String(a.sink)}`);
      if (a.muted !== undefined) sink.muted = a.muted;
      if (a.volume_percent !== undefined) sink.volume_percent = a.volume_percent;
      return undefined as T;
    }
    case "set_bluetooth_powered":
      state.bluetoothPowered = Boolean((args as { enabled?: boolean }).enabled);
      return undefined as T;
    default:
      throw new Error(`dev bridge has no fixture for command “${command}”`);
  }
}
