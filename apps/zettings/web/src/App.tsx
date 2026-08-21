import { useEffect, useState } from "react";
import type { RegistrySnapshotDto } from "@zettings/bindings";
import { invokeIpc } from "./lib/ipc";

type LoadState =
  | { phase: "loading" }
  | { phase: "ready"; snapshot: RegistrySnapshotDto }
  | { phase: "error"; message: string };

/**
 * Phase-1 shell proof: renders the registry seed graph fetched over typed
 * IPC. The full ZDL shell replaces this in PLAN Phase 4; structure (nav list,
 * honest loading/error states) is kept intentionally.
 */
export function App() {
  const [state, setState] = useState<LoadState>({ phase: "loading" });

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

  return (
    <main className="shell">
      <h1>Zettings</h1>
      {state.phase === "loading" && (
        <p role="status">Loading settings registry…</p>
      )}
      {state.phase === "error" && (
        <div role="alert">
          <p>Settings backend unavailable.</p>
          <p>{state.message}</p>
        </div>
      )}
      {state.phase === "ready" && (
        <nav aria-label="Settings categories">
          <ul>
            {state.snapshot.categories.map((category) => (
              <li key={category.id}>
                {category.title} — {category.description}
              </li>
            ))}
          </ul>
        </nav>
      )}
    </main>
  );
}
