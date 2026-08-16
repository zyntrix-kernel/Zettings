//! `zettings-ipc` — Tauri v2 Command Surface and Typed Binding Exporter.
//!
//! Exposes async Tauri commands to the webview shell and enforces `PolicyKit`
//! authorization checks via [`zettings_polkit::Authorizer`] prior to executing
//! privileged actions. All payload types are exported to TypeScript via `ts-rs`
//! into `packages/ts-bindings/src/generated/` so the React frontend invokes the
//! backend with strict typing — hand-typed duplicate frontend payloads are
//! forbidden (see `AGENTS.md`).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;
#[cfg(not(feature = "zettings-mock"))]
use zettings_polkit::ActionId;
#[cfg(feature = "zettings-mock")]
use zettings_polkit::{ActionId, Authorization, Authorizer, MockAuthorizer};

/// Backend health payload returned by the `zettings_health` command.
///
/// The frontend uses `is_mock` to surface a "mock backend" badge during Windows
/// frontend iteration and to gate UI affordances that require real Linux services.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "health.ts")]
pub struct Health {
    /// Zettings backend version (matches `CARGO_PKG_VERSION`).
    pub version: String,
    /// `true` when running against the `zettings-mock` state-machine backend
    /// (Windows dev loop). `false` on the real WSL2/Linux target.
    pub is_mock: bool,
}

/// Frontend-facing module descriptor returned by the `zettings_modules` command.
///
/// Mirrors the subset of [`zettings_core::manifest::ModuleManifest`] the sidebar
/// navigation needs. The full manifest stays backend-side.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "module_info.ts")]
pub struct ModuleInfo {
    /// Globally unique module id, e.g. `org.zyntrix.zettings.display`.
    pub id: String,
    /// Human-readable name shown in the sidebar.
    pub name: String,
    /// Lucide icon name, e.g. `monitor`.
    pub icon: String,
    /// Frontend route, e.g. `/display`.
    pub route: String,
}

/// Errors surfacing from IPC command execution. Serialized to the frontend as
/// a tagged enum so the React layer can pattern-match on `type`.
#[derive(Debug, Error, Serialize, Deserialize, TS)]
#[serde(tag = "type", content = "message")]
#[ts(export, export_to = "ipc_error.ts")]
pub enum IpcError {
    /// `PolicyKit` authorization was denied or dismissed by the user.
    #[error("PolicyKit authorization denied: {0}")]
    PolkitDenied(String),
    /// The underlying system service (`DBus`/`PipeWire`/etc.) was unreachable.
    #[error("System service unavailable: {0}")]
    ServiceUnavailable(String),
    /// The request payload failed validation.
    #[error("Invalid request payload: {0}")]
    InvalidPayload(String),
    /// An internal execution error not covered by a more specific variant.
    #[error("Internal execution error: {0}")]
    Internal(String),
}

/// Request payload to modify the system hostname.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "set_hostname_request.ts")]
pub struct SetHostnameRequest {
    /// The new hostname. Validated by `NetworkManager` on the backend.
    pub hostname: String,
}

/// Response payload for a hostname mutation.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "set_hostname_result.ts")]
pub struct SetHostnameResult {
    /// Whether the hostname was successfully applied.
    pub success: bool,
    /// The active hostname after the operation (echo-back for UI confirmation).
    pub active_hostname: String,
}

/// Sets the system hostname after verifying `PolicyKit` authorization.
///
/// Action ID: `org.zyntrix.zettings.network.set-hostname`.
///
/// Under the `zettings-mock` feature (Windows dev loop) the mock authorizer
/// always returns [`Authorization::Authorized`] and the new hostname is echoed
/// back directly. On the real Linux target the call is forwarded to
/// `org.freedesktop.NetworkManager.SetHostname` over `zbus` (Phase 5).
///
/// The `#[tauri::command]` attribute is applied in `apps/zettings/src/main.rs`
/// when this command is registered with `generate_handler!` (Phase 5). Keeping
/// the attribute off the lib definition avoids Tauri v2 macro-namespace
/// duplicate-definition errors when the lib is compiled independently of the
/// bin's `generate_handler!` invocation.
///
/// # Errors
/// - [`IpcError::PolkitDenied`] if the user dismisses the polkit dialog or is
///   not permitted to change the hostname.
/// - [`IpcError::ServiceUnavailable`] on the non-mock target until Phase 5
///   wires the real `zbus` integration.
///
/// > **Note:** This function is intentionally synchronous during Phase 1. The
/// > `async` keyword returns in Phase 5 once the real `zbus` integration lands.
//
// `needless_pass_by_value`: Tauri's `generate_handler!` deserializes the
// request payload from the webview into an owned `SetHostnameRequest` before
// calling this function, so the public IPC boundary must accept by value.
#[allow(clippy::needless_pass_by_value)]
pub fn network_set_hostname(request: SetHostnameRequest) -> Result<SetHostnameResult, IpcError> {
    set_hostname_impl(&request)
}

/// Backend implementation of [`network_set_hostname`], split out so the
/// `#[tauri::command]` macro expands over a thin wrapper instead of a body
/// containing `#[cfg]` blocks (which confuse the macro's helper-symbol
/// generation on Tauri v2).
fn set_hostname_impl(request: &SetHostnameRequest) -> Result<SetHostnameResult, IpcError> {
    let action = ActionId::zettings("network", "set-hostname");
    set_hostname_with_authorizer(request, &action)
}

/// Mock-path hostname mutation. The mock authorizer always permits the change.
#[cfg(feature = "zettings-mock")]
fn set_hostname_with_authorizer(
    request: &SetHostnameRequest,
    action: &ActionId,
) -> Result<SetHostnameResult, IpcError> {
    let authorizer = MockAuthorizer;
    let status = authorizer
        .check_authorization(action)
        .map_err(|e| IpcError::PolkitDenied(e.to_string()))?;
    match status {
        Authorization::Authorized => Ok(SetHostnameResult {
            success: true,
            active_hostname: request.hostname.clone(),
        }),
        Authorization::Challenge => Err(IpcError::PolkitDenied(
            "Authentication required via PolicyKit agent".into(),
        )),
        Authorization::Denied => Err(IpcError::PolkitDenied(
            "User denied authorization to change system hostname".into(),
        )),
    }
}

/// Real-target hostname mutation. The `zbus` call to
/// `org.freedesktop.NetworkManager.SetHostname` lands in Phase 5; until then
/// this surfaces a deterministic service-unavailable error so the frontend can
/// render a clear state rather than a silent no-op.
#[cfg(not(feature = "zettings-mock"))]
fn set_hostname_with_authorizer(
    _request: &SetHostnameRequest,
    _action: &ActionId,
) -> Result<SetHostnameResult, IpcError> {
    Err(IpcError::ServiceUnavailable(
        "Real DBus integration lands in Phase 5".into(),
    ))
}

#[cfg(test)]
mod bindings_export {
    //! Regenerates the TypeScript bindings under
    //! `packages/ts-bindings/src/generated/` when this crate is tested with the
    //! `export-bindings` feature.
    //!
    //! Run via:
    //! ```sh
    //! cargo test -p zettings-ipc --features export-bindings
    //! ```
    //! No environment variables required — the test computes the absolute
    //! path to the workspace `packages/ts-bindings/src/generated/` directory
    //! from `CARGO_MANIFEST_DIR`.

    use super::*;
    use std::path::PathBuf;

    /// Absolute path to `packages/ts-bindings/src/generated/` relative to
    /// this crate's `CARGO_MANIFEST_DIR` (`crates/zettings-ipc/`).
    fn generated_dir() -> PathBuf {
        let manifest_dir =
            std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is always set by cargo");
        PathBuf::from(manifest_dir)
            // crates/zettings-ipc -> crates/
            .parent()
            .expect("crate manifest dir has a parent")
            // crates/ -> workspace root
            .parent()
            .expect("crates dir has a parent (workspace root)")
            .join("packages")
            .join("ts-bindings")
            .join("src")
            .join("generated")
    }

    #[test]
    fn export_all_bindings() {
        // ts-rs 12 API: `TS::export_all(&cfg)` writes each type to
        // `cfg.export_dir.join(output_path)` where `output_path` is the
        // per-type `#[ts(export_to = "..")]` value (a filename like
        // "health.ts"). We compute the absolute workspace path so the
        // bindings always land in `packages/ts-bindings/src/generated/`
        // regardless of the caller's working directory.
        let cfg = ts_rs::Config::default().with_out_dir(generated_dir());
        Health::export_all(&cfg).expect("export Health bindings");
        ModuleInfo::export_all(&cfg).expect("export ModuleInfo bindings");
        IpcError::export_all(&cfg).expect("export IpcError bindings");
        SetHostnameRequest::export_all(&cfg).expect("export SetHostnameRequest bindings");
        SetHostnameResult::export_all(&cfg).expect("export SetHostnameResult bindings");
    }
}
