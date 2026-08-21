# Zettings

> Next-generation system settings application for **Zyntrix OS** (Kubuntu 24.04
> LTS / KDE Plasma). Built with Tauri v2 (Rust backend) and React 19
> (TypeScript frontend), integrating Linux system services through typed,
> capability-honest adapters.

Dual-licensed under **MIT OR Apache-2.0** at the user's option.

## Status — generation-2 rebuild

This branch (`rebuild/v2`) is a clean-slate rebuild executed phase-by-phase per
`PLAN.md`:

| Phase | Scope | State |
|---|---|---|
| 0 | Research baseline (`docs/research/`) | ✅ |
| 1 | Workspace architecture, typed IPC pipeline, security seams | ✅ |
| 2 | ZDL design language (`DESIGN.md` + token cascade + primitives) | ✅ |
| 3 | Motion engine (physics tokens, reduced-motion policy, frame monitor) | ✅ |
| 4 | Core framework (responsive shell, real search, theme engine, routing) | ✅ |
| 5 | Backend tier 1: zbus foundation, PolicyKit, power/network/session | ✅ |
| 6 | Frontend: System page live on adapters; category hubs honest-empty | ✅ core |
| 7 | Feature modules: audio (PulseAudio), Bluetooth (BlueZ), display (DRM) | ✅ tier 2 |
| 8 | Testing: Rust suites, vitest component/state suites, axe structural scans | ✅ part 1–2 |
| 9–12 | Optimization, packaging, full docs set, release gate | ⏳ planned |

Honest environment note: real D-Bus/PulseAudio/BlueZ behavior and launch/frame
benchmarks are verified on WSL2 Kubuntu / target hardware; Windows-host runs
use deterministic mock adapters by design.

## Architecture

```text
React shell ──typed IPC──▶ Tauri commands ──▶ BackendSet adapters ──▶ Linux/KDE services
     ▲                                                        │
     └── generated bindings (@zettings/bindings) ◀── ts-rs ◀──┘
```

| Crate | Responsibility |
|---|---|
| `zettings-core` | Domain model: registry graph, routes, capability states, errors |
| `zettings-ipc` | Wire DTOs; `ts-rs` → committed TypeScript bindings |
| `zettings-bus` | Typed tokio broadcast events (setting/capability changes) |
| `zettings-polkit` | Fail-closed authorization gateway seam (+ mock) |
| `zettings-plugin-sdk` | ed25519-signed module manifests |
| `zettings-search` | Weighted ranking kernel (spec §9 weights) |
| `zettings-backends` | Adapters: power profiles, NetworkManager, login1, PulseAudio, BlueZ, DRM sysfs — real impls on Linux, state-machine mocks elsewhere |

Diagrams with structured text alternatives: `docs/architecture/`.

## Quick start

| Task | Command |
|---|---|
| Install JS deps | `pnpm i` |
| Frontend dev (browser; honest no-runtime state) | `pnpm dev` |
| Full desktop app (Windows host = mock adapters) | `cargo run --manifest-path apps/zettings/Cargo.toml` |
| Real backend dev (WSL2 Kubuntu) | see `docs/setup/wsl2.md`, then run without mocks |
| Regenerate TS bindings | `pnpm bindings` |
| Typecheck | `pnpm -r typecheck` |
| Web unit + a11y tests | `pnpm -F zettings-web test` |
| Rust tests | `cargo test --workspace` |
| Full CI dry-run | `pnpm ci:check && pnpm -F zettings-web test` |

## Verification gates

Every change passes:

```cmd
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo check --workspace
pnpm -r typecheck
```

CI (`.github/workflows/ci.yml`) additionally verifies binding determinism and
compiles/tests the Linux adapter path on Ubuntu 24.04 runners.

## Security model

Zero Trust, least privilege: the webview holds only `core:default`
capabilities; privileged mutations flow through the PolicyKit gateway;
adapters never spawn shells; secrets never cross IPC. See
`docs/research/threat-model.md`.

## License

Dual MIT OR Apache-2.0. Contributions must be granted under the same terms.
