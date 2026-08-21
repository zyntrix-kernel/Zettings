# Prompt Compliance Matrix

> `prompt.txt` is a living acceptance specification (PLAN §0.2). Every
> requirement maps to a phase, an implementation locus, a verification method,
> and a status. No requirement may silently disappear.
>
> Status values: **PLANNED** → **IN_PROGRESS** → **PASS** / **BLOCKED(reason)**.

## Product & architecture

| Requirement (prompt.txt) | Phase | Locus | Verification | Status |
|---|---|---|---|---|
| Production-grade; no toys/placeholders/pseudocode | all | whole repo | review + gates | PLANNED |
| Rust + Tauri v2 + Tokio backend | 1,5 | workspace crates | cargo gates | PLANNED |
| React 19 + TypeScript + Vite frontend | 1,4,6 | web app | pnpm typecheck/build | PLANNED |
| Tailwind CSS v4 + CSS variables tokens | 2 | design system | token validation script | PLANNED |
| Modular, feature-based, plugin-capable, DI, message bus, repository pattern, state machines | 1 | core crates | architecture review + unit tests | PLANNED |
| Frontend-independent backend | 1 | IPC contract | ts-rs binding tests | PLANNED |

## Design language (ZDL)

| Requirement | Phase | Locus | Verification | Status |
|---|---|---|---|---|
| Original ZDL — not Windows/macOS/generic glassmorphism clone | 2 | DESIGN.md + tokens | visual review vs inspiration set | PLANNED |
| G2/G3 continuity geometry | 2,3 | squircle system + motion curves | math tests + visual | PLANNED |
| Liquid-glass material w/ depth, specular, blur control | 2 | material layer stack | browser verification (agent-browser, Chromium) | PLANNED |
| Light/Dark/OLED themes; accent dynamics; transparency/blur/roundedness controls; compact/comfort modes | 2,6 | token cascade + theme engine | theme snapshot tests | PLANNED |

## Motion

| Requirement | Phase | Locus | Verification | Status |
|---|---|---|---|---|
| Spring + Bezier interpolation, velocity preservation, momentum, elasticity | 3 | motion engine (motion-framer + react-spring-physics skills) | physics unit tests + frame harness | PLANNED |
| Micro-interactions, overshoot/anticipation/settle, secondary motion, parallax/depth where justified | 3,4 | motion primitives | component tests + visual | PLANNED |
| Context-aware durations; GPU-accelerated; no jank; never block rendering; 120 FPS target | 3,9 | engine + instrumentation | frame-time harness (perf-baseline.md protocol) | PLANNED |

## System integration

| Requirement | Phase | Locus | Verification | Status |
|---|---|---|---|---|
| Full adapter surface (display…thermals) per PLAN §13 | 5,7 | adapter crates | capability matrix cross-check | PLANNED |
| zbus/D-Bus, PolicyKit, systemd, NM, PipeWire/PulseAudio, UPower, PPD, AccountsService, BlueZ, KScreen/KWin, libinput, login1, PackageKit, Flatpak/Snap/AppImage, Debian packaging | 5,10 | adapters + packaging | WSL2 integration tests | PLANNED |
| Honest unavailable states; no fake settings | 5,7 | CapabilityState contract | adapter tests assert degraded paths | PLANNED |

## Search

| Requirement | Phase | Locus | Verification | Status |
|---|---|---|---|---|
| Registry-indexed search: titles/descriptions/aliases/keywords | 1,7 | search crate | index completeness tests | PLANNED |
| Fuzzy matching, misspellings, synonyms, natural-language queries | 7 | search crate | golden query suite incl. misspellings | PLANNED |
| Ranking: direct settings above categories; recent/pinned/frequent boosts; context awareness | 7 | ranking pipeline | ranking unit tests vs spec §9 weights | PLANNED |
| Keyboard-navigable instant results | 4,7 | search surface | keyboard-flow e2e | PLANNED |
| AI-assisted changes require explanation→preview→explicit confirm | 7+ | intent flow | flow test: no silent mutation possible | PLANNED |

## Deep links & navigation

| Requirement | Phase | Locus | Verification | Status |
|---|---|---|---|---|
| `zettings://` scheme; stable routes; route↔page bijection | 1,4 | route registry | bijection property test | PLANNED |
| Back/forward works; deep-linked pages load directly; failures don't break routing | 4 | shell router | routing integration tests | PLANNED |

## Security (threat-model.md)

| Requirement | Phase | Locus | Verification | Status |
|---|---|---|---|---|
| Zero Trust, least privilege, sandboxed webview capabilities | 1,5 | capability files + ACL | CI ACL diff check | PLANNED |
| PolicyKit for privileged ops; isolated validated privileged commands | 5 | zettings-polkit gateway | polkit action tests (WSL2) | PLANNED |
| Input sanitization; no unsafe arbitrary command execution | 1,5 | validators + argv allowlists | fuzz/proptest on validators | PLANNED |
| Audit logging of privileged changes | 1,5 | tracing-journald sink | audit trail integration test | PLANNED |

## Performance

| Requirement | Phase | Locus | Verification | Status |
|---|---|---|---|---|
| Cold <500ms · Hot <150ms · Mem <150MB · 120 FPS target | 9 | measured per perf-baseline.md | hyperfine/RSS/frame harness reports | NOT MEASURED YET |
| Lazy modules/backends; background indexing; async everything; no UI blocking | 1..7 | lazy init patterns | profile traces | PLANNED |
| Zero unnecessary re-renders; zero leaks | 8,9 | render discipline rules | profiler scenario + leak soak | PLANNED |

## Accessibility (WCAG AAA target)

| Requirement | Phase | Locus | Verification | Status |
|---|---|---|---|---|
| Screen readers, keyboard-only, focus indicators, logical order | 2,4,8 | primitives + shell | axe + manual AT pass (manual-testing skill) | PLANNED |
| Reduced motion, high contrast, large fonts, color-blind-safe encoding | 2,6 | tokens + theme engine | forced-colors/reduced-motion test matrix | PLANNED |
| Accessible state announcements | 4 | live regions (aria-live-regions skill) | SR announcement tests | PLANNED |

## Delivery

| Requirement | Phase | Locus | Verification | Status |
|---|---|---|---|---|
| CI/CD via GitHub Actions | 10 | .github/workflows | pipeline green | PLANNED |
| Debian packaging + desktop entry + MIME/deep-link registration + polkit policy install | 10 | packaging/ | install/upgrade/uninstall validation | PLANNED |
| Docs: README, dev/contrib guides, architecture, security model, threat model, testing/perf/ZDL/plugin/registry guides, WSL2 setup, release guide | 11 | docs/, README | docs review checklist | PLANNED |
| Release gate: all PASS or BLOCKED-documented | 12 | compliance matrix | final review | PLANNED |
