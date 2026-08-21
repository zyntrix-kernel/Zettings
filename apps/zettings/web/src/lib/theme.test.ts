import { afterEach, describe, expect, it } from "vitest";
import { resolveTheme, setThemeMode, type ThemeMode } from "./theme";

afterEach(() => {
  document.documentElement.removeAttribute("data-theme");
  document.documentElement.removeAttribute("data-theme-mode");
});

describe("theme engine", () => {
  it("resolves system to light in the jsdom default", () => {
    expect(resolveTheme("system")).toBe("light");
  });

  it("applies explicit themes to the root element", () => {
    const modes: ThemeMode[] = ["light", "dark", "oled", "hc"];
    for (const mode of modes) {
      setThemeMode(mode);
      expect(document.documentElement.dataset.themeMode).toBe(mode);
      expect(document.documentElement.dataset.theme).toBe(mode);
    }
  });

  it("system mode does not pin a data-theme attribute (CSS media queries own it)", () => {
    setThemeMode("dark");
    setThemeMode("system");
    expect(document.documentElement.dataset.themeMode).toBe("system");
    // resolve still answers for status text.
    expect(resolveTheme("system")).toBe("light");
  });

  it("survives unavailable storage without throwing", () => {
    const original = window.localStorage;
    Object.defineProperty(window, "localStorage", {
      configurable: true,
      value: {
        getItem: () => {
          throw new Error("blocked");
        },
        setItem: () => {
          throw new Error("blocked");
        },
      },
    });
    expect(() => setThemeMode("hc")).not.toThrow();
    expect(document.documentElement.dataset.theme).toBe("hc");
    Object.defineProperty(window, "localStorage", {
      configurable: true,
      value: original,
    });
  });
});
