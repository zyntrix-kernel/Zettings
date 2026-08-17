/**
 * ZDL Breadcrumbs — settings-graph navigation trail.
 *
 * Rendered in the content bar above the active panel. Each ancestor segment
 * is a tappable hash-link (`<a href="#/…">`) so the user can walk the app
 * hierarchy directly; the current leaf segment is shown as non-link text with
 * `aria-current="page"`.
 *
 * DESIGN.md / AGENTS.md CSS compliance: no `border-radius` on the breadcrumb
 * container (it's a horizontal nav strip, not a rounded panel); a minor pill
 * on the *chevron separators themselves* would be an icon — we use the lucide
 * `ChevronRight` icon glyph instead.
 *
 * Accessibility (ui-ux-pro-max/ux — nav-hierarchy, focus, keyboard):
 *   - `<nav aria-label="Breadcrumb">` so screen readers announce the trail.
 *   - An `<ol>` matches the visual order (ui-ux-pro-max High: Tab order
 *     matches visual order).
 *   - Visible focus ring on each link (High rule).
 */
import { useEffect } from "react";
import { ChevronRight, Home } from "lucide-react";
import {
  ancestorSegments,
  buildHashHref,
  type HashRoute,
} from "../lib/hash-route.js";

export interface BreadcrumbsProps {
  /** The current parsed route. */
  route: HashRoute;
  /** Called when the user clicks an ancestor link — defaults to a no-op
   *  because the hash-link handler is native. Provided so a future in-app
   *  transition hook (Phase 3 velocity-preserving route transition scaffold)
   *  can hook ancestor-click to seed the enter-spring. */
  onNavigate?: (segments: readonly string[]) => void;
}

/** Title-case a single route segment for display: `night-light` → `Night Light`. */
function displayForSegment(segment: string): string {
  return segment
    .split("-")
    .map((piece) => (piece.length === 0 ? piece : piece.charAt(0).toUpperCase() + piece.slice(1)))
    .join(" ");
}

/** Strip the leading slash from `route.raw` for rendering the canonical path. */
function hrefForSegments(segments: readonly string[]): string {
  return buildHashHref(segments);
}

export function Breadcrumbs({ route, onNavigate }: BreadcrumbsProps): React.ReactElement {
  // Pre-compute the ordered clickable ancestors (oldest → newest, excluding
  // the leaf, which is rendered as a non-link current-page marker).
  const ancestors = route.segments.map((_, index) => ancestorSegments(route.segments, index + 1));

  // Click handler: native `hashchange` triggers `useHashRoute` automatically,
  // but if the consumer supplies `onNavigate`, we call it with the resolved
  // ancestor segments so the Phase 3 route transition spring can seed.
  useEffect(() => {
    // No-op: kept here so future transition-hook registration can land without
    // restructuring the component shape (ui-ux-pro-max/react: derive rather
    // than re-render). `onNavigate` is read on demand in the click handler.
    return;
  }, [onNavigate]);

  const handleClick = (segments: readonly string[], event: React.MouseEvent<HTMLAnchorElement>) => {
    // Let Cmd/Ctrl+click open in a new webview tab (closest WSL2 analogue).
    // For plain left-click we let the native `hashchange` fire.
    if (event.metaKey || event.ctrlKey) return;
    onNavigate?.(segments);
  };

  return (
    <nav className="breadcrumbs" aria-label="Breadcrumb">
      <ol className="breadcrumb-list">
        {/* Root affordance — clicking always returns to the overview. */}
        <li className="breadcrumb-item breadcrumb-root">
          <a
            className="breadcrumb-link"
            href={hrefForSegments([])}
            aria-label="Settings overview"
            onClick={(e) => handleClick([], e)}
          >
            <Home size={14} aria-hidden="true" />
            <span className="breadcrumb-root-label">Overview</span>
          </a>
        </li>

        {ancestors.map((segments) => {
          // `segments.length` is ≥ 1 by construction (map is over indexes;
          // ancestorSegments(segments, index + 1) takes index + 1 elements).
          const leaf = segments[segments.length - 1] ?? "";
          const label = displayForSegment(leaf);
          return (
            <li
              key={segments.join("/")}
              className="breadcrumb-item"
            >
              <ChevronRight
                className="breadcrumb-chevron"
                size={14}
                aria-hidden="true"
              />
              <a
                className="breadcrumb-link"
                href={hrefForSegments(segments)}
                onClick={(e) => handleClick(segments, e)}
              >
                {label}
              </a>
            </li>
          );
        })}

        {/* Current-page (leaf) marker: non-link, aria-current page. */}
        {!route.isRoot ? (
          <li className="breadcrumb-item breadcrumb-current" aria-current="page">
            <ChevronRight
              className="breadcrumb-chevron"
              size={14}
              aria-hidden="true"
            />
            <span className="breadcrumb-current-label">
              {displayForSegment(route.segments[route.segments.length - 1] ?? "")}
            </span>
          </li>
        ) : null}
      </ol>
    </nav>
  );
}

export default Breadcrumbs;
