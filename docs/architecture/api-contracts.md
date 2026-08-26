# API Contracts (Phase 1 contract)

Signatures here are binding contracts between layers. Rust-facing surfaces are
written as real trait/impl signatures; the QML-facing surface is the cxx-qt
bridge inventory. Implementation phases must not widen or rename these without
updating this document in the same change.

## Backend adapter trait (`zettings-core`)

All adapters implement one trait; bridges depend on this, never on concrete
adapters (dependency inversion at the QML boundary).

```rust
#[async_trait::async_trait]
pub trait BackendAdapter: Send + Sync {
    fn id(&self) -> BackendId;

    /// Liveness probe used by the capability layer.
    async fn health(&self) -> Health;                    // Up | Degraded(reason) | Down(reason)

    /// Read a value declared by the registry.
    async fn read(&self, key: &ValueKey) -> Result<SettingValue, ZettingsError>;

    /// Apply a validated, already-authorized write.
    async fn write(&self, op: &Operation) -> Result<(), ZettingsError>;

    /// Subscribe to external change signals (D-Bus property changes etc.).
    fn subscribe(&self) -> broadcast::Receiver<BackendEvent>;
}
```

Guarantees:
- `write` is called only after validation + authorization (capability-model.md).
- Adapters are connection-loss tolerant: `health()` reflects reality; calls on a
  down adapter return `ZettingsError::BackendUnreachable`, they never block
  startup.
- All adapter methods are cancel-safe (tokio select-friendly).

## Error taxonomy (`zettings-core`)

```rust
#[derive(Debug, thiserror::Error)]
pub enum ZettingsError {
    #[error("backend {backend} unreachable: {source}")]
    BackendUnreachable { backend: BackendId, #[source] source: Box<dyn std::error::Error + Send + Sync> },
    #[error("authorization denied for {action}")]
    PolkitDenied { action: PolkitActionId },
    #[error("authorization dialog dismissed for {action}")]
    PolkitDismissed { action: PolkitActionId },
    #[error("validation failed for {setting}: {reason}")]
    Validation { setting: SettingId, reason: ConstraintViolation },
    #[error("hardware requirement not met: {probe:?}")]
    HardwareMissing { probe: HardwareProbe },
    #[error("unsupported on this platform: {reason}")]
    NotSupported { reason: UnsupportedReason },
    #[error("invalid route {route}")]
    InvalidRoute { route: String },
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
```

Bridge mapping: every invokable returns `Result<T, BridgeError>` where
`BridgeError { code: String, message: String }` — codes are the stable,
machine-readable subset of `ZettingsError` variants.

## cxx-qt bridge inventory (`zettings-bridge`)

One `#[cxx_qt::bridge]` module per domain (AGENTS.md rule — no monolith).
Each bridge owns its domain's `Arc<dyn BackendAdapter>` handles and exposes:

| Bridge module | Exposes to QML | Backends consumed |
|---|---|---|
| `router` | `navigate/replace/back/forward`, `currentRoute`, history depth | — |
| `registry` | page/section/setting models for current route (QAbstractListModel) | — |
| `search` | `search(query) → SearchHitsModel`, recent/pinned/frequent accessors | zettings-search |
| `system_datetime` | time/date/NTP/timezone properties + setters | timedate1 |
| `power` | battery, profiles, power-mode, lid/screen-off policies | UPower, power-profiles-daemon, login1 |
| `display` | outputs, resolution/scale/refresh, night light | KScreen / mutter-agnostic D-Bus (Phase 7 decision recorded there) |
| `audio` | sinks/sources, volume/mute, default device | PipeWire via pulse module, PulseAudio |
| `network` | wifi list/connect, ethernet state, airplane mode | NetworkManager |
| `bluetooth` | adapter power, discovery, pairing | BlueZ |
| `personalization` | accent, theme, wallpaper handoff to Plasma look-and-feel | plasma-apis via D-Bus/kwriteconfig5-equivalent |
| `capability` | per-setting evaluated state model | polkit gateway + probes |

Common bridge rules:
- Properties are read-mostly; backends push updates through
  `broadcast::Receiver<BackendEvent>`, each bridge runs one tokio task mapping
  events to Qt property changes (no polling from QML).
- Invokables are non-blocking: anything slow is spawned and reports completion
  via signals; invokables return an operation token immediately.
- Bridges never expose raw `String` errors to QML — only `BridgeError`.

## Search pipeline contract (`zettings-search`)

```rust
pub struct SearchIndex { /* built from RegistrySnapshot */ }

impl SearchIndex {
    pub fn build(snapshot: &RegistrySnapshot, usage: &UsageStats) -> Self;
    pub fn query(&self, q: &str, ctx: &QueryContext) -> Vec<SearchHit>;   // ranked, capped 50
}

pub struct QueryContext { pub route: Route, pub reduced_motion: bool /* reserved */ }
```

Ranking implements the PLAN §5 baseline table exactly (weights 100/80/70/50/35/30/10/5);
tie-break by registry order. Fuzzy stage uses bounded edit distance (max 2 edits
for tokens ≥ 4 chars). Semantic retrieval slot is defined but returns empty until
a local embedding provider lands — the pipeline position and weight (+30) are fixed now.

## Persistence contracts

| Store | Path | Format |
|---|---|---|
| Usage stats (recent/frequent) | `$XDG_STATE_HOME/zettings/usage.json` | JSON, atomic replace |
| Audit log | `$XDG_STATE_HOME/zettings/audit.log` | append-only JSONL |
| Session (window/route restore) | `$XDG_STATE_HOME/zettings/session.json` | JSON |

Audit entries record: timestamp, actor (polkit subject), setting id, verb,
outcome, polkit decision — never secret values (Wi-Fi passwords, etc. are
excluded by type: values marked sensitive in the registry are unloggable).

## Concurrency model

- One tokio runtime owned by the app shell; bridges and adapters run on it.
- Shared state flows through `Arc<>` + channels; no global mutable singletons.
- QML thread ↔ tokio: all crossings go through queued signals (cxx-qt handles
  the hop); no lock is ever held across a Qt boundary.
