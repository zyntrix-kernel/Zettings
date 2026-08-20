/**
 * PerfMonitor — Phase 9 dev-loop overlay for the launch-time, memory, and
 * 120 FPS audit.
 *
 * Renders a compact, non-interactive status card pinned to the bottom-right
 * corner. It is a **dev-only** tool: the component returns `null` unless
 * `import.meta.env.DEV` is true, and the production bundle never renders it.
 *
 * It surfaces:
 * - Cold-start latency (script start → first painted frame) with the
 *   `<500ms` Phase 9 budget.
 * - Hot-start latency (last route change → next painted frame) with the
 *   `<150ms` Phase 9 budget.
 * - Live FPS meter: p50/p95 frame time vs the 120 FPS (8.33 ms) compositor
 *   budget from DESIGN.md, plus a dropped-frame count.
 * - Backend telemetry from `zettings_perf_stats`: process uptime, startup
 *   latency, and resident set size (Linux only; `null` on the Windows mock
 *   dev loop). Validates the `<150MB` idle-RAM budget.
 *
 * Accessibility: the card is `role="status"` with `aria-live="polite"` so
 * screen readers can announce changing audit numbers without interrupting
 * (ui-ux-pro-max: live regions must not be overly chatty — values update at
 * the 250 ms report cadence, not per frame).
 */
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import type { PerfStatsDto } from "@zettings/bindings";
import {
  FPS_BUDGET_MS,
  PERF_MARKS,
  perfMeasure,
  useFpsMeter,
} from "../lib/perf.js";

/** Format a millisecond value for the overlay. */
function fmtMs(ms: number | undefined): string {
  return ms === undefined ? "—" : `${ms.toFixed(1)} ms`;
}

/** Format a byte count as MB. */
function fmtMb(bytes: number | null): string {
  return bytes === null ? "n/a (mock)" : `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

/**
 * Poll `zettings_perf_stats` on an interval. `is_mock` is part of the DTO, so
 * no separate health call is needed for the overlay badge.
 */
function useBackendPerf(intervalMs = 2000): PerfStatsDto | null {
  const [dto, setDto] = useState<PerfStatsDto | null>(null);

  useEffect(() => {
    let cancelled = false;
    let timer: ReturnType<typeof setInterval> | null = null;

    const poll = async (): Promise<void> => {
      try {
        const next = await invoke<PerfStatsDto>("zettings_perf_stats");
        if (!cancelled) setDto(next);
      } catch {
        // Backend unreachable (e.g. pure `vite` preview without Tauri) — the
        // overlay keeps showing launch/FPS data and omits backend telemetry.
      }
    };

    void poll();
    timer = setInterval(() => void poll(), intervalMs);
    return () => {
      cancelled = true;
      if (timer !== null) clearInterval(timer);
    };
  }, [intervalMs]);

  return dto;
}

/** Cold-start window, refreshed on each render. */
function useColdStartMs(): number | undefined {
  return perfMeasure("zettings:measure-cold-start", PERF_MARKS.scriptStart, PERF_MARKS.firstPaint);
}

/** Hot-start window for the most recent route navigation. */
function useHotStartMs(): number | undefined {
  const [hotMs, setHotMs] = useState<number | undefined>(undefined);

  useEffect(() => {
    const id = setInterval(() => {
      const ms = perfMeasure("zettings:measure-hot-start", PERF_MARKS.routeStart, PERF_MARKS.routePainted);
      if (ms !== undefined) setHotMs(ms);
    }, 250);
    return () => clearInterval(id);
  }, []);

  return hotMs;
}

export function PerfMonitor(): React.ReactElement | null {
  // Dev-only: the overlay is stripped from production builds at the call site
  // (`{import.meta.env.DEV ? <PerfMonitor /> : null}`) AND guarded here so
  // direct imports in tests are safe.
  if (!import.meta.env.DEV) return null;

  const fps = useFpsMeter();
  const backend = useBackendPerf();
  const coldStartMs = useColdStartMs();
  const hotStartMs = useHotStartMs();
  const uptimeText = backend ? `${backend.backend_uptime_ms.toFixed(0)} s` : "—";

  const coldBudget = 500; // PLAN.md Phase 9
  const hotBudget = 150; // PLAN.md Phase 9
  const coldOk = coldStartMs !== undefined && coldStartMs < coldBudget;
  const hotOk = hotStartMs !== undefined && hotStartMs < hotBudget;

  return (
    <div
      className="perf-monitor"
      role="status"
      aria-live="polite"
      data-testid="perf-monitor"
    >
      <dl className="perf-monitor-grid">
        <div>
          <dt>FPS</dt>
          <dd>{fps.fps.toFixed(0)}</dd>
        </div>
        <div>
          <dt>p95 frame</dt>
          <dd>{fps.p95FrameMs.toFixed(2)} ms</dd>
        </div>
        <div>
          <dt>budget</dt>
          <dd>{FPS_BUDGET_MS.toFixed(2)} ms</dd>
        </div>
        <div>
          <dt>dropped</dt>
          <dd>{fps.droppedFrames}</dd>
        </div>
        <div>
          <dt>cold start</dt>
          <dd className={coldOk ? "perf-ok" : "perf-warn"}>{fmtMs(coldStartMs)}</dd>
        </div>
        <div>
          <dt>hot start</dt>
          <dd className={hotOk ? "perf-ok" : "perf-warn"}>{fmtMs(hotStartMs)}</dd>
        </div>
        <div>
          <dt>uptime</dt>
          <dd>{uptimeText}</dd>
        </div>
        <div>
          <dt>RSS</dt>
          <dd>{fmtMb(backend?.memory_rss_bytes ?? null)}</dd>
        </div>
        <div>
          <dt>backend</dt>
          <dd>{backend ? (backend.is_mock ? "mock" : "linux") : "—"}</dd>
        </div>
      </dl>
    </div>
  );
}