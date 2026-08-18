# AGENTS.md — Zettings Repository Operating Directives

This file is the single source of truth for any agent (human or AI) operating in this repo. Read it before changing anything. The instructions here override personal preference.

## Repository Context
- **Working Path:** `C:\Users\USER\Desktop\Zyntrix\Zyntrix OS\Zettings-app`
- **Project Name:** **Zettings** — system settings application for **Zyntrix OS** (Kubuntu 24.04 / KDE Plasma 5.27).
- **Tech Stack:** Tauri v2 (Rust 1.97), React 19 + TypeScript (pnpm 11), Tailwind v4.
- **License:** Dual-licensed under MIT OR Apache-2.0 at the user's option.

## Single-Agent Operating Model
There are **no sub-agents** in this repository. The active agent is solely responsible for analysis, planning, implementation, verification, debugging, review, and completion.

- Do not delegate work to sub-agents.
- Do not create, invoke, or rely on `.opencode/agents/*.md` roles for execution.
- Do not split work into simulated specialist agents or ask another model/agent to perform a portion of the task.
- Treat all project skills as advisory specialist knowledge available to the same agent, not as separate workers.
- When a task spans multiple disciplines, the same agent must coordinate those concerns and resolve conflicts explicitly.

## Skill System & Non-Overlap Policy
Skills extend the single agent's knowledge and procedures. They **must not duplicate, override, or silently conflict with one another**.

### Skill Selection Rules
1. Before using a skill, identify its exact responsibility and read its `SKILL.md`.
2. Use the **smallest set of skills necessary** for the task. Do not load skills merely because they are installed.
3. Prefer one authoritative skill per concern. If two skills cover the same concern, select the more specific/current one and do not combine conflicting instructions.
4. General skills provide broad guidance; domain-specific skills provide deeper guidance for their domain. The domain-specific skill wins only within its explicit scope.
5. Project-local directives in this `AGENTS.md`, `DESIGN.md`, architecture docs, and existing implementation conventions take precedence over generic skills.
6. A skill must not redefine project architecture, naming, security policy, design tokens, verification gates, or other repository rules unless explicitly authorized by repository documentation.
7. Skills must be treated as **reference procedures**, not autonomous authorities. The agent remains responsible for deciding whether a skill applies.

### Skill Boundary Contract
Every installed skill should have one clearly defined primary responsibility. Its `SKILL.md` should avoid instructions belonging to another skill, especially:

- UI/UX strategy vs. Liquid Glass/material implementation
- Motion/animation vs. general visual design
- Accessibility vs. visual styling
- Frontend architecture vs. Tauri/Rust architecture
- Security vs. generic code quality
- Performance profiling vs. functional implementation
- Testing/QA vs. feature implementation

If overlap is unavoidable, the skill must explicitly state the boundary and defer to the project-level rules or the more specialized skill rather than redefining them.

### Conflict Resolution
When skill instructions disagree, resolve them in this order:
1. Repository safety/security requirements and explicit user requirements.
2. This `AGENTS.md` and other project-specific documentation.
3. Existing architecture and established implementation patterns in the codebase.
4. The most specific applicable skill.
5. Broader/general skills.
6. Generic model assumptions.

Never merge contradictory instructions into an improvised hybrid without first determining which rule has authority.

### Skill Duplication Check
When adding or reinstalling a skill:
- Search `.opencode/skills/` for existing skills with the same or overlapping responsibility.
- Inspect their descriptions and `SKILL.md` scope before installation.
- Do not install a second skill solely because it has a different name if it materially duplicates an existing skill.
- If two skills are retained because they cover genuinely different layers, document the boundary in their `SKILL.md` or in project documentation.
- Keep skills composable: a task should be understandable as a sequence of clearly bounded responsibilities, not a pile of competing instructions.

## Critical Conventions
- **Crate Naming:** All crates use `zettings-` prefix (with `t`). Rust import names: `zettings_polkit`, `zettings_core`, etc.
- **Tauri Commands:** Keep `#[tauri::command]` off lib functions; apply it in `main.rs` when registering with `generate_handler!`.
- **ts-rs v12:** Use `#[ts(export, export_to = "<filename>.ts")]` with just the filename. The export test computes the absolute workspace path from `CARGO_MANIFEST_DIR`.
- **Frontend Imports:** Import IPC payload types from `@zettings/bindings` (the workspace package). Never hand-type duplicate frontend payloads.
- **TypeScript Strict Mode:** `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`, `noUnusedLocals`, `noUnusedParameters`, `verbatimModuleSyntax` are all ON. Use `import type` for type-only imports.
- **CSS:** Standard `border-radius` is forbidden for major cards/panels — use G2/G3 squircle clip-paths instead.
- **Clippy:** `pedantic` is deny-by-default. Selective allows: `module_name_repetitions`, `must_use_candidate`, `missing_errors_doc`, `missing_panics_doc`, `missing_fields_in_debug`, `multiple_crate_versions`.

## Design Authority
The repository's `DESIGN.md` / Zyntrix Design Language (ZDL) specification is authoritative for project-specific visual behavior, including G2/G3 curvature, Liquid Glass material composition, token hierarchy, theme variants, and motion parameters.

- Do not replace ZDL with generic glassmorphism or generic design-system defaults.
- A Liquid Glass skill may provide implementation techniques, but ZDL project tokens and rules remain authoritative.
- UI/UX skills may advise on hierarchy and interaction design, but must not silently change ZDL geometry, tokens, or motion physics.

## Verification Gate
Run the applicable verification gates before considering work complete. For phase/commit completion, all four gates are mandatory:

```cmd
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo check --workspace
pnpm -r typecheck
```

For changes that affect tests, security, packaging, performance, or runtime integration, also run the relevant project-specific checks documented in `PLAN.md` and the affected module documentation.

## Build Runtimes
1. **Windows Host:** Frontend iteration using `zettings-mock` feature. Parent paths MUST NOT contain single quotes (`'`).
2. **WSL2 Kubuntu 24.04 LTS:** Real backend integration targeting `x86_64-unknown-linux-gnu` with system `zbus`, PipeWire, and PulseAudio. Document package install commands in `docs/setup/wsl2.md`. NEVER run `apt` automatically.

## Working Rules
- Inspect existing code and project documentation before introducing new abstractions.
- Reuse existing components, tokens, bindings, utilities, and crates when appropriate.
- Do not duplicate types, tokens, IPC payloads, or architectural responsibilities.
- Keep changes scoped to the requested task unless a necessary dependency or correctness issue requires adjacent changes.
- Never claim a build, test, benchmark, or verification passed unless it was actually run and observed to pass.
- When a check cannot be run in the current environment, state that explicitly and provide the exact command needed for verification.
- Preserve the repository's license and security constraints.
