/**
 * Wallpaper store — persisted Live Wallpaper preferences.
 *
 * A tiny external store (useSyncExternalStore-compatible) backed by
 * `localStorage` with `try/catch` guards so private-browsing/sandboxed
 * contexts degrade to in-memory defaults instead of crashing
 * (light-dark-mode skill: storage MUST NOT stop execution).
 *
 * UI state lives under the `zettings.wallpaper` key:
 *   { painter: PainterId, speed: number, intensity: number }
 * `speed`/`intensity` are clamped 0..1 user preferences; painters map them
 * to their own internal ranges.
 */

export const WALLPAINTER_IDS = ["aurora", "waves", "particles", "flow", "none"] as const;

export type WallpaperPainterId = (typeof WALLPAINTER_IDS)[number];

export interface WallpaperSettings {
  /** Active painter id. */
  painter: WallpaperPainterId;
  /** Animation speed preference, 0..1 (0 = slowest, 1 = fastest). */
  speed: number;
  /** Visual intensity preference, 0..1 (0 = subtle, 1 = vivid). */
  intensity: number;
}

export const DEFAULT_WALLPAPER: WallpaperSettings = {
  painter: "aurora",
  speed: 0.5,
  intensity: 0.6,
};

const STORAGE_KEY = "zettings.wallpaper";

function clamp01(value: number): number {
  if (!Number.isFinite(value)) return 0;
  return Math.min(1, Math.max(0, value));
}

function sanitize(raw: unknown): WallpaperSettings {
  if (typeof raw !== "object" || raw === null) return DEFAULT_WALLPAPER;
  const record = raw as Record<string, unknown>;
  const painter =
    typeof record.painter === "string" && (WALLPAINTER_IDS as readonly string[]).includes(record.painter)
      ? (record.painter as WallpaperPainterId)
      : DEFAULT_WALLPAPER.painter;
  return {
    painter,
    speed: clamp01(typeof record.speed === "number" ? record.speed : DEFAULT_WALLPAPER.speed),
    intensity: clamp01(
      typeof record.intensity === "number" ? record.intensity : DEFAULT_WALLPAPER.intensity,
    ),
  };
}

function load(): WallpaperSettings {
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (raw === null) return DEFAULT_WALLPAPER;
    return sanitize(JSON.parse(raw) as unknown);
  } catch {
    return DEFAULT_WALLPAPER;
  }
}

let current: WallpaperSettings = load();
const listeners = new Set<() => void>();

function emit(): void {
  for (const listener of listeners) listener();
}

/** Subscribe to wallpaper changes. Returns the unsubscribe function. */
export function subscribeWallpaper(listener: () => void): () => void {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

/** Read the current wallpaper settings snapshot. */
export function getWallpaper(): WallpaperSettings {
  return current;
}

/** Persist a partial update and notify subscribers. */
export function setWallpaper(patch: Partial<WallpaperSettings>): void {
  const next = sanitize({ ...current, ...patch });
  const changed =
    next.painter !== current.painter ||
    next.speed !== current.speed ||
    next.intensity !== current.intensity;
  if (!changed) return;
  current = next;
  try {
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(current));
  } catch {
    // Storage unavailable — the in-memory preference still applies this session.
  }
  emit();
}
