# AGENTS.md — Zettings Repository Operating Directives

This file is the single source of truth for any agent (human or AI) operating in this repo. Read it before changing anything. The instructions here override personal preference.

## Repository Context

* **Working Path:** `C:\Users\USER\Desktop\Zyntrix\Zyntrix OS\Zettings-app`
* **Project Name:** **Zettings** — system settings application for **Zyntrix OS** (Kubuntu 24.04 / KDE Plasma 5.27).
* **Tech Stack:** Tauri v2 (Rust 1.97), React 19 + TypeScript (pnpm 11), Tailwind v4.
* **License:** Dual-licensed under MIT OR Apache-2.0 at the user's option.

---

# 0. Permanent Behavioral Charter — `prompt.txt` (BINDING)

The repository root file **`prompt.txt`** is the permanent project charter. It is loaded into memory at every session start and its directives are binding for all work in this repository. Summary of its binding rules (the full text in `prompt.txt` remains authoritative):

* **Role:** Operate as an elite OS-level software engineering organization — Windows Settings engineering rigor, Apple HIG design taste, KDE integration depth, Tauri core-team fluency, Rust foundation-grade safety, security-engineer discipline.
* **Product:** Build **ZETTINGS**, the settings application for **Zyntrix OS** (Kubuntu + KDE Plasma). Not a clone — the next-generation Linux settings app; benchmark: exceed Windows 11 Settings, macOS System Settings, GNOME/KDE/COSMIC/elementary Settings.
* **Design language:** Original **Zyntrix Design Language (ZDL)** — inspired by Apple/Windows 11/Nothing OS/Arc/Linear/Raycast/Material 3/Fluent, never imitating them. Everything alive, physics-based motion (G2/G3 continuity, springs, momentum), GPU-accelerated, 120 FPS target.
* **Coding rules:** Never use placeholders. Never write pseudo-code. Never simplify. Production-ready code only. Every file compiles; every dependency justified; every module tested.
* **Quality gate:** Continuously ask "Would Apple ship this? Would Microsoft approve this? Would KDE merge this? Would Rust engineers accept this?" If no → rewrite until yes.
* **Delivery model:** Phase-gated professional roadmap (research → architecture → design system → motion engine → core framework → backend → frontend → features → testing → optimization → packaging → docs → release). Each phase completes before the next begins.

This charter operates **within** the rules below (single-agent model, skill gate, verification gates). Where prompt.txt's persona framing could be read as multi-agent delegation, AGENTS.md §1 prevails: one agent embodying all of those disciplines.

---

# 1. Single-Agent Operating Model

There are **no sub-agents** in this repository.

The active agent is solely responsible for:

* analysis
* planning
* skill selection
* skill loading
* implementation
* verification
* debugging
* review
* completion

Do not delegate work to sub-agents.

Do not create, invoke, or rely on `.opencode/agents/*.md` roles for execution.

Do not split work into simulated specialist agents or ask another model/agent to perform a portion of the task.

Treat installed skills as **knowledge/procedure modules available to the same agent**, not as separate workers.

---

# 2. HARD SKILL GATE — MANDATORY BEFORE ANY WORK

**NO REPOSITORY WORK MAY BEGIN UNTIL ALL NECESSARY SKILLS HAVE BEEN IDENTIFIED AND LOADED.**

This is a **hard prerequisite**, not a recommendation.

The agent MUST NOT:

* inspect source files for implementation purposes
* modify files
* create files
* delete files
* rename files
* install dependencies
* run build commands
* run tests
* run formatters
* run linters
* run typecheckers
* run scripts
* execute project tooling
* design an implementation
* begin coding
* perform debugging
* perform refactoring
* make architectural decisions
* claim a task is ready for implementation

until the applicable skills have been identified and loaded.

## 2.1 Required Pre-Work Sequence

For **every task**, the agent MUST follow this sequence:

### Step 1 — Understand the request

Determine exactly what the user is asking to change or investigate.

### Step 2 — Classify the task

Identify every technical/domain concern involved.

Examples:

* React UI
* TypeScript
* Tailwind/CSS
* ZDL visual design
* G2/G3 geometry
* Liquid Glass
* animation/motion
* accessibility
* frontend architecture
* Tauri
* Rust
* IPC
* Linux/KDE integration
* system settings
* security
* permissions/Polkit
* audio
* networking
* testing
* performance
* packaging
* documentation

### Step 3 — Discover applicable skills

Inspect the available skills and determine which skill(s) govern the identified concerns.

### Step 4 — Check for overlap

Before loading a skill, check whether another installed skill already covers the same responsibility.

Follow the Skill Selection Rules in Section 3.

### Step 5 — Load the necessary skills

Read the complete applicable `SKILL.md` instructions before performing work governed by that skill.

### Step 6 — Confirm skill coverage

The agent must be able to answer:

> "Which loaded skill governs each technical concern of this task?"

If any concern has no applicable loaded skill, **STOP**.

Do not continue implementation based on generic model knowledge.

### Step 7 — Only now inspect and modify the repository

After the required skills have been loaded, the agent may inspect project files, documentation, architecture, and implementation.

---

# 3. Skill System & Non-Overlap Policy

Skills extend the single agent's knowledge and procedures. They **must not duplicate, override, or silently conflict with one another**.

## 3.1 Skill Selection Rules

1. **Before using a skill, identify its exact responsibility and read its `SKILL.md`.**
2. **Use the smallest complete set of skills necessary for the task.**
3. **Never load a skill merely because it is installed.**
4. **Never perform work requiring a skill before that skill is loaded.**
5. Prefer one authoritative skill per concern.
6. If two skills cover the same concern, select the more specific/current one and do not combine conflicting instructions.
7. General skills provide broad guidance.
8. Domain-specific skills provide deeper guidance for their explicit domain.
9. The domain-specific skill wins only within its explicit scope.
10. Project-local directives in `AGENTS.md`, `DESIGN.md`, architecture docs, and existing implementation conventions take precedence over generic skills.
11. Skills must not redefine project architecture, naming, security policy, design tokens, verification gates, or other repository rules unless explicitly authorized by repository documentation.
12. Skills are **reference procedures**, not autonomous authorities. The active agent remains responsible for deciding whether a skill applies.

## 3.2 Skill Coverage Requirement

For each task, create an internal mapping:

```text
Task concern → Authoritative skill → Loaded? → Applicable?
```

Every applicable concern must have an authoritative loaded skill.

Example:

```text
React UI        → frontend skill       → YES → required
ZDL geometry    → ZDL/UI skill         → YES → required
Tauri IPC       → Tauri skill          → YES → required
Rust backend    → Rust skill           → YES → required
Testing         → testing skill        → YES → required
```

If a required skill is unavailable:

**STOP BEFORE IMPLEMENTATION.**

Report which skill is missing and why it is required.

Do not substitute generic model knowledge for a missing mandatory skill.

---

# 4. Skill Boundary Contract

Every installed skill should have one clearly defined primary responsibility.

Skills must not silently expand into other domains.

Examples of boundaries:

* UI/UX strategy ≠ Liquid Glass/material implementation
* Motion/animation ≠ general visual design
* Accessibility ≠ visual styling
* Frontend architecture ≠ Tauri/Rust architecture
* Security ≠ generic code quality
* Performance profiling ≠ functional implementation
* Testing/QA ≠ feature implementation

If overlap is unavoidable, the skill must explicitly state its boundary and defer to:

1. project-level rules;
2. the more specialized skill; or
3. existing project architecture.

---

# 5. Conflict Resolution

When skill instructions disagree, resolve them in this order:

1. Repository safety/security requirements and explicit user requirements.
2. This `AGENTS.md`.
3. Other project-specific documentation such as `DESIGN.md` and architecture documentation.
4. Existing architecture and established implementation patterns.
5. The most specific applicable loaded skill.
6. Broader/general loaded skills.
7. Generic model assumptions.

Never merge contradictory instructions into an improvised hybrid.

When instructions conflict and the authority cannot be determined:

**STOP and resolve the conflict before implementation.**

---

# 6. Skill Duplication Check

When adding or reinstalling a skill:

1. Search `.opencode/skills/` for existing skills with the same or overlapping responsibility.
2. Inspect their descriptions.
3. Read their `SKILL.md` scope.
4. Determine whether the new skill materially duplicates an existing skill.
5. Do not install a second skill solely because it has a different name.
6. If two skills are retained because they genuinely cover different layers, document their boundary.
7. Keep skills composable.

A task should be understandable as:

```text
Task
 ↓
Identify concerns
 ↓
Select authoritative skills
 ↓
Load skills
 ↓
Inspect repository
 ↓
Plan
 ↓
Implement
 ↓
Verify
```

Never:

```text
Task
 ↓
Start coding
 ↓
Maybe load skills later
```

---

# 7. Mandatory Stop Conditions

The agent MUST stop and ask for clarification or report the blocker if:

* a necessary skill cannot be found;
* a necessary skill cannot be loaded;
* two applicable skills conflict and authority cannot be determined;
* project documentation conflicts with the requested change and the conflict cannot be resolved;
* the requested implementation requires a domain outside the loaded skill coverage;
* the agent would otherwise have to rely on generic assumptions for a domain that should be governed by a project skill.

**Do not "just proceed" when a required skill is missing.**

---

# 8. Critical Conventions

* **Crate Naming:** All crates use `zettings-` prefix (with `t`). Rust import names: `zettings_polkit`, `zettings_core`, etc.
* **Tauri Commands:** Keep `#[tauri::command]` off lib functions; apply it in `main.rs` when registering with `generate_handler!`.
* **ts-rs v12:** Use `#[ts(export, export_to = "<filename>.ts")]` with just the filename. The export test computes the absolute workspace path from `CARGO_MANIFEST_DIR`.
* **Frontend Imports:** Import IPC payload types from `@zettings/bindings` (the workspace package). Never hand-type duplicate frontend payloads.
* **TypeScript Strict Mode:** `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`, `noUnusedLocals`, `noUnusedParameters`, `verbatimModuleSyntax` are all ON. Use `import type` for type-only imports.
* **CSS:** Standard `border-radius` is forbidden for major cards/panels — use G2/G3 squircle clip-paths instead.
* **Clippy:** `pedantic` is deny-by-default. Selective allows: `module_name_repetitions`, `must_use_candidate`, `missing_errors_doc`, `missing_panics_doc`, `missing_fields_in_debug`, `multiple_crate_versions`.

---

# 9. Design Authority

The repository's `DESIGN.md` / Zyntrix Design Language (ZDL) specification is authoritative for project-specific visual behavior, including:

* G2/G3 curvature
* Liquid Glass material composition
* token hierarchy
* theme variants
* motion parameters

Do not replace ZDL with generic glassmorphism or generic design-system defaults.

A Liquid Glass skill may provide implementation techniques, but ZDL project tokens and rules remain authoritative.

UI/UX skills may advise on hierarchy and interaction design, but must not silently change ZDL geometry, tokens, or motion physics.

**If a UI task requires ZDL knowledge, the applicable ZDL/design skill MUST be loaded before UI work begins.**

---

# 10. Repository Inspection Rule

After the Skill Gate has passed, inspect existing project documentation and implementation before introducing new abstractions.

Reuse:

* existing components
* tokens
* bindings
* utilities
* crates
* IPC contracts
* architecture patterns

Do not duplicate:

* types
* tokens
* IPC payloads
* architectural responsibilities

Keep changes scoped to the requested task unless a necessary dependency or correctness issue requires adjacent changes.

---

# 11. Verification Gate

Run the applicable verification gates before considering work complete.

For phase/commit completion, all four gates are mandatory:

```cmd
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo check --workspace
pnpm -r typecheck
```

For changes that affect:

* tests
* security
* packaging
* performance
* runtime integration

also run the relevant project-specific checks documented in `PLAN.md` and affected module documentation.

Never claim that a check passed unless it was actually run and observed to pass.

If a check cannot be run in the current environment:

1. state that explicitly;
2. do not claim success;
3. provide the exact command required for verification.

---

# 12. Build Runtimes

## Windows Host

Used for frontend iteration using the `zettings-mock` feature.

Parent paths MUST NOT contain single quotes (`'`).

## WSL2 Kubuntu 24.04 LTS

Used for real backend integration targeting:

```text
x86_64-unknown-linux-gnu
```

with:

* system `zbus`
* PipeWire
* PulseAudio

Document package installation commands in:

```text
docs/setup/wsl2.md
```

**NEVER run `apt` automatically.**

---

# 13. No Unskilled Work Principle

The following principle is absolute:

> **If the task requires a skill, that skill must be loaded before the agent performs the task.**

The agent must never use its generic model knowledge as a substitute for an applicable repository skill.

Generic reasoning may be used to coordinate already-loaded skills, interpret user requirements, resolve implementation details, and make decisions within the boundaries established by the loaded skills and repository documentation.

However:

**No applicable skill loaded → no corresponding work performed.**

---

# 14. Completion Requirements

A task is complete only when:

1. The necessary skills were identified.
2. The necessary skills were loaded before implementation.
3. Skill boundaries were respected.
4. Repository architecture and documentation were followed.
5. The requested implementation was completed.
6. Applicable verification gates were run.
7. Failures were resolved or explicitly reported.
8. No verification result was fabricated.
9. No unnecessary architectural changes were introduced.

---

# 15. Final Agent Checklist

Before starting:

```text
[ ] Read AGENTS.md
[ ] Understand the requested task
[ ] Identify every technical concern
[ ] Identify applicable skills
[ ] Check skill overlap
[ ] Load every necessary skill
[ ] Confirm every concern has skill coverage
```

Before implementation:

```text
[ ] Inspect DESIGN.md
[ ] Inspect relevant architecture/project documentation
[ ] Inspect existing implementation
[ ] Confirm existing patterns to reuse
[ ] Confirm no skill conflict exists
```

Before completion:

```text
[ ] Implementation complete
[ ] cargo fmt --all --check
[ ] cargo clippy --workspace --all-targets -- -D warnings
[ ] cargo check --workspace
[ ] pnpm -r typecheck
[ ] Additional task-specific checks completed
[ ] No unverified claims
```

## Absolute Rule

**NEVER START REPOSITORY WORK WITHOUT FIRST LOADING THE NECESSARY SKILLS.**

**WHEN A REQUIRED SKILL IS MISSING OR CANNOT BE LOADED, STOP.**

# MISSING REQUIRED SKILL — HARD STOP

If a task requires a skill and no applicable installed skill exists:

1. STOP the portion of work governed by that skill.
2. Search the available skills once more to confirm that no applicable skill exists.
3. Do NOT substitute PLAN.md, DESIGN.md, AGENTS.md, generic model knowledge, or another unrelated skill for the missing skill.
4. Do NOT implement the affected feature.
5. Report the exact missing skill/domain.
6. Request installation or loading of an appropriate skill.
7. Resume the affected work only after the required skill has been successfully loaded.

Example:

Task requires:
    Motion / animation

Available:
    Liquid Glass skill
    UI/UX skill
    No dedicated motion skill

Correct behavior:
    STOP motion implementation.

Incorrect behavior:
    "Liquid Glass + PLAN.md are sufficient, so I will implement motion anyway."

A project document is NOT a substitute for a required skill.

A related skill is NOT automatically a substitute for a missing specialized skill.

The agent may only continue with independent portions of the task when those portions have their own required skill coverage.