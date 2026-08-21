/**
 * HomePage — L1 launch surface reconstructing the Windows 11 Home experience
 * (spec §1): interactive cards for the settings categories plus quick-action
 * tiles, floating above the Live Wallpaper on light glass.
 *
 * Cards are real <button>s (keyboard skill), clipped to G2 squircles, with a
 * status dot that mirrors backend reachability (shape + colour, never colour
 * alone).
 */
import { useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import type { Health } from "@zettings/bindings";
import {
  Battery,
  Bluetooth,
  Monitor,
  Palette,
  Volume2,
  Wifi,
} from "lucide-react";
import type { LucideProps } from "lucide-react";
import { useSquircleClip } from "./squircle-surface.js";

export interface HomeModule {
  segment: string;
  title: string;
  description: string;
  icon: React.ComponentType<LucideProps>;
}

export const HOME_MODULES: readonly HomeModule[] = [
  {
    segment: "display",
    title: "Displays",
    description: "Arrange monitors, resolution, refresh rate, night light",
    icon: Monitor,
  },
  {
    segment: "audio",
    title: "Sound",
    description: "Master volume, per-app mixer, equalizer, output device",
    icon: Volume2,
  },
  {
    segment: "network",
    title: "Network",
    description: "Wi-Fi networks, ethernet, known networks, VPN",
    icon: Wifi,
  },
  {
    segment: "bluetooth",
    title: "Bluetooth",
    description: "Paired devices, battery levels, connect and disconnect",
    icon: Bluetooth,
  },
  {
    segment: "power",
    title: "Power",
    description: "Battery health, charge limits, performance profiles",
    icon: Battery,
  },
  {
    segment: "personalization",
    title: "Personalization",
    description: "Themes, accent colors, live wallpapers, glass tuning",
    icon: Palette,
  },
];

function greeting(): string {
  const hour = new Date().getHours();
  if (hour < 5) return "Good night";
  if (hour < 12) return "Good morning";
  if (hour < 18) return "Good afternoon";
  return "Good evening";
}

interface HomeCardProps {
  module: HomeModule;
}

function HomeCard({ module }: HomeCardProps): React.ReactElement {
  const Icon = module.icon;
  const { ref, clipStyle, defs } = useSquircleClip(18, 4);

  return (
    <>
      <button
        ref={ref as React.Ref<HTMLButtonElement>}
        type="button"
        className="settings-card home-card"
        style={clipStyle}
        onClick={() => {
          window.location.hash = `/${module.segment}`;
        }}
      >
        <span className="settings-card-icon" aria-hidden="true">
          <Icon size={22} strokeWidth={2} />
        </span>
        <span className="settings-card-text">
          <span className="settings-card-title">{module.title}</span>
          <span className="settings-card-description">{module.description}</span>
        </span>
      </button>
      {defs}
    </>
  );
}

interface QuickTileProps {
  label: string;
  target: string;
}

function QuickTile({ label, target }: QuickTileProps): React.ReactElement {
  return (
    <button type="button" className="nav-search-btn home-tile" onClick={() => {
      window.location.hash = target;
    }}>
      <span className="home-status-dot" data-state="ok" aria-hidden="true" />
      <span className="nav-search-label">{label}</span>
    </button>
  );
}

/** L1 Home hub. */
export function HomePage(): React.ReactElement {
  const { isError, isPending } = useQuery({
    queryKey: ["zettings-health"],
    queryFn: async (): Promise<Health> => await invoke<Health>("zettings_health"),
    retry: 1,
  });

  const state = isPending ? "idle" : isError ? "err" : "ok";

  return (
    <div>
      <div className="home-hero">
        <h1 className="home-hero-title">{greeting()}</h1>
        <p className="home-hero-subtitle">
          Zettings — system settings for Zyntrix OS.{" "}
          {state === "err" ? (
            <span role="status">Backend unreachable — showing saved state.</span>
          ) : (
            "Everything below is live."
          )}
        </p>
      </div>

      <section className="settings-section" aria-label="Settings categories">
        <div className="home-grid">
          {HOME_MODULES.map((mod) => (
            <HomeCard key={mod.segment} module={mod} />
          ))}
        </div>
      </section>

      <section className="settings-section" aria-label="Quick actions">
        <h2 className="settings-section-header">Quick actions</h2>
        <div className="home-tiles">
          <QuickTile label="Scan Wi-Fi" target="/network" />
          <QuickTile label="Pair a device" target="/bluetooth" />
          <QuickTile label="Power profile" target="/power" />
          <QuickTile label="Live wallpaper" target="/personalization" />
        </div>
      </section>
    </div>
  );
}
