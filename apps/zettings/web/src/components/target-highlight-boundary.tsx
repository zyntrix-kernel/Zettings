/**
 * Target route highlight utilities — deep-link target-control highlight
 * scaffold for Phase 7 panels.
 *
 * PLAN.md Phase 6.4 calls for "deep-linking router capability that highlights
 * target controls upon navigation". The breadcrumb + `useHashRoute` hooks
 * provide the routing half; this module provides the highlight half.
 *
 * Composable primitives (ui-ux-pro-max/react — prefer composition over config):
 *   - `useTargetMatch(routeSegments: readonly string[])` returns true when the
 *     current deep-link route is *exactly* `routeSegments`. Panels invoke it
 *     once per control and render a `data-target-active` attribute on the
 *     matching element.
 *   - `<TargetHighlightBoundary>` wraps a panel subtree and stampes the active
 *     route on a stable container attribute (`data-target-route`) so descendant
 *     CSS can apply contextual highlight without re-rendering each control on
 *     every route change (ui-ux-pro-max/react: Narrow Dependencies — primitive
 *     `route` atom drives the attribute, not a prop on every control).
 */
import { type ReactNode } from "react";
import { useHashRoute } from "../lib/hash-route.js";

/**
 * Returns `true` when the active deep-link route exactly equals `routeSegments`.
 *
 * Use it inside Phase 7 panels to apply a `data-target-active` attribute to the
 * matching control. Exact-equality is the right primitive: deep-link hash-routes
 * map 1:1 to a settings control; partial matches would over-highlight groups.
 */
export function useTargetMatch(routeSegments: readonly string[]): boolean {
  const route = useHashRoute();
  if (route.isRoot) return false;
  if (route.segments.length !== routeSegments.length) return false;
  return route.segments.every((segment, index) => segment === routeSegments[index]);
}

/** Serialize route segments into a stable string for container attributes. */
function stringifyRoute(segments: readonly string[]): string {
  return segments.length === 0 ? "/" : "/" + segments.join("/");
}

export interface TargetHighlightBoundaryProps {
  /** Panel subtree; the wrapper doesn't insert any DOM of its own. */
  children: ReactNode;
  /** Optional explicit route to write (defaults to the live `useHashRoute`). */
  route?: ReturnType<typeof useHashRoute>;
}

/**
 * Stamps the current deep-link route onto a wrapping `<div
 * data-target-route>` so descendant CSS can apply target-control highlight
 * without prop-drilling into every control.
 *
 * Rendered in `zettings.tsx` content-bar around the panel region.
 */
export function TargetHighlightBoundary({
  children,
  route: explicitRoute,
}: TargetHighlightBoundaryProps): React.ReactElement {
  const liveRoute = useHashRoute();
  const route = explicitRoute ?? liveRoute;
  return (
    <div className="target-highlight-boundary" data-target-route={stringifyRoute(route.segments)}>
      {children}
    </div>
  );
}

export default TargetHighlightBoundary;
