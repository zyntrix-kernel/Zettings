/**
 * G2/G3 continuous-curvature geometry (DESIGN.md §6).
 *
 * Generates superellipse ("squircle") SVG paths used as `clip-path: path()`
 * on major surfaces, where standard border-radius is forbidden.
 */

/** Curvature order for G2 surfaces (cards, inputs, rows). */
export const G2_ORDER = 4;
/** Curvature order for G3 surfaces (shell, dialogs, overlays). */
export const G3_ORDER = 6;
/** Perimeter samples for the generated cubic-bézier path. */
const SAMPLES = 128;

export interface SquircleOptions {
  /** Surface width in px. */
  width: number;
  /** Surface height in px. */
  height: number;
  /** Corner radius in px (clamped to half the minor axis). */
  radius: number;
  /** Superellipse order: {@link G2_ORDER} or {@link G3_ORDER}. */
  order?: typeof G2_ORDER | typeof G3_ORDER;
}

/**
 * Builds an SVG path string approximating a blended superellipse rectangle.
 *
 * The effective exponent interpolates from 2 (pure ellipse) to `order` as the
 * radius approaches half the minor axis, keeping small radii visually
 * consistent with large ones:
 * `n_eff = 2 + (n − 2) · r / (min(w,h)/2)`
 */
export function squirclePath({
  width,
  height,
  radius,
  order = G2_ORDER,
}: SquircleOptions): string {
  const minHalf = Math.min(width, height) / 2;
  const r = Math.max(0, Math.min(radius, minHalf));
  if (r === 0 || width <= 0 || height <= 0) {
    return `M0 0 H${width} V${height} H0 Z`;
  }
  // Exponent blending per DESIGN.md §6.
  const nEff = 2 + (order - 2) * (r / minHalf);
  const exp = 1 / nEff;

  const points: Array<[number, number]> = [];
  for (let i = 0; i < SAMPLES; i += 1) {
    const theta = (i / SAMPLES) * Math.PI * 2;
    const cosT = Math.cos(theta);
    const sinT = Math.sin(theta);
    // Superellipse in centered coordinates, scaled to the box.
    const x = Math.sign(cosT) * Math.abs(cosT) ** exp * (width / 2);
    const y = Math.sign(sinT) * Math.abs(sinT) ** exp * (height / 2);
    points.push([width / 2 + x, height / 2 + y]);
  }

  let d = `M${points[0]?.[0].toFixed(2)} ${points[0]?.[1].toFixed(2)}`;
  for (let i = 1; i < points.length; i += 1) {
    const [x, y] = points[i] ?? [0, 0];
    d += ` L${x.toFixed(2)} ${y.toFixed(2)}`;
  }
  return `${d} Z`;
}

/**
 * Convenience: a CSS `clip-path: path("...")` value for the given surface.
 * Callers recompute on resize (ResizeObserver) — paths are resolution-bound.
 */
export function squircleClipPath(options: SquircleOptions): string {
  return `path("${squirclePath(options)}")`;
}
