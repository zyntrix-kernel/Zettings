//! Typed IPC contracts between the ZETTINGS webview and the Rust backend.
//!
//! Payload types in this crate are the **single source of truth** for
//! frontend/backend communication. TypeScript bindings are generated from
//! these definitions via `ts-rs` into `packages/ts-bindings/src/generated/`
//! (run `pnpm bindings`). Hand-typed duplicate payloads are forbidden
//! (AGENTS.md §8).
//!
//! Types intentionally use plain `string` identifiers on the wire; richer
//! domain newtypes ([`zettings_core::CategoryId`], [`zettings_core::RouteId`])
//! are converted at the boundary so generated TS stays simple.

mod category;
#[cfg(feature = "export-bindings")]
mod export;
mod search;
mod system;

pub use category::{CategorySummaryDto, RegistrySnapshotDto};
pub use search::{SearchHitDto, SearchResponseDto};
pub use system::{
    AudioAreaDto, AudioSinkDto, BluetoothDeviceDto, BluetoothStatusDto, CapabilityStateDto,
    DisplayOutputDto, DisplayStatusDto, NetworkDeviceDto, NetworkStatusDto,
    PowerProfileSnapshotDto, SessionCapabilitiesDto, SystemSnapshotDto, TimedateSnapshotDto,
};
