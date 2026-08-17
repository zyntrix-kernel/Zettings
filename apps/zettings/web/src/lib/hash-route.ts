/**
 * ZDL hash-router — minimal deep-link router for the Phase 6.4 settings graph.
 *
 * The Zettings shell has no react-router dependency (per AGENTS.md, the
 * dependency footprint stays minimal). The Spotlight modal drives navigation
 * by setting `window.location.hash = entry.route` (e.g. `#/display/night-light`);
 * this hook observes the resulting `hashchange` events and exposes the parsed
 * route (`segments` + `raw`) so the Breadcrumbs component and per-route panels
 * can react to navigation and highlight the target control.
 *
 * The route schema is intentionally permissive: anything after `#` is a route.
 * A bare `#/` or empty hash resolves to the empty-route (root / overview).
 *
 * Accessibility (ui-ux-pro-max/ux — Navigation: nav-hierarchy):
 * the Breadcrumbs component built on top of this hook renders semantic
 * `<nav aria-label="Breadcrumb">` with ordered links so screen-reader users can
 * follow the hierarchy independently of visual cues.
 */
import { useEffect, useState } from "react";

/** A parsed hash route. */
export interface HashRoute {
  /** Canonical raw route string, e.g. `/display/night-light` (no leading `#`). */
  raw: string;
  /** Path segments, in order — `["display", "night-light"]` for the example. */
  segments: readonly string[];
  /** `true` when the user is on the root / overview page (empty segments). */
  isRoot: boolean;
}

/** The empty-route sentinel — returned when the hash is `#`, `#/`, or absent. */
const ROOT_ROUTE: HashRoute = {
  raw: "",
  segments: [],
  isRoot: true,
};

/** Parse a `window.location.hash` value (which may include the leading `#`). */
export function parseHashRoute(hash: string): HashRoute {
  // Drop the leading `#` (if any) and any `#/` prefix.
  const stripped = hash.replace(/^#\/?/, "");
  if (stripped.length === 0) return ROOT_ROUTE;
  const segments = stripped
    .split("/")
    .map((segment) => decodeURIComponent(segment))
    .filter((segment) => segment.length > 0);
  if (segments.length === 0) return ROOT_ROUTE;
  return {
    raw: "/" + segments.map(encodeURIComponent).join("/"),
    segments,
    isRoot: false,
  };
}

/**
 * Subscribe to `window.location.hash` changes and re-render the consumer on
 * each `hashchange` event. Per ui-ux-pro-max/react (Narrow Dependencies):
 * the effect only depends on the `hashchange` listener registration, never on
 * object references.
 */
export function useHashRoute(): HashRoute {
  const [route, setRoute] = useState<HashRoute>(() =>
    typeof window === "undefined" ? ROOT_ROUTE : parseHashRoute(window.location.hash),
  );

  useEffect(() => {
    if (typeof window === "undefined") return;
    const handler = (): void => {
      setRoute(parseHashRoute(window.location.hash));
    };
    window.addEventListener("hashchange", handler);
    // Sync once on mount in case the hash changed before subscribe.
    handler();
    return () => window.removeEventListener("hashchange", handler);
  }, []);

  return route;
}

/**
 * Build a hash URL (`#/…`) for a list of segments. Used by the Breadcrumbs
 * component to render ancestor links.
 */
export function buildHashHref(segments: readonly string[]): string {
  if (segments.length === 0) return "#/";
  return "#/" + segments.map(encodeURIComponent).join("/");
}

/**
 * Resolve the route segments up to (and including) a given depth.
 * `segments.slice(0, depth)` is wrapped here so callers can drop the
 * `noUncheckedIndexedAccess` clamp boilerplate.
 */
export function ancestorSegments(
  segments: readonly string[],
  depth: number,
): readonly string[] {
  const clamped = Math.max(0, Math.min(depth, segments.length));
  // `slice` on a readonly array returns a fresh array; safe.
  return segments.slice(0, clamped);
}
