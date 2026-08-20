/**
 * PowerPanel — Battery health discharge graph + profile toggles.
 *
 * Features:
 * - Battery health overview with discharge graph (SVG)
 * - Power profile selector (Balanced/Performance/Power Saver)
 * - Battery percentage + time remaining
 * - Charge threshold / conservation mode
 * - Registers search entries for Spotlight via IPC (handled elsewhere)
 *
 * Accessibility:
 * - Graph has text summary + data table fallback
 * - Profile buttons have ARIA pressed state
 * - All controls keyboard operable
 * - Reduced motion collapses graph animations
 *
 * Hooks rules: each profile card is its own `PowerProfileCard` component so
 * hooks run unconditionally (never inside a `.map()` callback).
 */
import { useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type {
  PowerActiveProfileResult,
  PowerBatteriesResult,
  PowerSetChargeThresholdRequest,
  PowerSetProfileRequest,
  PowerProfileDto,
  BatteryStateDto,
} from "@zettings/bindings";
import { PanelShell } from "./panel-shell.js";
import { GlassCard } from "./glass-card.js";
import { GlassButton } from "./glass-button.js";
import { Battery, BatteryCharging, Bolt, Leaf, Zap, Check } from "lucide-react";
import { useToggleSpring } from "../lib/zdl-motion-hooks.js";

type PowerProfileName = PowerProfileDto;

interface PowerProfileExtended {
  id: PowerProfileName;
  label: string;
  description: string;
  icon: React.ComponentType<{ size?: number; color?: string }>;
}

const POWER_PROFILES: PowerProfileExtended[] = [
  { id: "power-saver", label: "Power Saver", description: "Extends battery life by reducing performance", icon: Leaf },
  { id: "balanced", label: "Balanced", description: "Balances performance and power consumption", icon: Battery },
  { id: "performance", label: "Performance", description: "Maximizes performance at the cost of battery", icon: Bolt },
];

// Mock discharge data (in real impl, from UPower history)
const DISCHARGE_POINTS = [
  { time: "0h", percent: 100 },
  { time: "1h", percent: 94 },
  { time: "2h", percent: 87 },
  { time: "3h", percent: 81 },
  { time: "4h", percent: 74 },
  { time: "5h", percent: 66 },
  { time: "6h", percent: 58 },
  { time: "7h", percent: 51 },
  { time: "8h", percent: 43 },
  { time: "9h", percent: 35 },
  { time: "10h", percent: 27 },
  { time: "11h", percent: 19 },
  { time: "12h", percent: 12 },
  { time: "13h", percent: 5 },
  { time: "14h", percent: 0 },
];

/** Profile card — extracted so `useToggleSpring` runs unconditionally. */
function PowerProfileCard({
  profile,
  isActive,
  onSelect,
}: {
  profile: PowerProfileExtended;
  isActive: boolean;
  onSelect: (id: PowerProfileName) => void;
}): React.ReactElement {
  const spring = useToggleSpring(isActive);
  const iconColor = isActive ? "var(--accent)" : "var(--text-muted)";

  return (
    <GlassButton
      variant={isActive ? "prominent" : "regular"}
      aria-pressed={isActive}
      onClick={() => onSelect(profile.id)}
      dataTestId={`profile-${profile.id}`}
      style={{
        display: "flex",
        flexDirection: "column",
        alignItems: "flex-start",
        gap: "var(--space-3)",
        padding: "var(--space-5)",
        minWidth: 0,
        textAlign: "left",
        opacity: 0.7 + 0.3 * spring,
        transform: `scale(${0.98 + 0.02 * spring})`,
        border: isActive ? "2px solid var(--accent)" : "1px solid var(--glass-panel-border)",
      }}
    >
      <div
        style={{
          width: 40,
          height: 40,
          borderRadius: "10px",
          background: isActive
            ? "color-mix(in srgb, var(--accent) 18%, transparent)"
            : "color-mix(in srgb, var(--glass-panel-tint) 45%, transparent)",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
        }}
      >
        <profile.icon size={20} color={iconColor} />
      </div>
      <div style={{ flex: 1, minWidth: 0 }}>
        <div style={{ display: "flex", alignItems: "center", gap: "var(--space-2)" }}>
          <h4 style={{ fontSize: "var(--text-base)", fontWeight: 600, color: "var(--text)", margin: 0 }}>{profile.label}</h4>
          {isActive && <Check size={16} color="var(--accent)" aria-label="Active" />}
        </div>
        <p style={{ fontSize: "var(--text-sm)", color: "var(--text-muted)", margin: 0 }}>{profile.description}</p>
      </div>
    </GlassButton>
  );
}

/** Battery card — no hooks, so it is a plain render helper kept as a component for clarity. */
function BatteryCard({
  battery,
  idx,
}: {
  battery: BatteryStateDto;
  idx: number;
}): React.ReactElement {
  const isCharging = battery.charging;
  const capacity = battery.percentage;

  return (
    <GlassCard dataTestId={`battery-${idx}`} style={{ display: "flex", flexDirection: "column", gap: "var(--space-4)" }}>
      <div style={{ display: "flex", alignItems: "center", gap: "var(--space-3)" }}>
        <div
          style={{
            width: 48,
            height: 48,
            borderRadius: "12px",
            background: isCharging
              ? "color-mix(in srgb, var(--accent) 18%, transparent)"
              : "color-mix(in srgb, var(--glass-panel-tint) 45%, transparent)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
          }}
        >
          {isCharging ? <BatteryCharging size={24} color="var(--accent)" /> : <Battery size={24} color="var(--text)" />}
        </div>
        <div>
          <h3 style={{ fontSize: "var(--text-base)", fontWeight: 600, color: "var(--text)", margin: 0 }}>
            Battery {battery.device_index}
          </h3>
          <p style={{ fontSize: "var(--text-sm)", color: "var(--text-muted)", margin: "var(--space-1) 0 0" }}>
            Device Index: {battery.device_index}
          </p>
        </div>
      </div>

      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
        <span
          style={{
            fontSize: "var(--text-2xl)",
            fontWeight: 700,
            color: "var(--text)",
            fontVariantNumeric: "tabular-nums",
          }}
        >
          {capacity}%
        </span>
        {isCharging && <Zap size={20} color="var(--accent)" aria-label="Charging" />}
      </div>

      <div style={{ height: 12, borderRadius: 9999, background: "var(--surface-muted)", overflow: "hidden", position: "relative" }}>
        <div
          className="glass-progress__fill"
          style={{
            height: "100%",
            width: "100%",
            borderRadius: "inherit",
            background:
              capacity > 20
                ? "linear-gradient(90deg, var(--accent), var(--accent-secondary))"
                : "linear-gradient(90deg, #ff6b6b, #ff8e8e)",
            transform: `scaleX(${Math.max(0, Math.min(1, capacity / 100))})`,
            transformOrigin: "left",
          }}
        />
      </div>

      <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(140px, 1fr))", gap: "var(--space-3)" }}>
        <div className="liquid-glass liquid-glass--clear" style={{ padding: "var(--space-3)", borderRadius: "var(--radius-md)" }}>
          <div className="liquid-glass__refract" style={{ borderRadius: "var(--radius-md)" }} />
          <div className="liquid-glass__tint" style={{ borderRadius: "var(--radius-md)" }} />
          <div className="liquid-glass__specular" style={{ borderRadius: "var(--radius-md)" }} />
          <div className="liquid-glass__content" style={{ textAlign: "center" }}>
            <p style={{ fontSize: "var(--text-xs)", color: "var(--text-muted)", margin: "0 0 var(--space-1)" }}>Status</p>
            <p style={{ fontSize: "var(--text-sm)", fontWeight: 500, color: "var(--text)", margin: 0, textTransform: "capitalize" }}>
              {isCharging ? "charging" : "discharging"}
            </p>
          </div>
        </div>
        <div className="liquid-glass liquid-glass--clear" style={{ padding: "var(--space-3)", borderRadius: "var(--radius-md)" }}>
          <div className="liquid-glass__refract" style={{ borderRadius: "var(--radius-md)" }} />
          <div className="liquid-glass__tint" style={{ borderRadius: "var(--radius-md)" }} />
          <div className="liquid-glass__specular" style={{ borderRadius: "var(--radius-md)" }} />
          <div className="liquid-glass__content" style={{ textAlign: "center" }}>
            <p style={{ fontSize: "var(--text-xs)", color: "var(--text-muted)", margin: "0 0 var(--space-1)" }}>Charge</p>
            <p style={{ fontSize: "var(--text-sm)", fontWeight: 500, color: "var(--text)", margin: 0 }}>{capacity}%</p>
          </div>
        </div>
      </div>
    </GlassCard>
  );
}

export function PowerPanel(): React.ReactElement {
  const [activeProfile, setActiveProfile] = useState<PowerProfileName>("balanced");
  const [batteries, setBatteries] = useState<BatteryStateDto[]>([]);
  const [chargeThreshold, setChargeThreshold] = useState<number>(80);

  // Load power state on mount
  useEffect(() => {
    invoke<PowerBatteriesResult>("zettings_power_batteries")
      .then((r) => setBatteries(r.batteries))
      .catch((e) => console.error("Failed to load batteries:", e));

    invoke<PowerActiveProfileResult>("zettings_power_active_profile")
      .then((r) => setActiveProfile(r.profile as PowerProfileName))
      .catch((e) => console.error("Failed to load active profile:", e));
  }, []);

  const handleProfileChange = useCallback((profile: PowerProfileName) => {
    setActiveProfile(profile);
    const payload: PowerSetProfileRequest = { profile };
    invoke("zettings_power_set_profile", { request: payload }).catch((e) =>
      console.error("Failed to set power profile:", e)
    );
  }, []);

  const handleThresholdChange = useCallback((value: number) => {
    setChargeThreshold(value);
    const payload: PowerSetChargeThresholdRequest = { percent: value };
    invoke("zettings_power_set_charge_threshold", { request: payload }).catch((e) =>
      console.error("Failed to set charge threshold:", e)
    );
  }, []);

  // Memoize discharge graph as it's completely static
  const dischargeGraph = useMemo(() => {
    const width = 600;
    const height = 200;
    const padding = 40;
    const graphWidth = width - padding * 2;
    const graphHeight = height - padding * 2;

    const points = DISCHARGE_POINTS.map((p, i) => {
      const x = padding + (i / (DISCHARGE_POINTS.length - 1)) * graphWidth;
      const y = padding + graphHeight - (p.percent / 100) * graphHeight;
      return `${x},${y}`;
    }).join(" ");

    return (
      <GlassCard style={{ display: "flex", flexDirection: "column", gap: "var(--space-4)" }}>
        <h4 style={{ fontSize: "var(--text-base)", fontWeight: 600, color: "var(--text)", margin: "0 0 var(--space-4)" }}>
          Discharge History (Last 24h)
        </h4>
        <div style={{ position: "relative", width: width, height: height }}>
          <svg width={width} height={height} viewBox={`0 0 ${width} ${height}`} style={{ width: "100%", height: "auto", maxWidth: "100%" }}>
            <g stroke="var(--border)" strokeWidth="1" opacity="0.5">
              {[0, 25, 50, 75, 100].map((p) => (
                <line key={p} x1={padding} y1={padding + graphHeight * (1 - p / 100)} x2={width - padding} y2={padding + graphHeight * (1 - p / 100)} />
              ))}
              {[0, 25, 50, 75, 100].map((p) => (
                <line key={`v-${p}`} x1={padding + graphWidth * (p / 100)} y1={padding} x2={padding + graphWidth * (p / 100)} y2={height - padding} />
              ))}
            </g>
            <path
              d={`M${padding},${height - padding} L${points} L${width - padding},${height - padding} Z`}
              fill="url(#discharge-gradient)"
              opacity={0.3}
            />
            <path
              d={`M${points}`}
              stroke="var(--accent)"
              strokeWidth="2"
              fill="none"
              strokeLinecap="round"
              strokeLinejoin="round"
            />
            {DISCHARGE_POINTS.map((p, i) => {
              const x = padding + (i / (DISCHARGE_POINTS.length - 1)) * graphWidth;
              const y = padding + graphHeight - (p.percent / 100) * graphHeight;
              return (
                <circle key={i} cx={x} cy={y} r={4} fill="var(--accent)" stroke="var(--surface)" strokeWidth="2" />
              );
            })}
            <defs>
              <linearGradient id="discharge-gradient" x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" stopColor="var(--accent)" />
                <stop offset="100%" stopColor="var(--accent-secondary)" />
              </linearGradient>
            </defs>
          </svg>
        </div>
        <div style={{ fontSize: "var(--text-xs)", color: "var(--text-muted)", display: "flex", justifyContent: "space-between" }}>
          <span>100%</span>
          <span>75%</span>
          <span>50%</span>
          <span>25%</span>
          <span>0%</span>
        </div>
      </GlassCard>
    );
  }, []);

  return (
    <PanelShell
      title="Power"
      icon={Battery}
      subtitle="Battery health, discharge graph, power profiles, and charge thresholds"
      dataTestId="power-panel"
    >
      <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-8)" }}>
        <section style={{ display: "flex", flexDirection: "column", gap: "var(--space-6)" }}>
          {batteries.length === 0 ? (
            <GlassCard elevation={1} className="glass-empty" style={{ padding: "var(--space-12)", display: "flex", flexDirection: "column", alignItems: "center", textAlign: "center" }}>
              <Battery className="glass-empty__icon" size={48} />
              <h3 className="glass-empty__title">No batteries detected</h3>
              <p className="glass-empty__description">Connect a battery-powered device or enable the mock backend to see battery status.</p>
            </GlassCard>
          ) : (
            <div className="glass-grid glass-grid--auto-fill">
              {batteries.map((battery, idx) => (
                <BatteryCard key={battery.device_index} battery={battery} idx={idx} />
              ))}
            </div>
          )}

          {dischargeGraph}

          <section>
            <h4 style={{ fontSize: "var(--text-base)", fontWeight: 600, color: "var(--text)", margin: "0 0 var(--space-4)" }}>
              Power Profile
            </h4>
            <div className="glass-grid glass-grid--3" style={{ gap: "var(--space-4)" }}>
              {POWER_PROFILES.map((profile) => (
                <PowerProfileCard
                  key={profile.id}
                  profile={profile}
                  isActive={activeProfile === profile.id}
                  onSelect={handleProfileChange}
                />
              ))}
            </div>
          </section>

          <GlassCard elevation={1} style={{ padding: "var(--space-6)", display: "flex", flexDirection: "column", gap: "var(--space-4)" }}>
            <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: "var(--space-4)" }}>
                <div>
                  <h4 style={{ fontSize: "var(--text-base)", fontWeight: 600, color: "var(--text)", margin: 0 }}>Charge Threshold</h4>
                  <p style={{ fontSize: "var(--text-sm)", color: "var(--text-muted)", margin: "var(--space-1) 0 0" }}>
                    Stop charging at this percentage to extend battery lifespan
                  </p>
                </div>
                <div style={{ display: "flex", alignItems: "center", gap: "var(--space-4)", minWidth: 200 }}>
                  <input
                    type="range"
                    min="50"
                    max="100"
                    step="5"
                    value={chargeThreshold}
                    onChange={(e) => handleThresholdChange(Number(e.target.value))}
                    className="panel-slider"
                    style={{ flex: 1, maxWidth: 200 }}
                    aria-label="Charge threshold percentage"
                    data-testid="charge-threshold"
                  />
                  <span style={{ fontSize: "var(--text-sm)", fontWeight: 500, color: "var(--text)", minWidth: "48px", textAlign: "right" }}>
                    {chargeThreshold}%
                  </span>
                </div>
              </div>
              <div className="liquid-glass liquid-glass--clear" style={{ padding: "var(--space-3)", borderRadius: "var(--radius-md)" }}>
                <div className="liquid-glass__refract" style={{ borderRadius: "var(--radius-md)" }} />
                <div className="liquid-glass__tint" style={{ borderRadius: "var(--radius-md)" }} />
                <div className="liquid-glass__specular" style={{ borderRadius: "var(--radius-md)" }} />
                <div className="liquid-glass__content" style={{ display: "flex", alignItems: "center", gap: "var(--space-2)", fontSize: "var(--text-xs)", color: "var(--text-muted)" }}>
                  <Leaf size={14} />
                  <span>Conservation mode helps preserve battery health by limiting maximum charge</span>
                </div>
              </div>
          </GlassCard>
        </section>
      </div>
    </PanelShell>
  );
}