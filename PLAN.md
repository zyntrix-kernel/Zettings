# PLAN.md â€” ZETTINGS Master Implementation Roadmap

> **Status:** Authoritative implementation roadmap (native Qt 6 + Rust era â€” clean-slate rebuild)
>
> **Project:** ZETTINGS â€” Zyntrix OS Settings
>
> **Stack:** Native **Qt 6 (QML)** UI bridged from **Rust** via **cxx-qt**; backend pure Rust (tokio, zbus). No webview, no Node toolchain.
>
> **Target:** Kubuntu 24.04 / KDE Plasma 5.27
>
> **Primary inputs:** `prompt.txt`, `ZETTINGS_Windows_11_Settings_Deep_UI_Spec.md`, `AGENTS.md`, `DESIGN.md`
>
> **Rule:** `prompt.txt` is a mandatory product requirement. The Windows 11 specification is the reference for information architecture, navigation, reusable settings patterns, search, and observable UXâ€”not a license to copy Microsoft's proprietary implementation.

---

# 0. PLAN AUTHORITY AND NON-NEGOTIABLES

## 0.1 Source hierarchy

When requirements conflict, use this order:

1. Explicit user requirements.
2. `AGENTS.md` repository safety, architecture, skill, and verification rules.
3. `DESIGN.md` / Zyntrix Design Language (ZDL).
4. `prompt.txt` product requirements.
5. Windows 11 reconstruction specification as the reference model.
6. Existing repository architecture and established implementation conventions.
7. Loaded skills.
8. Generic model assumptions.

If two requirements genuinely conflict and cannot be resolved, **STOP** before implementation.

## 0.2 Prompt compliance is continuous

`prompt.txt` is not merely an initial prompt. It is a **living acceptance specification**.

Every phase, feature, refactor, and release must be checked against it.

An implementation is incomplete if it works technically but violates any applicable requirement from `prompt.txt`.

The agent must maintain a **Prompt Compliance Matrix** throughout the project:

```text
Prompt requirement
    â†“
PLAN phase / feature
    â†“
Implementation
    â†“
Verification
    â†“
Evidence
    â†“
PASS / FAIL / BLOCKED
```

No requirement may silently disappear because it was not convenient to implement.

## 0.3 No phase skipping

The project uses sequential phases:

```text
Phase 0 â†’ Phase 1 â†’ Phase 2 â†’ ... â†’ Phase 12
```

A phase must be completed and accepted before the next phase begins.

A later phase may not be used to hide unfinished mandatory work from an earlier phase.

---

# 1. PRODUCT VISION

ZETTINGS is a next-generation Linux system settings application for Zyntrix OS.

It must combine:

- Windows 11's strong settings information architecture and discoverability;
- Apple's refinement, hierarchy, responsiveness, and interaction quality;
- KDE/Linux-native system integration;
- Rust/Qt security and maintainability;
- an original Zyntrix Design Language (ZDL).

It is **not a Windows clone**.

The Windows reference explicitly defines the target as the Windows Settings mental model plus Windows/Fluent grammar, KDE/Linux-native implementation, data-driven registry, strong search, deep links, native adapters, accessibility, and responsive navigation.

`prompt.txt` requires the finished product to exceed major desktop settings applications in UI, UX, animation, architecture, performance, integration, accessibility, consistency, extensibility, and maintainability.

---

# 2. MASTER ACCEPTANCE TARGET

At release, ZETTINGS must satisfy all applicable requirements in these domains:

## Product

- Production-grade system settings application.
- No toy examples.
- No fake integrations presented as real.
- No unexplained placeholders.
- No pseudo-code in production implementation.
- No arbitrary shortcuts.

## UI / UX

- Original ZDL.
- Windows-inspired information architecture.
- Premium visual hierarchy.
- Responsive navigation.
- Reusable settings primitives.
- Consistent interaction behavior.
- Deep linking.
- Breadcrumb/context navigation.
- Excellent empty/loading/error/permission states.

## Motion

- G2/G3 continuity where defined by ZDL.
- Bezier interpolation.
- Spring interpolation.
- Velocity preservation.
- Momentum.
- Elasticity.
- Dynamic easing.
- Micro-interactions.
- Anticipation / overshoot / settle.
- Secondary motion.
- Parallax/depth where justified.
- Adaptive and context-aware duration.
- GPU-accelerated rendering.
- No janky or blocking animation.
- Target 120 FPS.

These requirements are explicitly specified by `prompt.txt`.

## System integration

The architecture must support the complete requested integration surface, including:

- display / resolution / scaling / HDR / night light / refresh rate / GPU / color profiles;
- audio / microphone;
- Bluetooth / Wi-Fi / Ethernet / VPN / firewall;
- updates;
- storage / battery / power / performance modes;
- processes / startup apps;
- users / groups;
- accessibility / keyboard / mouse / touchpad / tablet / stylus;
- fonts / regional settings / language / notifications;
- privacy / security / developer options;
- Wayland / X11;
- virtual desktops / window rules / KWin / KDE effects;
- package management / drivers / printers / USB / Thunderbolt / DisplayLink / docking;
- file associations / default apps;
- themes / wallpaper / accent / animations / cursor / icons;
- shell / terminal;
- network shares / remote desktop / SSH;
- containers / Flatpak / Snap / AppImage / Docker / Podman / VMs;
- logs / crash reports / services / systemd / journal;
- environment variables / power profiles;
- BIOS / kernel / CPU / GPU / memory / sensors / fans / thermals.

This inventory comes directly from `prompt.txt`; unsupported hardware or platform capabilities must be represented honestly rather than faked.

---

# 3. ARCHITECTURAL PRINCIPLE

Use a data-driven settings graph:

```text
Canonical Registry
      â†“
Category
      â†“
Page
      â†“
Section
      â†“
Setting Definition
      â†“
Reusable Control Renderer
      â†“
Capability / Permission Layer
      â†“
Backend Adapter
      â†“
Native Linux / KDE subsystem
```

The Windows reconstruction reference specifically recommends a registry â†’ page definition â†’ section definition â†’ setting definition â†’ control renderer â†’ native Linux backend pipeline.

Every setting must have at least:

```text
stable ID
title
description
category
page
section
route
aliases
keywords
icon
control type
current-value provider
set-value action
validation
permission requirements
hardware requirements
reboot requirements
search weight
backend capability
```

---

# 4. WINDOWS SETTINGS RECONSTRUCTION BASELINE

Use Windows Settings as the **engineering and information-architecture reference**, not as proprietary source code.

The current top-level model is:

```text
Home
System
Bluetooth & devices
Network & internet
Personalization
Apps
Accounts
Time & language
Gaming
Accessibility
Privacy & security
Windows Update
```

The reference identifies this as the recognizable Windows Settings mental model.

Use the following hierarchy:

```text
L0 Application shell
L1 Category / hub
L2 Settings page
L3 Section / group
L4 Setting entity / card / row
L5 Inline expander / sub-setting
L6 Dialog / flyout / picker / advanced page
```

This hierarchy is explicitly identified in the Windows reconstruction specification.

Reusable templates must include:

```text
CategoryPageTemplate
L2SettingsPageTemplate
SettingsGroupTemplate
SettingsCardTemplate
SettingsExpanderTemplate
DeviceListTemplate
DetailPageTemplate
SearchResultTemplate
PermissionTemplate
ErrorTemplate
EmptyStateTemplate
```

---

# 5. SEARCH ARCHITECTURE

ZETTINGS search is a first-class subsystem, not a text filter.

Required capabilities:

- natural-language search;
- fuzzy matching;
- AI-assisted ranking;
- synonyms;
- aliases;
- misspelling tolerance;
- keyboard navigation;
- instant results;
- recent settings;
- pinned settings;
- frequently used settings;
- context awareness;
- predictive suggestions;
- searchable settings index.

These requirements are explicit in `prompt.txt`.

Recommended pipeline:

```text
Query
 â†“
Normalization
 â†“
Exact title/ID match
 â†“
Keyword / alias match
 â†“
Fuzzy match
 â†“
Semantic retrieval
 â†“
Context/recent/frequency boosts
 â†“
Ranked settings
 â†“
Direct route
```

Baseline ranking from the Windows reconstruction:

```text
exact title match       +100
exact keyword match      +80
alias match              +70
page/category match      +50
description match       +35
semantic similarity      +30
recently used            +10
fuzzy spelling            +5
```

Direct settings must rank above broad category pages.

AI-assisted setting changes must always use:

```text
Intent
 â†“
Candidate setting
 â†“
Explanation
 â†“
Preview
 â†“
Explicit user confirmation
 â†“
Validated backend action
 â†“
Result
```

Never allow AI to silently change arbitrary system configuration.

---

# 6. DEEP LINKING

Use ZETTINGS-owned routes.

Examples:

```text
zettings://system
zettings://system/display
zettings://system/sound
zettings://system/power
zettings://devices/bluetooth
zettings://devices/printers
zettings://network/wifi
zettings://network/ethernet
zettings://personalization/background
zettings://personalization/themes
zettings://apps/installed
zettings://accounts/sign-in
zettings://accessibility/display
zettings://privacy/microphone
zettings://updates
```

The Windows specification explicitly recommends a ZETTINGS route scheme rather than copying `ms-settings:`.

Rules:

- Every navigable setting has a stable route.
- Every route maps to exactly one page definition.
- Routes must be serializable and testable.
- Back/forward navigation must work.
- Deep-linked pages must load directly.
- Permission/hardware failures must not break routing.

---

# 7. DESIGN SYSTEM â€” ZDL

`prompt.txt` requires an original design language named **Zyntrix Design Language (ZDL)**.

Do not copy Windows, macOS, GNOME, KDE, or another product.

Take inspiration from them while producing an original visual system.

ZDL must define:

```text
Color tokens
Typography
Spacing
Density
Elevation
Materials
Transparency
Blur
Geometry
G2/G3 curvature
Controls
Icons
Focus
Selection
Hover
Pressed states
Disabled states
Dark mode
OLED mode
Light mode
Motion tokens
Accessibility variants
```

Repository `DESIGN.md` remains authoritative for exact ZDL values.

The Windows reference supplies reusable UI anatomy such as SettingsCard, SettingsExpander, navigation, constrained content, and standard controls; ZDL must translate those concepts into Zyntrix's own geometry and materials.

---

# 8. MOTION ENGINE

Build the motion system before feature-heavy UI.

Architecture:

```text
Interaction
 â†“
Motion intent
 â†“
Context
 â†“
Physics parameters
 â†“
Interpolation
 â†“
GPU-composited animation
 â†“
Accessibility/reduced-motion policy
```

Motion engine must support:

- G2/G3 continuity;
- Bezier interpolation;
- spring interpolation;
- velocity preservation;
- momentum;
- elasticity;
- dynamic easing;
- micro-interactions;
- anticipation;
- overshoot;
- settle;
- secondary motion;
- parallax;
- depth;
- adaptive duration;
- context-aware motion.

`prompt.txt` explicitly requires these motion primitives and a 120 FPS target.

Reduced-motion mode must disable or simplify non-essential motion without breaking interaction.

---

# 9. SECURITY ARCHITECTURE

Security is a product feature.

Required principles:

- Zero Trust.
- Least privilege.
- Sandboxing.
- Secure IPC.
- PolicyKit.
- Capability-based permissions.
- Input sanitization.
- Permission validation.
- Audit logging.
- Threat modeling.
- No unsafe arbitrary command execution.
- Privileged operations isolated in backend adapters.

These are explicit `prompt.txt` requirements.

Additional rules:

```text
QML/UI
  NEVER directly modifies system files.

UI
  â†’ typed cxx-qt invokable / D-Bus call
  â†’ capability/authorization layer
  â†’ backend adapter
  â†’ native API
```

System changes must go through backend adapters and privileged operations must be isolated and validated.

---

# 10. PERFORMANCE BUDGET

Target:

```text
Cold launch       < 500 ms
Hot launch        < 150 ms
Memory            < 150 MB
Target rendering  120 FPS
```

Required strategies:

- lazy-loaded feature modules;
- lazy backend initialization;
- background search indexing;
- asynchronous system queries;
- parallel independent backend queries;
- memoized selectors;
- no unnecessary QML re-evaluation or over-draw;
- resource cleanup;
- no memory leaks;
- GPU-accelerated animations;
- no blocking IPC;
- bounded caches;
- incremental rendering for large lists.

These targets and techniques are specified in `prompt.txt`.

Every performance claim must be measured.

---

# 11. ACCESSIBILITY

Target:

- WCAG AAA where applicable;
- screen-reader support;
- keyboard-only operation;
- reduced motion;
- high contrast;
- large-font support;
- color-blind-safe information encoding;
- visible focus;
- accessible animation;
- voice-navigation-ready semantics.

These requirements are explicitly listed in `prompt.txt`.

Every interactive control must have:

```text
accessible name
role
state/value
keyboard operation
focus behavior
screen-reader semantics
error/disabled explanation where relevant
```

---

# 12. THEMING

Required modes:

```text
Light
Dark
OLED
```

Required customization:

```text
Dynamic accent colors
Wallpaper-derived colors
Custom themes
Roundedness
Transparency
Blur
Animation intensity
Compact mode
Comfort mode
Developer mode
```

These requirements come from `prompt.txt`.

All theme values must flow through ZDL tokens.

No feature may hard-code theme colors or geometry without a documented exception.

---

# 13. BACKEND INTEGRATION MATRIX

Build adapters rather than embedding platform calls into UI components.

Primary backend technologies required by `prompt.txt` include:

```text
Rust
Qt 6 (QML)
cxx-qt
Tokio
zbus
D-Bus
PolicyKit
systemd
NetworkManager
PipeWire
PulseAudio
UPower
PowerProfilesDaemon
AccountsService
BlueZ
KScreen
KWin APIs
libinput
login1
PackageKit
Flatpak
Snap
AppImage
Debian packaging
```


Suggested adapter boundaries:

```text
DisplayAdapter
AudioAdapter
BluetoothAdapter
NetworkAdapter
VpnAdapter
FirewallAdapter
UpdateAdapter
StorageAdapter
BatteryAdapter
PowerAdapter
ProcessAdapter
StartupAdapter
AccountsAdapter
AccessibilityAdapter
InputAdapter
FontAdapter
LocaleAdapter
NotificationAdapter
PrivacyAdapter
SecurityAdapter
KWinAdapter
DesktopAdapter
PackageAdapter
PrinterAdapter
UsbAdapter
ContainerAdapter
VirtualizationAdapter
DiagnosticsAdapter
SystemAdapter
```

Each adapter exposes typed capabilities and reports unavailable functionality honestly.

---

# 14. PHASE 0 â€” RESEARCH

## Objectives

Establish the factual and architectural baseline before implementation.

## Tasks

- Read and reconcile `prompt.txt`.
- Read the Windows reconstruction specification completely.
- Read `AGENTS.md`.
- Read `DESIGN.md`.
- Inventory existing repository architecture.
- Inventory existing skills.
- Identify authoritative skill for every technical concern.
- Load all required skills before repository work.
- Build Prompt Compliance Matrix.
- Build Windows â†’ Zyntrix feature mapping.
- Identify Linux backend feasibility.
- Identify unsupported/optional platform features.
- Establish terminology.
- Establish security threat model.
- Establish performance measurement methodology.

## Deliverables

```text
docs/research/
docs/research/prompt-compliance.md
docs/research/windows-to-zyntrix-mapping.md
docs/research/backend-capability-matrix.md
docs/research/threat-model.md
docs/research/performance-baseline.md
```

## Exit gate

No unresolved architectural contradiction.

---

# 15. PHASE 1 â€” ARCHITECTURE

## Objectives

Create the production architecture.

## Tasks

- Define workspace boundaries.
- Define Rust crates using `zettings-` naming.
- Define cxx-qt bridge architecture (QML â†” Rust) and D-Bus IPC architecture.
- Define frontend/backend boundary.
- Define capability system.
- Define backend adapter interface.
- Define settings registry.
- Define settings graph.
- Define routing.
- Define search index.
- Define plugin architecture.
- Define dependency injection.
- Define message bus.
- Define repository pattern where appropriate.
- Define state machines.
- Define error model.
- Define observability.
- Define audit logging.
- Define permissions model.

## Deliverables

```text
architecture diagrams
sequence diagrams
component diagrams
data-flow diagrams
API contracts
settings registry schema
route schema
capability model
```

## Exit gate

Architecture review passes and Prompt Compliance Matrix remains complete.

---

# 16. PHASE 2 â€” DESIGN SYSTEM

## Objectives

Implement ZDL foundations.

## Tasks

- Define tokens.
- Define typography.
- Define spacing.
- Define density.
- Define surfaces/materials.
- Define G2/G3 geometry.
- Define control states.
- Define icons.
- Define focus states.
- Define light/dark/OLED themes.
- Define accessibility variants.
- Define responsive breakpoints.
- Define component primitives.
- Define SettingsCard.
- Define SettingsExpander.
- Define navigation row.
- Define info bar.
- Define picker primitives.

## Exit gate

Core components visually and semantically satisfy `DESIGN.md` and Windows reference hierarchy without becoming a Windows clone.

---

# 17. PHASE 3 â€” MOTION ENGINE

## Objectives

Create reusable ZDL motion infrastructure.

## Tasks

- Motion token system.
- Spring system.
- Bezier system.
- G2/G3 transition support.
- Velocity tracking.
- Gesture state.
- Enter/exit transitions.
- Navigation transitions.
- Expansion transitions.
- Hover/press/focus feedback.
- Modal/flyout transitions.
- Reduced-motion behavior.
- GPU compositing strategy.
- Frame-time instrumentation.

## Exit gate

Motion primitives are reusable, measurable, accessible, and do not block rendering.

---

# 18. PHASE 4 â€” CORE FRAMEWORK

## Objectives

Build the application shell.

## Tasks

- Window shell.
- Navigation shell.
- Responsive navigation.
- Search surface.
- Back/forward navigation.
- Breadcrumbs.
- Routing.
- Settings registry loading.
- Error boundaries.
- Loading states.
- Empty states.
- Permission states.
- Notification/InfoBar system.
- Keyboard navigation.
- Accessibility tree.
- Theme engine.
- State management.

## Navigation model

Use:

```text
Desktop
  expanded navigation

Medium
  compact navigation

Narrow
  overlay/minimal navigation
```

The Windows specification explicitly warns against keeping a giant always-open sidebar at every width.

---

# 19. PHASE 5 â€” BACKEND INTEGRATION

## Objectives

Connect ZETTINGS to real Linux/KDE systems.

## Tasks

Implement adapters incrementally:

1. D-Bus / zbus foundation.
2. PolicyKit.
3. systemd/login1.
4. NetworkManager.
5. PipeWire/PulseAudio.
6. BlueZ.
7. KScreen.
8. KWin.
9. UPower / power profiles.
10. AccountsService.
11. libinput/input.
12. udisks/storage.
13. CUPS/printers.
14. PackageKit.
15. Flatpak.
16. Snap.
17. AppImage.
18. Diagnostics/system information.
19. Containers/VM integrations.
20. Remaining feature adapters.

No UI should call native system APIs directly.

## Exit gate

Each adapter has:

```text
typed API
capability detection
permission handling
error handling
tests
mock implementation where required
real Linux integration path
```

---

# 20. PHASE 6 â€” FRONTEND

## Objectives

Build the complete user-facing shell on the framework.

## Tasks

Implement:

```text
Home
System
Bluetooth & devices
Network & internet
Personalization
Apps
Accounts
Time & language
Gaming
Accessibility
Privacy & security
Updates
```

The Windows reference establishes these categories as the baseline information architecture.

Do not blindly reproduce every historical Windows Settings URI. The reference explicitly warns that some entries are deprecated or conditional.

---

# 21. PHASE 7 â€” FEATURE MODULES

Build modules from highest-value/core system capabilities to extended integrations.

## Tier 1 â€” Core desktop

- Display.
- Audio.
- Network.
- Bluetooth.
- Power.
- Storage.
- Appearance.
- Input.
- Users.
- Accessibility.

## Tier 2 â€” System management

- Apps.
- Packages.
- Updates.
- Printers.
- Privacy.
- Security.
- Developer options.
- Services.
- Startup.
- Processes.

## Tier 3 â€” KDE/Zyntrix

- KWin.
- Window rules.
- Virtual desktops.
- KDE effects.
- Plasma integration.
- Shell.
- Terminal.
- Theme engine.
- Zyntrix-specific settings.

## Tier 4 â€” Advanced ecosystem

- Containers.
- Flatpak.
- Snap.
- AppImage.
- Docker.
- Podman.
- Virtual machines.
- SSH.
- Remote desktop.
- Diagnostics.
- Logs.
- Crash reports.
- Hardware telemetry.

Each module must consume the canonical registry and reusable components.

---

# 22. PHASE 8 â€” TESTING

## Test layers

### Rust

- unit tests;
- integration tests;
- backend adapter tests;
- permission tests;
- error-path tests;
- serialization tests;
- IPC contract tests.

### Frontend

- component tests;
- state tests;
- routing tests;
- keyboard navigation tests;
- accessibility tests;
- search tests;
- theme tests;
- reduced-motion tests.

### Integration

- real D-Bus;
- real NetworkManager;
- real PipeWire;
- real BlueZ;
- real KScreen;
- real KWin;
- PolicyKit;
- systemd/login1.

### Visual

- screenshot regression;
- responsive breakpoints;
- theme comparisons;
- state comparisons;
- animation/frame-time checks.

### Performance

Measure the explicit `prompt.txt` targets rather than claiming them.

---

# 23. PHASE 9 â€” OPTIMIZATION

## Objectives

Meet or improve the performance budget.

## Tasks

- cold launch profiling;
- hot launch profiling;
- memory profiling;
- QML scene-graph/render-loop profiling;
- Rust profiling;
- IPC latency;
- backend query parallelization;
- search indexing optimization;
- lazy loading;
- cache tuning;
- GPU compositing;
- animation frame profiling;
- leak detection.

## Acceptance

```text
Cold launch < 500 ms
Hot launch  < 150 ms
Memory      < 150 MB
Target      120 FPS
```

If hardware/environment prevents a target from being met, document the measured result and bottleneck instead of fabricating compliance.

---

# 24. PHASE 10 â€” PACKAGING

## Objectives

Produce installable Zyntrix artifacts.

## Tasks

- Debian packaging.
- Application binary + QML asset packaging.
- Dependency declarations.
- Desktop entry.
- Icons.
- MIME/deep-link registration.
- Permissions.
- PolicyKit installation.
- Upgrade behavior.
- Uninstall behavior.
- Offline/install validation.
- Versioning.
- Release artifacts.

---

# 25. PHASE 11 â€” DOCUMENTATION

Required documentation:

```text
README
Developer Guide
Contribution Guide
Architecture documentation
Backend integration guide
Security model
Threat model
Testing strategy
Performance guide
ZDL documentation
Plugin development guide
Settings registry guide
WSL2 setup
Packaging guide
Release guide
```

`prompt.txt` explicitly requires folder/architecture diagrams, sequence/component/data-flow diagrams, modules, API contracts, testing strategy, CI/CD, documentation, README, Developer Guide, and Contribution Guide.

---

# 26. PHASE 12 â€” RELEASE

## Final release gate

Every applicable requirement must be:

```text
PASS
```

or explicitly:

```text
BLOCKED â€” documented reason
```

No silent omissions.

Final verification:

```cmd
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo check --workspace
qmllint <changed .qml files>
qmlformat --check <changed .qml files>
```

Then run all relevant tests, security checks, packaging checks, runtime integration checks, performance benchmarks, and visual/accessibility checks required by the affected modules.

---

# 27. PROMPT COMPLIANCE MATRIX

Maintain this matrix throughout the project.

| Requirement domain | Required | Evidence |
|---|---:|---|
| Production-grade implementation | YES | Build + review |
| Original ZDL | YES | `DESIGN.md` + visual review |
| Windows-inspired architecture | YES | Registry/navigation tests |
| Apple-quality interaction target | YES | UX/visual review |
| KDE/Linux-native backend | YES | Integration tests |
| Rust + Qt 6 via cxx-qt | YES | Workspace |
| QML frontend (single module URI) | YES | Frontend |
| Advanced motion engine | YES | Motion tests |
| G2/G3 | YES | Motion/design tests |
| 120 FPS target | YES | Performance benchmark |
| Search | YES | Search test suite |
| AI ranking | YES | Ranking tests |
| Deep links | YES | Route tests |
| Enterprise modular architecture | YES | Architecture review |
| Zero Trust / least privilege | YES | Security review |
| PolicyKit | YES | Permission integration |
| Audit logging | YES | Security tests |
| Performance budget | YES | Benchmarks |
| WCAG AAA target | YES | Accessibility audit |
| Light/Dark/OLED | YES | Theme tests |
| Dynamic/custom themes | YES | Theme tests |
| Full requested system integration | YES | Capability matrix |
| Documentation | YES | Docs review |
| CI/CD | YES | CI pipeline |
| Packaging | YES | Install artifacts |
| Release readiness | YES | Final gate |

---

# 28. PHASE COMPLETION CONTRACT

A phase is complete only when:

```text
[ ] All phase objectives implemented
[ ] All required skills were loaded before relevant work
[ ] Prompt Compliance Matrix updated
[ ] Architecture constraints respected
[ ] ZDL constraints respected
[ ] Security constraints respected
[ ] Accessibility constraints respected
[ ] Tests completed
[ ] Applicable verification gates passed
[ ] Documentation updated
[ ] No known unresolved blocker
[ ] Phase review accepted
```

Only then may the next phase begin.

---

# 29. ABSOLUTE DEVELOPMENT RULE

The agent must continuously ask:

```text
Does this satisfy prompt.txt?
Does this satisfy AGENTS.md?
Does this satisfy DESIGN.md?
Does this use the correct loaded skills?
Does this preserve the Windows information architecture without cloning Windows?
Does this use native Linux/KDE backends?
Is it production-ready?
Is it secure?
Is it accessible?
Is it performant?
Is it maintainable?
```

If any answer is **NO**, the work is not complete.

`prompt.txt` explicitly establishes the quality bar as production-ready, fully tested, benchmarked, and continuously evaluated against Apple, Microsoft, KDE, and Rust engineering standards.

---

# 30. FINAL PRODUCT DEFINITION

ZETTINGS is successful when it becomes:

> A beautiful, intelligent, fluid, secure, accessible, deeply integrated, Linux-native settings application for Zyntrix OS that uses the recognizable information architecture and discoverability principles of Windows Settings while remaining an original Zyntrix product.

The Windows reference explicitly defines the final reconstruction target as Windows information architecture + Fluent grammar + KDE/Linux-native implementation + data-driven registry + strong search + stable deep links + native adapters + accessibility + responsive navigation.

The product goal from `prompt.txt` is broader: a flagship Qt + Rust + KDE Plasma application that is native, fluid, intelligent, deeply integrated, modular, secure, maintainable, and suitable for long-term evolution into Zyntrix OS's official settings application.
