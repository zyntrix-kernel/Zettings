/**
 * PersonalizationPanel — Accent picker + squircle/blur sliders.
 *
 * Features:
 * - Wallpaper accent color extraction (via palette_extract IPC)
 * - Custom accent color picker with live preview
 * - Squircle roundness slider (G2 order 4 → G3 order 6)
 * - Glass blur intensity slider
 * - Theme variant selector (Light/Dark/OLED/HC)
 * - Registers search entries for Spotlight
 *
 * Accessibility:
 * - Color picker has text input fallback
 * - Sliders keyboard operable with Arrow keys
 * - Live preview updates respect reduced motion
 * - All controls have visible labels (no placeholder-as-label)
 */
import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AccentPaletteDto, PaletteExtractResult } from "@zettings/bindings";
import { PanelShell } from "./panel-shell.js";
import { GlassCard } from "./glass-card.js";
import { GlassButton } from "./glass-button.js";
import {
  Palette,
  Image,
  Sun,
  Moon,
  Contrast,
  Eye,
  Check,
  RotateCcw,
  Sparkles
} from "lucide-react";
import { useSpring, ZDL_SPRINGS } from "../lib/zdl-motion-hooks.js";

const THEME_VARIANTS = [
  { id: "light", name: "Light", icon: Sun, description: "Clean light theme" },
  { id: "dark", name: "Dark", icon: Moon, description: "Easy on the eyes" },
  { id: "oled", name: "OLED", icon: Contrast, description: "True black for OLED displays" },
  { id: "hc", name: "High Contrast", icon: Eye, description: "Maximum accessibility" },
] as const;

type ThemeVariantId = (typeof THEME_VARIANTS)[number]["id"];

// Default Zyntrix Aurora colors
const DEFAULT_ACCENT = "#58aebc";
const DEFAULT_ACCENT_ON = "#0c0a09";
const DEFAULT_ACCENT_SECONDARY = "#7f62cf";

function hexToRgb(hex: string): [number, number, number] {
  const clean = hex.replace("#", "");
  const r = parseInt(clean.slice(0, 2), 16) / 255;
  const g = parseInt(clean.slice(2, 4), 16) / 255;
  const b = parseInt(clean.slice(4, 6), 16) / 255;
  return [r, g, b];
}

function rgbToHex(r: number, g: number, b: number): string {
  const toHex = (v: number) => Math.round(Math.max(0, Math.min(1, v)) * 255).toString(16).padStart(2, "0");
  return `#${toHex(r)}${toHex(g)}${toHex(b)}`;
}

function getContrastColor(r: number, g: number, b: number): string {
  // Relative luminance
  const luminance = 0.2126 * r + 0.7152 * g + 0.0722 * b;
  return luminance > 0.5 ? "#0c0a09" : "#fafaf9";
}

interface PreviewBoxProps {
  color: string;
  size?: number;
}

function PreviewBox({ color, size = 48 }: PreviewBoxProps): React.ReactElement {
  const [r, g, b] = hexToRgb(color);
  const contrast = getContrastColor(r, g, b);
  return (
    <GlassCard width={size} height={size} radius={12} elevation={1} dataTestId={`preview-${color}`} style={{ display: "flex", alignItems: "center", justifyContent: "center", flexShrink: 0 }}>
      <div style={{ background: color, borderRadius: "calc(var(--radius-md) - 2px)", width: "calc(100% - 4px)", height: "calc(100% - 4px)", display: "flex", alignItems: "center", justifyContent: "center", color: contrast, fontSize: "var(--text-xs)", fontWeight: 600 }}>
        {color}
      </div>
    </GlassCard>
  );
}

interface ThemeCardProps {
  variant: (typeof THEME_VARIANTS)[number];
  isActive: boolean;
  onClick: () => void;
}

/** Theme variant card. Extracted so the panel never calls hooks in a `.map()`. */
function ThemeCard({ variant, isActive, onClick }: ThemeCardProps): React.ReactElement {
  const cardSpring = useSpring(isActive ? 1 : 0, ZDL_SPRINGS.slider);

  return (
    <GlassButton
      key={variant.id}
      variant="regular"
      onClick={onClick}
      aria-pressed={isActive}
      dataTestId={`theme-${variant.id}`}
      style={{
        display: "flex",
        flexDirection: "column",
        gap: "var(--space-3)",
        padding: "var(--space-5)",
        flex: 1,
        minWidth: 0,
        textAlign: "center",
        height: "auto",
        opacity: 0.7 + 0.3 * cardSpring.position,
        transform: `scale(${0.98 + 0.02 * cardSpring.position})`,
        border: isActive ? "2px solid var(--accent)" : "none",
        transition: "opacity var(--motion-duration-fast) var(--motion-ease-out), transform var(--motion-duration-fast) var(--motion-ease-out), border-color var(--motion-duration-fast) var(--motion-ease-out)",
      }}
    >
      <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-3)", alignItems: "center" }}>
        <GlassCard
          width={48}
          height={48}
          radius={12}
          elevation={1}
          style={{ display: "flex", alignItems: "center", justifyContent: "center", border: isActive ? "2px solid var(--accent)" : "1px solid var(--border)" }}
        >
          <variant.icon size={24} color={isActive ? "var(--accent)" : "var(--text-muted)"} />
        </GlassCard>
        <div>
          <h4 style={{ fontSize: "var(--text-base)", fontWeight: 600, color: "var(--text)", margin: 0 }}>{variant.name}</h4>
          <p style={{ fontSize: "var(--text-sm)", color: "var(--text-muted)", margin: "var(--space-1) 0 0" }}>{variant.description}</p>
        </div>
        {isActive && <Check size={20} color="var(--accent)" aria-label="Active" />}
      </div>
    </GlassButton>
  );
}

interface GlassSliderProps {
  label: string;
  value: number;
  min: number;
  max: number;
  step: number;
  onChange: (v: number) => void;
  unit?: string;
  marks?: { value: number; label: string }[];
  id: string;
}

/** Glass slider with spring-animated value display. */
function GlassSlider({ label, value, min, max, step, onChange, unit = "", marks, id }: GlassSliderProps): React.ReactElement {
  const spring = useSpring(value, ZDL_SPRINGS.slider);

  return (
    <GlassCard elevation={1} dataTestId={`glass-slider-${id}`} style={{ padding: "var(--space-4)", display: "flex", flexDirection: "column", gap: "var(--space-3)" }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <label htmlFor={id} className="panel-field-label" style={{ margin: 0 }}>{label}</label>
        <span style={{ fontSize: "var(--text-sm)", fontWeight: 500, color: "var(--accent)", fontVariantNumeric: "tabular-nums" }}>
          {spring.position.toFixed(step < 1 ? 1 : 0)}{unit}
        </span>
      </div>
      <input
        id={id}
        type="range"
        min={min}
        max={max}
        step={step}
        value={value}
        onChange={(e) => onChange(Number(e.target.value))}
        className="panel-slider"
        style={{ width: "100%" }}
        data-testid={id}
      />
      {marks && (
        <div style={{ display: "flex", justifyContent: "space-between", fontSize: "var(--text-xs)", color: "var(--text-subtle)" }}>
          {marks.map((m) => (
            <span key={m.value}>{m.label}</span>
          ))}
        </div>
      )}
    </GlassCard>
  );
}

interface ColorInputProps {
  label: string;
  value: string;
  onChange: (v: string) => void;
  id: string;
  previewSize?: number;
}

function ColorInput({ label, value, onChange, id, previewSize = 40 }: ColorInputProps): React.ReactElement {
  return (
    <GlassCard elevation={1} dataTestId={`color-input-${id}`} style={{ padding: "var(--space-4)", display: "flex", flexDirection: "column", gap: "var(--space-3)" }}>
      <label htmlFor={id} className="panel-field-label" style={{ margin: 0 }}>{label}</label>
      <div style={{ display: "flex", alignItems: "center", gap: "var(--space-4)" }}>
        <GlassCard
          width={previewSize}
          height={previewSize}
          radius={12}
          elevation={1}
          style={{ display: "flex", alignItems: "center", justifyContent: "center", flexShrink: 0 }}
        >
          <div style={{ background: value, borderRadius: "calc(var(--radius-md) - 2px)", width: "calc(100% - 4px)", height: "calc(100% - 4px)" }} />
        </GlassCard>
        <div style={{ flex: 1, display: "flex", flexDirection: "column", gap: "var(--space-2)" }}>
          <input
            id={id}
            type="color"
            value={value}
            onChange={(e) => onChange(e.target.value)}
            className="panel-input"
            style={{ width: "100%", height: 40, borderRadius: "var(--radius-sm)", cursor: "pointer", padding: "2px" }}
            data-testid={id}
          />
          <input
            type="text"
            value={value}
            onChange={(e) => {
              const v = e.target.value;
              if (/^#[0-9a-fA-F]{6}$/.test(v)) onChange(v);
            }}
            className="panel-input"
            placeholder="#RRGGBB"
            style={{ width: "100%" }}
            aria-label={`${label} hex value`}
          />
        </div>
      </div>
    </GlassCard>
  );
}

export function PersonalizationPanel(): React.ReactElement {
  const [accent, setAccent] = useState<string>(DEFAULT_ACCENT);
  const [accentOn, setAccentOn] = useState<string>(DEFAULT_ACCENT_ON);
  const [accentSecondary, setAccentSecondary] = useState<string>(DEFAULT_ACCENT_SECONDARY);
  const [squircleOrder, setSquircleOrder] = useState<4 | 6>(4);
  const [glassBlur, setGlassBlur] = useState<number>(16); // 8-32px
  const [glassSaturate, setGlassSaturate] = useState<number>(180); // 100-250%
  const [themeVariant, setThemeVariant] = useState<ThemeVariantId>("dark");
  const [extracting, setExtracting] = useState<boolean>(false);
  const [extractedPalette, setExtractedPalette] = useState<AccentPaletteDto | null>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  // Apply accent colors to CSS custom properties
  const applyAccent = useCallback((primary: string, on?: string, secondary?: string) => {
    const [r, g, b] = hexToRgb(primary);
    const root = document.documentElement;
    root.style.setProperty("--accent", primary);
    root.style.setProperty("--accent-rgb", `${Math.round(r * 255)}, ${Math.round(g * 255)}, ${Math.round(b * 255)}`);
    if (on) root.style.setProperty("--accent-on", on);
    if (secondary) root.style.setProperty("--accent-secondary", secondary);
  }, []);

  // Apply theme variant
  const applyTheme = useCallback((variant: string) => {
    document.documentElement.setAttribute("data-theme", variant);
  }, []);

  // Apply squircle order
  const applySquircle = useCallback((order: number) => {
    document.documentElement.style.setProperty("--squircle-order", String(order));
  }, []);

  // Apply glass blur/saturate
  const applyGlass = useCallback((blur: number, saturate: number) => {
    document.documentElement.style.setProperty("--glass-panel-blur", `${blur}px`);
    document.documentElement.style.setProperty("--glass-panel-saturate", `${saturate}%`);
  }, []);

  // Initialize on mount
  useEffect(() => {
    applyAccent(accent, accentOn, accentSecondary);
    applyTheme(themeVariant);
    applySquircle(squircleOrder);
    applyGlass(glassBlur, glassSaturate);
  }, [accent, accentOn, accentSecondary, themeVariant, squircleOrder, glassBlur, glassSaturate, applyAccent, applyTheme, applySquircle, applyGlass]);

  const handleExtractPalette = useCallback(async () => {
    if (!fileInputRef.current?.files?.[0]) return;
    setExtracting(true);
    try {
      const file = fileInputRef.current.files[0];
      const arrayBuffer = await file.arrayBuffer();
      const bytes = new Uint8Array(arrayBuffer);
      const result = await invoke<PaletteExtractResult>("zettings_palette_extract", { imageBytes: Array.from(bytes) });
      if (result.palette) {
        setExtractedPalette(result.palette);
        const [r, g, b] = result.palette.accent;
        const newAccent = rgbToHex(r, g, b);
        const [ro, go, bo] = result.palette.on_accent;
        const newAccentOn = rgbToHex(ro, go, bo);
        const [rs, gs, bs] = result.palette.secondary;
        const newAccentSecondary = rgbToHex(rs, gs, bs);
        setAccent(newAccent);
        setAccentOn(newAccentOn);
        setAccentSecondary(newAccentSecondary);
      }
    } catch (e) {
      console.error("Failed to extract palette:", e);
    } finally {
      setExtracting(false);
    }
  }, []);

  const handleFileSelect = useCallback(() => {
    fileInputRef.current?.click();
  }, []);

  const handleApplyExtracted = useCallback(() => {
    if (extractedPalette) {
      const [r, g, b] = extractedPalette.accent;
      const newAccent = rgbToHex(r, g, b);
      const [ro, go, bo] = extractedPalette.on_accent;
      const newAccentOn = rgbToHex(ro, go, bo);
      const [rs, gs, bs] = extractedPalette.secondary;
      const newAccentSecondary = rgbToHex(rs, gs, bs);
      setAccent(newAccent);
      setAccentOn(newAccentOn);
      setAccentSecondary(newAccentSecondary);
    }
  }, [extractedPalette]);

  const handleReset = useCallback(() => {
    setAccent(DEFAULT_ACCENT);
    setAccentOn(DEFAULT_ACCENT_ON);
    setAccentSecondary(DEFAULT_ACCENT_SECONDARY);
    setExtractedPalette(null);
  }, []);

  return (
    <PanelShell
      title="Personalization"
      icon={Palette}
      subtitle="Accent colors, corner roundness, glass blur, and theme variants"
      dataTestId="personalization-panel"
    >
      <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-8)" }}>
        {/* Accent color extraction */}
        <GlassCard elevation={1} style={{ padding: "var(--space-6)", display: "flex", flexDirection: "column", gap: "var(--space-6)" }}>
          <div>
            <h4 style={{ fontSize: "var(--text-base)", fontWeight: 600, color: "var(--text)", margin: 0 }}>Wallpaper Accent Extraction</h4>
            <p style={{ fontSize: "var(--text-sm)", color: "var(--text-muted)", margin: "var(--space-1) 0 0" }}>Upload a wallpaper image to automatically extract an accent color palette</p>
          </div>
          <div style={{ display: "flex", flexWrap: "wrap", gap: "var(--space-4)", alignItems: "center" }}>
            <input
              ref={fileInputRef}
              type="file"
              accept="image/*"
              onChange={handleExtractPalette}
              style={{ display: "none" }}
              data-testid="wallpaper-upload"
            />
            <GlassButton onClick={handleFileSelect} disabled={extracting} dataTestId="choose-image">
              <Image size={16} />
              {extracting ? "Extracting…" : "Choose Image"}
            </GlassButton>
            {extractedPalette && (
              <>
                <div style={{ display: "flex", alignItems: "center", gap: "var(--space-3)", flexWrap: "wrap" }}>
                  <PreviewBox color={rgbToHex(...extractedPalette.accent)} size={40} />
                  <PreviewBox color={rgbToHex(...extractedPalette.on_accent)} size={40} />
                  <PreviewBox color={rgbToHex(...extractedPalette.secondary)} size={40} />
                </div>
                <GlassButton variant="prominent" onClick={handleApplyExtracted} dataTestId="apply-palette">
                  <Sparkles size={16} /> Apply Palette
                </GlassButton>
              </>
            )}
            <GlassButton onClick={handleReset} dataTestId="reset-personalization">
              <RotateCcw size={16} /> Reset
            </GlassButton>
          </div>
        </GlassCard>

        {/* Custom accent color pickers */}
        <GlassCard elevation={1} style={{ padding: "var(--space-6)", display: "flex", flexDirection: "column", gap: "var(--space-4)" }}>
          <h4 style={{ fontSize: "var(--text-base)", fontWeight: 600, color: "var(--text)", margin: 0 }}>Custom Accent Colors</h4>
          <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(280px, 1fr))", gap: "var(--space-4)" }}>
            <ColorInput label="Primary Accent" value={accent} onChange={setAccent} id="accent-primary" />
            <ColorInput label="On Primary (Text)" value={accentOn} onChange={setAccentOn} id="accent-on" />
            <ColorInput label="Secondary Accent" value={accentSecondary} onChange={setAccentSecondary} id="accent-secondary" />
          </div>
        </GlassCard>

        {/* Glass material controls */}
        <GlassCard elevation={1} style={{ padding: "var(--space-6)", display: "flex", flexDirection: "column", gap: "var(--space-4)" }}>
          <h4 style={{ fontSize: "var(--text-base)", fontWeight: 600, color: "var(--text)", margin: 0 }}>Glass Material</h4>
          <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(280px, 1fr))", gap: "var(--space-4)" }}>
            <GlassSlider
              label="Corner Roundness (Squircle Order)"
              value={squircleOrder}
              min={4}
              max={6}
              step={2}
              onChange={(v) => setSquircleOrder(v as 4 | 6)}
              id="squircle-order"
              marks={[
                { value: 4, label: "G2 (Order 4)" },
                { value: 6, label: "G3 (Order 6)" },
              ]}
            />
            <GlassSlider
              label="Glass Blur Intensity"
              value={glassBlur}
              min={8}
              max={32}
              step={2}
              onChange={setGlassBlur}
              unit="px"
              id="glass-blur"
              marks={[
                { value: 8, label: "Subtle" },
                { value: 16, label: "Standard" },
                { value: 24, label: "Strong" },
                { value: 32, label: "Maximum" },
              ]}
            />
            <GlassSlider
              label="Glass Saturation"
              value={glassSaturate}
              min={100}
              max={250}
              step={10}
              onChange={setGlassSaturate}
              unit="%"
              id="glass-saturate"
              marks={[
                { value: 100, label: "Natural" },
                { value: 150, label: "Enhanced" },
                { value: 180, label: "Standard" },
                { value: 250, label: "Vibrant" },
              ]}
            />
          </div>
        </GlassCard>

        {/* Theme variant selector */}
        <GlassCard elevation={1} style={{ padding: "var(--space-6)", display: "flex", flexDirection: "column", gap: "var(--space-4)" }}>
          <h4 style={{ fontSize: "var(--text-base)", fontWeight: 600, color: "var(--text)", margin: 0 }}>Theme Variant</h4>
          <div className="glass-grid glass-grid--4" style={{ gap: "var(--space-4)" }}>
            {THEME_VARIANTS.map((variant) => (
              <ThemeCard key={variant.id} variant={variant} isActive={themeVariant === variant.id} onClick={() => setThemeVariant(variant.id)} />
            ))}
          </div>
        </GlassCard>

        {/* Live preview section */}
        <GlassCard elevation={1} className="glass-empty" style={{ padding: "var(--space-8)", display: "flex", flexDirection: "column", alignItems: "center", textAlign: "center" }}>
          <Sparkles className="glass-empty__icon" size={48} />
          <h3 className="glass-empty__title">Live Preview Active</h3>
          <p className="glass-empty__description">All changes above are applied instantly. Adjust colors, blur, and roundness to see real-time updates across the entire application.</p>
        </GlassCard>
      </div>
    </PanelShell>
  );
}