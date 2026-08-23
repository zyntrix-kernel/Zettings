/**
 * Live-wallpaper preference (user-personalization skill): an explicit,
 * persisted choice between the animated aurora and a static rendition.
 *
 * Storage follows the personalization contract — try/catch access, values
 * validated against an allowlist, applied via `data-aurora` on the root so
 * CSS owns presentation. `system` is not offered because the OS has no
 * equivalent signal; the default is ON (it is decorative and fully disabled
 * under prefers-reduced-motion by the stylesheet regardless of this value).
 */

export type WallpaperMotion = "live" | "static";

const STORAGE_KEY = "zettings.wallpaper-motion";
const ALLOWED: ReadonlyArray<WallpaperMotion> = ["live", "static"];

function apply(value: WallpaperMotion): void {
  document.documentElement.dataset.aurora = value;
}

/** Reads and applies the stored preference; returns it. */
export function initWallpaper(): WallpaperMotion {
  let stored: string | null = null;
  try {
    stored = window.localStorage.getItem(STORAGE_KEY);
  } catch {
    // Unavailable storage keeps the current-page default.
  }
  const value: WallpaperMotion =
    stored !== null && (ALLOWED as readonly string[]).includes(stored)
      ? (stored as WallpaperMotion)
      : "live";
  apply(value);
  return value;
}

/** Persists (best-effort) and applies the chosen wallpaper motion. */
export function setWallpaperMotion(value: WallpaperMotion): void {
  try {
    window.localStorage.setItem(STORAGE_KEY, value);
  } catch {
    // Preference still applies for this session.
  }
  apply(value);
}
