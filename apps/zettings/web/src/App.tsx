import { useEffect, useState } from "react";
import type { CategorySummaryDto, RegistrySnapshotDto } from "@zettings/bindings";
import { invokeIpc } from "./lib/ipc";
import { NavRow } from "./components/zdl";

type LoadState =
  | { phase: "loading" }
  | { phase: "ready"; snapshot: RegistrySnapshotDto }
  | { phase: "error"; message: string };

/**
 * Phase-2 shell proof: renders the registry seed graph through ZDL
 * primitives over typed IPC. The full responsive shell replaces this in
 * PLAN Phase 4.
 */
export function App() {
  const [state, setState] = useState<LoadState>({ phase: "loading" });
  const [currentId, setCurrentId] = useState<string | null>(null);

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

  const openCategory = (category: CategorySummaryDto) => {
    setCurrentId(category.id);
  };

  return (
    <main style={{ maxWidth: 1000, margin: "0 auto", padding: "var(--space-8)" }}>
      <h1 style={{ font: "var(--text-display)", marginBlockEnd: "var(--space-6)" }}>
        Zettings
      </h1>
      {state.phase === "loading" && <p role="status">Loading settings registry…</p>}
      {state.phase === "error" && (
        <div role="alert">
          <p>Settings backend unavailable.</p>
          <p>{state.message}</p>
        </div>
      )}
      {state.phase === "ready" && (
        <nav aria-label="Settings categories">
          <div style={{ display: "grid", gap: "var(--space-1)" }}>
            {state.snapshot.categories.map((category) => (
              <NavRow
                key={category.id}
                label={category.title}
                current={currentId === category.id}
                onActivate={() => openCategory(category)}
              />
            ))}
          </div>
        </nav>
      )}
    </main>
  );
}
