# PLAN.md — ZETTINGS Enterprise 13-Phase Master Roadmap

> **Target Platform:** Zyntrix OS (Kubuntu 24.04 LTS / KDE Plasma 5.27 / Frameworks 6)[cite: 1]
> **Architecture:** Tauri v2 + Rust 1.97 Workspace + React 19 + TypeScript + Tailwind v4[cite: 1]
> **Design Language:** Zyntrix Design Language (ZDL) — G2/G3 Continuous Squircles, Liquid Glass, Spring Physics Engine[cite: 1]

---

## Strict Verification Gate Protocol
Every phase must pass all four gate commands before landing the phase commit[cite: 1]:
1. `cargo fmt --all --check`[cite: 1]
2. `cargo clippy --workspace --all-targets -- -D warnings`[cite: 1]
3. `cargo check --workspace`[cite: 1]
4. `pnpm -r typecheck`[cite: 1]

---

## Phase 0: Research & System Architectural Benchmarking (COMPLETED)[cite: 1, 2]
- [x] **Competitive Research:** Deep-dive analysis of navigation graph, search ranking, deep linking, and caching in Windows 11 Settings, macOS System Settings, GNOME Settings, and KDE System Settings[cite: 1].
- [x] **Integration Matrix:** Mapped DBus system service interfaces (`zbus`) for NetworkManager, PipeWire, PulseAudio, BlueZ, UPower, AccountsService, KScreen, KWin, systemd, logind, and PolicyKit[cite: 1].
- [x] **Performance Benchmarks:** Established target metrics: Cold start <500ms, Hot start <150ms, Idle RAM <150MB, 120 FPS compositor animation budget[cite: 1].

---

## Phase 1: Architecture, Crate Workspace DAG & IPC Pipeline (COMPLETED)[cite: 1, 2]
- [x] **Cargo & pnpm Workspace Setup:** Pinned Rust 1.97 and pnpm 11 workspace DAG (`apps/zettings`, 7 core crates, `packages/ts-bindings`)[cite: 1, 2].
- [x] **Core Crates Skeleton:** Implemented `zsettings-core`, `zsettings-bus`, `zettings-ipc`, `zettings-polkit`, `zettings-plugin-sdk`, `zettings-palette`, and `zettings-search`[cite: 1, 2].
- [x] **IPC Binding Pipeline:** Configured `ts-rs` v12 payload derivation exporting typed interfaces to `@zettings/bindings`[cite: 1, 2].
- [x] **Mock Execution Layer:** Created `zsettings-mock` feature flag enabling full state-machine mock backend execution on Windows development hosts[cite: 1].
- [x] **Capabilities & Assets:** Resolved Tauri ACL capabilities (`core:webview:allow-internal-toggle-devtools`) and generated PNG/ICO assets[cite: 2].
- [x] **Architecture Diagrams:** Authored 5 Mermaid system diagrams under `docs/architecture/`[cite: 1, 2].

**Gate Commit:** `git commit -m "phase(1): complete architecture scaffold and crate workspace"`[cite: 1]

---

## Phase 2: Zyntrix Design Language (ZDL) Specification & Token Cascade (COMPLETED)
- [x] **Primitive & Semantic Design Tokens:** Create CSS variable system for surface elevation, translucent glass tints, specular edge highlights, and wallpaper accent colors[cite: 2].
- [x] **Continuous Curvature Math Engine:** Implement G2 ($n=4$) and G3 ($n=6$) superellipse squircle clip-path generators in React/TypeScript[cite: 1].
- [x] **Liquid Glass Material Composition:** Construct multi-layered CSS backdrop-filter panels (`blur(24px)`, `saturate(180%)`) with dynamic contrast adjustments[cite: 2].
- [x] **Tailwind v4 `@theme` Integration:** Map ZDL primitive tokens directly into Tailwind v4 workspace configurations[cite: 2].
- [x] **Tauri Decorum Shell Window:** Integrate custom transparent window frame with native window controls[cite: 2].

**Gate Commit:** `git commit -m "phase(2): build ZDL token cascade and liquid glass visual shell"`[cite: 1]

---

## Phase 3: 120 FPS Motion Engine & Physics Solvers (COMPLETED)
- [x] **Spring Physics Solver:** Build analytical and RK4 spring-damper solvers (`stiffness`, `damping`, `mass`, `velocity`) running on compositor thread[cite: 1].
- [x] **Continuous Curvature Transitions:** Implement page route transition hooks maintaining velocity across tab changes[cite: 1].
- [x] **Micro-Interaction System:** Build spring-animated sliders, toggle switches, expanders, and press-feedback cards with physical weight[cite: 1].
- [x] **Reduced Motion Adaptive Fallbacks:** Implement automatic fallback to opacity cross-fades when system `prefers-reduced-motion` is active[cite: 1].

**Gate Commit:** `git commit -m "phase(3): build 120 FPS spring motion engine and physics solvers"`[cite: 1]

---

## Phase 4: Core Framework, Plugin SDK & Tokio Message Bus `(COMPLETED)`
- [x] **`zsettings-bus` Event Router:** Build async tokio broadcast channel for live system event streaming (e.g., link state, audio volume, display re-plug)[cite: 1, 2].
- [x] **`zsettings-core` Plugin Registry:** Implement dynamic plugin discovery, `ring` ed25519 signature verification, and manifest parsing[cite: 1, 2].
- [x] **`zsettings-plugin-sdk` Capability ACLs:** Enforce capability boundaries on external settings modules[cite: 1, 2].
- [x] **`zsettings-polkit` Auth Gateway:** Implement PolicyKit privilege check pipeline returning structured Authorization/Challenge/Denied responses[cite: 1, 2].

**Gate Commit:** `git commit -m "phase(4): build tokio message bus, plugin loader, and polkit authorization gateway"`[cite: 1]

---

## Phase 5: Deep System Backend Integration (`zbus` & Linux Services)
- [x] **Display (`zsettings-display`):** `zbus` bindings to `org.kde.KScreen` for resolution, refresh rate, scaling, and night color management[cite: 1, 2].
- [x] **Audio (`zsettings-audio`):** `libpulse-binding` and `pipewire` hooks for per-application volume sliders, stream routing, and device selection[cite: 1, 2].
- [x] **Network & Bluetooth (`zsettings-network`, `zsettings-bluetooth`):** `zbus` bindings to `NetworkManager` and `BlueZ` for Wi-Fi scanning, VPN, and device pairing[cite: 1, 2]. _(Bluetooth/VPN deferred — `zettings-network` covers hostname + Wi-Fi scan; `BlueZ` merges into network per AGENTS.md "Network & Bluetooth Engineer".)_
- [x] **Power & Battery (`zsettings-power`):** `UPower` and `power-profiles-daemon` hooks for charge thresholds and performance profile switching[cite: 1, 2].
- [ ] **Accounts & System (`zsettings-accounts`):** `AccountsService` and `logind` integration for user profile management and session control[cite: 1, 2]. _(Deferred to Phase 6+ — no `zettings-bus` event type requires it yet.)_

**Gate Commit:** `git commit -m "phase(5): implement zbus linux service integration across domain crates"`[cite: 1]

---

## Phase 6: Frontend Shell, Spotlight Search & Navigation Graph
- [x] **In-Memory Tantivy Index (`zsettings-search`):** Build schema-driven search index for settings options, sub-menus, keywords, and aliases[cite: 1, 2].
- [x] **Sub-5ms Fuzzy Search:** Combine Tantivy with `strsim` Levenshtein distance for typo-tolerant query results over IPC[cite: 1, 2].
- [x] **Spotlight Modal Component:** Build Apple-inspired centered glass search overlay triggered by global shortcut (`Super+I` / `Ctrl+Space`)[cite: 1, 2].
- [x] **Breadcrumbs & Settings Graph Navigation:** Implement deep-linking router capability that highlights target controls upon navigation[cite: 1, 2].

**Gate Commit:** `git commit -m "phase(6): build frontend shell, deep search engine, and navigation graph"`[cite: 1]

---

## Phase 7: Domain Feature Modules (UI Panels)
- [ ] **Display & Monitor Canvas Panel:** Interactive drag-and-arrange monitor arrangement canvas with snapping alignment[cite: 1].
- [ ] **Sound Mixer & Equalizer Panel:** Per-app audio mixer cards with live VU volume meters[cite: 1].
- [ ] **Network & Wi-Fi Panel:** Access point connection lists with signal strength meters and security credential prompts[cite: 1].
- [ ] **Bluetooth & Peripherals Panel:** Paired device management cards with battery percentage indicators[cite: 1].
- [ ] **Power & Performance Panel:** Interactive battery health discharge graphs and power profile toggles[cite: 1].
- [ ] **Personalization & ZDL Theme Panel:** Dynamic wallpaper accent color picker, squircle roundness sliders, and blur controls[cite: 1].

**Gate Commit:** `git commit -m "phase(7): construct complete domain settings feature modules"`[cite: 1]

---

## Phase 8: Comprehensive Testing Suite & Security Audit
- [ ] **Backend Unit & Integration Tests:** Run `cargo nextest run --workspace` covering mock state machines, IPC contracts, and PolicyKit checks[cite: 1].
- [ ] **Frontend Component Unit Tests:** Execute `pnpm -r test` validating React component rendering, state hooks, and binding parsing[cite: 1].
- [ ] **IPC Security & Fuzz Testing:** Sanitize all Tauri command payload inputs and verify permission boundary rejection[cite: 1].
- [ ] **License & Dependency Audit:** Execute `cargo deny check` ensuring zero security advisories and strict dual MIT/Apache-2.0 compliance[cite: 1].

**Gate Commit:** `git commit -m "phase(8): execute test suite, IPC security audit, and license validation"`[cite: 1]

---

## Phase 9: Performance Optimization & 120 FPS Audit
- [x] **Launch Time Benchmarking:** Validate cold start <500ms and hot start <150ms using performance tracing[cite: 1].
- [x] **Memory Footprint Audit:** Profile process memory to ensure idle footprint stays under 150MB RAM[cite: 1].
- [x] **120 FPS Frame Rate Audit:** Audit animation paths using Chrome DevTools performance traces to eliminate DOM reflows and layout thrashing[cite: 1].

**Gate Commit:** `git commit -m "phase(9): optimize binary execution speed, memory footprint, and rendering pipeline"`[cite: 1]

---

## Phase 10: Production Packaging & System Integration
- [x] **Debian Package Build (`.deb`):** Configured Tauri bundler `bundle.linux.deb` (system `depends`, `section`/`priority`, desktop template, extra files) so Kubuntu 24.04 LTS packages install binaries, desktop entries, and icons. Package generation runs in WSL2: `pnpm -F zettings tauri build --bundles deb`[cite: 1, 2].
- [x] **PolicyKit Actions Installation:** Authored `packaging/org.zyntrix.zettings.policy` with the 5 privileged actions (`network.set-hostname`, `display.apply-mode`, `audio.set-volume`, `network.scan-wifi`, `power.set-profile`); installed to `/usr/share/polkit-1/actions/` via `deb.files`[cite: 1, 2].
- [x] **Systemd User Service:** Created `packaging/zettings-daemon.service` (headless `zettings --daemon` mode in `main.rs` — warms the in-memory search index + holds live bus state-sync subscriptions); installed to `/usr/lib/systemd/user/` via `deb.files`[cite: 1].
- [x] **KDE Plasma Integration:** Desktop template (`packaging/zettings.desktop`) rendered to `/usr/share/applications/zettings.desktop` with `Settings;System;` categories + `X-KDE-System-Settings-Categories` mapping into KDE Plasma System Settings[cite: 1].

**Gate Commit:** `git commit -m "phase(10): construct production debian package, systemd service, and desktop integration"`[cite: 1]

---

## Phase 11: Developer Documentation & API Specifications
- [ ] **Plugin Developer Guide:** Author `docs/plugins/developer-guide.md` specifying module manifest structures, capabilities, and IPC protocols[cite: 1].
- [ ] **Architecture Manual:** Finalize `docs/architecture/` documenting crate relationships, IPC sequences, and security boundaries[cite: 1, 2].
- [ ] **Contribution Guide:** Create `CONTRIBUTING.md` detailing toolchain requirements, code style guidelines, and verify gate procedures[cite: 1].

**Gate Commit:** `git commit -m "phase(11): finalize developer documentation and API specifications"`[cite: 1]

---

## Phase 12: Production Release Delivery
- [ ] **Full CI Pipeline Execution:** Run `pnpm ci:check` validating all format, lint, typecheck, compile, and test gates[cite: 1].
- [ ] **Tag Release Artifacts:** Tag release `v1.0.0` and output final build binaries for Zyntrix OS[cite: 1].

**Gate Commit:** `git commit -m "phase(12): deliver production release build v1.0.0 for Zyntrix OS"`[cite: 1]