/**
 * Theme selector (light-dark-mode skill pattern): a labelled group of native
 * buttons with `aria-pressed`; selection changes ONLY on activation — focus
 * and hover never preview or apply a theme.
 */
import { useEffect, useState } from "react";
import {
  initTheme,
  setThemeMode,
  resolveTheme,
  type ThemeMode,
} from "../../lib/theme";

const OPTIONS: ReadonlyArray<{ mode: ThemeMode; label: string }> = [
  { mode: "system", label: "System" },
  { mode: "light", label: "Light" },
  { mode: "dark", label: "Dark" },
  { mode: "oled", label: "OLED" },
  { mode: "hc", label: "High contrast" },
];

export function ThemeSelector() {
  const [mode, setMode] = useState<ThemeMode>(() => initTheme().mode);
  const [, bumpResolved] = useState(0);

  useEffect(() => {
    const query = window.matchMedia("(prefers-color-scheme: dark)");
    const onChange = (): void => bumpResolved((n) => n + 1);
    query.addEventListener("change", onChange);
    return () => query.removeEventListener("change", onChange);
  }, []);

  return (
    <div role="group" aria-label="Color theme" className="zdl-theme-group">
      {OPTIONS.map((option) => (
        <button
          key={option.mode}
          type="button"
          className="zdl-theme-btn"
          aria-pressed={mode === option.mode ? "true" : "false"}
          onClick={() => {
            setThemeMode(option.mode);
            setMode(option.mode);
          }}
        >
          <span>{option.label}</span>
        </button>
      ))}
      {/* Resolved-system status; polite, non-duplicative. */}
      <span className="visually-hidden" role="status">
        {mode === "system"
          ? `System preference currently uses ${resolveTheme("system")} appearance.`
          : ""}
      </span>
    </div>
  );
}
