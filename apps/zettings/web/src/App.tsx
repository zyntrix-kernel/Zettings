import { useEffect, useRef, useState, type ReactNode } from "react";
import { AnimatePresence, motion } from "motion/react";
import type { CategorySummaryDto, RegistrySnapshotDto } from "@zettings/bindings";
import { invokeIpc } from "./lib/ipc";
import { currentRoute, navigateToRoute, type Route } from "./lib/router";
import { useMotionPolicy } from "./lib/motion";
import { categoryIcon } from "./lib/category-icons";
import { AppShell, routeFromDeepLink } from "./components/shell/app-shell";
import { EmptyState, ErrorBar, Loading } from "./components/shell/status";
import { SettingsCard } from "./components/zdl";
import { SystemPage } from "./SystemPage";

type LoadState =
  | { phase: "loading" }
  | { phase: "ready"; snapshot: RegistrySnapshotDto }
  | { phase: "error"; message: string };

function routeTitle(route: Route, categories: CategorySummaryDto[]): string {
  if (route.kind === "home") return "Home";
  return (
    categories.find((c) => c.id === route.category)?.title ?? "Settings"
  );
}

export function App() {
  const [state, setState] = useState<LoadState>({ phase: "loading" });
  const [route, setRoute] = useState<Route>(() => currentRoute());
  const headingRef = useRef<HTMLHeadingElement>(null);
  const policy = useMotionPolicy();

  useEffect(() => {
    let cancelled = false;
    invokeIpc<RegistrySnapshotDto>("registry_snapshot")
      .then((snapshot) => {
        if (!cancelled) setState({ phase: "ready", snapshot });
      })
      .catch((cause: unknown) => {
        if (!cancelled) {
          const message = cause instanceof Error ? cause.message : String(cause);
          setState({ phase: "error", message });
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  // Browser history owns Back/Forward; hashchange is the single source.
  useEffect(() => {
    const onHashChange = (): void => setRoute(currentRoute());
    window.addEventListener("hashchange", onHashChange);
    return () => window.removeEventListener("hashchange", onHashChange);
  }, []);

  // Client-side routing contract: title update + focus on page heading.
  const categories =
    state.phase === "ready" ? state.snapshot.categories : [];
  useEffect(() => {
    document.title = `${routeTitle(route, categories)} · Zettings`;
    headingRef.current?.focus();
  }, [route, categories]);

  const navigate = (next: Route): void => navigateToRoute(next);
  const openDeepLink = (link: string): void =>
    setRoute(routeFromDeepLink(link));

  let content: ReactNode;
  if (state.phase === "loading") {
    content = <Loading label="Loading settings…" />;
  } else if (state.phase === "error") {
    content = (
      <ErrorBar
        title="Settings backend unavailable"
        detail={state.message}
        onRetry={() => window.location.reload()}
      />
    );
  } else if (route.kind === "home") {
    content = (
      <>
        <h1 ref={headingRef} tabIndex={-1} className="zdl-page-title">
          Home
        </h1>
        <div className="zdl-card-grid">
          {state.snapshot.categories.map((category) => {
            const Icon = categoryIcon(category.id);
            return (
              <SettingsCard
                key={category.id}
                title={category.title}
                description={category.description}
                icon={<Icon size={20} />}
                onActivate={() =>
                  navigate({ kind: "category", category: category.id })
                }
              />
            );
          })}
        </div>
      </>
    );
  } else {
    const category = state.snapshot.categories.find(
      (c) => c.id === route.category,
    );
    content = category === undefined
      ? (
        <EmptyState
          title="Unknown location"
          explanation="That settings address does not exist. Use search or the navigation pane to find a setting."
          action={
            <button type="button" className="zdl-button" onClick={() => navigate({ kind: "home" })}>
              Go to Home
            </button>
          }
        />
      )
      : (
        <>
          <h1 ref={headingRef} tabIndex={-1} className="zdl-page-title">
            {category.title}
          </h1>
          <p className="zdl-page-description">{category.description}</p>
          {category.id === "system" ? (
            <SystemPage />
          ) : (
            /* Honest empty state: pages arrive with their adapter phases. */
            <EmptyState
              title="No settings pages yet"
              explanation={`Settings for ${category.title} connect as system adapters are integrated. The category is registered and searchable; its pages are not built yet.`}
            />
          )}
        </>
      );
  }

  return (
    <AppShell
      route={route}
      categories={categories}
      onNavigate={navigate}
      onOpenDeepLink={openDeepLink}
    >
      <AnimatePresence mode="wait" initial={false}>
        <motion.div
          key={
            state.phase === "ready"
              ? `${route.kind}:${route.kind === "category" ? route.category : ""}`
              : state.phase
          }
          className="zdl-page"
          initial={policy.page.initial}
          animate={policy.page.animate}
          exit={policy.page.exit}
          transition={policy.page.transition}
        >
          {content}
        </motion.div>
      </AnimatePresence>
    </AppShell>
  );
}
