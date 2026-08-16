 /**
  * ZDL Squircle — continuous-curvature container (G2 n=4 / G3 n=6).
  *
  * Renders children clipped by a superellipse path produced by
  * `generateSquirclePath` from `zdl-motion.ts`. Per AGENTS.md CSS rules,
  * `border-radius` is forbidden for major panels — this clip-path is the
  * only mechanism used to produce rounded corners on the surface.
  *
  * Performance note (from ui-ux-pro-max/react stack guidance): the component
  * is wrapped in `React.memo` because squircle surfaces often render many
  * times as parents re-render. A custom comparator avoids recomputation of
  * the SVG clip-path when only non-geometry props change.
  */
import { memo, useId, type CSSProperties, type ReactNode } from "react";
import { generateSquirclePath, type SquircleOrder } from "../lib/zdl-motion.js";

export interface SquircleProps {
  /** Container width in CSS pixels. */
  width: number;
  /** Container height in CSS pixels. */
  height: number;
  /** Corner radius in CSS pixels. Defaults to a quarter of the shorter side. */
  radius?: number;
  /** Superellipse order: 4 = G2 continuity, 6 = G3 continuity. Defaults to 4. */
  order?: SquircleOrder;
  /** Optional className applied to the clipped content wrapper. */
  className?: string;
  /** Optional inline style applied to the clipped content wrapper. */
  style?: CSSProperties;
  /** Children rendered inside the clipped region. */
  children?: ReactNode;
}

function SquircleImpl({
  width,
  height,
  radius,
  order = 4,
  className,
  style,
  children,
}: SquircleProps): React.ReactElement {
  const w = width;
  const h = height;
  const r = radius ?? Math.min(w, h) / 4;
  // useId gives a stable, SSR-safe unique id per element instance.
  const rawId = useId();
  // CSS clip-path ids must not contain colons — strip them from useId output.
  const clipId = `zdl-squircle-${rawId.replace(/:/g, "")}`;

  // generateSquirclePath returns an absolute SVG path in user-space pixel
  // coordinates, so the clipPath must use the default `userSpaceOnUse` units.
  const path = generateSquirclePath(w, h, r, order);
  const clipStyle: CSSProperties = {
    ...style,
    width: w,
    height: h,
    clipPath: `url(#${clipId})`,
  };

  return (
    <div className={className} style={clipStyle}>
      <svg
        width={w}
        height={h}
        viewBox={`0 0 ${w} ${h}`}
        aria-hidden="true"
        style={{ position: "absolute", width: 0, height: 0, overflow: "hidden" }}
      >
        <defs>
          <clipPath id={clipId}>
            <path d={path} />
          </clipPath>
        </defs>
      </svg>
      {children}
    </div>
  );
}

// Custom comparator: only re-render when geometry or passthrough props change.
// The `useId` value is stable across renders, so the clip-path url(...) string
// only needs to be rebuilt when the path itself changes.
function squirclePropsEqual(prev: SquircleProps, next: SquircleProps): boolean {
  return (
    prev.width === next.width &&
    prev.height === next.height &&
    prev.radius === next.radius &&
    prev.order === next.order &&
    prev.className === next.className &&
    prev.style === next.style &&
    prev.children === next.children
  );
}

/**
 * Memoized squircle container. Use this anywhere a G2/G3 continuous-curvature
 * surface is needed. The squircle engine guarantees screen-edge continuity
 * for the corner tangent — for major cards/panels never substitute CSS
 * `border-radius` (forbidden by AGENTS.md).
 */
export const Squircle = memo(SquircleImpl, squirclePropsEqual);
