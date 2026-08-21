/**
 * Application shell (PLAN §18): responsive navigation pane, search surface,
 * breadcrumbs, landmarks, and global keyboard contract.
 *
 * Navigation modes per spec §16 breakpoints: expanded (>1100px), compact
 * rail (800–1100px), overlay disclosure (<800px, non-modal per navigation
 * skill). The rail is a composite widget using roving tabindex.
 */
import { useCallback, useEffect, useRef, useState, type ReactNode } from "react";
import type { CategorySummaryDto } from "@zettings/bindings";
import type { Route } from "../../lib/router";
import { NavRow } from "../zdl";
import { SearchSurface } from "./search-surface";
import { ThemeSelector } from "./theme-selector";

type NavMode = "expanded" | "compact" | "overlay";

function useNavMode(): NavMode {
  const query = (): NavMode => {
    const width = window.innerWidth;
    if (width > 1100) return "expanded";
    if (width >= 800) return "compact";
    return "overlay";
  };
  const [mode, setMode] = useState<NavMode>(query);
  useEffect(() => {
    const onResize = (): void => setMode(query());
    window.addEventListener("resize", onResize);
    return () => window.removeEventListener("resize", onResize);
  }, []);
  return mode;
}

export interface AppShellProps {
  route: Route;
  categories: CategorySummaryDto[];
  /** Navigates to a category hub. */
  onNavigate: (route: Route) => void;
  /** Opens a search hit's deep link directly. */
  onOpenDeepLink: (link: string) => void;
  children: ReactNode;
}

export function AppShell({
  route,
  categories,
  onNavigate,
  onOpenDeepLink,
  children,
}: AppShellProps) {
  const navMode = useNavMode();
  const [overlayOpen, setOverlayOpen] = useState(false);
  const overlayToggleRef = useRef<HTMLButtonElement>(null);
  const railRef = useRef<HTMLDivElement>(null);

  // Esc closes the overlay nav and restores focus to its toggle
  // (non-modal disclosure contract).
  useEffect(() => {
    if (navMode !== "overlay" || !overlayOpen) return;
    const onKey = (event: KeyboardEvent): void => {
      if (event.key === "Escape") {
        setOverlayOpen(false);
        overlayToggleRef.current?.focus();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [navMode, overlayOpen]);

  // Close the overlay nav whenever width leaves the overlay mode.
  useEffect(() => {
    if (navMode !== "overlay") setOverlayOpen(false);
  }, [navMode]);

  // Alt+Backspace → back (DESIGN.md §8.4).
  useEffect(() => {
    const onKey = (event: KeyboardEvent): void => {
      if (event.altKey && event.key === "Backspace") {
        event.preventDefault();
        window.history.back();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  // Roving tabindex across the rail (keyboard skill, APG).
  const onRailKeyDown = useCallback((event: React.KeyboardEvent) => {
    const container = railRef.current;
    if (container === null) return;
    const items = Array.from(
      container.querySelectorAll<HTMLButtonElement>(".zdl-nav-row"),
    );
    if (items.length === 0) return;
    const current = items.findIndex((el) => el === document.activeElement);
    let next = -1;
    if (event.key === "ArrowDown") next = (current + 1 + items.length) % items.length;
    else if (event.key === "ArrowUp") next = (current - 1 + items.length) % items.length;
    else if (event.key === "Home") next = 0;
    else if (event.key === "End") next = items.length - 1;
    if (next < 0) return;
    event.preventDefault();
    items.forEach((el, index) => {
      el.tabIndex = index === next ? 0 : -1;
    });
    items[next]?.focus();
  }, []);

  const activeCategory =
    route.kind === "category"
      ? (categories.find((c) => c.id === route.category) ?? null)
      : null;

  const navRows = (
    <>
      <NavRow label="Home" current={route.kind === "home"} onActivate={() => onNavigate({ kind: "home" })} />
      {categories.map((category) => (
        <NavRow
          key={category.id}
          label={navMode === "expanded" ? category.title : category.title.split(" ")[0] ?? category.title}
          current={route.kind === "category" && route.category === category.id}
          onActivate={() => onNavigate({ kind: "category", category: category.id })}
        />
      ))}
    </>
  );

  const railBody = (
    <>
      <div className="zdl-rail__search">
        <SearchSurface onOpen={(hit) => onOpenDeepLink(hit.route)} />
      </div>
      <div
        className="zdl-rail__rows"
        ref={railRef}
        role="navigation"
        aria-label="Settings categories"
        onKeyDown={onRailKeyDown}
      >
        {navRows}
      </div>
      <div className="zdl-rail__footer">
        <ThemeSelector />
      </div>
    </>
  );

  return (
    <div className="zdl-shell" data-nav-mode={navMode}>
      <a className="skip-link" href="#main-content">
        Skip to main content
      </a>

      {navMode === "overlay" && (
        <header className="zdl-topbar">
          <button
            ref={overlayToggleRef}
            type="button"
            className="zdl-button"
            aria-expanded={overlayOpen ? "true" : "false"}
            aria-controls="app-nav"
            onClick={() => setOverlayOpen((v) => !v)}
          >
            Menu
          </button>
          <span className="zdl-topbar__title">Zettings</span>
        </header>
      )}

      <div className="zdl-shell__body">
        {navMode === "overlay" ? (
          <div id="app-nav" hidden={!overlayOpen} className="zdl-rail zdl-rail--overlay">
            {railBody}
          </div>
        ) : (
          <aside id="app-nav" className={`zdl-rail zdl-rail--${navMode}`}>
            {railBody}
          </aside>
        )}

        <main id="main-content" tabIndex={-1} className="zdl-content">
          <nav aria-label="Breadcrumb" hidden={activeCategory === null}>
            <ol className="zdl-breadcrumbs">
              <li>
                <button type="button" className="zdl-link" onClick={() => onNavigate({ kind: "home" })}>
                  Home
                </button>
              </li>
              {activeCategory !== null && (
                <li aria-current="page">{activeCategory.title}</li>
              )}
            </ol>
          </nav>
          {children}
        </main>
      </div>
    </div>
  );
}

/** Converts a deep link into a Route for the router. */
export function routeFromDeepLink(link: string): Route {
  const hash = link.startsWith("zettings://")
    ? `#${link.slice("zettings://".length)}`
    : "#/home";
  const path = hash.replace(/^#\/?/, "");
  const head = path.split("/")[0] ?? "";
  return head === "" || head === "home" ? { kind: "home" } : { kind: "category", category: head };
}
