//! Public SDK for authoring Zettings modules.
//!
//! This crate is the only surface a third-party module links against.
//! It declares:
//! - the `Capability` enum (capability-based permissions),
//! - shared error types,
//! - the trait shape modules must implement (in a later phase).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use serde::{Deserialize, Serialize};

/// Capability a module may request in its manifest.
///
/// Capabilities replace blanket root access. A module that does not declare
/// a capability cannot exercise it at runtime — the shell's ACL rejects the
/// IPC call before it reaches the backend.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum Capability {
    /// Read system configuration without modifying it.
    ReadSystemConfig,
    /// Modify system configuration (e.g. `KScreen`, `NetworkManager`).
    WriteSystemConfig,
    /// Spawn processes as the user (never as root).
    SpawnUserProcess,
    /// Trigger a polkit-1 authorization dialog for a named action.
    RequestPrivilege,
    /// Access the user's secret store (`libsecret` / `KWallet`).
    ReadSecrets,
    /// Read location/timezone/auto-env.
    ReadLocation,
    /// Read network state without joining networks.
    ReadNetworkState,
    /// Open deep links via the `zettings://` scheme.
    OpenDeepLink,
}

/// Errors that may surface from SDK-level operations.
#[derive(Debug, thiserror::Error)]
pub enum SdkError {
    /// The module was loaded but lacks a capability it tried to exercise.
    #[error("missing capability: {0:?}")]
    MissingCapability(Capability),
    /// The module was loaded with an unsupported SDK version.
    #[error("unsupported sdk version: {0}")]
    UnsupportedSdkVersion(semver_hack::Version),
}

/// Private shim so we don't pull in `semver` for one type. Replace with the
/// real `semver` crate when SDK version negotiation moves to Phase 4.
mod semver_hack {
    use std::fmt;

    /// SHIM replacement for `semver::Version` used by
    /// [`SdkError::UnsupportedSdkVersion`] until the real `semver` crate is
    /// wired in Phase 4. Stores the raw version string.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Version(pub String);

    impl fmt::Display for Version {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(&self.0)
        }
    }
}

pub use semver_hack::Version;
