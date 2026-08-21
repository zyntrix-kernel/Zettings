/**
 * Hash router for ZETTINGS deep links (PLAN §6).
 *
 * Routes mirror the `zettings://` scheme: `#/system/display` ↔
 * `zettings://system/display`. Browser history owns back/forward — no
 * parallel stack to desynchronize. Unknown routes resolve to a safe home.
 */

export type Route =
  | { kind: "home" }
  | { kind: "category"; category: string };

/** Parses a location hash into a Route. */
export function parseHash(hash: string): Route {
  const path = hash.replace(/^#\/?/, "").replace(/\/+$/, "");
  if (path === "" || path === "home") return { kind: "home" };
  const segments = path.split("/");
  const category = segments[0]?.toLowerCase() ?? "";
  // Every segment must be a valid slug; unknown-but-well-formed categories
  // resolve to an honest "unknown location" page in the shell.
  const wellFormed =
    category !== "" &&
    /^[a-z0-9-]+$/.test(category) &&
    segments.every((s) => /^[a-z0-9-]*$/.test(s));
  return wellFormed ? { kind: "category", category } : { kind: "home" };
}

/** Current route. */
export function currentRoute(): Route {
  return parseHash(window.location.hash);
}

/** Navigates programmatically (pushes history so Back works). */
export function navigateToRoute(route: Route): void {
  const target = route.kind === "home" ? "#/home" : `#/${route.category}`;
  if (window.location.hash !== target) {
    window.location.hash = target;
  }
}

/** Converts a registry deep link (`zettings://x[/y]`) to an app hash. */
export function deepLinkToHash(link: string): string | null {
  return link.startsWith("zettings://") ? `#${link.slice("zettings://".length)}` : null;
}
