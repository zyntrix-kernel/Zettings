/**
 * Theme engine (DESIGN.md §9, light-dark-mode skill).
 *
 * Modes: system | light | dark | oled | hc. The selected mode and the
 * resolved appearance are separate values (`data-theme-mode` vs
 * `data-theme`). Theme changes happen ONLY on explicit activation — never on
 * focus or hover. Storage failures degrade to session-only behavior.
 */

export type ThemeMode = "system" | "light" | "dark" | "oled" | "hc";
export type ResolvedTheme = "light" | "dark" | "oled" | "hc";

const STORAGE_KEY = "zettings.theme-mode";
const MODES: readonly ThemeMode[] = ["system", "light", "dark", "oled", "hc"];

const prefersDarkQuery = (): MediaQueryList =>
  window.matchMedia("(prefers-color-scheme: dark)");

function readStoredMode(): ThemeMode {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    return MODES.includes(raw as ThemeMode) ? (raw as ThemeMode) : "system";
  } catch {
    return "system";
  }
}

function writeStoredMode(mode: ThemeMode): void {
  try {
    localStorage.setItem(STORAGE_KEY, mode);
  } catch {
    /* storage unavailable; mode still applies for this session */
  }
}

export function resolveTheme(mode: ThemeMode): ResolvedTheme {
  if (mode !== "system") return mode;
  return prefersDarkQuery().matches ? "dark" : "light";
}

function apply(mode: ThemeMode): void {
  const root = document.documentElement;
  root.dataset.themeMode = mode;
  const resolved = resolveTheme(mode);
  // `system` resolves to light/dark only; explicit oled/hc apply directly.
  if (mode === "system") {
    delete root.dataset.theme;
    if (resolved === "dark") root.dataset.theme = "dark";
  } else {
    root.dataset.theme = resolved;
  }
}

/** Applies the stored/system theme; called once at shell boot. */
export function initTheme(): { mode: ThemeMode; resolved: ResolvedTheme } {
  const mode = readStoredMode();
  apply(mode);
  // Keep system mode live under OS preference changes without rewriting the
  // stored selection.
  prefersDarkQuery().addEventListener("change", () => {
    if ((document.documentElement.dataset.themeMode ?? "system") === "system") {
      apply("system");
    }
  });
  return { mode, resolved: resolveTheme(mode) };
}

/** Explicitly activates a mode (the ONLY path that changes the theme). */
export function setThemeMode(mode: ThemeMode): void {
  writeStoredMode(mode);
  apply(mode);
}
