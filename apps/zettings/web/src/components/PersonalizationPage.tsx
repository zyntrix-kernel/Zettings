/**
 * Personalization L2 page — theme & appearance. The theme selector is REAL
 * (applies light/dark/OLED/high-contrast through the ZDL token cascade and
 * persists per user-personalization rules). Remaining areas render as
 * explained "arriving" rows for their KDE integrations.
 */
import { ThemeSelector } from "./shell/theme-selector";
import { categoryIcon } from "../lib/category-icons";
import { SettingsCard } from "./zdl";

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
  const planned = PLANNED_AREAS.map((area) => ({ ...area, Icon: categoryIcon("colors") }));

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

      <h2 className="zdl-section-title">More appearance settings</h2>
      <div className="zdl-card-grid">
        {planned.map(({ title, description, via }) => (
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
