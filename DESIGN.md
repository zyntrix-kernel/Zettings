# DESIGN.md — Zyntrix Design Language (ZDL)

> **Authority.** This document is the single source of truth for all visual,
> spatial, material, motion, and accessibility-affecting presentation in
> ZETTINGS. Project rules (`AGENTS.md`) and product requirements (`prompt.txt`)
> override it only where they explicitly conflict. Implementation lives in
> `apps/zettings/qml/org/zyntrix/zettings/Style/ZdlTheme.qml` (tokens) and
> `apps/zettings/qml/org/zyntrix/zettings/Components/` (primitives); this
> document specifies intent and exact values.
>
> **Originality clause.** ZDL is an original language for Zyntrix OS. The
> Windows 11 reconstruction spec supplies information architecture and
> interaction grammar only. Fluent, Apple HIG, Material, and Nothing OS are
> inspiration sources — never templates. No proprietary asset, metric table,
> or font from another platform is reproduced.

---

## 1. Design principles

1. **Calm authority** — a settings app is infrastructure; decoration never
   competes with comprehension.
2. **Everything responds** — every interactive surface acknowledges hover,
   press, focus, and state change through the shared motion vocabulary.
3. **Depth is honest** — elevation and translucency communicate real layering;
   glass never obscures legibility (§5 fallbacks).
4. **One excellent component** — pages are compositions of registry-driven
   primitives; bespoke one-off styling is forbidden without a documented
   exception in this file.
5. **Accessible by construction** — contrast pairs, focus treatment, target
   sizes, and reduced-motion behavior are token-level guarantees, not
   per-component afterthoughts.

## 2. Color system

### 2.1 Primitive ramp — `--zdl-base-*`

A cool neutral ramp (zinc family) anchors all surfaces:

| Token | Value | | Token | Value |
|---|---|---|---|---|
| `--zdl-base-50` | `#fafafa` | | `--zdl-base-500` | `#71717a` |
| `--zdl-base-100` | `#f4f4f5` | | `--zdl-base-600` | `#52525b` |
| `--zdl-base-200` | `#e4e4e7` | | `--zdl-base-700` | `#3f3f46` |
| `--zdl-base-300` | `#d4d4d8` | | `--zdl-base-800` | `#27272a` |
| `--zdl-base-400` | `#a1a1aa` | | `--zdl-base-900` | `#18181b` |
| | | | `--zdl-base-950` | `#09090b` |

### 2.2 Accent — "Zyntrix Aurora"

The brand accent is a saturated violet in light appearances and a luminous
lavender in dark appearances. It is used for selection, active navigation,
primary actions, icon washes (`--accent-soft`), and focus rings — never for
body text.

| Role | Light | Dark / OLED |
|---|---|---|
| `--accent` | `#7c3aed` | `#a78bfa` |
| `--accent-strong` (hover/pressed) | `#6d28d9` | `#c4b5fd` |
| `--on-accent` (text on accent fills) | `#ffffff` | `#241145` |
| `--accent-soft` (selection wash) | violet @ 12% | lavender @ 16% |

Validated pairs: `--accent` on white ≈ 5.7:1 (AA text); dark accent on
near-black ≈ 6.9:1; `--on-accent` pairs ≥ 4.5:1 in both appearances.

Secondary signal (warnings/destructive confirmations) uses amber `#b45309`
(light) / `#fbbf24` (dark); danger uses `#b91c1c` / `#f87171`. Status is never
encoded by color alone (icon + label always accompany it).

### 2.3 Semantic tokens (theme-aware)

Components consume **only** semantic tokens:

| Token | Light | Dark | OLED |
|---|---|---|---|
| `--surface` | base-50 | base-900 | `#000000` |
| `--surface-elevated` | `#ffffff` | base-800 | `#101010` |
| `--surface-muted` | base-100 | base-800/60% | `#0a0a0a` |
| `--surface-sunken` | base-200 | base-950 | `#000000` |
| `--text` | base-900 | base-50 | base-50 |
| `--text-muted` | base-600 | base-400 | base-400 |
| `--text-subtle` | base-500 | base-500 | base-500 |
| `--border` | base-300 | base-700 | base-800 |
| `--border-strong` (control outlines) | base-500 | base-500 | base-600 |
| `--focus-ring` | `--accent` | `--accent` | `--accent` |
| `--shadow-color` | 12% black | 40% black | 60% black |

### 2.4 Contrast contract (WCAG 2.2)

Validated pairs (computed ratios):

- `--text` on `--surface`: ≥ 13:1 in every theme (AAA).
- `--text-muted` on `--surface`: ≥ 6.5:1 (AA+, near AAA).
- `--text-subtle` is reserved for ≥ 18.67 px bold / 24 px text or decorative
  separators; never for instructional copy.
- `--border-strong` on `--surface`: ≥ 3:1 — required wherever a control
  boundary is the identifying affordance (unchecked toggle track, slider
  track, checkbox).
- `--on-accent` on `--accent`: ≥ 4.5:1 in every theme.
- Focus ring: 3 px `outline` at ≥ 3:1 against any surface it crosses
  (§8.3).

New token pairs must be added to the validated-pairs table in
`docs/research/prompt-compliance.md` evidence before use.

### 2.5 High-contrast theme (`data-theme="hc"`)

User-selectable always; automatically approximated under
`forced-colors: active`. Rules: pure `#000`/`#fff` inversions, all borders
2 px `currentColor`, **glass disabled** (opaque surfaces), shadows removed,
focus outline widened to 4 px, selection uses `Highlight` system color.

## 3. Typography

Stack: `"Inter", "Cantarell", "Segoe UI", system-ui, sans-serif`;
monospace: `"JetBrains Mono", "Fira Code", ui-monospace, monospace`.
KDE-native fallbacks (Cantarell, Noto Sans) render acceptably; Inter ships
with Zyntrix OS.

| Token | Size / line-height | Weight | Use |
|---|---|---|---|
| `--text-display` | 28 px / 36 px | 600 | Page titles (L1/L2) |
| `--text-title-lg` | 20 px / 28 px | 600 | Section heroes, dialog titles |
| `--text-title` | 16 px / 24 px | 600 | Card titles, group headers |
| `--text-body` | 14 px / 21 px | 400 | Default copy, descriptions |
| `--text-caption` | 12 px / 18 px | 400 | Meta, timestamps, badges |

Minimum rendered size is 12 px. Text scales with OS font settings; layouts
must survive 200% text zoom without clipping (reflow-safe single column).

## 4. Spacing, density, layout

Base unit **4 px**. Tokens `--space-1…--space-16` = 4…64 px.

| Context | Value |
|---|---|
| Card internal padding | `--space-4` (16 px) |
| Gap between cards in a group | `--space-2` (8 px) |
| Gap between groups | `--space-6` (24 px) |
| Page horizontal padding | `--space-8` (32 px) |
| Content max width | 1000 px (Windows-spec constrained column) |

Density modes: **comfortable** (default; settings rows 56 px min-height) and
**compact** (rows 44 px — the touch-target floor). Both keep interactive
targets ≥ 24 px absolute minimum; primary pointers land ≥ 44 px.

Responsive breakpoints (content width, per spec §16): >1100 expanded nav ·
800–1100 compact rail · 560–800 overlay nav · <560 single column.

## 5. Materials & elevation

### 5.1 Liquid Glass stack

Glass is the app's primary material and is layered over an **aurora
wallpaper** (`.zdl-aurora`): a fixed violet/teal mesh gradient whose slow
drift gives translucency something to refract. Composition per surface class:

```text
Full refraction (≤3 instances per view — rail panel, search flyout, topbar):
  L0 refract  backdrop-filter: blur(18px) saturate(160%)
              [+ SVG feDisplacementMap edge lensing in Chromium]
  L1 tint     --glass-tint (light .42 white / dark .55 zinc-900)
  L2 specular 4-edge inset rim highlights
  L3 content

Frost (bulk content — cards, lists):
  translucent gradient + hairline border + specular top rim; NO per-card
  backdrop-filter. The aurora behind is a smooth gradient, so skipping the
  blur is visually identical at a fraction of the compositor cost.
```

OLED and hc themes collapse every glass surface to opaque `--surface-elevated`.

### 5.2 Fallback ladder (legibility wins)

1. RHI hardware path: full refraction pipeline (glass tint + aurora behind).
2. Software rendering (llvmpipe under WSLg): blur+tint only, effect counts
   reduced to the ≤3 full-refraction surfaces.
3. OLED theme / high-contrast theme / reduced-transparency preference:
   **opaque** `--surface-elevated`; zero refraction layers.

Text over glass always sits on the tint layer validated against the *least
favorable* wallpaper region; if validation fails, opacity increases before
any text size/weight compensation is considered.

### 5.3 Elevation

| Level | Shadow (dark-adjusted via `--shadow-color`) | Use |
|---|---|---|
| 1 | `0 1px 2px` 6% | resting cards |
| 2 | `0 2px 8px` 10% | hovered cards, nav pane |
| 3 | `0 8px 24px` 14% | dialogs, flyouts |
| 4 | `0 16px 48px` 20% | search overlay |

Elevation is communicated by shadow **and** surface shift together; hc theme
replaces both with 2 px borders.

## 6. Geometry — G2/G3 continuous curvature

Standard `border-radius` is **forbidden** on major surfaces (cards, panels,
dialogs, shell). ZDL uses superellipse squircles:

```text
|x/a|^n + |y/b|^n = 1
G2 (n=4): cards, inputs, list rows, buttons
G3 (n=6): app shell frame, floating dialogs, search overlay
```

Radius blending interpolates the exponent between ellipse (n=2) and target
order as `r/min(w,h)/2` grows, so small radii stay visually consistent:

```text
n_eff = 2 + (n − 2) · r / (min(w,h)/2)
```

Implementation: the QML `Squircle` primitive samples the superellipse into a
96-point `PathSvg` inside a `Shape`, recomputed on resize; `exponent` follows
the blending rule above so small radii stay visually consistent. Radii tokens:
`--radius-control` 10 · `--radius-card` 14 · `--radius-panel` 20 ·
`--radius-overlay` 28. Pill shapes (toggles, chips) use true capsules.
Focus outlines are drawn as part of the squircle stroke so clipping never
hides them.

## 7. Iconography

Lucide stroke icons (24 px grid, 2 px stroke, round caps/joins) — matches the
seed-graph icon names already shipped in the registry. Rules: icons are
decorative unless alone; icon-only controls require `aria-label`; status
icons always pair with text; `currentColor` everywhere so themes and forced
colors adapt.

## 8. States & interaction

### 8.1 Component states

Every interactive primitive implements: rest · hover · pressed · focus ·
selected/checked · disabled · loading. Hover lifts elevation 1→2 and shifts
surface toward `--surface-elevated`; press compresses scale to 0.98 with
spring return (Phase 3 engine); disabled keeps full visibility, drops to
`--text-subtle` + `--border`, and explains itself via description when the
reason isn't obvious (spec §15).

### 8.2 Selection semantics

A card is either a toggle or a navigation affordance — never ambiguous
(spec §18). Navigation rows end in a chevron; toggle cards carry the switch
on the right; mixed-action cards separate the zones into distinct targets.

### 8.3 Focus

```css
:focus-visible {
  outline: 3px solid var(--focus-ring);
  outline-offset: 2px;
}
```

Never removed without equal replacement. Under `forced-colors`, outline
switches to `Highlight`. Focus and selection are visually distinct
(outline ≠ fill). Focus order follows DOM order; composite widgets (nav rail)
use roving tabindex with arrow keys per APG.

### 8.4 Keyboard contract (app-wide)

Ctrl+F search · Alt+Backspace back · arrows navigate lists/rail · Enter/Space
activate · Esc dismisses overlays · Tab traversal logical. Dialogs use Qt
Quick Controls `Dialog` with `modality`, explicit initial focus, and focus
restore on close.

## 9. Themes

Selection model follows the three-option pattern (System / Light / Dark +
OLED and High-contrast entries), stored as `system|light|dark|oled|hc`,
applied through `ZdlTheme.themeMode`. The `system` entry resolves from the
desktop color-scheme signal and is wired when the desktop-integration phase
lands; the four explicit modes are fully functional now. Theme changes occur
**only** on explicit activation — never on focus/hover.
OLED forces opaque blacks and disables glass; hc per §2.5.

## 10. Motion (token layer; engine lands Phase 3)

Durations: `--motion-instant` 80 ms · `--motion-quick` 140 ms ·
`--motion-normal` 220 ms · `--motion-deliberate` 320 ms.
Easing: standard `cubic-bezier(0.2, 0, 0, 1)`; emphasized transitions use
spring physics `{stiffness, damping}` pairs — navigation 220/28, controls
320/22, modals 180/24 (mass 1.0/0.6/1.2). Velocity is preserved across
interruptions; overshoot allowed only on control-scale feedback (≤ 4%).

Reduced motion: the platform reduce-motion preference (surfaced as
`ZdlTheme.reducedMotion`, wired to the desktop setting in the
desktop-integration phase) collapses all transforms to opacity cross-fades
≤ 120 ms or zero duration; parallax and secondary motion are disabled
entirely. Theme changes never animate color.

GPU discipline: animate only `transform`/`opacity`/`clip-path`/`filter`;
layout properties are never animated. Frame budget 8.3 ms; instrumentation
lands with the Phase 3 harness.

## 11. Component anatomy (canonical primitives)

```text
SettingsCard      [icon] title/description …… control | chevron
SettingsExpander  header card + aria-expanded region, ONE level deep
NavRow            icon + label (+ badge) ; roving tabindex in rail
ToggleSwitch      role=switch, 40×24 track, 20×20 knob, ≥44 px hit area
InfoBar           icon + message + optional action; polite live region
Dialog            native <dialog>, G3 overlay surface, focus restore
SearchOverlay     level-4 elevation, results ranked by zettings-search
Skeleton          shimmer-free pulse (reduced-motion safe) placeholder
EmptyState        icon + explanation + action; honest unavailable states
```

Anatomy deviations require a DESIGN.md amendment first, implementation second.

## 12. Compliance hooks

- Token usage enforced by review + future `validate-tokens` script (no raw hex
  outside `zdl.css`).
- Every primitive ships with its state matrix exercised in tests (Phase 8:
  axe scans per theme, keyboard walkthroughs, forced-colors snapshots).
- This file changes only through commits that update both spec and tokens
  atomically.
