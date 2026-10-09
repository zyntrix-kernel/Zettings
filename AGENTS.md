# AGENTS.md — ZETTINGS Repository Operating Directives

This document is the authoritative operating contract for every human or AI agent working in the ZETTINGS repository.

Read it before performing repository work.

These rules govern engineering process, architecture, skill acquisition, implementation quality, verification, and completion.

---

# 1. PROJECT IDENTITY

**Project:** ZETTINGS
**Operating System:** Zyntrix OS
**Base Platform:** Kubuntu + KDE Plasma
**Target:** Linux Desktop

ZETTINGS is the next-generation system settings application for Zyntrix OS.

It is not a clone of Windows Settings, macOS System Settings, GNOME Settings, KDE System Settings, or any other existing settings application.

The product may learn from established operating-system settings applications, but its implementation, information architecture, visual language, interaction model, and engineering should be distinctly Zyntrix.

---

# 2. CURRENT TECHNOLOGY STACK

## Frontend

* Flutter
* Dart
* Flutter Desktop
* Material 3
* Material 3 theming
* Flutter accessibility and semantics
* Flutter animation framework

## Backend

* Rust
* Tokio
* zbus
* D-Bus
* PolicyKit / polkit
* systemd / logind

## Linux / KDE Integration

Where supported and required:

* NetworkManager
* PipeWire
* PulseAudio compatibility
* BlueZ
* UPower
* Power Profiles Daemon
* AccountsService
* systemd
* logind
* KScreen
* KWin
* KDE services
* PackageKit
* Flatpak
* Snap
* AppImage

The agent must verify actual platform APIs and capabilities before implementing integrations.

---

# 3. ARCHITECTURAL DIRECTION

The active application architecture is:

```text
Flutter UI
    ↓
Presentation / Application Layer
    ↓
Typed Application Services
    ↓
Rust Boundary
    ↓
D-Bus / Linux / KDE System Services
```

The architecture must maintain a clear separation between:

* presentation
* application logic
* domain logic
* system integration
* privileged operations
* persistence
* IPC

Flutter widgets must not become system-integration implementations.

Rust must not become a dumping ground for presentation logic.

System-facing functionality must remain independently testable.

---

# 4. OBSOLETE TECHNOLOGIES

The following technologies are **not part of the current architecture**:

* Qt
* QML
* cxx-qt
* Tauri
* React
* TypeScript
* Vite
* Tailwind CSS
* webview-based UI

Do not reintroduce them unless the user explicitly authorizes an architectural change.

Historical implementations may exist in archive locations. They are reference material only and must not silently influence the current architecture.

---

# 5. PROJECT CHARTER — prompt.txt

The repository root contains:

```text
prompt.txt
```

`prompt.txt` is the permanent product charter.

It defines:

* product vision
* engineering philosophy
* design goals
* UX expectations
* motion philosophy
* system integration goals
* accessibility expectations
* security expectations
* performance goals
* development phases
* quality standards

The complete `prompt.txt` remains authoritative for product intent.

This `AGENTS.md` defines how that intent must be executed inside the repository.

If an ambiguity exists:

1. explicit user requirement
2. safety/security requirements
3. this `AGENTS.md`
4. `prompt.txt`
5. project architecture/documentation
6. applicable skills
7. generic model assumptions

Never silently invent a resolution for a genuine architectural conflict.

---

# 6. NO DESIGN.md REQUIREMENT

`DESIGN.md` is not required.

Do not create or depend on `DESIGN.md` unless explicitly requested by the user.

The active design authority is:

```text
prompt.txt
+
Material 3
+
Zyntrix Design Language
+
repository theme/tokens
+
existing approved implementation
```

Visual decisions must remain centralized and consistent.

---

# 7. ZYNTRIX DESIGN LANGUAGE

The application uses an original design language called:

**Zyntrix Design Language — ZDL**

ZDL should feel:

* refined
* modern
* responsive
* dimensional
* calm
* intelligent
* precise
* distinctly Zyntrix

ZDL may take conceptual inspiration from excellent operating-system and application design, including:

* Windows
* macOS
* KDE
* GNOME
* Material Design
* Fluent Design
* Nothing OS
* Linear
* Raycast
* Notion
* other high-quality interfaces

However:

> **Do not copy another product's visual identity.**

Do not build a Windows clone.

Do not build a macOS clone.

Do not build a generic Material demo.

Material 3 is the implementation foundation; ZDL is the product identity.

---

# 8. MATERIAL 3

Material 3 is the default Flutter design system.

Use:

```dart
ThemeData(
  useMaterial3: true,
)
```

Prefer native Material 3 components when they satisfy the requirement.

Examples include:

* NavigationRail
* NavigationDrawer
* SearchBar
* SearchAnchor
* ListTile
* Card
* FilledButton
* FilledButton.tonal
* OutlinedButton
* TextButton
* Switch
* Checkbox
* Radio
* Slider
* SegmentedButton
* DropdownMenu
* Dialog
* MenuAnchor
* Tooltip
* SnackBar
* ProgressIndicator
* Chip
* Badge

Do not recreate standard Material controls without a demonstrated product requirement.

Custom ZDL components should compose Material 3 behavior where practical.

---

# 9. THEME ARCHITECTURE

Theme values must be centralized.

Use Flutter's:

* `ThemeData`
* `ColorScheme`
* `TextTheme`
* `ThemeExtension`
* Material 3 shape and typography systems

where appropriate.

Avoid scattering raw visual constants throughout feature widgets.

Prefer semantic values.

Bad:

```dart
Container(
  color: const Color(0xFF6C3BFF),
)
```

Better:

```dart
Container(
  color: Theme.of(context).colorScheme.primary,
)
```

Project-specific tokens should be represented through centralized theme infrastructure.

---

# 10. THEMES

The application should support, where applicable:

* Light
* Dark
* OLED-oriented dark
* dynamic accent configuration
* accessible contrast modes
* large-text configurations

Theme behavior must remain accessible and readable.

Never sacrifice readability merely to achieve a visual effect.

---

# 11. RESPONSIVE DESKTOP DESIGN

ZETTINGS is a desktop-first application.

It must support:

* maximized windows
* restored windows
* window resizing
* different desktop resolutions
* laptop displays
* large monitors
* different DPI/scaling configurations

Use Flutter layout systems such as:

* `LayoutBuilder`
* `MediaQuery`
* `Flexible`
* `Expanded`
* `ConstrainedBox`
* `Wrap`

Do not hard-code one screen resolution.

---

# 12. NAVIGATION

Navigation should use Material 3 patterns where appropriate.

Possible responsive behavior:

```text
Large window
    → NavigationRail / expanded navigation

Medium window
    → compact navigation

Small window
    → NavigationDrawer / compact layout
```

Navigation must preserve appropriate:

* selected location
* hierarchy
* back navigation
* keyboard focus
* scroll state
* deep-link state

---

# 13. SEARCH

Search is a core product capability.

The architecture should support:

* exact matching
* fuzzy matching
* synonyms
* keywords
* typo tolerance
* category matching
* descriptions
* recent settings
* frequently used settings
* keyboard navigation
* deep links
* contextual results

Search ranking must be deterministic and measurable.

Do not label a search system "AI-powered" unless an actual AI component is intentionally implemented and justified.

---

# 14. ACCESSIBILITY

Accessibility is a first-class requirement.

Support:

* keyboard-only navigation
* screen readers
* semantic labels
* logical focus traversal
* visible focus indicators
* large text
* text scaling
* high contrast
* sufficient color contrast
* accessible controls
* accessible feedback
* reduced motion

Use Flutter accessibility primitives such as:

```text
Semantics
Focus
FocusNode
FocusTraversalGroup
Shortcuts
Actions
```

where appropriate.

Visual appearance must never be the only indication of system state.

---

# 15. REDUCED MOTION

The application must respect reduced-motion preferences.

When reduced motion is active:

* minimize decorative animation
* reduce transition complexity
* remove unnecessary parallax
* reduce scaling effects
* avoid continuous motion

Functional transitions must remain understandable.

---

# 16. MOTION SYSTEM

Motion must have purpose.

Use Flutter's animation infrastructure appropriately:

* `AnimationController`
* `Tween`
* `CurvedAnimation`
* `TweenAnimationBuilder`
* `AnimatedSwitcher`
* `AnimatedContainer`
* `SpringSimulation`

Use spring physics and velocity-aware transitions when they materially improve the interaction.

Motion may include:

* continuity
* momentum
* anticipation
* overshoot
* settle
* contextual easing
* adaptive duration
* secondary motion

Do not animate everything.

Do not use animation merely to demonstrate technical capability.

---

# 17. MOTION QUALITY

Every animation must be evaluated for:

* responsiveness
* clarity
* accessibility
* performance
* visual hierarchy
* consistency

Avoid:

* excessive bounce
* slow transitions
* distracting movement
* unnecessary parallax
* continuous decorative animation
* motion that interferes with reading

Performance targets must be measured rather than assumed.

---

# 18. SYSTEM INTEGRATION

ZETTINGS must interact with the actual Linux/KDE system.

Never fabricate system state.

Potential areas include:

* display
* resolution
* scaling
* refresh rate
* HDR
* night light
* GPU
* color profiles
* audio
* microphone
* Bluetooth
* Wi-Fi
* Ethernet
* VPN
* firewall
* updates
* storage
* battery
* power
* power profiles
* users
* groups
* accessibility
* keyboard
* mouse
* touchpad
* tablet/stylus
* language
* regional settings
* notifications
* privacy
* security
* developer settings
* Wayland
* X11
* virtual desktops
* KWin
* window rules
* effects
* packages
* printers
* USB
* file associations
* default applications
* themes
* wallpapers
* accent colors
* cursors
* icons
* terminal configuration
* network shares
* remote desktop
* SSH
* containers
* virtual machines
* logs
* crash reports
* services
* system information
* CPU
* GPU
* memory
* kernel
* sensors
* thermals

Only implement capabilities actually supported by the target platform.

---

# 19. RUST SYSTEM BOUNDARY

Rust owns system-facing operations.

Flutter must not directly execute arbitrary privileged system commands.

The Flutter/Rust boundary should expose typed operations.

Each operation should clearly define:

```text
Request
Response
Error
Permission
Availability
Timeout
State synchronization
```

Avoid generic "execute command" APIs.

---

# 20. D-BUS

D-Bus communication should be:

* asynchronous
* typed where practical
* isolated from widgets
* reusable
* testable
* error-aware

Never put raw D-Bus logic directly inside Flutter presentation widgets.

---

# 21. SECURITY

Security is mandatory.

Use:

* least privilege
* capability-oriented permissions
* strict input validation
* safe IPC
* PolicyKit where appropriate
* explicit authorization
* secure error handling
* auditability where required

Never:

* execute arbitrary user-controlled commands
* interpolate untrusted input into shell commands
* trust UI state as authorization
* grant unnecessary privileges
* silently elevate privileges

Authorization belongs at the system-facing boundary.

---

# 22. PRIVILEGED OPERATIONS

Every privileged operation must:

1. determine the required capability;
2. validate the request;
3. verify authorization;
4. perform the operation;
5. verify the result;
6. return the actual result;
7. update application state.

Never assume an operation succeeded merely because no immediate exception occurred.

---

# 23. ERROR HANDLING

Errors must be explicit.

Where useful, distinguish:

```text
Unavailable
Unsupported
PermissionDenied
InvalidInput
Timeout
ServiceUnavailable
HardwareUnavailable
SystemError
```

User-facing errors should explain:

* what happened
* why it matters
* what can be done next

Technical diagnostics may be exposed separately.

---

# 24. EXTERNAL STATE

System state can change outside ZETTINGS.

Architecture must account for external changes where relevant:

```text
Linux/KDE system
       ↓
Rust integration
       ↓
Application state
       ↓
Flutter UI
```

Do not assume ZETTINGS is the only process changing settings.

---

# 25. ASYNC REQUIREMENTS

System operations must never block the Flutter UI.

Avoid:

* blocking the UI isolate
* synchronous long-running operations
* unnecessary polling
* excessive rebuilds

CPU-heavy or long-running work must remain outside the UI-critical path.

---

# 26. PERFORMANCE

Performance goals are measured targets.

Measure where applicable:

* cold launch
* hot launch
* navigation latency
* search latency
* memory usage
* frame performance
* widget rebuilds
* backend latency
* D-Bus latency
* system-service latency

Never claim performance without measurement.

Optimize based on profiling evidence.

---

# 27. FLUTTER PERFORMANCE

Avoid:

* unnecessary widget rebuilds
* enormous widget trees
* uncontrolled listeners
* excessive animations
* expensive rendering effects
* unnecessary polling
* unnecessary image decoding
* synchronous system operations on the UI isolate

Use lazy construction for large settings collections.

---

# 28. DEPENDENCY POLICY

Every dependency requires justification.

Before adding one:

1. check whether Flutter/Dart already provides the capability;
2. check existing project dependencies;
3. evaluate maintenance;
4. evaluate licensing;
5. evaluate security;
6. evaluate platform compatibility;
7. evaluate performance;
8. document non-obvious reasons.

Do not add libraries merely for convenience.

---

# 29. NO PLACEHOLDER IMPLEMENTATIONS

Never claim a feature is implemented when it contains:

```text
TODO
FIXME
placeholder
dummy backend
fake state
mock data
coming soon
pseudo-code
unfinished implementation
```

Mocks are permitted only inside tests or explicitly identified development tooling.

Production UI must never silently substitute fake system state.

---

# 30. ARCHITECTURE QUALITY

Avoid:

* god classes
* god widgets
* god services
* circular dependencies
* duplicated domain logic
* duplicated IPC contracts
* duplicated theme values
* feature-specific system hacks

Keep features modular and independently understandable.

---

# 31. FEATURE STRUCTURE

Where appropriate, features may follow:

```text
features/
    feature_name/
        presentation/
        application/
        domain/
        data/
```

Do not force this structure onto trivial features.

Architecture should serve the product rather than become ceremony.

---

# 32. TESTING

Testing should exist at appropriate levels.

## Dart

Use:

* unit tests
* widget tests
* integration tests

where applicable.

## Rust

Use:

* unit tests
* integration tests
* system integration tests

where appropriate.

## System integrations

Use mocks for deterministic unit testing and real Linux/KDE environments for actual integration verification.

---

# 33. VERIFICATION

Never claim a check passed unless it was actually run and observed to pass.

Minimum applicable Flutter checks:

```bash
dart format --output=none --set-exit-if-changed .
flutter analyze
flutter test
```

Linux build verification:

```bash
flutter build linux
```

When Rust exists:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo check --workspace
cargo test --workspace
```

Additional feature-specific checks must be executed where applicable.

---

# 34. RUNTIME VERIFICATION

Compilation does not prove runtime correctness.

Distinguish:

```text
Compiled
Analyzed
Unit-tested
Integration-tested
Runtime-tested
```

A Linux-specific feature must be runtime-tested on an appropriate Linux environment before claiming that the integration works.

If the environment cannot perform a required test, state that explicitly.

---

# 35. PACKAGE VERIFICATION

When packaging is affected, verify the actual package.

Potential targets include:

* `.deb`
* AppImage
* Flatpak

Do not claim packaging support merely because configuration files exist.

---

# 36. PLAN.md

`PLAN.md` is the active implementation roadmap.

For substantial work it should contain:

* objectives
* architecture
* implementation phases
* dependencies
* milestones
* verification gates
* risks
* acceptance criteria

The plan must describe the current Flutter + Rust architecture.

Do not preserve obsolete Qt/QML/Tauri architecture in the active plan.

---

# 37. PHASE-GATED DEVELOPMENT

Major development should follow:

```text
Phase 0 — Research
        ↓
Phase 1 — Architecture
        ↓
Phase 2 — Material 3 / ZDL Design System
        ↓
Phase 3 — Motion System
        ↓
Phase 4 — Core Application Framework
        ↓
Phase 5 — Rust / Linux Integration
        ↓
Phase 6 — Flutter Application Shell
        ↓
Phase 7 — Feature Modules
        ↓
Phase 8 — Testing
        ↓
Phase 9 — Performance Optimization
        ↓
Phase 10 — Packaging
        ↓
Phase 11 — Documentation
        ↓
Phase 12 — Release
```

Each major phase should satisfy its acceptance criteria before dependent work proceeds.

Small isolated fixes may use proportional planning.

---

# 38. WEB RESEARCH

When the task requires current information, unfamiliar technology, platform APIs, or specialized implementation guidance, the agent must research authoritative sources.

Prefer:

1. official documentation
2. official project repositories
3. upstream specifications
4. authoritative engineering documentation
5. reputable technical sources

Do not rely on stale assumptions for rapidly changing technologies.

---

# 39. SKILL SYSTEM

Skills are specialized knowledge/procedure modules used by the single active agent.

They are not sub-agents.

The agent must use the smallest complete set of applicable skills.

Before specialized work:

```text
Identify concern
      ↓
Find applicable skill
      ↓
Load SKILL.md
      ↓
Confirm coverage
      ↓
Work
```

---

# 40. AUTOMATIC SKILL DISCOVERY

If a necessary skill is not installed locally, **do not immediately stop**.

The agent must:

```text
Local skill search
      ↓
Web research
      ↓
Candidate evaluation
      ↓
Security review
      ↓
Download
      ↓
Install
      ↓
Read SKILL.md
      ↓
Confirm applicability
      ↓
Continue
```

The agent should acquire suitable skills automatically when safe and technically possible.

---

# 41. SKILL WEB RESEARCH

When researching a missing skill, evaluate:

* source reputation
* maintenance
* supported technology/version
* documentation quality
* scope
* overlap
* license
* security
* compatibility
* usefulness

Prefer:

* official skill repositories
* official project repositories
* well-maintained open-source skill collections
* reputable technical organizations

Avoid:

* unknown download sites
* suspicious repositories
* executable installers
* untrusted scripts
* obfuscated content

---

# 42. SKILL SECURITY REVIEW

Downloaded skills must be treated as untrusted content until inspected.

Before installation, inspect for:

* arbitrary command execution
* credential collection
* data exfiltration
* destructive commands
* privilege escalation
* hidden binaries
* obfuscated scripts
* unexpected dependency installation
* unrelated filesystem modifications

Prefer skills consisting primarily of:

```text
SKILL.md
documentation
references
templates
safe helper files
```

If a skill appears malicious or unsafe:

**DO NOT INSTALL IT.**

Find another candidate.

---

# 43. SKILL INSTALLATION

Install acquired skills using the repository/environment's established skill mechanism.

The preferred local location is:

```text
.opencode/skills/
```

unless the environment specifies another authoritative location.

Do not blindly execute third-party installation scripts.

Prefer downloading and installing the required skill files directly.

---

# 44. SKILL PROVENANCE

When practical, retain:

```text
Skill name
Source repository
Source URL
Version or commit
Acquisition date
Purpose
```

Do not create redundant metadata if the skill-management system already records this information.

---

# 45. SKILL DUPLICATION

Before installing a skill:

1. search installed skills;
2. inspect their scope;
3. compare responsibilities;
4. determine whether the new skill materially duplicates an existing skill.

Do not install redundant skills simply because they have different names.

If two skills genuinely cover different layers, document their boundaries internally.

---

# 46. SKILL CONFLICTS

When skills disagree:

1. explicit user requirements
2. security requirements
3. `AGENTS.md`
4. `prompt.txt`
5. project architecture
6. most specific applicable skill
7. broader skill
8. generic model knowledge

Never combine contradictory rules into an arbitrary hybrid.

If authority cannot be determined:

**STOP and resolve the conflict.**

---

# 47. MISSING SKILL HARD STOP

Only stop after reasonable discovery and acquisition attempts have failed.

Report:

```text
Required domain
Research performed
Sources evaluated
Candidates rejected
Reason no suitable skill could be acquired
Required next step
```

Do not silently replace a missing specialized skill with generic knowledge.

---

# 48. USER APPROVAL FOR EXTERNAL CHANGES

The agent may automatically acquire safe repository-local skills.

The agent must request user approval before:

* installing system-wide software
* modifying system packages
* executing privileged commands
* accessing credentials
* changing external accounts
* making destructive system changes
* accepting incompatible licenses
* modifying files outside the repository when unnecessary

Never use `apt` or `apt-get` automatically.

---

# 49. REPOSITORY SCOPE

Keep changes scoped.

Do not modify unrelated files simply because they could be improved.

Adjacent changes are justified only when required for:

* correctness
* security
* buildability
* architecture
* dependency resolution
* testing
* requested functionality

---

# 50. GIT HYGIENE

Keep commits focused.

Avoid mixing unrelated:

* features
* refactors
* dependency changes
* formatting changes
* generated files

Do not delete historical work without authorization.

---

# 51. UI DEFINITION OF DONE

A significant UI component should be reviewed for:

```text
[ ] Material 3 consistency
[ ] ZDL consistency
[ ] Light theme
[ ] Dark theme
[ ] OLED behavior where appropriate
[ ] High contrast
[ ] Large text
[ ] Keyboard navigation
[ ] Focus state
[ ] Hover state
[ ] Pressed state
[ ] Disabled state
[ ] Loading state
[ ] Error state
[ ] Responsive behavior
[ ] Reduced motion
[ ] Screen-reader semantics
[ ] Performance
```

---

# 52. FEATURE DEFINITION OF DONE

A feature is complete only when applicable:

```text
[ ] Requirements implemented
[ ] Architecture respected
[ ] Existing components reused
[ ] Real system integration implemented
[ ] Errors handled
[ ] Permissions handled
[ ] External state considered
[ ] Accessibility implemented
[ ] Tests added/updated
[ ] Formatting passed
[ ] Static analysis passed
[ ] Build passed
[ ] Runtime integration verified
[ ] Documentation updated
```

---

# 53. ABSOLUTE QUALITY RULES

Never:

* fabricate verification results
* fabricate system state
* hide errors
* bypass authorization
* execute arbitrary commands
* introduce unexplained dependencies
* duplicate architecture
* create unnecessary abstractions
* sacrifice accessibility for aesthetics
* sacrifice correctness for animation
* sacrifice performance for decoration
* claim unsupported platform capabilities
* silently ignore failed operations

---

# 54. FINAL CHECKLIST

## Before work

```text
[ ] Read AGENTS.md
[ ] Read relevant prompt.txt requirements
[ ] Understand the request
[ ] Identify technical concerns
[ ] Search installed skills
[ ] Research missing skills if necessary
[ ] Acquire applicable skills
[ ] Inspect acquired skills
[ ] Load required SKILL.md files
[ ] Confirm skill coverage
```

## Before implementation

```text
[ ] Inspect current architecture
[ ] Inspect relevant components
[ ] Inspect theme infrastructure
[ ] Inspect affected system integrations
[ ] Inspect tests
[ ] Inspect dependencies
[ ] Determine verification requirements
[ ] Update PLAN.md if substantial
```

## Before completion

```text
[ ] Implementation complete
[ ] No placeholder production code
[ ] No fabricated system state
[ ] Dart formatting passed
[ ] Flutter analyze passed
[ ] Flutter tests passed
[ ] Rust formatting passed where applicable
[ ] Rust clippy passed where applicable
[ ] Rust checks/tests passed where applicable
[ ] Linux build verified where applicable
[ ] Runtime integration verified where applicable
[ ] Accessibility reviewed
[ ] Performance reviewed
[ ] Documentation updated
[ ] No unverified claims
```

---

# 55. ABSOLUTE OPERATING RULE

> **DO NOT BEGIN SPECIALIZED REPOSITORY WORK UNTIL THE NECESSARY SKILLS HAVE BEEN IDENTIFIED AND LOADED.**

If a required skill is missing:

> **RESEARCH IT.**

If a suitable skill exists online:

> **ACQUIRE IT.**

If it is safe and compatible:

> **INSTALL IT.**

Then:

> **READ IT, VERIFY COVERAGE, AND PROCEED.**

Only stop when reasonable discovery and acquisition have genuinely failed.

ZETTINGS must be developed as one coherent, production-grade **Flutter + Rust + Linux/KDE system**, with Material 3 as its UI foundation and Zyntrix Design Language as its product identity.
