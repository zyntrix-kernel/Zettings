# AGENTS.md — Zettings Repository Operating Directives

This file is the single source of truth for any agent (human or AI) operating in this repo. Read it before changing anything. The instructions here override personal preference.

## Repository Context
- **Working Path:** `C:\Users\USER\Desktop\Zyntrix\Zyntrix OS\Zettings-app`
- **Project Name:** **Zettings** — system settings application for **Zyntrix OS** (Kubuntu 24.04 LTS / KDE Plasma 5.27).
- **Tech Stack:** Tauri v2 (Rust 1.97), React 19 + TypeScript (pnpm 11), Tailwind v4.
- **License:** Dual-licensed under MIT OR Apache-2.0 at the user's option.

## OpenCode Sub-Agent Team

Agent definitions live in `.opencode/agents/*.md`. The Lead Architect (`@zyn-architect`) delegates to these sub-agents:

### Core Infrastructure Agents
| Agent | Role | When to Delegate |
|-------|------|-----------------|
| `@rust-core` | Senior Systems Engineer | Cargo workspace crates, Tauri v2 commands, `zbus` DBus integration, `zettings-polkit` rules, ts-rs payload export |
| `@motion-ui` | Creative UI Engineer | React 19 webview code, ZDL tokens, G2/G3 squircle curvature, liquid glass, spring physics, Tailwind v4 `@theme` |
| `@search-core` | Search & Navigation Engineer | Tantivy indexing, `strsim` Levenshtein fuzzy matching, sub-5ms route mapping, deep-link targets |
| `@qa-auditor` | Performance & Security Auditor | Read-only verification gates, polkit security audits, 120Hz render profiling, license compliance |

### Domain Module Agents (Phase 5+)
| Agent | Role | Domain |
|-------|------|--------|
| `@zettings-display` | Display Module Engineer | KScreen integration, monitor canvas, night color |
| `@zettings-audio` | Audio Module Engineer | PipeWire/PulseAudio, sound mixer, equalizer |
| `@zettings-network` | Network & Bluetooth Engineer | NetworkManager/BlueZ, Wi-Fi scanning, device pairing |
| `@zettings-power` | Power Module Engineer | UPower/power-profiles-daemon, battery health, performance profiles |
| `@zettings-personalization` | Personalization Engineer | ZDL theme panel, wallpaper accent extraction, squircle/blur controls |

## Delegation Protocol
1. `@zyn-architect` analyzes requests and forms an execution plan.
2. Tasks are delegated to the appropriate sub-agent(s) via the `task` tool with `subagent_type` matching the agent name.
3. Multiple independent tasks are delegated in parallel for speed.
4. All sub-agents must run verification gates before reporting completion.

## Critical Conventions
- **Crate Naming:** All crates use `zettings-` prefix (with `t`). Rust import names: `zettings_polkit`, `zettings_core`, etc.
- **Tauri Commands:** Keep `#[tauri::command]` off lib functions; apply it in `main.rs` when registering with `generate_handler!`.
- **ts-rs v12:** Use `#[ts(export, export_to = "<filename>.ts")]` with just the filename. The export test computes the absolute workspace path from `CARGO_MANIFEST_DIR`.
- **Frontend Imports:** Import IPC payload types from `@zettings/bindings` (the workspace package). Never hand-type duplicate frontend payloads.
- **TypeScript Strict Mode:** `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`, `noUnusedLocals`, `noUnusedParameters`, `verbatimModuleSyntax` are all ON. Use `import type` for type-only imports.
- **CSS:** Standard `border-radius` is forbidden for major cards/panels — use G2/G3 squircle clip-paths instead.
- **Clippy:** `pedantic` is deny-by-default. Selective allows: `module_name_repetitions`, `must_use_candidate`, `missing_errors_doc`, `missing_panics_doc`, `missing_fields_in_debug`, `multiple_crate_versions`.

## Verification Gate (Must pass before every phase commit)
```cmd
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo check --workspace
pnpm -r typecheck
```

## Build Runtimes
1. **Windows Host:** Frontend iteration using `zettings-mock` feature. Parent paths MUST NOT contain single quotes (`'`).
2. **WSL2 Kubuntu 24.04 LTS:** Real backend integration targeting `x86_64-unknown-linux-gnu` with system `zbus`, PipeWire, and PulseAudio. Document package install commands in `docs/setup/wsl2.md`. NEVER run `apt` automatically.
