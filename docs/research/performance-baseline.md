# Performance Baseline & Measurement Methodology

> prompt.txt targets: cold launch < 500 ms · hot launch < 150 ms · memory
> < 150 MB · 120 FPS rendering target. PLAN §10: every claim measured, never
> assumed. This document defines HOW numbers will be produced so Phase 9 has a
> reproducible protocol and Phase 0 has an honest baseline of "not yet measured".

## Environments

| Env | Role | Notes |
|---|---|---|
| WSL2 Kubuntu 24.04 (bare metal where available) | authoritative measurements | real backend, `x86_64-unknown-linux-gnu` |
| Windows host | frontend-only smoke checks via `zettings-mock` | numbers NOT release-authoritative |

Record per run: CPU model, RAM, storage type, kernel, compositor state,
power profile. Results without environment metadata are void.

## Launch latency

- **Cold:** process start → first interactive frame.
  - Rust side: instrument `main()` with `tracing` spans (`boot.total`,
    `boot.plugins`, `boot.window_shown`).
  - Webview side: `performance.timing` equivalent — `navigationStart` →
    first contentful paint → shell `ready` event posted back to Rust.
  - Command: hyperfine (≥20 runs, warmup discarded) around the binary;
    report median + p95.
- **Hot:** second launch with OS caches warm + window-state restore; same spans.

## Memory

- Peak RSS via `/proc/self/status` VmHWM read at exit + sampled every 2 s by an
  external sampler during scripted interaction.
- Webview heap: Chromium `--js-flags=--max-old-space-size` left default;
  measure via CDP `Performance.getMetrics` through agent-browser when needed.
- Budget split (initial): webview ≤ 90 MB, Rust core ≤ 40 MB, headroom 20 MB.

## Frame rate / motion

- Instrumentation hook in the motion engine records frame intervals via
  `requestAnimationFrame`; exports p50/p95/p99 frame time + longest task.
- Automated check: run scripted navigation/expansion scenarios under
  agent-browser (CDP tracing) on the dev host; assert p95 frame time budget
  (<8.3 ms for 120 Hz-class work; document hardware ceiling otherwise).
- GPU compositing verified via layer breakdown (CDP layer tree), not assumed.

## React render discipline

- React DevTools profiler in CI-lite form: scripted scenario asserts that
  registry-driven page switch causes O(page) renders, not O(app).
- Lint rule: no anonymous object/arrow props into list-row components
  (enforced by eslint config once frontend scaffold exists).

## IPC latency

- Benchmark command: typed ping command round-trip, 1k iterations, report
  median/p95. Target < 5 ms local round-trip for small payloads.

## Search indexing

- Index build time for full registry; incremental re-index cost per changed page.
- Query latency p50/p95 at 60 entries and at projected 500+ entry scale
  (synthetic registry expansion).

## Regression gates (Phase 9 acceptance)

```text
cargo bench (criterion) for: registry lookup, search query, serialization
hyperfine launch script committed to repo (tools/bench/launch.sh)
frame-time harness runnable from CLI, results emitted as JSON
```

## Current status (Phase 0)

**No measurements exist yet — nothing is claimed.** All targets above are
budgets, not achievements. First real numbers land with the Phase 4 skeleton
(launch/memory) and Phase 3 (frame timing).
