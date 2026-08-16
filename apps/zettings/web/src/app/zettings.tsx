import { useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import type { Health } from "@zettings/bindings";
import { ShellFrame } from "../components/shell-frame.js";

async function fetchHealth(): Promise<Health> {
  return await invoke<Health>("zettings_health");
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

  const { data, isLoading, isError } = useQuery({
    queryKey: ["zettings-health"],
    queryFn: fetchHealth,
  });

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
            <h2>System</h2>
          </header>
          <section className="content-body">
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
          </section>
        </main>
      </div>
    </ShellFrame>
  );
}
