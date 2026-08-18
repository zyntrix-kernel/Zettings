import { useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { Suspense, useEffect, useState, lazy } from "react";
import type { Health } from "@zettings/bindings";
import { Breadcrumbs } from "../components/breadcrumbs.js";
import { ShellFrame } from "../components/shell-frame.js";
import { SpotlightModal } from "../components/spotlight-modal.js";
import { TargetHighlightBoundary } from "../components/target-highlight-boundary.js";
import { useHashRoute } from "../lib/hash-route.js";
import { useSpotlightStore } from "../stores/spotlight-store.js";

// Shared icon mappings for sidebar and quick actions
const ICONS: Record<string, React.ComponentType<{ size?: number; color?: string }>> = {
  Wifi: () => <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M5 13a10 10 0 0 1 14 0M5 17a14 14 0 0 1 14 0"/></svg>,
  Bluetooth: () => <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><polyline points="6.5 6.5 17.5 17.5 12 23 12 1 17.5 6.5 6.5 17.5"/></svg>,
  Battery: () => <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><rect x="1" y="4" width="18" height="16" rx="2"/><line x1="23" y1="9" x2="23" y2="11"/></svg>,
  Palette: () => <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><circle cx="13.5" cy="6.5" r=".5"/><circle cx="17.5" cy="10.5" r=".5"/><circle cx="8.5" cy="15.5" r=".5"/><circle cx="4.5" cy="11.5" r=".5"/><path d="M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10c.926 0 1.648-.746 1.648-1.688 0-.437-.18-.835-.437-1.12.324.403.89.727 1.534.727 1.043 0 1.534-.982 1.534-2.188 0-1.145-.825-1.93-1.714-1.93-.877 0-1.516.671-1.516 1.5 0 1.027.706 1.543 1.564 1.737A4.97 4.97 0 0 1 13 21.5c3.59 0 6.5-2.91 6.5-6.5S15.59 2 12 2z"/></svg>,
};

// Lazy-loaded panel components (code splitting per route - ui-ux-pro-max Medium)
const DisplayPanel = lazy(() => import("../components/display-panel.js").then((m) => ({ default: m.DisplayPanel })));
const AudioPanel = lazy(() => import("../components/audio-panel.js").then((m) => ({ default: m.AudioPanel })));
const NetworkPanel = lazy(() => import("../components/network-panel.js").then((m) => ({ default: m.NetworkPanel })));
const BluetoothPanel = lazy(() => import("../components/bluetooth-panel.js").then((m) => ({ default: m.BluetoothPanel })));
const PowerPanel = lazy(() => import("../components/power-panel.js").then((m) => ({ default: m.PowerPanel })));
const PersonalizationPanel = lazy(() => import("../components/personalization-panel.js").then((m) => ({ default: m.PersonalizationPanel })));

// Overview page (root route)
function OverviewPage(): React.ReactElement {
  const queryClient = useQueryClient();
  const { data, isLoading, isError } = useQuery({
    queryKey: ["zettings-health"],
    queryFn: async (): Promise<Health> => await invoke<Health>("zettings_health"),
  });

  if (isLoading) return <p>Connecting to backend…</p>;
  if (isError) return <p role="alert">Backend unreachable.</p>;
  if (!data) return <div />;

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-8)" }}>
      <header style={{ paddingBottom: "var(--space-6)", borderBottom: "1px solid var(--border)" }}>
        <h2 style={{ fontSize: "var(--text-2xl)", lineHeight: "var(--text-2xl-lh)", fontWeight: 600, margin: 0 }}>
          Zettings v{data.version}
          {data.is_mock ? <span style={{ fontSize: "var(--text-sm)", fontWeight: 400, color: "var(--text-muted)", marginLeft: "var(--space-2)" }}> (mock backend)</span> : null}
        </h2>
        <p style={{ fontSize: "var(--text-base)", color: "var(--text-muted)", margin: "var(--space-2) 0 0" }}>
          System settings for Zyntrix OS. Use the sidebar or press <kbd style={{ fontFamily: "var(--font-mono)", padding: "2px 6px", background: "var(--surface-muted)", borderRadius: "4px", border: "1px solid var(--border)" }}>Super+I</kbd> to open Spotlight search.
        </p>
      </header>

      <section aria-labelledby="modules-heading">
        <h3 id="modules-heading" style={{ fontSize: "var(--text-lg)", fontWeight: 600, color: "var(--text)", margin: "0 0 var(--space-4)" }}>
          Settings Modules
        </h3>
        <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(260px, 1fr))", gap: "var(--space-4)" }}>
          {MODULE_CARDS.map((mod) => (
            <ModuleCard key={mod.route} module={mod} />
          ))}
        </div>
      </section>

      <section aria-labelledby="quick-heading" style={{ paddingTop: "var(--space-4)", borderTop: "1px solid var(--border)" }}>
        <h3 id="quick-heading" style={{ fontSize: "var(--text-lg)", fontWeight: 600, color: "var(--text)", margin: "0 0 var(--space-4)" }}>
          Quick Actions
        </h3>
        <div style={{ display: "flex", flexWrap: "wrap", gap: "var(--space-3)" }}>
          <QuickAction label="Scan Wi-Fi" icon="Wifi" onClick={() => window.location.hash = "/network"} />
          <QuickAction label="Pair Bluetooth" icon="Bluetooth" onClick={() => window.location.hash = "/bluetooth"} />
          <QuickAction label="Power Profile" icon="Battery" onClick={() => window.location.hash = "/power"} />
          <QuickAction label="Extract Colors" icon="Palette" onClick={() => window.location.hash = "/personalization"} />
        </div>
      </section>
    </div>
  );
}

interface ModuleCardProps {
  module: {
    route: string;
    title: string;
    description: string;
    icon: React.ComponentType<{ size?: number; color?: string }>;
    keywords: string[];
  };
}

function ModuleCard({ module }: ModuleCardProps): React.ReactElement {
  const Icon = ICONS[module.icon];
  return (
    <button
      onClick={() => window.location.hash = module.route}
      style={{
        display: "flex",
        flexDirection: "column",
        gap: "var(--space-3)",
        padding: "var(--space-5)",
        background: "var(--surface-muted)",
        border: "1px solid var(--border)",
        borderRadius: "12px",
        cursor: "pointer",
        textAlign: "left",
        transition: "border-color var(--motion-duration-fast) var(--motion-ease-out), box-shadow var(--motion-duration-fast) var(--motion-ease-out), background var(--motion-duration-fast) var(--motion-ease-out)",
      }}
      onMouseEnter={(e) => {
        e.currentTarget.style.borderColor = "color-mix(in srgb, var(--accent) 40%, var(--border))";
        e.currentTarget.style.boxShadow = "var(--shadow-1)";
        e.currentTarget.style.background = "var(--surface-elevated)";
      }}
      onMouseLeave={(e) => {
        e.currentTarget.style.borderColor = "var(--border)";
        e.currentTarget.style.boxShadow = "none";
        e.currentTarget.style.background = "var(--surface-muted)";
      }}
      onFocus={(e) => {
        e.currentTarget.style.outline = "2px solid var(--ring)";
        e.currentTarget.style.outlineOffset = "2px";
      }}
      onBlur={(e) => {
        e.currentTarget.style.outline = "none";
      }}
    >
      <div style={{ display: "flex", alignItems: "center", gap: "var(--space-3)" }}>
        <div style={{ width: 44, height: 44, borderRadius: "12px", background: "color-mix(in srgb, var(--accent) 14%, transparent)", display: "flex", alignItems: "center", justifyContent: "center" }}>
          <Icon size={22} color="var(--accent)" />
        </div>
        <div style={{ flex: 1, minWidth: 0 }}>
          <h4 style={{ fontSize: "var(--text-base)", fontWeight: 600, color: "var(--text)", margin: 0, whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis" }}>
            {module.title}
          </h4>
          <p style={{ fontSize: "var(--text-sm)", color: "var(--text-muted)", margin: "var(--space-1) 0 0" }}>
            {module.description}
          </p>
        </div>
      </div>
      <div style={{ fontSize: "var(--text-xs)", color: "var(--text-subtle)" }}>
        Keywords: {module.keywords.slice(0, 3).join(", ")}…
      </div>
    </button>
  );
}

interface QuickActionProps {
  label: string;
  icon: string;
  onClick: () => void;
}

function QuickAction({ label, icon: IconName, onClick }: QuickActionProps): React.ReactElement {
  const Icon = ICONS[IconName] || ICONS.Wifi;

  return (
    <button
      onClick={onClick}
      className="panel-button panel-button-secondary"
      style={{ display: "inline-flex", alignItems: "center", gap: "var(--space-2)" }}
    >
      <Icon size={16} />
      {label}
    </button>
  );
}

const MODULE_CARDS = [
  { route: "/display", title: "Displays", description: "Arrange monitors, resolution, refresh rate, night light", icon: "Wifi", keywords: ["monitor", "screen", "arrange", "resolution", "night light"] },
  { route: "/audio", title: "Sound", description: "Master volume, per-app mixer, equalizer, output device", icon: "Bluetooth", keywords: ["volume", "mixer", "equalizer", "output", "bluetooth audio"] },
  { route: "/network", title: "Network", description: "Wi-Fi networks, ethernet, known networks, VPN", icon: "Battery", keywords: ["wifi", "ethernet", "scan", "connect", "password"] },
  { route: "/bluetooth", title: "Bluetooth", description: "Paired devices, battery levels, connect/disconnect", icon: "Palette", keywords: ["pair", "headphones", "mouse", "keyboard", "battery"] },
  { route: "/power", title: "Power", description: "Battery health, discharge graph, performance profiles", icon: "Wifi", keywords: ["battery", "profile", "performance", "power saver", "cycles"] },
  { route: "/personalization", title: "Personalization", description: "Accent colors, corner roundness, glass blur, themes", icon: "Bluetooth", keywords: ["accent", "theme", "dark mode", "squircle", "glass", "blur"] },
];

async function fetchHealth(): Promise<Health> {
  return await invoke<Health>("zettings_health");
}

function isSpotlightShortcut(event: KeyboardEvent): boolean {
  if (event.type !== "keydown") return false;
  if (event.code === "KeyI" && (event.metaKey || event.ctrlKey) && !event.altKey) {
    return true;
  }
  if (event.code === "Space" && event.ctrlKey && !event.metaKey && !event.altKey) {
    return true;
  }
  return false;
}

export function Zettings(): React.ReactElement {
  const [reducedMotion, setReducedMotion] = useState(false);
  useEffect(() => {
    const mq = window.matchMedia("(prefers-reduced-motion: reduce)");
    setReducedMotion(mq.matches);
    const handler = (e: MediaQueryListEvent): void => setReducedMotion(e.matches);
    mq.addEventListener("change", handler);
    return () => mq.removeEventListener("change", handler);
  }, []);

  const openSpotlight = useSpotlightStore((state) => state.open);
  const closeSpotlight = useSpotlightStore((state) => state.close);

  useEffect(() => {
    const handler = (event: KeyboardEvent): void => {
      if (isSpotlightShortcut(event)) {
        event.preventDefault();
        openSpotlight();
      } else if (event.key === "Escape") {
        closeSpotlight();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [openSpotlight, closeSpotlight]);

  // Phase 6.4 — hash-route state driving breadcrumbs + target highlight
  const route = useHashRoute();

  // Resolve panel component from route
  const PanelComponent = ((): React.ComponentType | null => {
    if (route.isRoot) return OverviewPage;
    const segment = route.segments[0];
    switch (segment) {
      case "display": return DisplayPanel;
      case "audio": return AudioPanel;
      case "network": return NetworkPanel;
      case "bluetooth": return BluetoothPanel;
      case "power": return PowerPanel;
      case "personalization": return PersonalizationPanel;
      default: return OverviewPage;
    }
  })();

  return (
    <ShellFrame>
      <div className="zettings-shell" data-reduced-motion={reducedMotion}>
        <aside className="sidebar">
          <header className="sidebar-title">
            <span className="brand-glyph" aria-hidden />
            <h1>Zettings</h1>
          </header>
          <nav aria-label="Settings modules">
            <ul style={{ listStyle: "none", padding: 0, margin: 0, display: "flex", flexDirection: "column", gap: "var(--space-1)" }}>
              {MODULE_CARDS.map((mod) => {
                const Icon = ICONS[mod.icon] || ICONS.Wifi;
                return (
                  <li key={mod.route}>
                    <button
                      onClick={() => window.location.hash = mod.route}
                      style={{
                        display: "flex",
                        alignItems: "center",
                        gap: "var(--space-3)",
                        width: "100%",
                        padding: "var(--space-3) var(--space-4)",
                        background: route.segments[0] === mod.route.replace("/", "") ? "color-mix(in srgb, var(--accent) 10%, transparent)" : "transparent",
                        border: "none",
                        borderRadius: "8px",
                        color: route.segments[0] === mod.route.replace("/", "") ? "var(--accent)" : "var(--text)",
                        cursor: "pointer",
                        textAlign: "left",
                        fontSize: "var(--text-sm)",
                        fontWeight: route.segments[0] === mod.route.replace("/", "") ? 600 : 400,
                        transition: "background var(--motion-duration-fast) var(--motion-ease-out), color var(--motion-duration-fast) var(--motion-ease-out)",
                      }}
                      onMouseEnter={(e) => {
                        if (route.segments[0] !== mod.route.replace("/")) {
                          e.currentTarget.style.background = "var(--surface-muted)";
                        }
                      }}
                      onMouseLeave={(e) => {
                        if (route.segments[0] !== mod.route.replace("/")) {
                          e.currentTarget.style.background = "transparent";
                        }
                      }}
                    >
                      <Icon size={18} style={{ color: route.segments[0] === mod.route.replace("/", "") ? "var(--accent)" : "var(--text-muted)" }} />
                      <span>{mod.title}</span>
                    </button>
                  </li>
                );
              })}
            </ul>
          </nav>
        </aside>
        <main className="content">
          <header className="content-bar">
            <Breadcrumbs route={route} />
          </header>
          <section className="content-body">
            <TargetHighlightBoundary>
              <Suspense fallback={<PanelSkeleton />}>
                {PanelComponent ? <PanelComponent /> : <OverviewPage />}
              </Suspense>
            </TargetHighlightBoundary>
          </section>
        </main>
      </div>
      <SpotlightModal />
    </ShellFrame>
  );
}

function PanelSkeleton(): React.ReactElement {
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-6)" }}>
      <div style={{ height: "48px", background: "var(--surface-muted)", borderRadius: "12px", animation: "pulse 1.5s ease-in-out infinite" }} />
      <div style={{ height: "200px", background: "var(--surface-muted)", borderRadius: "12px", animation: "pulse 1.5s ease-in-out infinite 0.2s" }} />
      <div style={{ height: "200px", background: "var(--surface-muted)", borderRadius: "12px", animation: "pulse 1.5s ease-in-out infinite 0.4s" }} />
      <style jsx>{`
        @keyframes pulse {
          0%, 100% { opacity: 0.6; }
          50% { opacity: 1; }
        }
      `}</style>
    </div>
  );
}