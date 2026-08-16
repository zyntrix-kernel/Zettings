/**
 * ZDL Squircle hook — React binding for the continuous curvature engine.
 *
 * Returns a CSS `clip-path` value (using an inline SVG path) and the raw SVG
 * path string so callers can use the same geometry for both clipping and
 * masking. See DESIGN.md section 1 for the G2/G3 superellipse spec.
 */
import { useMemo } from "react";
import { generateSquirclePath } from "./zdl-motion";
import type { SquircleOrder } from "./zdl-motion";

export interface SquircleGeometry {
  /** SVG path string for the squircle outline. */
  path: string;
  /**
   * CSS `clip-path` value referencing an inline SVG data URL. Apply to the
   * `style` prop of any element to clip it to the squircle shape.
   */
  clipPath: string;
}

export interface UseSquircleArgs {
  width: number;
  height: number;
  /** Corner radius in px; clamped to `min(width, height) / 2`. */
  radius: number;
  /** G2 (4) or G3 (6) continuity. Defaults to G2. */
  order?: SquircleOrder;
}

/**
 * Computes a squircle clip-path for the given dimensions. Memoised on all
 * inputs so consumers can pass the raw props through without extra caching.
 */
export function useSquircle(args: UseSquircleArgs): SquircleGeometry {
  const { width, height, radius, order } = args;
  return useMemo<SquircleGeometry>(() => {
    const path = generateSquirclePath(width, height, radius, order ?? 4);
    // Inline SVG data URL — `fill-rule:evenodd` is irrelevant for a single
    // closed subpath; the path is convex so nonzero winding fills it.
    const svg =
      `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" ` +
      `height="${height}" preserveAspectRatio="none"><path d="${path}" /></svg>`;
    const encoded = encodeURIComponent(svg).replace(/'/g, "%27").replace(/"/g, "%22");
    const clipPath = `path("data:image/svg+xml,${encoded}")`;
    return { path, clipPath };
  }, [width, height, radius, order]);
}
