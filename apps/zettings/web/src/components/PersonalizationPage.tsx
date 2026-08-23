/**
 * Personalization L2 page — theme & appearance. The theme selector and the
 * live-wallpaper preference are REAL (applied through the ZDL token cascade
 * / wallpaper module and persisted per user-personalization rules).
 * Remaining areas render as explained "arriving" rows for their KDE
 * integrations.
 */
import { useState } from "react";
import { ThemeSelector } from "./shell/theme-selector";
import {
  initWallpaper,
  setWallpaperMotion,
  type WallpaperMotion,
} from "../lib/wallpaper";
import { SettingsCard, ToggleSwitch } from "./zdl";

const PLANNED_AREAS: ReadonlyArray<{ title: string; description: string; via: string }> = [
  {
    title: "Colors",
    description: "System accent color across windows and controls.",
    via: "KDE color scheme integration",
  },
  {
    title: "Fonts",
    description: "Interface and monospace typefaces.",
    via: "fontconfig / KDE fonts",
  },
  {
    title: "Cursor",
    description: "Pointer theme and size.",
    via: "KDE cursor themes",
  },
];

export function PersonalizationPage() {
  const [motion, setMotion] = useState<WallpaperMotion>(() => initWallpaper());

  const setLive = (live: boolean): void => {
    setWallpaperMotion(live ? "live" : "static");
    setMotion(live ? "live" : "static");
  };

  return (
    <>
      <h1 tabIndex={-1} className="zdl-page-title">
        Theme &amp; appearance
      </h1>
      <p className="zdl-page-description">
        Choose how Zettings looks. Changes apply immediately.
      </p>

      <h2 className="zdl-section-title">Theme</h2>
      <div className="zdl-card-grid">
        <SettingsCard
          title="Color theme"
          description="Applies immediately and is remembered on this device."
          control={<ThemeSelector />}
        />
      </div>

      <h2 className="zdl-section-title">Wallpaper</h2>
      <div className="zdl-card-grid">
        <SettingsCard
          title="Live wallpaper"
          description={
            motion === "live"
              ? "The aurora drifts slowly behind the interface. It is always paused when your system requests reduced motion."
              : "The aurora renders as a still image."
          }
          control={
            <ToggleSwitch
              label="Live wallpaper"
              checked={motion === "live"}
              onChange={setLive}
            />
          }
        />
      </div>

      <h2 className="zdl-section-title">More appearance settings</h2>
      <div className="zdl-card-grid">
        {PLANNED_AREAS.map(({ title, description, via }) => (
          <SettingsCard
            key={title}
            title={title}
            description={description}
            control={<span className="zdl-area-status">Arriving with {via}</span>}
          />
        ))}
      </div>
    </>
  );
}
