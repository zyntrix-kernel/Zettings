# Zettings

> Next-generation system settings application for **Zyntrix OS** (Kubuntu 24.04 LTS / KDE Plasma).
> Built with Tauri v2 (Rust backend) and React 19 (TypeScript frontend), integrated with KDE and
> Linux system services via DBus (`zbus`), PulseAudio, PipeWire, NetworkManager, BlueZ, UPower,
> AccountsService, KScreen, KWin, systemd, PackageKit, logind and PolicyKit.

Dual-licensed under **MIT OR Apache-2.0** at the user's option.

## Status

Phase 1 — Architecture scaffold (this commit):

- Cargo + pnpm workspace
- 7 Rust crates (`zettings-core`, `zettings-bus`, `zettings-ipc`,
  `zettings-polkit`, `zettings-plugin-sdk`, `zettings-palette`,
  `zettings-search`) and the Tauri v2 shell (`apps/zettings`)
- React 19 + Tailwind v4 webview with a typed IPC contract pipeline
  (`ts-rs` => `packages/ts-bindings`)
- Mermaid architecture diagrams under `docs/architecture/`
- Verify gates: `cargo fmt --check`, `cargo clippy -D warnings`,
  `cargo check --workspace`, `pnpm -r typecheck`

See the roadmap in `AGENTS.md`. Future phases land Display, Audio,
Bluetooth, Network and the remaining 50+ panels as loadable plugins.

## Quick start

| Task | Command |
|---|---|
| Install JS deps | `pnpm i` |
| Frontend dev (Windows: mock backend) | `pnpm dev` |
| Real backend dev (WSL2 Kubuntu) | `pnpm dev:linux` |
| Typecheck | `pnpm -r typecheck` |
| Lint | `pnpm -r lint` |
| Rust format + lint | `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings` |
| Full CI dry-run | `pnpm ci:check` |

## Build environments

- **Windows host** — frontend development against a `zettings-mock` feature
  that supplies a state-machine mock backend so the webview can iterate
  without Linux services.
- **WSL2 Kubuntu 24.04 LTS** — real backend integration. Build with
  `cargo check --workspace --target x86_64-unknown-linux-gnu` to confirm
  `zbus` / `pipewire` / `libpulse-binding` linking.

See `docs/setup/wsl2.md` for the packages the Linux host must install.
The Zettings project never runs `apt`, `winget`, or `brew` for you; the
docs spell out every install command.

## Architecture

Diagrams: `docs/architecture/` (Mermaid)

- `crate-graph.mmd` — Rust workspace DAG
- `ipc-sequence.mmd` — end-to-end elevated command flow, polkit path included
- `plugin-lifecycle.mmd` — module load + signature + capability state machine
- `search-dataflow.mmd` — tantivy indexing + ranking pipeline
- `theme-cascade.mmd` — primitive -> semantic -> component token cascade with wallpaper accent

## License

Dual MIT OR Apache-2.0. Contributions must be granted under the same terms.
