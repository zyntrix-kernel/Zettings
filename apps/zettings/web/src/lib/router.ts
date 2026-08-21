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
  const [head, ...rest] = path.split("/");
  const category = head?.toLowerCase() ?? "";
  // Only the category segment is addressable until L2 pages land; deeper
  // segments are preserved in the URL but currently resolve to their hub.
  return category === "" || rest.some((s) => !/^[a-z0-9-]*$/.test(s))
    ? { kind: "home" }
    : { kind: "category", category };
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
