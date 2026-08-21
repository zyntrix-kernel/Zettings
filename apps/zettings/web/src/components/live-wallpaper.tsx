/**
 * ZDL LiveWallpaper — animated canvas backdrop behind the glass UI.
 *
 * Renders the active painter (see `lib/live-wallpapers.ts`) into a fixed,
 * pointer-transparent canvas at z-index 0 so every Liquid Glass surface in
 * the shell refracts it. This is what makes the shell feel "alive"
 * (prompt.txt visual philosophy) while content stays primary
 * (liquid-glass-foundations: glass is the overlay, never the content).
 *
 * Performance contract:
 *   - ONE rAF loop for the whole app; painters are pure functions of time.
 *   - Loop pauses on `document.hidden` and unmounts cleanly.
 *   - Device pixel ratio clamped to 2 (retina-crisp without 4K fill-rate).
 *   - `prefers-reduced-motion`: renders exactly one static frame and stops —
 *     the wallpaper remains beautiful but never moves (DESIGN.md §4).
 *
 * Accessibility: the layer is `aria-hidden` and `pointer-events: none`;
 * contrast of foreground text is guaranteed by the glass tint stack
 * (`--glass-panel-*`), not by the wallpaper itself.
 */

import { useEffect, useRef, useSyncExternalStore } from "react";
import {
  WALLPAPER_PAINTERS,
  type WallpaperPalette,
} from "../lib/live-wallpapers.js";
import {
  getWallpaper,
  subscribeWallpaper,
} from "../lib/wallpaper-store.js";
import { usePrefersReducedMotion } from "../lib/zdl-motion-hooks.js";

/** Map speed preference 0..1 to a time multiplier (0.25× .. 1.75×). */
function speedMultiplier(speed: number): number {
  return 0.25 + speed * 1.5;
}

function resolvePalette(): WallpaperPalette {
  const styles = getComputedStyle(document.documentElement);
  const accent = styles.getPropertyValue("--accent").trim() || "#58aebc";
  const accentSecondary =
    styles.getPropertyValue("--accent-secondary").trim() || "#7f62cf";
  const surface = styles.getPropertyValue("--surface").trim() || "#0c0a09";
  const theme = document.documentElement.getAttribute("data-theme");
  return {
    accent,
    accentSecondary,
    surface,
    isDark: theme === "dark" || theme === "oled",
  };
}

export function LiveWallpaper(): React.ReactElement {
  const settings = useSyncExternalStore(subscribeWallpaper, getWallpaper);
  const reducedMotion = usePrefersReducedMotion();
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (canvas === null) return;
    const ctx = canvas.getContext("2d");
    if (ctx === null) return;

    let rafId = 0;
    let disposed = false;
    let lastTs: number | null = null;
    let simTime = 3.2; // start mid-flow so the first frame is representative
    let palette = resolvePalette();

    const resize = (): void => {
      const dpr = Math.min(window.devicePixelRatio || 1, 2);
      const width = Math.max(1, Math.floor(window.innerWidth));
      const height = Math.max(1, Math.floor(window.innerHeight));
      canvas.width = Math.floor(width * dpr);
      canvas.height = Math.floor(height * dpr);
      canvas.style.width = `${width}px`;
      canvas.style.height = `${height}px`;
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    };

    const paintFrame = (): void => {
      const width = window.innerWidth;
      const height = window.innerHeight;
      WALLPAPER_PAINTERS[settings.painter]?.paint(
        ctx,
        width,
        height,
        simTime,
        palette,
        settings.intensity,
      );
    };

    const tick = (ts: number): void => {
      if (disposed) return;
      if (lastTs !== null && !document.hidden) {
        // Clamp dt so a background-tab stall doesn't teleport the animation.
        const dt = Math.min((ts - lastTs) / 1000, 0.05);
        simTime += dt * speedMultiplier(settings.speed);
      }
      lastTs = ts;
      paintFrame();
      rafId = requestAnimationFrame(tick);
    };

    resize();

    if (reducedMotion || settings.painter === "none") {
      // Single static frame; repaint only on resize/theme change.
      paintFrame();
    } else {
      rafId = requestAnimationFrame(tick);
    }

    const onResize = (): void => {
      resize();
      paintFrame();
    };
    const onVisibility = (): void => {
      if (document.hidden) {
        cancelAnimationFrame(rafId);
        lastTs = null;
      } else if (!reducedMotion && settings.painter !== "none") {
        rafId = requestAnimationFrame(tick);
      }
    };

    // Repaint when the theme token cascade changes (light/dark/oled/hc).
    const themeObserver = new MutationObserver(() => {
      palette = resolvePalette();
      paintFrame();
    });
    themeObserver.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["data-theme"],
    });

    window.addEventListener("resize", onResize);
    document.addEventListener("visibilitychange", onVisibility);

    return () => {
      disposed = true;
      cancelAnimationFrame(rafId);
      window.removeEventListener("resize", onResize);
      document.removeEventListener("visibilitychange", onVisibility);
      themeObserver.disconnect();
    };
  }, [settings.painter, settings.speed, settings.intensity, reducedMotion]);

  return (
    <div className="wallpaper-layer" aria-hidden="true">
      <div className="wallpaper-fallback" />
      <canvas ref={canvasRef} className="wallpaper-canvas" />
    </div>
  );
}

export interface WallpaperPreviewProps {
  /** Painter id to preview. */
  painterId: keyof typeof WALLPAPER_PAINTERS;
  /** Intensity preference used for the static frame. */
  intensity: number;
}

/**
 * Static single-frame thumbnail of a painter, used by the Personalization
 * picker. Renders once per size/painter/intensity change — no rAF loop.
 */
export function WallpaperPreview({
  painterId,
  intensity,
}: WallpaperPreviewProps): React.ReactElement {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (canvas === null) return;
    const ctx = canvas.getContext("2d");
    if (ctx === null) return;

    const render = (): void => {
      const rect = canvas.getBoundingClientRect();
      const dpr = Math.min(window.devicePixelRatio || 1, 2);
      const width = Math.max(1, Math.floor(rect.width));
      const height = Math.max(1, Math.floor(rect.height));
      canvas.width = Math.floor(width * dpr);
      canvas.height = Math.floor(height * dpr);
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      WALLPAPER_PAINTERS[painterId]?.paint(
        ctx,
        width,
        height,
        3.2,
        resolvePalette(),
        intensity,
      );
    };

    render();
    const observer = new ResizeObserver(render);
    observer.observe(canvas);
    return () => observer.disconnect();
  }, [painterId, intensity]);

  return <canvas ref={canvasRef} className="wallpaper-thumb-canvas" aria-hidden="true" />;
}
