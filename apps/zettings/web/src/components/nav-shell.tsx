/**
 * NavShell — L0 application shell: Liquid Glass navigation pane + content
 * viewport, reconstructing the Win11 NavigationView model (spec §2.1/§3.1):
 *
 *   [Search…]                      ← opens Spotlight (Super+I / Ctrl+Space)
 *   ◉ Home
 *   ▣ Displays … Personalization   ← aria-current="page" tracks the route
 *
 * Responsive behaviour (spec §16, CSS-driven):
 *   >1100px  full pane (icon + label)
 *   800–1100 compact icon rail
 *   <800px   minimal: pane becomes an overlay drawer behind a scrim,
 *            opened by the hamburger toggle (non-modal disclosure pattern:
 *            no inert, Escape closes + restores focus, scrim click closes).
 *
 * Accessibility (navigation skill): <nav aria-label="Settings"> landmark,
 * aria-current="page" + accent pill indicator (shape cue, not colour-only),
 * visible focus rings, logical DOM order.
 */
import { useEffect, useRef, useState, type ReactNode } from "react";
import { House, Menu, Search } from "lucide-react";
import type { LucideProps } from "lucide-react";
import { useSpotlightStore } from "../stores/spotlight-store.js";

export interface NavModule {
  /** Hash route segment, e.g. "display". */
  segment: string;
  /** Display label. */
  title: string;
  /** Lucide icon component. */
  icon: React.ComponentType<LucideProps>;
}

export interface NavShellProps {
  /** Category entries below Home. */
  modules: readonly NavModule[];
  /** First route segment, or null when on Home. */
  activeSegment: string | null;
  /** Toolbar content between the back button and the trailing spacer. */
  toolbar?: ReactNode;
  /** Scrollable page content. */
  children: ReactNode;
}

const NARROW_QUERY = "(max-width: 800px)";

export function NavShell({
  modules,
  activeSegment,
  toolbar,
  children,
}: NavShellProps): React.ReactElement {
  const [drawerOpen, setDrawerOpen] = useState(false);
  const toggleRef = useRef<HTMLButtonElement>(null);
  const openSpotlight = useSpotlightStore((state) => state.open);

  // Close the drawer when the viewport leaves the narrow range.
  useEffect(() => {
    const mq = window.matchMedia(NARROW_QUERY);
    const onChange = (event: MediaQueryListEvent): void => {
      if (!event.matches) setDrawerOpen(false);
    };
    mq.addEventListener("change", onChange);
    return () => mq.removeEventListener("change", onChange);
  }, []);

  // Escape closes the drawer and restores focus to the toggle.
  useEffect(() => {
    if (!drawerOpen) return;
    const onKey = (event: KeyboardEvent): void => {
      if (event.key === "Escape") {
        event.preventDefault();
        setDrawerOpen(false);
        toggleRef.current?.focus();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [drawerOpen]);

  const navigate = (hash: string): void => {
    window.location.hash = hash;
    setDrawerOpen(false);
  };

  return (
    <div className="shell-body">
      {drawerOpen ? (
        <button
          type="button"
          className="nav-backdrop"
          aria-label="Close navigation"
          onClick={() => setDrawerOpen(false)}
        />
      ) : null}

      <aside id="zettings-nav-pane" className="nav-pane" data-open={drawerOpen}>
        <button
          type="button"
          className="nav-search-btn"
          onClick={openSpotlight}
          aria-label="Search settings"
        >
          <Search size={16} aria-hidden="true" />
          <span className="nav-search-label">Search settings</span>
          <kbd className="nav-search-kbd" aria-hidden="true">
            Ctrl+I
          </kbd>
        </button>

        <nav aria-label="Settings">
          <ul className="nav-list">
            <li className="nav-item">
              <button
                type="button"
                className="nav-item-btn"
                aria-current={activeSegment === null ? "page" : undefined}
                onClick={() => navigate("#/")}
              >
                <span className="nav-item-icon" aria-hidden="true">
                  <House size={18} strokeWidth={2} />
                </span>
                <span className="nav-item-label">Home</span>
              </button>
            </li>
            {modules.map((mod) => {
              const Icon = mod.icon;
              const current = activeSegment === mod.segment;
              return (
                <li key={mod.segment} className="nav-item">
                  <button
                    type="button"
                    className="nav-item-btn"
                    aria-current={current ? "page" : undefined}
                    onClick={() => navigate(`#/${mod.segment}`)}
                  >
                    <span className="nav-item-icon" aria-hidden="true">
                      <Icon size={18} strokeWidth={2} />
                    </span>
                    <span className="nav-item-label">{mod.title}</span>
                  </button>
                </li>
              );
            })}
          </ul>
        </nav>
      </aside>

      <div className="shell-content">
        <div className="content-toolbar">
          <button
            ref={toggleRef}
            type="button"
            className="back-button nav-toggle"
            aria-expanded={drawerOpen}
            aria-controls="zettings-nav-pane"
            aria-label="Toggle navigation"
            onClick={() => setDrawerOpen((open) => !open)}
          >
            <Menu size={18} aria-hidden="true" />
          </button>
          {toolbar}
        </div>
        <div className="content-scroll">{children}</div>
      </div>
    </div>
  );
}
