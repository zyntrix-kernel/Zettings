# ZETTINGS — Implementation Plan

**Project:** Zyntrix OS Settings
**Platform:** Zyntrix OS / Kubuntu + KDE Plasma
**Frontend:** Flutter + Dart + Material 3
**System Backend:** Rust
**IPC:** Secure typed IPC / D-Bus
**Status:** Planning

---

# 1. PURPOSE

This document defines the implementation plan for ZETTINGS.

ZETTINGS is a production-grade desktop settings application for Zyntrix OS.

The project must be developed incrementally.

Do not build hundreds of disconnected settings pages first.

Establish the architecture, design system, system abstraction layer, navigation, search, and core integrations before expanding into the full settings surface.

---

# 2. CORE ARCHITECTURE

The application follows this architecture:

```text
┌─────────────────────────────────────────┐
│              ZETTINGS UI                │
│                                         │
│ Flutter + Dart + Material 3             │
│ Zyntrix Design Language                 │
└───────────────────┬─────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│         Application / Domain Layer      │
│                                         │
│ Settings Registry                        │
│ Routing                                  │
│ Search                                   │
│ State                                    │
│ Preferences                              │
│ Feature Services                         │
└───────────────────┬─────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│          System Abstraction Layer       │
│                                         │
│ DisplayService                           │
│ NetworkService                           │
│ AudioService                             │
│ PowerService                             │
│ BluetoothService                         │
│ AccountService                           │
│ PackageService                           │
│ SystemInfoService                        │
└───────────────────┬─────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│              Rust Backend               │
│                                         │
│ Typed IPC                                │
│ Permissions                              │
│ D-Bus                                    │
│ Linux APIs                               │
│ KDE APIs                                 │
└───────────────────┬─────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│              Linux System               │
│                                         │
│ KDE Plasma                               │
│ KWin                                     │
│ NetworkManager                            │
│ PipeWire                                 │
│ BlueZ                                    │
│ systemd                                  │
│ UPower                                   │
│ AccountsService                           │
│ PolicyKit                                │
│ etc.                                     │
└─────────────────────────────────────────┘
```

---

# 3. DEVELOPMENT PRINCIPLES

The following principles are mandatory.

## 3.1 Material 3 First

Material 3 is the foundation.

Prefer existing Flutter Material 3 components.

Custom components are allowed only when:

* Material 3 has no suitable component
* a desktop-specific adaptation is required
* the component provides genuine product value

Do not recreate standard Material controls unnecessarily.

---

## 3.2 Feature First

Organize functionality by feature rather than technical type.

Preferred:

```text
features/
  display/
  network/
  audio/
  power/
```

rather than:

```text
screens/
services/
controllers/
widgets/
```

containing unrelated functionality.

---

## 3.3 Backend Independence

Flutter must not know how Linux implements a setting.

Example:

```text
Flutter
   ↓
DisplayService
   ↓
DisplayRepository
   ↓
Rust IPC
   ↓
KScreen / Wayland / KDE APIs
```

The UI should not contain D-Bus calls or shell commands.

---

## 3.4 Real Functionality Only

Never create fake controls.

If a system API is unavailable:

* detect it
* represent the capability correctly
* explain the limitation

Do not make a switch appear functional when it is not.

---

# 4. PHASE 0 — REPOSITORY AUDIT

## Objectives

Before changing code:

* inspect the repository
* inspect existing Flutter configuration
* inspect existing Rust/backend code
* inspect dependencies
* inspect assets
* inspect documentation
* identify existing functionality
* identify broken functionality
* identify technical debt

## Deliverables

```text
RESEARCH.md
ARCHITECTURE.md
```

## Acceptance Criteria

The agent can explain:

* current architecture
* current dependencies
* existing features
* platform constraints
* build process
* testing process
* known problems

No major implementation should begin before this phase is understood.

---

# 5. PHASE 1 — ARCHITECTURE FOUNDATION

## Objectives

Establish the application architecture.

Implement:

* application bootstrap
* dependency injection
* routing
* feature boundaries
* service interfaces
* repository interfaces
* error model
* system capability model
* logging
* configuration

## Core concepts

Create explicit abstractions for:

```text
SystemCapability
SystemError
PermissionState
ServiceAvailability
SettingDefinition
SettingCategory
SettingRoute
```

## Acceptance Criteria

The application can:

* launch
* navigate
* resolve routes
* inject services
* handle errors
* determine system capability availability

without implementing the entire settings catalog.

---

# 6. PHASE 2 — MATERIAL 3 DESIGN SYSTEM

## Objectives

Create the Zyntrix visual foundation on top of Material 3.

Implement:

* ThemeData
* ColorScheme
* typography
* spacing
* shape system
* elevation
* navigation components
* setting rows
* section headers
* cards
* dialogs
* menus
* loading states
* error states
* empty states

## Material 3 requirements

Use:

```dart
ThemeData(
  useMaterial3: true,
)
```

Centralize theme configuration.

Do not hard-code component colors.

Use semantic Material color roles.

---

# 7. PHASE 3 — ZDL

## Objectives

Define:

**Zyntrix Design Language**

ZDL extends Material 3.

Define:

* desktop spacing
* density
* surface hierarchy
* icon sizing
* navigation behavior
* settings-page composition
* accent behavior
* subtle translucency
* motion conventions

## Deliverables

```text
DESIGN.md
```

and reusable Flutter components.

## Acceptance Criteria

Every major screen can use the same visual primitives.

No page should need its own independent visual language.

---

# 8. PHASE 4 — APPLICATION SHELL

Build the primary desktop shell.

Required:

* application window layout
* navigation rail/drawer
* content area
* page title
* breadcrumbs where useful
* search
* responsive resizing
* keyboard navigation
* theme switching

Target structure:

```text
┌─────────────────────────────────────────────┐
│ Search / global actions                     │
├──────────────┬──────────────────────────────┤
│              │                              │
│ Navigation   │       Page Content           │
│              │                              │
│ Categories   │                              │
│              │                              │
└──────────────┴──────────────────────────────┘
```

The exact layout may evolve after usability testing.

---

# 9. PHASE 5 — ROUTING

Implement centralized routing.

Every settings page must have:

* stable route
* title
* category
* searchable metadata
* deep-link support

Example:

```text
zettings://display
zettings://network/wifi
zettings://audio/output
zettings://appearance/theme
```

Navigation must support:

* forward
* back
* direct navigation
* search navigation
* deep links

---

# 10. PHASE 6 — SETTINGS REGISTRY

Create the central settings registry.

Each entry should contain:

```text
id
title
description
category
subcategory
route
icon
keywords
synonyms
searchWeight
requiredCapabilities
availability
```

The registry becomes the source of truth for:

* navigation
* search
* deep links
* indexing
* breadcrumbs
* contextual recommendations

Do not duplicate this metadata across screens.

---

# 11. PHASE 7 — SEARCH

Implement local indexed search.

## Search pipeline

```text
Query
 ↓
Normalization
 ↓
Tokenization
 ↓
Exact matching
 ↓
Prefix matching
 ↓
Fuzzy matching
 ↓
Synonym matching
 ↓
Ranking
 ↓
Results
```

Ranking should consider:

1. exact title match
2. title prefix
3. keywords
4. category
5. description
6. synonyms
7. recent usage
8. frequency

Search must remain responsive with a large settings registry.

---

# 12. PHASE 8 — RUST IPC FOUNDATION

Build the system backend communication layer.

Define typed commands and responses.

Example:

```text
get_system_info
get_display_configuration
get_network_interfaces
get_audio_devices
get_power_state
get_bluetooth_devices
```

Do not create a generic:

```text
execute_command
```

interface.

IPC must validate:

* command type
* arguments
* permissions
* response type
* errors

---

# 13. PHASE 9 — SYSTEM INFORMATION

Implement the first real system integration.

Expose:

* OS
* kernel
* desktop environment
* CPU
* GPU
* memory
* storage
* displays
* firmware
* display server

This feature is intentionally first because it establishes:

* hardware discovery
* backend communication
* caching
* error handling
* capability detection

---

# 14. PHASE 10 — DISPLAY

Implement:

* connected displays
* resolution
* refresh rate
* scaling
* orientation
* primary display
* arrangement

Where supported:

* HDR
* color profiles
* night-light integration

Handle unsupported capabilities explicitly.

---

# 15. PHASE 11 — NETWORK

Implement:

* Wi-Fi
* Ethernet
* VPN
* connection status
* network interfaces
* DNS where supported
* proxy where supported

Use NetworkManager APIs where appropriate.

Do not implement network management using arbitrary shell commands.

---

# 16. PHASE 12 — AUDIO

Implement:

* output devices
* input devices
* volume
* mute
* default devices
* application audio where practical
* microphone status

Use PipeWire as the primary modern Linux audio architecture.

Maintain compatibility where required.

---

# 17. PHASE 13 — POWER

Implement:

* battery status
* charging
* power source
* sleep
* screen timeout
* power profiles
* performance/balanced/power-saving modes where supported

Integrate with:

* UPower
* Power Profiles Daemon
* logind

---

# 18. PHASE 14 — BLUETOOTH

Implement:

* Bluetooth state
* adapter discovery
* device discovery
* pairing
* connection
* disconnection
* device information

Use BlueZ/D-Bus.

Handle unavailable adapters gracefully.

---

# 19. PHASE 15 — ACCOUNTS

Implement:

* current user
* users
* user information
* account state
* groups where appropriate
* login/session configuration where supported

Use AccountsService and appropriate system APIs.

Never expose sensitive account information unnecessarily.

---

# 20. PHASE 16 — PERSONALIZATION

Implement:

* appearance
* theme
* accent
* wallpaper
* icons
* cursor
* fonts
* animations
* desktop appearance

Integrate with KDE configuration mechanisms.

Do not modify configuration files blindly.

---

# 21. PHASE 17 — ACCESSIBILITY

Implement:

* accessibility settings
* text scaling
* high contrast
* reduced motion
* pointer settings
* keyboard accessibility
* screen-reader-compatible semantics

The application itself must remain accessible while configuring accessibility.

---

# 22. PHASE 18 — STORAGE & APPLICATIONS

Implement:

* storage overview
* mounted filesystems
* disk information
* application management where supported
* default applications
* file associations

Then integrate:

* Flatpak
* Snap
* AppImage

Only expose functionality actually supported by each package system.

---

# 23. PHASE 19 — PRIVACY & SECURITY

Implement:

* application permissions where available
* firewall status/control where supported
* screen capture permissions
* privacy status
* security information

Avoid pretending Linux has a single universal permissions model.

Expose platform-specific capabilities accurately.

---

# 24. PHASE 20 — UPDATES

Implement system update functionality using the appropriate package/update APIs.

Provide:

* update status
* available updates
* update history where available
* repository status
* firmware updates where supported

Never perform updates without explicit user confirmation.

---

# 25. PHASE 21 — KDE / WINDOW MANAGEMENT

Implement appropriate KDE-specific controls:

* KWin
* virtual desktops
* window behavior
* window rules
* effects
* compositor configuration
* workspace behavior

Only expose settings that can be reliably controlled.

---

# 26. PHASE 22 — DEVELOPER & ADVANCED SETTINGS

Implement:

* Wayland/X11 information
* debugging
* logs
* system services
* environment information
* kernel information
* developer options

Advanced functionality must be clearly separated from normal user settings.

---

# 27. PHASE 23 — ERROR & RECOVERY SYSTEM

Create a consistent error framework.

Every system operation must handle:

```text
Success
PermissionDenied
Unsupported
Unavailable
Timeout
BackendError
InvalidInput
HardwareUnavailable
ServiceUnavailable
```

UI should provide:

* clear explanation
* recovery action where possible
* retry
* technical diagnostics where useful

---

# 28. PHASE 24 — ACCESSIBILITY AUDIT

Perform a dedicated accessibility pass.

Verify:

* semantic labels
* focus order
* keyboard operation
* focus visibility
* text scaling
* contrast
* reduced motion
* screen reader behavior

Do not treat accessibility testing as part of final cleanup only.

---

# 29. PHASE 25 — PERFORMANCE AUDIT

Measure before optimizing.

Measure:

* cold startup
* warm startup
* navigation latency
* search latency
* frame performance
* memory
* backend latency
* rebuild counts

Use Flutter profiling tools.

Investigate:

* unnecessary rebuilds
* expensive layouts
* oversized widget trees
* synchronous system operations
* excessive animations
* unnecessary backend calls

---

# 30. PHASE 26 — TESTING

Maintain tests throughout development.

## Unit tests

Test:

* models
* search
* ranking
* repositories
* services
* state logic

## Widget tests

Test:

* settings controls
* navigation
* dialogs
* error states
* accessibility semantics

## Integration tests

Test:

* real navigation
* backend communication
* system changes
* permissions
* failure states

---

# 31. PHASE 27 — PACKAGING

Prepare:

* Debian package
* Flatpak
* AppImage where appropriate

Verify:

* desktop entry
* icon
* application metadata
* permissions
* dependencies
* upgrades
* uninstall

Test installation from a clean system.

---

# 32. PHASE 28 — DOCUMENTATION

Finalize:

```text
README.md
ARCHITECTURE.md
DESIGN.md
BACKEND.md
SECURITY.md
ACCESSIBILITY.md
TESTING.md
CONTRIBUTING.md
CHANGELOG.md
```

Documentation must correspond to the actual implementation.

---

# 33. PHASE 29 — RELEASE CANDIDATE

Before release:

* clean build succeeds
* all tests pass
* packaging succeeds
* application launches correctly
* navigation works
* search works
* themes work
* keyboard navigation works
* accessibility works
* permissions work
* backend failures are handled
* unsupported hardware is handled
* no debug UI remains
* no fake functionality remains
* logs contain no unexpected errors

---

# 34. FEATURE COMPLETION CHECKLIST

A feature may only be marked:

**COMPLETE**

when all are true:

```text
[ ] Architecture defined
[ ] Backend implemented
[ ] IPC implemented
[ ] Real Linux integration verified
[ ] UI implemented
[ ] Material 3 compliant
[ ] Light theme tested
[ ] Dark theme tested
[ ] Keyboard navigation tested
[ ] Accessibility tested
[ ] Error states implemented
[ ] Permission states implemented
[ ] Unsupported states implemented
[ ] Unit tests written
[ ] Widget tests written
[ ] Integration tests written where appropriate
[ ] Performance measured
[ ] Documentation updated
```

---

# 35. DEFINITION OF PROJECT COMPLETION

ZETTINGS is complete only when:

```text
Core architecture
        +
Material 3 design system
        +
Navigation
        +
Search
        +
Deep links
        +
Rust backend
        +
Linux/KDE integration
        +
Feature modules
        +
Accessibility
        +
Security
        +
Testing
        +
Performance validation
        +
Packaging
        +
Documentation
```

are all production-ready.

---

# 36. AGENT EXECUTION RULE

The implementation agent must follow this order:

```text
Inspect
  ↓
Research
  ↓
Plan
  ↓
Architect
  ↓
Implement
  ↓
Test
  ↓
Profile
  ↓
Review
  ↓
Document
  ↓
Mark complete
```

Never:

```text
Design random screen
→
add fake data
→
move to next screen
```

---

# 37. PHASE GATE

A phase may only be considered complete when:

1. Its implementation exists.
2. Its tests pass.
3. Its documented acceptance criteria are satisfied.
4. No known critical defects remain.
5. Existing functionality still works.
6. The implementation does not violate the architecture.

If a phase fails its gate, fix it before proceeding.

---

# 38. PRIORITY ORDER

When trade-offs occur, prioritize:

```text
1. Security
2. Correctness
3. System integration
4. Accessibility
5. Reliability
6. Maintainability
7. Performance
8. UX
9. Visual polish
10. Decorative effects
```

A beautiful incorrect setting is worse than a plain correct setting.

---

# 39. FINAL PRINCIPLE

ZETTINGS should not win by having the most effects, the most screens, or the most code.

It should win through:

**clarity + correctness + native integration + accessibility + performance + thoughtful design.**

Material 3 provides the foundation.

Zyntrix provides the identity.

Flutter provides the UI platform.

Rust provides the secure systems boundary.

KDE and Linux provide the underlying system.

ZETTINGS connects them into one coherent experience.
