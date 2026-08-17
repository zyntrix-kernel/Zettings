import { useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import type { Health } from "@zettings/bindings";
import { Breadcrumbs } from "../components/breadcrumbs.js";
import { ShellFrame } from "../components/shell-frame.js";
import { SpotlightModal } from "../components/spotlight-modal.js";
import { TargetHighlightBoundary } from "../components/target-highlight-boundary.js";
import { useHashRoute } from "../lib/hash-route.js";
import { useSpotlightStore } from "../stores/spotlight-store.js";

async function fetchHealth(): Promise<Health> {
  return await invoke<Health>("zettings_health");
}

/**
 * Returns `true` if the given keyboard event matches one of the Spotlight
 * modal global shortcuts:
 *   - `Super+I`  (Tauri reports the Super key as `metaKey` on Linux/macOS
 *                 and `ctrlKey` is *not* pressed simultaneously)
 *   - `Ctrl+Space`
 *
 * Defensively checks both `meta` and `ctrl` modifiers for the `Super+I`
 * binding to behaviour-identical on Windows / macOS / KDE host keyboards.
 */
function isSpotlightShortcut(event: KeyboardEvent): boolean {
  if (event.type !== "keydown") return false;
  if (event.code === "KeyI" && (event.metaKey || event.ctrlKey) && !event.altKey) {
    // Super+I (metaKey on Linux/KDE, ctrlKey on some Windows keyboards) —
    // do not also fire when Alt is held (Alt+Super+I is reserved).
    return true;
  }
  if (event.code === "Space" && event.ctrlKey && !event.metaKey && !event.altKey) {
    return true;
  }
  return false;
}

export function Zettings(): React.ReactElement {
  const [reducedMotion, setReducedMotion] = useState(false);
  useEffect(() => {
    const mq = window.matchMedia("(prefers-reduced-motion: reduce)");
    setReducedMotion(mq.matches);
    const handler = (e: MediaQueryListEvent): void => setReducedMotion(e.matches);
    mq.addEventListener("change", handler);
    return () => mq.removeEventListener("change", handler);
  }, []);

  // Spotlight modal store hooks — `open()` and `close()` are stable refs
  // returned by zustand, so the effect below can depend on them once.
  const openSpotlight = useSpotlightStore((state) => state.open);
  const closeSpotlight = useSpotlightStore((state) => state.close);

  // Global keydown listener for the Spotlight shortcut (Super+I / Ctrl+Space).
  // Phase 6.3: document-level listener is the compliant substitute for the
  // not-yet-installed `tauri-plugin-global-shortcut` (see PLAN.md Phase 6.3).
  useEffect(() => {
    const handler = (event: KeyboardEvent): void => {
      if (isSpotlightShortcut(event)) {
        event.preventDefault();
        openSpotlight();
      } else if (event.key === "Escape") {
        closeSpotlight();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [openSpotlight, closeSpotlight]);

  const { data, isLoading, isError } = useQuery({
    queryKey: ["zettings-health"],
    queryFn: fetchHealth,
  });

  // Phase 6.4 — hash-route state driving breadcrumbs + the target-control
  // highlight boundary. `useHashRoute` subscribes to `window.hashchange`.
  const route = useHashRoute();

  return (
    <ShellFrame>
      <div className="zettings-shell" data-reduced-motion={reducedMotion}>
        <aside className="sidebar">
          <header className="sidebar-title">
            <span className="brand-glyph" aria-hidden />
            <h1>Zettings</h1>
          </header>
          <nav aria-label="Settings modules">
            <p className="sidebar-placeholder">Loading modules…</p>
          </nav>
        </aside>
        <main className="content">
          <header className="content-bar">
            <Breadcrumbs route={route} />
          </header>
          <section className="content-body">
            <TargetHighlightBoundary>
              {isLoading ? (
                <p>Connecting to backend…</p>
              ) : isError ? (
                <p role="alert">Backend unreachable.</p>
              ) : data === undefined ? null : (
                <p>
                  Zettings v{data.version}
                  {data.is_mock ? " (mock backend)" : ""}
                </p>
              )}
            </TargetHighlightBoundary>
          </section>
        </main>
      </div>
      {/*
        Spotlight modal is rendered last so it overlays the shell via
        position:fixed. It unmounts itself when closed (the spring exit
        transition completes before DOM teardown).
      */}
      <SpotlightModal />
    </ShellFrame>
  );
}
