/**
 * ZDL GlassCard — token-driven liquid-glass card surface.
 *
 * Replaces hand-rolled `.liquid-glass` div stacks + forbidden `border-radius`
 * in panels with a single G2 squircle surface that composes the ZDL glass
 * stack (backdrop refraction + tint + specular + elevation) entirely from the
 * token cascade (`--glass-panel-bg`, `--glass-panel-tint`, `--glass-panel-blur`,
 * `--glass-panel-saturate`, `--glass-panel-border`, `--shadow-*`). Theme
 * variants (light/dark/oled/hc) re-tune the material automatically.
 *
 * Sizing: an optional explicit `width`/`height` lets the Squircle clip-path be
 * computed immediately. Without them the card measures itself via
 * `useElementSize` and renders the squircle once laid out. The measurement is
 * batched through `requestAnimationFrame` so no layout thrash occurs.
 *
 * Accessibility: the card is a presentational surface, not an interactive
 * control. If it is clickable, wrap its content in a real `<button>` (or use
 * `GlassButton`); do not add `onClick` to this surface. Optional
 * `tabIndex`/`role`/`onKeyDown`/`onMouseDown` passthrough is provided for
 * draggable arrangement surfaces (e.g. display-canvas monitors) where the
 * card itself must be focusable — pair it with `role` and keyboard handlers.
 */
import { type CSSProperties, type KeyboardEvent as ReactKeyboardEvent, type MouseEvent as ReactMouseEvent, type ReactNode } from "react";
import { type SquircleOrder } from "../lib/zdl-motion.js";
import { useElementSize } from "../lib/use-element-size.js";
import { Squircle } from "./squircle.js";

export type GlassElevation = 1 | 2 | 3 | 4;

export interface GlassCardProps {
  /** Optional explicit width in CSS px. When omitted, the card self-measures. */
  width?: number;
  /** Optional explicit height in CSS px. When omitted, the card self-measures. */
  height?: number;
  /** Corner radius in CSS px. Defaults to 20 (G2). */
  radius?: number;
  /** Superellipse order: 4 = G2 (default), 6 = G3. */
  order?: SquircleOrder;
  /** Elevation index 1..4 mapping to `--shadow-1`..`--shadow-4`. Defaults to 2. */
  elevation?: GlassElevation;
  /** Optional className applied to the squircle-clipped content wrapper. */
  className?: string;
  /** Optional inline style applied to the squircle-clipped content wrapper. */
  style?: CSSProperties;
  /** Optional data-testid for e2e targeting. */
  dataTestId?: string;
  /** Optional tabindex for focusable arrangement surfaces (e.g. drag canvas). */
  tabIndex?: number;
  /** Optional ARIA role for focusable arrangement surfaces. */
  role?: string;
  /** Optional keydown handler (paired with `tabIndex` for keyboard drag). */
  onKeyDown?: (event: ReactKeyboardEvent<HTMLDivElement>) => void;
  /** Optional mousedown handler (for drag initiation on arrangement surfaces). */
  onMouseDown?: (event: ReactMouseEvent<HTMLDivElement>) => void;
  /** Optional click handler (e.g. stop-propagation on modal card surfaces). */
  onClick?: (event: ReactMouseEvent<HTMLDivElement>) => void;
  /** Card content. */
  children?: ReactNode;
}

const SHADOW_TOKEN: Record<GlassElevation, string> = {
  1: "var(--shadow-1)",
  2: "var(--shadow-2)",
  3: "var(--shadow-3)",
  4: "var(--shadow-4)",
};

export function GlassCard({
  width,
  height,
  radius = 20,
  order = 4,
  elevation = 2,
  className,
  style,
  dataTestId,
  tabIndex,
  role,
  onKeyDown,
  onMouseDown,
  onClick,
  children,
}: GlassCardProps): React.ReactElement {
  const [ref, measured] = useElementSize<HTMLDivElement>();
  const w = width ?? measured?.width ?? 0;
  const h = height ?? measured?.height ?? 0;

  const surfaceStyle: CSSProperties = {
    position: "relative",
    background: "var(--glass-panel-bg)",
    backdropFilter:
      "blur(var(--glass-panel-blur)) saturate(var(--glass-panel-saturate))",
    WebkitBackdropFilter:
      "blur(var(--glass-panel-blur)) saturate(var(--glass-panel-saturate))",
    border: "1px solid var(--glass-panel-border)",
    boxShadow: SHADOW_TOKEN[elevation],
    width: width !== undefined ? `${width}px` : "100%",
    height: height !== undefined ? `${height}px` : "auto",
    isolation: "isolate",
  };

  const contentStyle: CSSProperties = {
    ...style,
    position: "relative",
    zIndex: 10,
    padding: "var(--space-4)",
  };

  if (w <= 0 || h <= 0) {
    // Pre-measurement frame: no clip yet, but the glass material is present.
    // Use `ref` on the surface so the observer can start reporting size.
    return (
      <div ref={ref} data-testid={dataTestId} className={className} style={surfaceStyle} tabIndex={tabIndex} role={role} onKeyDown={onKeyDown} onMouseDown={onMouseDown} onClick={onClick}>
        <div style={contentStyle}>{children}</div>
      </div>
    );
  }

  return (
    <div ref={ref} data-testid={dataTestId} className={className} style={surfaceStyle} tabIndex={tabIndex} role={role} onKeyDown={onKeyDown} onMouseDown={onMouseDown} onClick={onClick}>
      <Squircle width={w} height={h} radius={radius} order={order}>
        <div style={contentStyle}>{children}</div>
      </Squircle>
    </div>
  );
}