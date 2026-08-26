# Zettings Architecture (PLAN Phase 1)

Authoritative structural specification of the native stack: **Qt 6 (QML)** UI
bridged from **Rust** via **cxx-qt**, backend logic in pure Rust over D-Bus.
This directory is the deliverable set of PLAN Phase 1; implementation phases
must keep it synchronized in the same change that alters any contract.

| Document | Content |
|---|---|
| `crate-graph.mmd` | Workspace DAG and Qt boundary |
| `ipc-sequence.mmd` | Setting write path incl. polkit authorization |
| `module-lifecycle.mmd` | Startup/shutdown lifecycle |
| `search-dataflow.mmd` | Index construction and query ranking |
| `api-contracts.md` | Trait + bridge signatures, error taxonomy, persistence |
| `settings-registry-schema.md` | Registry data model and invariants |
| `route-schema.md` | Deep-link grammar and router contract |
| `capability-model.md` | Requirement evaluation and authorization flow |

Diagram rendering note (per mermaid skill): `.mmd` files use `accTitle`/
`accDescr` syntax; GitHub.com Markdown renders Mermaid with a platform-managed
renderer (version not pinned here). The structured alternatives below are the
accessible equivalent of every diagram.

## Layering rules

1. `zettings-core` depends on nothing internal; it defines types, traits,
   errors. No tokio, no zbus.
2. Domain crates (`search`, `polkit`, `backends`) depend only on core (and
   polkit where authorization is enforced).
3. `zettings-bridge` is the ONLY crate containing `#[cxx_qt::bridge]` modules;
   QML-visible types are born there and never hand-duplicated in C++.
4. `apps/zettings` composes bridges + QML modules; it contains no business
   rules. Backend/domain crates must never depend on it.
5. All cross-layer async state flows through channels/`Arc`; no global mutable
   singletons; no lock held across a Qt↔tokio boundary.

## Crate inventory (alternative for crate-graph.mmd)

| Crate (layer) | Depends on | Responsibility |
|---|---|---|
| `zettings-core` (L0) | — | Registry schema types, route model, capability model, error taxonomy, `BackendAdapter` trait |
| `zettings-search` (L1) | core | Inverted index, normalization/synonyms, fuzzy matching, PLAN §5 ranking pipeline |
| `zettings-polkit` (L1) | core | Only code path touching `org.freedesktop.PolicyKit1`; audit log |
| `zettings-backends` (L2) | core, polkit | Concrete D-Bus adapters: timedate1, NetworkManager, BlueZ, UPower, power-profiles-daemon, login1, PulseAudio/PipeWire, AccountsService |
| `zettings-bridge` (L3) | core, search, backends, polkit | Per-domain `#[cxx_qt::bridge]` QObjects (see api-contracts.md inventory) |
| `apps/zettings` (L4) | zettings-bridge, Qt 6 via CMake/Corrosion | Executable, QML shell/pages/components/style under single URI `org.zyntrix.zettings` |

Workspace mechanics per AGENTS.md §8: virtual manifest at repo root; shared
metadata/deps/lints in `[workspace.package]` / `[workspace.dependencies]` /
`[workspace.lints]`; members opt in with `.workspace = true`.

## Write-path messages (alternative for ipc-sequence.mmd)

| # | From → To | Message / outcome |
|---|---|---|
| 1 | QML page → bridge | Invokable (e.g. `setNtpEnabled(true)`); returns operation token immediately |
| 2 | Bridge → capability layer | Evaluate `CapabilityRequirement` |
| 3 | Capability → polkit gateway → PolicyKit1 | `CheckAuthorization`; KDE agent dialog when required; denial = audited error |
| 4 | Bridge → registry validation | Value vs `ValueConstraint` |
| 5 | Bridge → adapter | `write(Operation)` |
| 6 | Adapter → daemon | D-Bus method call; ack/error |
| 7 | Adapter → bridge → QML | Operation result + property-change signal refreshes bound views |
| 8 | Bridge → audit log | Append entry: actor, setting, verb, outcome, decision |

## Startup sequence (alternative for module-lifecycle.mmd)

1. Single-instance check — second launch forwards its URL to the running
   instance (`org.zyntrix.Zettings /activate`) and exits.
2. Init tracing + audit log; start tokio runtime.
3. Build `RegistrySnapshot`; invariant violations hard-fail (schema rule).
4. Construct `SearchIndex`; load usage stats.
5. Connect adapters in parallel with per-backend timeouts; each reports
   `Up/Degraded/Down` independently — degraded daemons never block startup.
6. Load QML engine and Shell window; resolve route (deep link → replace;
   else session restore or hub default).

Shutdown (user-initiated): flush usage/session stores atomically, close
broadcast channels, join adapter tasks with timeout, exit cleanly.

## Search flow summary (alternative for search-dataflow.mmd)

Indexing (startup): normalize/tokenize → synonym+alias expansion → inverted
index; usage stats attach recency/frequency/pinned rank metadata.
Query (debounced): normalize → exact id/title (+100) → keyword (+80)/alias
(+70)/page (+50)/description (+35) → bounded fuzzy ≤2 edits (+5) → reserved
semantic slot (+30, empty provider until a local embedder lands) → context
boosts (recent +10, pinned, frequency, current category) → deterministic sort
→ top 50 → activation navigates directly to the hit's route.

## Deliberate scope decisions

- **No privileged daemon in v2**: mutations run in-process behind the polkit
  gateway; `packaging/zettings-daemon.service` template remains parked
  unused until an out-of-process requirement materializes (documented, not wired).
- **Plugins v1 are compile-time feature modules** registered through the
  registry builder; out-of-process providers are a documented future extension
  point (registry-driven pages make the seam explicit), not scaffolding built now.
- **Display backend** final target (KScreen vs compositor-specific D-Bus) is
  decided in Phase 7 with its own spike; the adapter trait already isolates it.
