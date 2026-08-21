/**
 * Frame-time instrumentation (DESIGN.md §10, perf-baseline.md protocol).
 *
 * A dependency-free rAF sampler that records frame intervals while active and
 * exposes percentile summaries. The Phase 8/9 harnesses and dev overlay
 * consume this; it never runs unless started explicitly.
 */

export interface FrameStats {
  /** Sampled frames. */
  samples: number;
  /** Median frame interval (ms). */
  p50: number;
  /** 95th percentile frame interval (ms). */
  p95: number;
  /** 99th percentile frame interval (ms). */
  p99: number;
  /** Longest observed interval (ms). */
  longest: number;
}

function percentile(sorted: number[], p: number): number {
  const idx = Math.min(sorted.length - 1, Math.ceil((p / 100) * sorted.length) - 1);
  return sorted[Math.max(0, idx)] ?? 0;
}

/**
 * Samples `requestAnimationFrame` intervals. Start/stop is idempotent;
 * `summary()` may be called at any time.
 */
export class FrameMonitor {
  #rafId: number | null = null;
  #last: number | null = null;
  readonly #intervals: number[] = [];

  /** Begins sampling; no-op when already running. */
  start(): void {
    if (this.#rafId !== null) return;
    this.#last = null;
    const tick = (now: number) => {
      if (this.#last !== null) {
        this.#intervals.push(now - this.#last);
      }
      this.#last = now;
      this.#rafId = requestAnimationFrame(tick);
    };
    this.#rafId = requestAnimationFrame(tick);
  }

  /** Stops sampling; the rAF chain is cancelled. */
  stop(): void {
    if (this.#rafId !== null) {
      cancelAnimationFrame(this.#rafId);
      this.#rafId = null;
    }
    this.#last = null;
  }

  /** Drops collected samples (keeps running state). */
  reset(): void {
    this.#intervals.length = 0;
  }

  /** Current percentile summary over collected samples. */
  summary(): FrameStats {
    const sorted = [...this.#intervals].sort((a, b) => a - b);
    const n = sorted.length;
    return {
      samples: n,
      p50: n > 0 ? percentile(sorted, 50) : 0,
      p95: n > 0 ? percentile(sorted, 95) : 0,
      p99: n > 0 ? percentile(sorted, 99) : 0,
      longest: n > 0 ? (sorted[n - 1] ?? 0) : 0,
    };
  }
}
