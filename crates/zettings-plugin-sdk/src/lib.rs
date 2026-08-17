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

/// Capability access-control list. A snapshot of the capabilities a module
/// was granted at load time; the IPC boundary consults it before forwarding
/// a request to the backend.
///
/// Constructed from a module manifest's declared `capabilities` field. The
/// list is immutable for the lifetime of the module mount — capability
/// amplification at runtime is forbidden by design.
#[derive(Debug, Clone, Default)]
pub struct Acl {
    granted: std::collections::HashSet<Capability>,
}

impl Acl {
    /// Construct an `Acl` granting exactly the listed capabilities.
    #[must_use]
    pub fn from_granted<I>(capabilities: I) -> Self
    where
        I: IntoIterator<Item = Capability>,
    {
        Self {
            granted: capabilities.into_iter().collect(),
        }
    }

    /// Construct an empty `Acl` (no capabilities granted — sandboxed module).
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Returns `Ok(())` if `capability` was granted, otherwise an
    /// [`SdkError::MissingCapability`] describing the denial.
    ///
    /// This is the single enforcement point every IPC handler must call before
    /// touching a privileged backend service.
    ///
    /// # Errors
    /// - [`SdkError::MissingCapability`] when the capability is not present.
    pub fn enforce(&self, capability: Capability) -> Result<(), SdkError> {
        if self.granted.contains(&capability) {
            Ok(())
        } else {
            Err(SdkError::MissingCapability(capability))
        }
    }

    /// Returns `true` when `capability` was granted. Read-only check; use
    /// [`Acl::enforce`] at the IPC boundary so the denial is structured.
    #[must_use]
    pub fn has(&self, capability: Capability) -> bool {
        self.granted.contains(&capability)
    }

    /// Iterator over the granted capabilities, in unspecified order.
    pub fn iter(&self) -> impl Iterator<Item = Capability> + '_ {
        self.granted.iter().copied()
    }

    /// Number of capabilities granted.
    #[must_use]
    pub fn len(&self) -> usize {
        self.granted.len()
    }

    /// `true` when zero capabilities are granted.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.granted.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_acl_denies_everything() {
        let acl = Acl::empty();
        assert!(acl.is_empty());
        let err = acl.enforce(Capability::ReadSystemConfig).unwrap_err();
        assert!(matches!(
            err,
            SdkError::MissingCapability(Capability::ReadSystemConfig)
        ));
    }

    #[test]
    fn granted_capability_enforces_ok() {
        let acl = Acl::from_granted([Capability::ReadSystemConfig, Capability::OpenDeepLink]);
        assert_eq!(acl.len(), 2);
        assert!(acl.has(Capability::ReadSystemConfig));
        assert!(acl.has(Capability::OpenDeepLink));
        assert!(!acl.has(Capability::WriteSystemConfig));
        acl.enforce(Capability::ReadSystemConfig).expect("granted");
        acl.enforce(Capability::OpenDeepLink).expect("granted");
        assert!(acl.enforce(Capability::WriteSystemConfig).is_err());
    }
}
