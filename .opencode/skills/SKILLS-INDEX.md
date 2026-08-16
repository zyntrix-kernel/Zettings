# Skills Index — Accessibility Manifest for Subagents

> **Purpose:** This file is the single entry point for any DSH `subagent` or
> OpenCode agent to discover and load skills from `.opencode/skills/`. DSH
> subagents do NOT share the orchestrator's conversation context, but they
> CAN read files in the workspace. Read this file first, then `read` the
> `SKILL.md` and referenced files you need directly from disk using the
> absolute paths below.

**Repo root:** `C:\Users\USER\Desktop\Zyntrix\Zyntrix OS\Zettings-app`
**Skills root:** `C:\Users\USER\Desktop\Zyntrix\Zyntrix OS\Zettings-app\.opencode\skills\`

## How to Use a Skill

1. Read this index to find the skill matching your task.
2. `read` the skill's `SKILL.md` at the absolute path listed for its overview.
3. `read` the specific `references/*.md` files listed for detailed knowledge.
4. Scripts (when present) are optional helpers in `scripts/`; treat their
   output as advisory — do not execute them unless explicitly asked.

## Available Skills

### 1. `design-system` — Token architecture & component specs
**Most relevant for Zettings Phase 2 (ZDL token cascade) and Tailwind v4 `@theme`.**
- **SKILL.md:** `.opencode\skills\design-system\SKILL.md`
- **Description:** Three-layer token cascade (primitive → semantic → component),
  CSS variable systems, spacing/typography scales, component specs, Tailwind
  theme configuration, design-to-code handoff.
- **References (read for ZDL work):**
  - `references\token-architecture.md` — why 3 layers, layer responsibilities
  - `references\primitive-tokens.md` — color scales, spacing, typography, radius, shadow, motion
  - `references\semantic-tokens.md` — color/spacing/typography semantics, dark mode overrides
  - `references\component-tokens.md` — button/input/card/badge/alert/dialog/table tokens
  - `references\component-specs.md` — per-component specs
  - `references\states-and-variants.md` — hover/focus/disabled/loading/error states
  - `references\tailwind-integration.md` — CSS var setup, tailwind.config.ts, dark mode toggle

### 2. `ui-ux-pro-max` — Searchable UI/UX design intelligence
**Use for picking styles, colors, typography, UX patterns.**
- **SKILL.md:** `.opencode\skills\ui-ux-pro-max\SKILL.md`
- **Description:** 67 styles, 161 color palettes, 57 font pairings, 99 UX
  guidelines, 25 chart types across 22 tech stacks (React included). Has
  Python-based search script (`scripts\search.py`) — optional and read-only.
- **Relevant data:** `data\react.csv`, `data\styles.csv`, `data\colors.csv`,
  `data\typography.csv`, `data\motion.csv`, `data\ux-guidelines.csv`,
  `data\ui-reasoning.csv`, `data\stacks\react.csv`, `data\stacks\shadcn.csv`.
- **References:** None (blank); structured guidance lives in the CSV data.

### 3. `ui-styling` — shadcn/ui + Tailwind theming reference
**Use for shadcn component patterns, accessibility, responsive utilities.**
- **SKILL.md:** `.opencode\skills\ui-styling\SKILL.md`
- **Description:** Tailwind customization, shadcn/ui theming, accessibility.
- **References:**
  - `references\canvas-design-system.md`
  - `references\shadcn-accessibility.md`
  - `references\shadcn-components.md`
  - `references\shadcn-theming.md`
  - `references\tailwind-customization.md`
  - `references\tailwind-responsive.md`
  - `references\tailwind-utilities.md`

### 4. `brand` — Brand identity & visual system
**Optional. Use if Zyntrix brand guidelines need enforcing.**
- **SKILL.md:** `.opencode\skills\brand\SKILL.md`
- **References:** color-palette-management, consistency-checklist,
  logo-usage-rules, messaging-framework, typography-specifications,
  visual-identity, voice-framework (all under `references\`).

### 5. `design` — Logo / icon / slide generation
**Optional creative skill — generally not needed for Zettings system UI.**
- **SKILL.md:** `.opencode\skills\design\SKILL.md`
- **References:** logo-design, icon-design, cip-*, slide-* (under `references\`).

### 6. `design-system` overlap: `slides` / `banner-design`
- `slides` (`slides\SKILL.md`) — pure presentation-authoring skill.
- `banner-design` (`banner-design\SKILL.md`) — banner sizing reference.
- **Neither is needed for the Zettings app** (we are not building slides).

## Quick-Reference for Zettings Phase 2 (ZDL Shell)

For Phase 2 the **only skills that matter** are `design-system` and
`ui-ux-pro-max` (for the React stack data). Load these two `SKILL.md` files
plus the token + Tailwind references they list, before implementing any
component. The token cascade, squircle engine, glass material, and Tailwind v4
`@theme` integration are all already in place at
`apps\zettings\web\src\styles\zdl.css` and `apps\zettings\web\src\lib\zdl-motion.ts`
— read those existing files first to match conventions before adding anything.
