import { useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { Suspense, useEffect, useRef, useState, lazy } from "react";
import type { Health } from "@zettings/bindings";
import {
  Battery,
  Bluetooth,
  Contrast,
  Monitor,
  Palette,
  Volume2,
  Wifi,
} from "lucide-react";
import { Breadcrumbs } from "../components/breadcrumbs.js";
import { PerfMonitor } from "../components/perf-monitor.js";
import { ShellFrame } from "../components/shell-frame.js";
import { SpotlightModal } from "../components/spotlight-modal.js";
import { TargetHighlightBoundary } from "../components/target-highlight-boundary.js";
import { useHashRoute } from "../lib/hash-route.js";
import { PERF_MARKS, perfMark } from "../lib/perf.js";
import { useSpotlightStore } from "../stores/spotlight-store.js";

// Shared icon mappings for sidebar and quick actions (lucide-react)
const ICONS: Record<string, React.FC<{ size?: number; color?: string }>> = {
  Display: (props) => <Monitor size={props.size ?? 16} color={props.color ?? "currentColor"} strokeWidth={2} />,
  Sound: (props) => <Volume2 size={props.size ?? 16} color={props.color ?? "currentColor"} strokeWidth={2} />,
  Network: (props) => <Wifi size={props.size ?? 16} color={props.color ?? "currentColor"} strokeWidth={2} />,
  Bluetooth: (props) => <Bluetooth size={props.size ?? 16} color={props.color ?? "currentColor"} strokeWidth={2} />,
  Power: (props) => <Battery size={props.size ?? 16} color={props.color ?? "currentColor"} strokeWidth={2} />,
  Personalization: (props) => <Palette size={props.size ?? 16} color={props.color ?? "currentColor"} strokeWidth={2} />,
  Contrast: (props) => <Contrast size={props.size ?? 16} color={props.color ?? "currentColor"} strokeWidth={2} />,
  Wifi: (props) => <Wifi size={props.size ?? 16} color={props.color ?? "currentColor"} strokeWidth={2} />,
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
    icon: string; // Key into ICONS
    keywords: string[];
  };
}

function ModuleCard({ module }: ModuleCardProps): React.ReactElement {
  const Icon = ICONS[module.icon] ?? ICONS.Wifi!;
  return (
    <button
      onClick={() => window.location.hash = module.route}
      className="liquid-glass"
      style={{
        position: "relative",
        overflow: "hidden",
        borderRadius: "16px",
        display: "flex",
        flexDirection: "column",
        gap: "var(--space-3)",
        padding: "var(--space-5)",
        cursor: "pointer",
        textAlign: "left",
        background: "transparent",
        border: "none",
      }}
    >
      {/* Layer 0: Refraction */}
      <div className="liquid-glass__refract" style={{ borderRadius: "16px" }} />
      {/* Layer 1: Tint */}
      <div className="liquid-glass__tint" style={{ borderRadius: "16px" }} />
      {/* Layer 2: Specular */}
      <div className="liquid-glass__specular" style={{ borderRadius: "16px" }} />
      {/* Layer 3: Content */}
      <div className="liquid-glass__content">
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
  const Icon = ICONS[IconName] ?? ICONS.Wifi!;

  return (
    <button
      onClick={onClick}
      className="liquid-glass"
      style={{
        position: "relative",
        overflow: "hidden",
        borderRadius: "10px",
        display: "inline-flex",
        alignItems: "center",
        gap: "var(--space-2)",
        padding: "var(--space-2) var(--space-3)",
        border: "none",
        cursor: "pointer",
        background: "transparent",
        color: "var(--text)",
        fontSize: "var(--text-sm)",
        fontWeight: 500,
      }}
    >
      {/* Layer 0: Refraction */}
      <div className="liquid-glass__refract" style={{ borderRadius: "10px" }} />
      {/* Layer 1: Tint */}
      <div className="liquid-glass__tint" style={{ borderRadius: "10px" }} />
      {/* Layer 2: Specular */}
      <div className="liquid-glass__specular" style={{ borderRadius: "10px" }} />
      {/* Layer 3: Content */}
      <div className="liquid-glass__content" style={{ display: "inline-flex", alignItems: "center", gap: "var(--space-2)" }}>
        <Icon size={16} color="var(--accent)" />
        <span>{label}</span>
      </div>
    </button>
  );
}

const MODULE_CARDS = [
  { route: "/display", title: "Displays", description: "Arrange monitors, resolution, refresh rate, night light", icon: "Display", keywords: ["monitor", "screen", "arrange", "resolution", "night light"] },
  { route: "/audio", title: "Sound", description: "Master volume, per-app mixer, equalizer, output device", icon: "Sound", keywords: ["volume", "mixer", "equalizer", "output", "bluetooth audio"] },
  { route: "/network", title: "Network", description: "Wi-Fi networks, ethernet, known networks, VPN", icon: "Network", keywords: ["wifi", "ethernet", "scan", "connect", "password"] },
  { route: "/bluetooth", title: "Bluetooth", description: "Paired devices, battery levels, connect/disconnect", icon: "Bluetooth", keywords: ["pair", "headphones", "mouse", "keyboard", "battery"] },
  { route: "/power", title: "Power", description: "Battery health, discharge graph, performance profiles", icon: "Power", keywords: ["battery", "profile", "performance", "power saver", "cycles"] },
  { route: "/personalization", title: "Personalization", description: "Accent colors, corner roundness, glass blur, themes", icon: "Personalization", keywords: ["accent", "theme", "dark mode", "squircle", "glass", "blur"] },
];

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

  // Phase 9 hot-start tracing: mark the route dispatch and the first frame
  // after the new panel commits. `requestAnimationFrame` posts the paint mark
  // after the browser has produced a frame for the committed DOM, giving an
  // upper-bound proxy for "new route visible" (see docs/performance/audit.md).
  const prevRouteRef = useRef<string>(route.raw);
  useEffect(() => {
    const prev = prevRouteRef.current;
    prevRouteRef.current = route.raw;
    if (prev === route.raw) return;
    perfMark(PERF_MARKS.routeStart);
    const rafId = requestAnimationFrame(() => {
      perfMark(PERF_MARKS.routePainted);
    });
    return () => cancelAnimationFrame(rafId);
  }, [route.raw]);

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
      {/* Liquid Glass SVG filter definition — defines the refraction displacement map */}
      <svg className="liquid-glass-filter" aria-hidden="true">
        <defs>
          <filter id="liquid-glass-refract" color-interpolation-filters="sRGB" x="0%" y="0%" width="100%" height="100%">
            <feImage result="dispMap" x="0" y="0" width="400" height="400" preserveAspectRatio="none"
              href="data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSI0MDAiIGhlaWdodD0iNDAwIj48ZGVmcz48cmFkaWFsR3JhZGllbnQgaWQ9ImciIHI9IjUwJSIgY3g9IjUwJSIgY3k9IjUwJSI+PHN0b3Agb2Zmc2V0PSIwJSIgc3RvcC1jb2xvcj0iIzgwODA4MCIvPjxzdG9wIG9mZnNldD0iNzAlIiBzdG9wLWNvbG9yPSIjODA4MDgwIi8+PHN0b3Agb2Zmc2V0PSIxMDAlIiBzdG9wLWNvbG9yPSIjYzBjMGMwIi8+PC9yYWRpYWxHcmFkaWVudD48L2RlZnM+PHJlY3Qgd2lkdGg9IjEwMCUiIGhlaWdodD0iMTAwJSIgZmlsbD0idXJsKCNnKSIvPjwvc3ZnPg==" />
            <feDisplacementMap in="SourceGraphic" in2="dispMap" scale="-30" xChannelSelector="R" yChannelSelector="G" />
          </filter>
        </defs>
      </svg>
      <div className="zettings-shell" data-reduced-motion={reducedMotion}>
        <aside className="sidebar">
          <header className="sidebar-title">
            <span className="brand-glyph" aria-hidden />
            <h1>Zettings</h1>
          </header>
          <nav aria-label="Settings modules">
            <ul style={{ listStyle: "none", padding: 0, margin: 0, display: "flex", flexDirection: "column", gap: "var(--space-1)" }}>
              {MODULE_CARDS.map((mod) => {
                const Icon = ICONS[mod.icon] ?? ICONS.Wifi!;
                const isActive = route.segments[0] === mod.route.replace("/", "");
                return (
                  <li key={mod.route}>
                    <button
                      onClick={() => window.location.hash = mod.route}
                      className="liquid-glass"
                      style={{
                        position: "relative",
                        overflow: "hidden",
                        borderRadius: "12px",
                        display: "flex",
                        alignItems: "center",
                        gap: "var(--space-3)",
                        width: "100%",
                        padding: "var(--space-3) var(--space-4)",
                        border: "none",
                        color: isActive ? "var(--accent)" : "var(--text)",
                        cursor: "pointer",
                        textAlign: "left",
                        fontSize: "var(--text-sm)",
                        fontWeight: isActive ? 600 : 400,
                        background: "transparent",
                      }}
                    >
                      {/* Layer 0: Refraction */}
                      <div className="liquid-glass__refract" style={{ borderRadius: "12px" }} />
                      {/* Layer 1: Tint */}
                      <div className="liquid-glass__tint" style={{ borderRadius: "12px", background: isActive ? "rgba(var(--accent-rgb), 0.18)" : "rgba(255, 255, 255, 0.12)" }} />
                      {/* Layer 2: Specular */}
                      <div className="liquid-glass__specular" style={{ borderRadius: "12px" }} />
                      {/* Layer 3: Content */}
                      <div className="liquid-glass__content" style={{ display: "flex", alignItems: "center", gap: "var(--space-3)", width: "100%" }}>
                        <Icon size={18} color={isActive ? "var(--accent)" : "var(--text-muted)"} />
                        <span>{mod.title}</span>
                      </div>
                    </button>
                  </li>
                );
              })}
            </ul>
          </nav>
        </aside>
        <main className="content">
          <header className="content-bar liquid-glass" style={{ position: "relative", overflow: "hidden", borderRadius: "var(--radius-lg)" }}>
            {/* Layer 0: Refraction */}
            <div className="liquid-glass__refract" style={{ borderRadius: "var(--radius-lg)" }} />
            {/* Layer 1: Tint */}
            <div className="liquid-glass__tint" style={{ borderRadius: "var(--radius-lg)" }} />
            {/* Layer 2: Specular */}
            <div className="liquid-glass__specular" style={{ borderRadius: "var(--radius-lg)" }} />
            {/* Layer 3: Content */}
            <div className="liquid-glass__content">
              <Breadcrumbs route={route} />
            </div>
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
      {import.meta.env.DEV ? <PerfMonitor /> : null}
    </ShellFrame>
  );
}

function PanelSkeleton(): React.ReactElement {
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-6)" }}>
      <div className="pulse" style={{ height: "48px", background: "var(--surface-muted)", borderRadius: "12px" }} />
      <div className="pulse" style={{ height: "200px", background: "var(--surface-muted)", borderRadius: "12px", animationDelay: "0.2s" }} />
      <div className="pulse" style={{ height: "200px", background: "var(--surface-muted)", borderRadius: "12px", animationDelay: "0.4s" }} />
    </div>
  );
}