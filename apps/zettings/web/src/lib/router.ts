/**
 * Hash router for ZETTINGS deep links (PLAN §6).
 *
 * Routes mirror the `zettings://` scheme: `#/system/bluetooth` ↔
 * `zettings://system/bluetooth`. L1 = category hub, L2 = settings page.
 * Browser history owns back/forward — no parallel stack to desynchronize.
 * Unknown routes resolve to a safe home.
 */

export type Route =
  | { kind: "home" }
  | { kind: "category"; category: string; sub?: string };

const SLUG = /^[a-z0-9-]+$/;
/** Sub-segments may be empty strings when trailing; validated separately. */
const SUB_SLUG = /^[a-z0-9-]+$/;

/** Parses a location hash into a Route. */
export function parseHash(hash: string): Route {
  const path = hash.replace(/^#\/?/, "").replace(/\/+$/, "");
  if (path === "" || path === "home") return { kind: "home" };
  const segments = path.split("/");
  const category = segments[0] ?? "";
  const sub = segments.length > 1 ? (segments[1] ?? "") : undefined;
  // Every segment must be a valid slug; unknown-but-well-formed categories
  // resolve to an honest "unknown location" page in the shell.
  const wellFormed =
    SLUG.test(category) && (sub === undefined || SUB_SLUG.test(sub));
  return wellFormed
    ? sub !== undefined && sub !== ""
      ? { kind: "category", category, sub }
      : { kind: "category", category }
    : { kind: "home" };
}

/** Current route. */
export function currentRoute(): Route {
  return parseHash(window.location.hash);
}

/** Navigates programmatically (pushes history so Back works). */
export function navigateToRoute(route: Route): void {
  const target =
    route.kind === "home"
      ? "#/home"
      : route.sub !== undefined
        ? `#/${route.category}/${route.sub}`
        : `#/${route.category}`;
  if (window.location.hash !== target) {
    window.location.hash = target;
  }
}

/** Converts a registry deep link (`zettings://x[/y]`) to an app hash. */
export function deepLinkToHash(link: string): string | null {
  return link.startsWith("zettings://") ? `#${link.slice("zettings://".length)}` : null;
}
