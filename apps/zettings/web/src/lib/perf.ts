/**
 * Zettings performance tooling — Phase 9 launch-time + 120 FPS audit surface.
 *
 * Provides:
 * - `performance.mark`/`measure` helpers for cold-start and hot-start tracing
 *   (cold start = script start → first painted frame; hot start = route change
 *   → next painted frame).
 * - `useFpsMeter` — a `requestAnimationFrame`-driven frame-time meter that
 *   reports instantaneous FPS plus p50/p95 frame times and the count of
 *   dropped frames against the 120 FPS compositor budget (8.33 ms/frame) from
 *   `DESIGN.md`.
 *
 * All helpers are safe to call in production builds: the PerfMonitor overlay
 * component gates its own rendering on `import.meta.env.DEV`, and the raw
 * `performance` API calls are no-ops in environments that do not expose them.
 *
 * Accessibility: the FPS meter uses `requestAnimationFrame` (never timers that
 * starve main-thread paint) and the overlay is implemented as plain text with
 * `role="status"` semantics so screen readers can announce the audit numbers.
 */
import { useEffect, useRef, useState } from "react";

/** Well-known mark/measure names used across the app. */
export const PERF_MARKS = {
  /** Module-eval time of `main.tsx` — the earliest possible script timestamp. */
  scriptStart: "zettings:script-start",
  /** React `createRoot(...).render()` call site. */
  reactMount: "zettings:react-mount",
  /** First `requestAnimationFrame` after React commits — earliest paint proxy. */
  firstPaint: "zettings:first-paint",
  /** A route navigation was dispatched (`hashchange`). */
  routeStart: "zettings:route-start",
  /** First frame after the route's panel is committed. */
  routePainted: "zettings:route-painted",
} as const;

/** Emit a named `performance.mark` (no-op when `performance.mark` is absent). */
export function perfMark(name: string): void {
  if (typeof performance !== "undefined" && "mark" in performance) {
    performance.mark(name);
  }
}

/**
 * Emit a `performance.measure` between two marks, returning its duration in
 * milliseconds, or `undefined` when the Performance API or one of the marks is
 * unavailable (e.g. first call in a fresh page before either mark exists).
 */
export function perfMeasure(name: string, from: string, to: string): number | undefined {
  if (typeof performance === "undefined" || !("measure" in performance)) {
    return undefined;
  }
  const startEntry = performance.getEntriesByName(from).at(-1);
  const endEntry = performance.getEntriesByName(to).at(-1);
  if (!startEntry || !endEntry) return undefined;
  const start = startEntry.startTime;
  const end = endEntry.startTime;
  if (end < start) return undefined;
  performance.measure(name, { start, end });
  return end - start;
}

/** Convenience: `performance.now()` or `0` for environments without it. */
export function perfNow(): number {
  return typeof performance !== "undefined" ? performance.now() : 0;
}

/** A single frame-time sample from the FPS meter. */
export interface FrameSample {
  /** Wall-clock timestamp of the frame (ms, `performance.now` base). */
  readonly timestamp: number;
  /** Frame duration in ms (delta from the previous frame). */
  readonly frameMs: number;
}

/** Aggregate frame-time statistics for a rolling window. */
export interface FpsStats {
  /** Rolling window length in ms (default 1000 ms). */
  readonly windowMs: number;
  /** Frames sampled in the current window. */
  readonly frameCount: number;
  /** Frames in the window whose frame time exceeded the 120 FPS budget. */
  readonly droppedFrames: number;
  /** Instantaneous FPS (`frameCount` scaled to one second). */
  readonly fps: number;
  /** p50 frame time in ms over the current window. */
  readonly p50FrameMs: number;
  /** p95 frame time in ms over the current window. */
  readonly p95FrameMs: number;
  /** 120 FPS compositor budget in ms per frame (8.33 ms). */
  readonly budgetMs: number;
  /** `true` when the p95 frame time stays within the 120 FPS budget. */
  readonly withinBudget: boolean;
}

/** 120 FPS compositor budget — 1000 / 120 ≈ 8.33 ms (DESIGN.md). */
export const FPS_BUDGET_MS = 1000 / 120;

/**
 * Sort a numeric sample array ascending for percentile extraction.
 * `samples` is copied so the caller's array is never mutated.
 */
function sortedCopy(values: readonly number[]): number[] {
  return [...values].sort((a, b) => a - b);
}

/** Percentile at `p` (0..1) of a pre-sorted ascending array. */
function percentile(sorted: readonly number[], p: number): number {
  if (sorted.length === 0) return 0;
  const idx = Math.min(sorted.length - 1, Math.max(0, Math.ceil(p * sorted.length) - 1));
  return sorted[idx] ?? 0;
}

/**
 * Compute window statistics from raw frame-time samples, pruning samples older
 * than `windowMs` from the tail of the current window.
 */
export function summarizeFrames(
  samples: readonly FrameSample[],
  now: number,
  windowMs: number,
): FpsStats {
  const cutoff = now - windowMs;
  const inWindow = samples.filter((s) => s.timestamp >= cutoff);
  const frameTimes = inWindow.map((s) => s.frameMs);
  const frameCount = inWindow.length;
  const elapsed = frameCount === 0 ? 1 : inWindow[inWindow.length - 1]!.timestamp - inWindow[0]!.timestamp;
  const fps = elapsed > 0 ? (frameCount * 1000) / elapsed : 0;
  const droppedFrames = frameTimes.filter((ms) => ms > FPS_BUDGET_MS).length;
  const sorted = sortedCopy(frameTimes);
  return {
    windowMs,
    frameCount,
    droppedFrames,
    fps,
    p50FrameMs: percentile(sorted, 0.5),
    p95FrameMs: percentile(sorted, 0.95),
    budgetMs: FPS_BUDGET_MS,
    withinBudget: percentile(sorted, 0.95) <= FPS_BUDGET_MS,
  };
}

/**
 * FPS meter hook — samples `requestAnimationFrame` frame deltas into a rolling
 * ring buffer and re-computes `FpsStats` roughly every `reportIntervalMs`.
 *
 * The loop keeps a single rAF callback alive (no per-frame React state writes),
 * so the meter itself contributes zero layout cost; consumers re-render at the
 * report cadence, not the frame cadence.
 *
 * @param windowMs   Rolling window length (default 1000 ms).
 * @param reportIntervalMs  How often the returned stats are refreshed
 *                          (default 250 ms).
 */
export function useFpsMeter(
  windowMs = 1000,
  reportIntervalMs = 250,
): FpsStats {
  const [stats, setStats] = useState<FpsStats>(() => summarizeFrames([], perfNow(), windowMs));
  const samplesRef = useRef<FrameSample[]>([]);
  const lastFrameRef = useRef<number>(perfNow());
  const reportTimerRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const rafRef = useRef<number | null>(null);

  useEffect(() => {
    const tick = (now: number): void => {
      const prev = lastFrameRef.current;
      const frameMs = Math.min(now - prev, 1000); // clamp tab-throttle gaps
      lastFrameRef.current = now;
      samplesRef.current.push({ timestamp: now, frameMs });
      rafRef.current = requestAnimationFrame(tick);
    };

    const report = (): void => {
      setStats(summarizeFrames(samplesRef.current, perfNow(), windowMs));
    };

    lastFrameRef.current = perfNow();
    rafRef.current = requestAnimationFrame(tick);
    reportTimerRef.current = setInterval(report, reportIntervalMs);

    return () => {
      if (rafRef.current !== null) cancelAnimationFrame(rafRef.current);
      rafRef.current = null;
      if (reportTimerRef.current !== null) clearInterval(reportTimerRef.current);
      reportTimerRef.current = null;
      samplesRef.current = [];
    };
  }, [windowMs, reportIntervalMs]);

  return stats;
}