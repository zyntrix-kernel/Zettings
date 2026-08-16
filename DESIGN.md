# DESIGN.md — Zyntrix Design Language (ZDL) Specification

## 1. Geometry: Continuous Curvature (G2/G3 Squircles)
Standard CSS `border-radius` is forbidden for major cards and modal containers. Radii must follow continuous curvature $G2/G3$ superellipses defined by:

$$\left|\frac{x}{a}\right|^n + \left|\frac{y}{b}\right|^n = 1$$

- **G2 Continuity ($n=4$):** Standard cards, input fields, toggles, list items.
- **G3 Continuity ($n=6$):** Main application shell, floating dialogs, spotlight search overlay.

### Radius Blending
The `radius` parameter controls corner roundness, blending between a sharp rectangle (`radius=0`) and a full superellipse. The blending is achieved by interpolating the effective exponent between 2 (pure ellipse) and the target order (4 for G2, 6 for G3) based on the radius ratio:

$$n_{eff} = 2 + (n - 2) \cdot \frac{r}{\min(w, h) / 2}$$

### SVG Path Output
Squircle paths use cubic bezier (`C`) commands with 128 perimeter samples for smooth curves. The `useSquircle` hook returns both the SVG path string and a CSS `clip-path` value.

## 2. Liquid Glass Material Stack
Glass panels use a multi-layered composition:
- **Layer 1 (Backdrop):** `backdrop-filter: blur(24px) saturate(180%)`
- **Layer 2 (Tint):** `rgba(28, 25, 23, 0.65)` (Dark) / `rgba(255, 255, 255, 0.45)` (Light)
- **Layer 3 (Specular Highlight):** `1px solid rgba(255, 255, 255, 0.12)` top border

### Elevation Levels
| Level | Shadow | Use Case |
|-------|--------|----------|
| 1 | `0 1px 2px rgba(0,0,0,0.04)` | Inline cards, list items |
| 2 | `0 2px 8px rgba(0,0,0,0.08)` | Sidebar, content panels |
| 3 | `0 8px 24px rgba(0,0,0,0.12)` | Floating dialogs, modals |
| 4 | `0 16px 48px rgba(0,0,0,0.16)` | Spotlight search overlay |

## 3. ZDL Token Cascade (3-Tier System)

### Tier 1: Primitive Tokens
Raw color values, spacing, radii, typography scales. Theme-agnostic.
- Neutrals: `--zdl-base-50` through `--zdl-base-950` (stone palette)
- Accents: `--accent`, `--accent-on`, `--accent-secondary`
- Spacing: `--space-1` (4px) through `--space-16` (64px) on 4/8 rhythm
- Typography: `--font-sans`, `--font-mono`, `--text-xs` through `--text-2xl`
- Radii: `--radius-sm` (6px), `--radius-md` (10px), `--radius-lg` (14px), `--radius-xl` (20px)
- Glass: `--glass-blur` (24px), `--glass-saturate` (180%), `--glass-tint`, `--glass-specular`
- Shadows: `--shadow-1` through `--shadow-4`

### Tier 2: Semantic Tokens
Theme-aware aliases that reference primitives. Override per theme via `[data-theme="..."]`.
- Surfaces: `--surface`, `--surface-elevated`, `--surface-muted`
- Borders: `--border`
- Text: `--text`, `--text-muted`, `--text-subtle`
- Ring: `--ring` (focus indicator)

### Tier 3: Component Tokens
Component-specific compositions referencing semantic tokens.
- Sidebar: `--sidebar-bg`, `--sidebar-border`
- Content: `--content-bg`, `--content-bar-border`
- Glass panel: `--glass-panel-bg`, `--glass-panel-blur`, `--glass-panel-border`

### Theme Variants
| Theme | `data-theme` | Background | Key Difference |
|-------|-------------|------------|----------------|
| Light | `light` (default) | `--zdl-base-50` | High contrast on white |
| Dark | `dark` | `--zdl-base-950` | Inverted, dark surfaces |
| OLED | `oled` | `#000000` | True black, pure OLED |
| High Contrast | `hc` | Boosted | WCAG AAA, thick borders |

## 4. Motion Engine Physics Parameters
- **Navigation Slide:** `stiffness: 220, damping: 28, mass: 1.0`
- **Interactive Toggles:** `stiffness: 320, damping: 22, mass: 0.6`
- **Sliders & Knobs:** `stiffness: 400, damping: 30, mass: 0.5`
- **Modal Dialog:** `stiffness: 180, damping: 24, mass: 1.2`
- **Target Compositor Speed:** **120 FPS** with zero synchronous DOM reflows.

### Reduced Motion Fallback
When `prefers-reduced-motion: reduce` is active, all spring animations fall back to opacity cross-fades with `0ms` duration. The `data-reduced-motion` attribute on the shell element gates this behavior.

## 5. Tailwind v4 @theme Integration
ZDL primitive tokens are mapped into Tailwind v4's `@theme` block, enabling utility classes like:
- `bg-surface`, `bg-surface-elevated`, `bg-surface-muted`
- `text-default`, `text-muted`, `text-subtle`
- `border-border`, `ring-ring`
- `shadow-1` through `shadow-4`
- `font-sans`, `font-mono`
