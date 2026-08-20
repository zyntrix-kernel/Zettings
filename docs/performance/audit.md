# Phase 9 Performance Audit

Zettings Phase 9 optimizes the binary execution speed, memory footprint, and
rendering pipeline. The three budgets below come from `PLAN.md` Phase 9 and
are validated by the tooling described in this document.

| Budget | Target | Instrumented |
| --- | --- | --- |
| Cold start (launch to interactive) | `<500ms` | frontend `performance` marks + backend `tracing` events |
| Hot start (relaunch with warm cache) | `<150ms` | frontend `performance` marks |
| Idle memory footprint (RSS) | `<150MB` | `zettings_perf_stats` + WSL2 `ps`/`smem` |
| Rendering | 120 FPS budget (`8.33ms`/frame) | dev-only `PerfMonitor` overlay + DevTools traces |

## Instrumentation surface

### Rust backend (`apps/zettings/src/main.rs`)

- `PROCESS_START` (`OnceLock<Instant>`) is captured before anything else runs.
- `SETUP_DONE` is captured at the end of the Tauri `setup` closure.
- `zettings_perf_stats` command returns `PerfStatsDto` (`zettings-ipc`):
  `backend_uptime_ms`, `backend_startup_ms`, `memory_rss_bytes`, `is_mock`.
  RSS is read via `/proc/self/status` (`VmRSS`) on Linux only and is `None`
  on the Windows mock dev loop. Field widths are `u32` so the JSON IPC
  protocol delivers plain JS `number` (no `bigint` mismatch).
- The same numbers are emitted as `tracing::info!` events
  (`event = "main.enter"`, `event = "setup.complete"`) for the WSL2 commands
  below.

### Frontend (`apps/zettings/web`)

- `src/lib/perf.ts` records `performance` marks (`scriptStart`, `reactMount`,
  `firstPaint`, `routeStart`, `routePainted`) and hosts the `useFpsMeter`
  hook that samples `requestAnimationFrame` deltas and summarizes them into
  per-window FPS, p50/p95 frame time, and dropped-frame count
  (a frame is dropped when it exceeds `1000/120` ms).
- `src/components/perf-monitor.tsx` is a **dev-only** overlay
  (`import.meta.env.DEV`) that shows the live FPS, p95, dropped frames,
  cold/hot start time, backend uptime, and RSS. It renders `role="status"`
  with `aria-live="polite"` so screen readers announce budget breaches.

## How to run the audit

### Windows host (frontend iteration, `zettings-mock`)

```powershell
pnpm dev
```

Open the app and observe the `PerfMonitor` overlay (bottom-right). The
overlay reports cold-start from `scriptStart -> firstPaint`, hot-start from
`routeStart -> routePainted`, and the live 120 FPS budget summary. It only
mounts in `DEV` builds.

### WSL2 Kubuntu 24.04 (real backend)

Build and run the real backend (see `docs/setup/wsl2.md` for the one-time
package installs):

```bash
cd /mnt/c/Users/USER/Desktop/Zyntrix/Zyntrix\ OS/Zettings-app
pnpm -F zettings-web build
cargo build --release --manifest-path apps/zettings/Cargo.toml --target x86_64-unknown-linux-gnu
RUST_LOG=zettings=trace ./target/x86_64-unknown-linux-gnu/release/zettings
```

The backend logs the launch-time budget directly:

```text
TRACE zettings: event="main.enter" zettings backend starting
TRACE zettings: event="setup.complete" backend_startup_ms=NNN zettings backend ready
```

`backend_startup_ms` is the Rust-process portion of cold start
(`PROCESS_START` to `SETUP_DONE`). Add the frontend `scriptStart -> firstPaint`
window from DevTools to get the full cold-start number; the target is `<500ms`
combined on the Kubuntu 24.04 target hardware.

### Memory footprint (WSL2)

```bash
# Peak + steady-state RSS of the running backend
pid=$(pgrep -f 'release/zettings')
cat /proc/$pid/status | grep -E 'VmRSS|VmHWM'
ps -o pid,rss,vsz,args -p $pid

# Aggregate view across the app + webview helper processes
smem -tk -P zettings
```

Compare `VmRSS` against the `<150MB` idle budget. The `PerfMonitor` overlay
shows the same `memory_rss_bytes` value live on Linux builds.

### 120 FPS rendering audit

1. Run `pnpm dev`, open DevTools (F12), and switch to the **Performance**
   panel.
2. Record a 5-10s trace while driving the app: scroll settings pages, drag
   sliders, toggle switches, watch the VU meter and battery gauge animate.
3. In the trace, filter the **Rendering** layer and confirm:
   - no **Layout Shift** / forced reflow markers appear during animation,
   - the animation frames land in the **Compositing** layer (green),
   - no continuous `Recalculate Style`/`Layout` blocks on the main thread.
4. Confirm the `PerfMonitor` overlay shows `withinBudget: yes` (p95 frame
   time under `8.33ms`) with zero dropped frames.

Phase 9 rendering fixes that eliminate layout-thrash:

- Progress fills (battery, EQ, VU meter) animate `transform: scaleX/scaleY`
  with a fixed `transform-origin` instead of animating `width`/`height`
  (which forces a layout pass per frame).
- The VU meter bar/peak animate `scaleY`/`translateY` on the compositor
  thread with `will-change: transform`.
- Signal-strength bars scope their transition to the paint-only `background`
  property instead of `all`.
- The shimmer effect runs on a pseudo-element `transform` animation instead
  of background-position, and is disabled under `prefers-reduced-motion`.

## Known tooling gaps on the Windows host

- `cargo deny` is not installed on this host, so the `ci:check` step
  `cargo deny check` cannot run here. Install it with
  `cargo install cargo-deny` (or run on the WSL2 toolchain) before a release.
- The `pnpm lint` gate (ESLint 9) previously had no flat config; Phase 9
  added `eslint.config.js` (`typescript-eslint` + `eslint-config-prettier`)
  so the gate now passes.

## Acceptance checklist

- [ ] Cold start `<500ms` (combined backend + frontend) on Kubuntu 24.04
- [ ] Hot start `<150ms` from the dev overlay
- [ ] Idle RSS `<150MB` from `zettings_perf_stats` / `smem`
- [ ] 120 FPS budget met: p95 frame time `<8.33ms`, no dropped frames in a
      DevTools trace, no forced reflows during animation