/**
 * WallpaperSettings — Live Wallpaper picker for the Personalization page.
 *
 * Renders one static-frame preview per painter (no rAF loops — previews are
 * single frames; only the global LiveWallpaper layer animates) plus speed and
 * intensity sliders. Selection uses native buttons with `aria-pressed`
 * (light-dark-mode skill pattern: selection never happens on focus/hover).
 */
import { useSyncExternalStore } from "react";
import { WallpaperPreview } from "./live-wallpaper.js";
import {
  WALLPAINTER_IDS,
  getWallpaper,
  setWallpaper,
  subscribeWallpaper,
} from "../lib/wallpaper-store.js";
import { wallpaperPickerEntries } from "../lib/live-wallpapers.js";

export function WallpaperSettings(): React.ReactElement {
  const settings = useSyncExternalStore(subscribeWallpaper, getWallpaper);
  const entries = wallpaperPickerEntries();

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-4)" }}>
      <div>
        <h4 style={{ fontSize: "var(--text-base)", fontWeight: 600, color: "var(--text)", margin: 0 }}>
          Live Wallpaper
        </h4>
        <p style={{ fontSize: "var(--text-sm)", color: "var(--text-muted)", margin: "var(--space-1) 0 0" }}>
          Animated backdrop rendered behind every glass surface. Respects reduced motion.
        </p>
      </div>

      <div
        role="group"
        aria-label="Live wallpaper style"
        className="wallpaper-grid"
      >
        {entries.map((entry) => (
          <button
            key={entry.id}
            type="button"
            className="wallpaper-thumb"
            aria-pressed={settings.painter === entry.id}
            onClick={() => setWallpaper({ painter: entry.id })}
          >
            <WallpaperPreview painterId={entry.id} intensity={settings.intensity} />
            <span className="wallpaper-thumb-name">{entry.label}</span>
          </button>
        ))}
      </div>

      {(WALLPAINTER_IDS as readonly string[]).includes(settings.painter) &&
      settings.painter !== "none" ? (
        <div
          style={{
            display: "grid",
            gridTemplateColumns: "repeat(auto-fit, minmax(240px, 1fr))",
            gap: "var(--space-4)",
          }}
        >
          <div className="panel-field">
            <label htmlFor="wallpaper-speed" className="panel-field-label">
              Animation speed — {Math.round(settings.speed * 100)}%
            </label>
            <input
              id="wallpaper-speed"
              type="range"
              min={0}
              max={1}
              step={0.05}
              value={settings.speed}
              onChange={(event) => setWallpaper({ speed: Number(event.target.value) })}
              className="panel-slider"
            />
          </div>
          <div className="panel-field">
            <label htmlFor="wallpaper-intensity" className="panel-field-label">
              Intensity — {Math.round(settings.intensity * 100)}%
            </label>
            <input
              id="wallpaper-intensity"
              type="range"
              min={0}
              max={1}
              step={0.05}
              value={settings.intensity}
              onChange={(event) => setWallpaper({ intensity: Number(event.target.value) })}
              className="panel-slider"
            />
          </div>
        </div>
      ) : null}
    </div>
  );
}
