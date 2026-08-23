import { useEffect, useRef, useState, type ReactNode } from "react";
import { AnimatePresence, motion } from "motion/react";
import type { CategorySummaryDto, RegistrySnapshotDto } from "@zettings/bindings";
import { invokeIpc } from "./lib/ipc";
import { currentRoute, navigateToRoute, type Route } from "./lib/router";
import { useMotionPolicy } from "./lib/motion";
import { categoryIcon } from "./lib/category-icons";
import { findLiveArea } from "./lib/pages";
import { AppShell, routeFromDeepLink } from "./components/shell/app-shell";
import { EmptyState, ErrorBar, Loading } from "./components/shell/status";
import { SettingsCard } from "./components/zdl";
import { CategoryHub } from "./components/CategoryHub";
import { BluetoothPage } from "./components/BluetoothPage";
import { NetworkPage } from "./components/NetworkPage";
import { PersonalizationPage } from "./components/PersonalizationPage";
import { TimeLanguagePage } from "./components/TimeLanguagePage";
import { SystemPage } from "./SystemPage";

type LoadState =
  | { phase: "loading" }
  | { phase: "ready"; snapshot: RegistrySnapshotDto }
  | { phase: "error"; message: string };

/** Live L2 pages keyed by `<category>/<sub>`. */
const LIVE_PAGES: Readonly<Record<string, () => ReactNode>> = {
  "devices/bluetooth": () => <BluetoothPage />,
  "network/status": () => <NetworkPage />,
  "personalization/theme": () => <PersonalizationPage />,
  "time-language/date-time": () => <TimeLanguagePage />,
};

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
      : route.category === "system"
        ? /* System's hub IS its multi-section live page (power/network/
             audio/bluetooth/display sections on real adapters). */
          <SystemPage />
        : route.sub !== undefined && findLiveArea(route.category, route.sub) !== null
          ? (LIVE_PAGES[`${route.category}/${route.sub}`]?.() ?? null)
          : (
            <CategoryHub
              category={route.category}
              title={category.title}
              description={category.description}
              onOpenArea={(slug) =>
                navigate({ kind: "category", category: route.category, sub: slug })
              }
            />
          );
  }

  // Focus target: hubs/home own the h1 via headingRef.
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
              ? `${route.kind}:${route.kind === "category" ? route.category : ""}:${route.kind === "category" ? (route.sub ?? "") : ""}`
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
