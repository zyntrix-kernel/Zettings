/**
 * ZDL Live Wallpaper painters.
 *
 * Each painter renders one animation frame into a 2D canvas context as a pure
 * function of time — no internal state, no allocations per frame beyond the
 * gradient objects the canvas API requires. This keeps the rAF loop in
 * `LiveWallpaper` trivially pausable/resumable and lets thumbnails render a
 * single static frame by passing `t = 0`.
 *
 * Performance contract (prompt.txt: 120 FPS, never block rendering):
 *   - No DOM reads/writes inside painters.
 *   - No `ctx.filter` (forces slow software paths); glow comes from radial
 *     gradients composited with `globalCompositeOperation`.
 *   - Particle counts scale with area but are clamped.
 *
 * Colors are resolved from the ZDL token cascade by the caller so theme
 * switches (light/dark/oled/hc) repaint automatically.
 */

import type { WallpaperPainterId } from "./wallpaper-store.js";

/** Theme-resolved colors passed to every painter. */
export interface WallpaperPalette {
  /** `--accent` resolved value. */
  accent: string;
  /** `--accent-secondary` resolved value. */
  accentSecondary: string;
  /** `--surface` resolved value (base fill). */
  surface: string;
  /** True when the active theme is dark/oled (painters bias luminance). */
  isDark: boolean;
}

export type PainterFn = (
  ctx: CanvasRenderingContext2D,
  width: number,
  height: number,
  timeSeconds: number,
  palette: WallpaperPalette,
  intensity: number,
) => void;

/** Parse `#rrggbb` / `#rgb` into [r,g,b]; returns null on other formats. */
function parseHex(color: string): [number, number, number] | null {
  const value = color.trim();
  const short = /^#([0-9a-f])([0-9a-f])([0-9a-f])$/i.exec(value);
  if (short !== null) {
    return [
      parseInt(short[1] ?? "0", 16) * 17,
      parseInt(short[2] ?? "0", 16) * 17,
      parseInt(short[3] ?? "0", 16) * 17,
    ];
  }
  const full = /^#([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})$/i.exec(value);
  if (full !== null) {
    return [
      parseInt(full[1] ?? "0", 16),
      parseInt(full[2] ?? "0", 16),
      parseInt(full[3] ?? "0", 16),
    ];
  }
  return null;
}

/** `rgba(r,g,b,a)` from any hex token value with a safe fallback. */
function rgba(color: string, alpha: number): string {
  const rgb = parseHex(color) ?? [88, 174, 188];
  return `rgba(${rgb[0]}, ${rgb[1]}, ${rgb[2]}, ${alpha})`;
}

/** Deterministic hash-based pseudo-random in [0,1) from an integer seed. */
function seeded(seed: number): number {
  const x = Math.sin(seed * 127.1 + 311.7) * 43758.5453;
  return x - Math.floor(x);
}

/* ---------------------------------------------------------------------------
   Aurora — large breathing gradient blobs on Lissajous drifts. */

const auroraBlobs = [
  { hueMix: 0.0, rx: 0.30, ry: 0.26, sx: 0.11, sy: 0.07, px: 0.25, py: 0.30 },
  { hueMix: 0.5, rx: 0.34, ry: 0.30, sx: 0.07, sy: 0.12, px: 0.72, py: 0.62 },
  { hueMix: 1.0, rx: 0.26, ry: 0.34, sx: 0.13, sy: 0.05, px: 0.50, py: 0.85 },
  { hueMix: 0.75, rx: 0.22, ry: 0.20, sx: 0.05, sy: 0.10, px: 0.15, py: 0.80 },
] as const;

const paintAurora: PainterFn = (ctx, w, h, t, palette, intensity) => {
  ctx.fillStyle = palette.surface;
  ctx.fillRect(0, 0, w, h);
  const baseAlpha = (palette.isDark ? 0.34 : 0.26) * (0.4 + intensity * 0.9);
  ctx.globalCompositeOperation = palette.isDark ? "screen" : "multiply";
  for (let i = 0; i < auroraBlobs.length; i++) {
    const blob = auroraBlobs[i]!;
    const cx = (blob.px + Math.sin(t * blob.sx * 2 + i * 1.7) * blob.rx) * w;
    const cy = (blob.py + Math.cos(t * blob.sy * 2 + i * 2.3) * blob.ry) * h;
    const radius = Math.max(w, h) * (0.32 + 0.06 * Math.sin(t * 0.35 + i));
    const color = blob.hueMix < 0.5 ? palette.accent : palette.accentSecondary;
    const gradient = ctx.createRadialGradient(cx, cy, 0, cx, cy, radius);
    gradient.addColorStop(0, rgba(color, baseAlpha));
    gradient.addColorStop(1, rgba(color, 0));
    ctx.fillStyle = gradient;
    ctx.fillRect(0, 0, w, h);
  }
  ctx.globalCompositeOperation = "source-over";
};

/* ---------------------------------------------------------------------------
   Waves — stacked sine bands drifting at phase offsets. */

const paintWaves: PainterFn = (ctx, w, h, t, palette, intensity) => {
  ctx.fillStyle = palette.surface;
  ctx.fillRect(0, 0, w, h);
  const layers = 5;
  for (let layer = 0; layer < layers; layer++) {
    const progress = layer / (layers - 1);
    const amplitude = h * (0.05 + 0.05 * progress) * (0.5 + intensity);
    const yBase = h * (0.55 + 0.09 * progress);
    const speed = 0.5 + progress * 0.7;
    const wavelength = w / (1.6 + progress * 1.4);
    const mixColor = progress < 0.5 ? palette.accent : palette.accentSecondary;
    ctx.beginPath();
    ctx.moveTo(0, h);
    for (let x = 0; x <= w; x += 8) {
      const phase = (x / wavelength) * Math.PI * 2 + t * speed;
      const y =
        yBase +
        Math.sin(phase) * amplitude +
        Math.sin(phase * 0.5 + layer) * amplitude * 0.4;
      ctx.lineTo(x, y);
    }
    ctx.lineTo(w, h);
    ctx.closePath();
    const gradient = ctx.createLinearGradient(0, yBase - amplitude, 0, h);
    gradient.addColorStop(0, rgba(mixColor, (palette.isDark ? 0.22 : 0.16) * (0.4 + intensity)));
    gradient.addColorStop(1, rgba(mixColor, 0));
    ctx.fillStyle = gradient;
    ctx.fill();
  }
};

/* ---------------------------------------------------------------------------
   Particles — drifting glowing motes with depth parallax. */

const PARTICLE_MAX = 90;

const paintParticles: PainterFn = (ctx, w, h, t, palette, intensity) => {
  ctx.fillStyle = palette.surface;
  ctx.fillRect(0, 0, w, h);
  const count = Math.min(PARTICLE_MAX, Math.round(((w * h) / 26000) * (0.5 + intensity)));
  for (let i = 0; i < count; i++) {
    const depth = 0.3 + seeded(i * 3 + 1) * 0.7;
    const size = (1.2 + seeded(i * 7 + 2) * 2.6) * depth;
    const speedX = (8 + seeded(i * 11 + 3) * 18) * depth * (0.4 + intensity);
    const speedY = (4 + seeded(i * 13 + 5) * 10) * depth * (0.4 + intensity);
    const x = ((seeded(i * 17 + 7) * w + t * speedX) % (w + 40)) - 20;
    const y = ((seeded(i * 19 + 9) * h + t * speedY) % (h + 40)) - 20;
    const twinkle = 0.55 + 0.45 * Math.sin(t * (1 + depth * 2) + i);
    const color = i % 3 === 0 ? palette.accentSecondary : palette.accent;
    const alpha = (palette.isDark ? 0.5 : 0.38) * twinkle * (0.35 + intensity * 0.65);
    const glow = ctx.createRadialGradient(x, y, 0, x, y, size * 6);
    glow.addColorStop(0, rgba(color, alpha));
    glow.addColorStop(1, rgba(color, 0));
    ctx.fillStyle = glow;
    ctx.beginPath();
    ctx.arc(x, y, size * 6, 0, Math.PI * 2);
    ctx.fill();
    ctx.fillStyle = rgba(color, Math.min(1, alpha * 1.8));
    ctx.beginPath();
    ctx.arc(x, y, size, 0, Math.PI * 2);
    ctx.fill();
  }
};

/* ---------------------------------------------------------------------------
   Flow — slowly rotating translucent color sheets (mesh-gradient feel). */

const paintFlow: PainterFn = (ctx, w, h, t, palette, intensity) => {
  ctx.fillStyle = palette.surface;
  ctx.fillRect(0, 0, w, h);
  const diagonal = Math.sqrt(w * w + h * h);
  ctx.globalCompositeOperation = palette.isDark ? "screen" : "multiply";
  for (let i = 0; i < 3; i++) {
    const angle = t * (0.08 + i * 0.03) + (i * Math.PI * 2) / 3;
    const cx = w / 2 + Math.cos(t * 0.1 + i * 2.1) * w * 0.18;
    const cy = h / 2 + Math.sin(t * 0.13 + i * 1.3) * h * 0.18;
    ctx.save();
    ctx.translate(cx, cy);
    ctx.rotate(angle);
    const gradient = ctx.createLinearGradient(-diagonal / 2, 0, diagonal / 2, 0);
    const color = i % 2 === 0 ? palette.accent : palette.accentSecondary;
    gradient.addColorStop(0, rgba(color, 0));
    gradient.addColorStop(0.5, rgba(color, (palette.isDark ? 0.30 : 0.20) * (0.4 + intensity)));
    gradient.addColorStop(1, rgba(color, 0));
    ctx.fillStyle = gradient;
    ctx.fillRect(-diagonal / 2, -diagonal / 2, diagonal, diagonal);
    ctx.restore();
  }
  ctx.globalCompositeOperation = "source-over";
};

/* ---------------------------------------------------------------------------
   Registry ----------------------------------------------------------------- */

interface PainterEntry {
  /** Human-readable name shown in the Personalization picker. */
  label: string;
  paint: PainterFn;
}

export const WALLPAPER_PAINTERS: Record<WallpaperPainterId, PainterEntry> = {
  aurora: { label: "Aurora", paint: paintAurora },
  waves: { label: "Waves", paint: paintWaves },
  particles: { label: "Particles", paint: paintParticles },
  flow: { label: "Flow", paint: paintFlow },
  none: {
    label: "Static",
    paint: (ctx, w, h, _t, palette) => {
      ctx.fillStyle = palette.surface;
      ctx.fillRect(0, 0, w, h);
    },
  },
};

/** Ordered picker entries (id + label) for UI iteration. */
export function wallpaperPickerEntries(): ReadonlyArray<{
  id: WallpaperPainterId;
  label: string;
}> {
  return (Object.keys(WALLPAPER_PAINTERS) as WallpaperPainterId[]).map((id) => ({
    id,
    label: WALLPAPER_PAINTERS[id]!.label,
  }));
}
