/**
 * ZDL squircle clip utilities.
 *
 * `SquircleSurface` — declarative container whose entire surface (background,
 * backdrop-filter, border) is clipped to a G2/G3 superellipse.
 *
 * `useSquircleClip` — hook form for interactive elements (<button>, <a>) that
 * must carry the clip themselves; returns the ref, computed style, and the
 * SVG <defs> fragment that must be rendered alongside the element.
 *
 * Because `clip-path` clips away `box-shadow`, elevation on clipped surfaces
 * uses `filter: var(--drop-shadow-*)` tokens which follow the silhouette.
 */
import { useId, type CSSProperties, type ReactElement, type ReactNode, type RefObject } from "react";
import { generateSquirclePath, type SquircleOrder } from "../lib/zdl-motion.js";
import { useElementSize } from "../lib/use-element-size.js";

export interface SquircleClip {
  /** Attach to the element that must be clipped. */
  ref: RefObject<HTMLElement | null>;
  /** Merge into the element's style (adds clipPath once measured). */
  clipStyle: CSSProperties;
  /** Render next to the element (fragment may be null pre-measurement). */
  defs: ReactElement | null;
}

/** Hook variant of the squircle clip for polymorphic/interactive elements. */
export function useSquircleClip(
  radius = 14,
  order: SquircleOrder = 4,
): SquircleClip {
  const [ref, measured] = useElementSize<HTMLElement>();
  const rawId = useId();
  const clipId = `zdl-clip-${rawId.replace(/:/g, "")}`;

  const w = measured?.width ?? 0;
  const h = measured?.height ?? 0;
  const clipped = w > 0 && h > 0;

  const clipStyle: CSSProperties = clipped ? { clipPath: `url(#${clipId})` } : {};
  const defs: ReactElement | null = clipped ? (
    <svg
      width={0}
      height={0}
      aria-hidden="true"
      focusable="false"
      style={{ position: "absolute", width: 0, height: 0, overflow: "hidden" }}
    >
      <defs>
        <clipPath id={clipId} clipPathUnits="userSpaceOnUse">
          <path d={generateSquirclePath(w, h, radius, order)} />
        </clipPath>
      </defs>
    </svg>
  ) : null;

  return { ref, clipStyle, defs };
}

export interface SquircleSurfaceProps {
  /** Corner radius in CSS px. Defaults to 16. */
  radius?: number;
  /** Superellipse order: 4 = G2 (default), 6 = G3. */
  order?: SquircleOrder;
  /** Applied to the clipped surface element. */
  className?: string;
  /** Inline style merged under the computed clip-path. */
  style?: CSSProperties;
  /** Surface content. */
  children?: ReactNode;
}

/** Declarative squircle-clipped surface container. */
export function SquircleSurface({
  radius = 16,
  order = 4,
  className,
  style,
  children,
}: SquircleSurfaceProps): React.ReactElement {
  const { ref, clipStyle, defs } = useSquircleClip(radius, order);
  return (
    <>
      <div ref={ref as React.Ref<HTMLDivElement>} className={className} style={{ ...style, ...clipStyle }}>
        {children}
      </div>
      {defs}
    </>
  );
}
