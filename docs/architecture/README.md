# Architecture Diagrams

Mermaid sources with embedded accessible titles/descriptions (`accTitle:` /
`accDescr:`). Per the Mermaid accessibility rules, each diagram below also
has a **structured text alternative** — the tables are the authoritative
reference; the diagrams are visual aids.

## crate-graph.mmd — Workspace dependency DAG

| Crate | Layer | Depends on | Responsibility |
|---|---|---|---|
| `zettings-core` | L1 domain | (none internal) | Registry/route/capability/error model |
| `zettings-ipc` | L2 contracts | core | Wire DTOs + ts-rs → TS bindings |
| `zettings-bus` | L2 messaging | core | Typed broadcast events |
| `zettings-polkit` | L3 authorization | core | PolicyKit gateway seam (+ mock) |
| `zettings-plugin-sdk` | L2 extension | core | Module manifests, ed25519 verification |
| `zettings-search` | L2 retrieval | core | Query normalization + weighted ranking |
| `apps/zettings` | L4 shell | all above | Tauri commands, composition root |

Adapter crates (display/audio/network/power/…) join in PLAN Phase 5 as L3
consumers of core + polkit + bus.

## ipc-sequence.mmd — Registry snapshot round trip

1. React shell calls `invokeIpc("registry_snapshot")`.
2. Inside the desktop runtime the bridge forwards to the Tauri command.
3. The command builds the snapshot from the compiled-in seed graph.
4. The payload returns typed via generated bindings.
5. In a plain browser session the bridge raises an honest error instead of
   fabricating data.

## module-lifecycle.mmd — Module states

| Transition | Guard / trigger | Outcome state |
|---|---|---|
| Discovered → VerifySignature | manifest found on disk | — |
| VerifySignature → Rejected | invalid ed25519 signature | terminal |
| VerifySignature → CheckPolicy | valid signature | — |
| CheckPolicy → Rejected | capability not policy-allowed | terminal |
| CheckPolicy → Active | allowed | definitions admitted |
| Active ⇄ RegistryLoaded | registry merge completes | steady state |
| Active → Unloaded | user/policy removal | terminal |

## search-dataflow.mmd — Ranking pipeline

Ordered stages: normalize query → score every setting (weights: title 100,
keyword 80, alias 70, page/category 50, description 35, semantic 30,
recency 10, fuzzy ≥5) → filter positive → deterministic sort (score desc,
then stable id) → top hit deep-links to its route.
