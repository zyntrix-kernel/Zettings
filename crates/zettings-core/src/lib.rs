//! Plugin loader and core orchestration for Zettings.
//!
//! This crate owns:
//! - discovering modules in `/usr/lib/zyntrix/zettings/modules/` and
//!   `~/.local/share/zyntrix/zettings/modules/`,
//! - verifying each module's ed25519 signature via `ring`,
//! - enforcing the capability set the module claims against the shell ACL,
//! - mounting the module into the in-process registry on `zettings-bus`.
//!
//! Backend domain services are NOT part of this crate. They live in
//! `zettings-display`, `zettings-audio`, etc. and integrate via the SDK.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod discovery;
pub mod manifest;
pub mod registry;
pub mod signature;

pub use discovery::{DiscoveredModule, DiscoveryError};
pub use manifest::ModuleManifest;
pub use registry::{Registry, RegistryError};
pub use signature::{PublicKey, SignatureError};
