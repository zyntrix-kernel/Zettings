/**
 * ZDL GlassPanel — multi-layered "liquid glass" material.
 *
 * Composes three visual layers on a G2/G3 squircle surface:
 *   1. translucent tint (`--glass-panel-bg`) + backdrop blur + saturate
 *   2. 1px specular edge highlight (`--glass-panel-border`)
 *   3. elevation shadow (`--shadow-1`..`--shadow-4`)
 *
 * Tokens are sourced from the existing 3-tier cascade in `zdl.css` so theme
 * variants (light/dark/oled/hc) automatically re-tune the glass.
 *
 * Accessibility: respects `prefers-reduced-motion` via the global token
 * override in `zdl.css` (motion durations collapse to 0ms). The component
 * does no continuous/infinite animation — backdrop-filter re-paint is a
 * compositor-side effect only.
 */
import { type CSSProperties, type ReactNode } from "react";
import { type SquircleOrder } from "../lib/zdl-motion.js";
import { Squircle } from "./squircle.js";

/** Elevation index maps directly to `--shadow-1`..`--shadow-4` tokens. */
export type GlassElevation = 1 | 2 | 3 | 4;

export interface GlassPanelProps {
  /** Optional explicit width in CSS px. Defaults to intrinsic block size. */
  width?: number;
  /** Optional explicit height in CSS px. Defaults to intrinsic block size. */
  height?: number;
  /** Corner radius in CSS px. Defaults to 20. */
  radius?: number;
  /** Superellipse order. Defaults to 4 (G2). */
  order?: SquircleOrder;
  /** Elevation index 1..4. Defaults to 2. */
  elevation?: GlassElevation;
  /** Optional className passed to the inner content wrapper. */
  className?: string;
  /** Optional inline style passed to the inner content wrapper. */
  style?: CSSProperties;
  /** Panel content. */
  children?: ReactNode;
}

const SHADOW_TOKEN: Record<GlassElevation, string> = {
  1: "var(--shadow-1)",
  2: "var(--shadow-2)",
  3: "var(--shadow-3)",
  4: "var(--shadow-4)",
};

export function GlassPanel({
  width,
  height,
  radius = 20,
  order = 4,
  elevation = 2,
  className,
  style,
  children,
}: GlassPanelProps): React.ReactElement {
  // The outer wrapper carries backdrop-filter + tint + specular border + shadow.
  // The Squircle inside clips everything to the continuous-curvature shape.
  const surfaceStyle: CSSProperties = {
    position: "relative",
    background: "var(--glass-panel-bg)",
    backdropFilter: "blur(var(--glass-panel-blur)) saturate(var(--glass-panel-saturate))",
    WebkitBackdropFilter: "blur(var(--glass-panel-blur)) saturate(var(--glass-panel-saturate))",
    border: "1px solid var(--glass-panel-border)",
    boxShadow: SHADOW_TOKEN[elevation],
    // Outer box carries the size; Squircle inherits via 100%.
    width: width !== undefined ? `${width}px` : "100%",
    height: height !== undefined ? `${height}px` : "auto",
  };

  // Inner content wrapper separates layout concerns from the squircle clip.
  const innerStyle: CSSProperties = {
    position: "relative",
    // Lift content above the absolute-positioned SVG defs in <Squircle>.
    zIndex: 1,
    padding: "var(--space-6)",
    ...style,
  };

  // When dimensions are explicit, Squircle can compute its own clip-path.
  // When they are auto/intrinsic, we use a fallback pseudo-blank so the
  // clip-path tracks the content — squircle behaves via 100%-driven sizing.
  const sqWidth = width ?? 0;
  const sqHeight = height ?? 0;

  return (
    <div className={className} style={surfaceStyle}>
      {sqWidth > 0 && sqHeight > 0 ? (
        <Squircle width={sqWidth} height={sqHeight} radius={radius} order={order}>
          <div style={innerStyle}>{children}</div>
        </Squircle>
      ) : (
        // Intrinsic-size path: leave content clipped to a fallback squircle
        // only after measurement — for now, render flat with the glass
        // material intact. Phase 3 hooks the ResizeObserver-driven path.
        <div style={innerStyle}>{children}</div>
      )}
    </div>
  );
}
