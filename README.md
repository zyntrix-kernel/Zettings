# ZETTINGS

The settings application for **Zyntrix OS** — Kubuntu 24.04 LTS / KDE Plasma 5.27.

Native **Qt 6 (QML)** UI bridged from **Rust** via [cxx-qt](https://github.com/KDAB/cxx-qt). Backend logic is pure Rust (tokio, zbus, D-Bus/PolicyKit/systemd integration). No webview. No Node toolchain.

> This repository was restarted clean-slate on the native stack. The previous Tauri v2
> implementation is fully preserved at `archive/tauri-v2` (branch + tag) as reference
> material only — never a merge target.

## Product goals

* Exceed Windows 11 Settings, macOS System Settings, and GNOME/KDE/COSMIC Settings in UI, UX, motion, architecture, performance, integration, accessibility, and maintainability.
* Original **Zyntrix Design Language (ZDL)**: G2/G3 curvature, Liquid Glass materials, physics-based (spring/momentum) motion, GPU-accelerated, 120 FPS target.
* Windows-inspired information architecture with a data-driven settings registry, first-class search, and stable deep links (`zyntrix-settings:`).
* Honest system integration: real D-Bus backends; unavailable hardware reported honestly, never faked.

## Repository layout

```text
AGENTS.md            Operating directives for any agent working here (read first)
PLAN.md              Master phase-gated implementation roadmap
DESIGN.md            Zyntrix Design Language specification (authoritative for UI)
prompt.txt           Permanent behavioral charter (binding)
ZETTINGS_Windows_11_Settings_Deep_UI_Spec.md   Windows 11 settings reference analysis
docs/
  research/          Stack-agnostic research: capability matrix, threat model,
                     performance baseline, Windows→Zyntrix mapping, prompt compliance
  setup/wsl2.md      WSL2 Kubuntu 24.04 development environment setup
packaging/           Polkit policy, systemd unit, desktop entry templates
crates/              Rust crates (zettings-* prefix) — populated in PLAN Phase 1+
apps/zettings        Application shell — populated in PLAN Phase 4+
```

## Development environment

Primary development and verification environment: **WSL2 Kubuntu 24.04** targeting
`x86_64-unknown-linux-gnu`. See [`docs/setup/wsl2.md`](docs/setup/wsl2.md) for the full,
manual package installation procedure (Qt 6 dev stack, Rust 1.97 via `rust-toolchain.toml`,
integration daemons, polkit sandbox rule).

The Windows host is documentation/planning only — the product targets Linux exclusively.

## Verification gates

All gates run in WSL2:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo check --workspace
qmllint <changed .qml files>          # once QML sources exist
qmlformat --check <changed .qml files>
```

## Governance

| Document | Role |
| --- | --- |
| `prompt.txt` | Permanent behavioral charter — binding at every session |
| `AGENTS.md` | Skill gate, conventions, verification rules |
| `PLAN.md` | Phase-gated roadmap; no phase skipping |
| `DESIGN.md` | ZDL tokens, geometry, materials, motion — authoritative |

## License

Dual-licensed under [MIT](LICENSE-MIT) OR [Apache-2.0](LICENSE-APACHE) at your option.
