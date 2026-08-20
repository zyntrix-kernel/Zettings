/**
 * ZDL `useElementSize` — measures a DOM node via `ResizeObserver`.
 *
 * The `Squircle` clip-path needs explicit pixel dimensions (it renders an SVG
 * path in `userSpaceOnUse` units), but glass cards in auto-fill grids and
 * intrinsic-height layouts do not know their size ahead of time. This hook
 * observes a node and reports its content-box size so `Squircle` can render
 * the exact continuous-curvature path. Returns `undefined` until the first
 * measurement so callers can render a zero-dimension (no-clip) fallback frame
 * without a flash of a clipped shape.
 *
 * Accessibility / perf note (ui-ux-pro-max React guidance): the hook batches
 * ResizeObserver callbacks through `requestAnimationFrame` so layout thrash is
 * not triggered on the main thread, and it never measures hidden nodes
 * (`display: none` reports 0×0).
 */
import { useEffect, useRef, useState } from "react";

export interface ElementSize {
  width: number;
  height: number;
}

export function useElementSize<T extends HTMLElement>(): [
  React.RefObject<T | null>,
  ElementSize | undefined,
] {
  const ref = useRef<T | null>(null);
  const [size, setSize] = useState<ElementSize | undefined>(undefined);

  useEffect(() => {
    const node = ref.current;
    if (!node || typeof ResizeObserver === "undefined") {
      // SSR / very old runtime: fall back to the node's layout size.
      if (node) {
        setSize({ width: node.offsetWidth, height: node.offsetHeight });
      }
      return;
    }

    let frame = 0;
    const observer = new ResizeObserver((entries) => {
      const entry = entries[0];
      if (!entry) return;
      const { width, height } = entry.contentRect;
      if (width === 0 && height === 0) return; // hidden or detached
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(() => {
        setSize({ width, height });
      });
    });

    observer.observe(node);
    return () => {
      cancelAnimationFrame(frame);
      observer.disconnect();
    };
  }, []);

  return [ref, size];
}