//! Bus identity constants and system capability reporting model.

use serde::{Deserialize, Serialize};
use zvariant::Type;

/// Well-known bus name of the daemon.
pub const BUS_NAME: &str = "org.zyntrix.Zettings";
/// Object path served by the daemon.
pub const OBJECT_PATH: &str = "/org/zyntrix/Zettings";
/// Interface implemented at [`OBJECT_PATH`].
pub const INTERFACE: &str = "org.zyntrix.Zettings";

/// Availability strings. Identical to the Dart `ServiceAvailability` names so
/// the Flutter side maps them without translation.
pub mod availability {
    pub const AVAILABLE: &str = "available";
    pub const UNAVAILABLE: &str = "unavailable";
    pub const UNSUPPORTED: &str = "unsupported";
    pub const PERMISSION_DENIED: &str = "permissionDenied";
    pub const UNKNOWN: &str = "unknown";
}

/// One probed system capability.
///
/// `reason` is a plain string (empty = none) because D-Bus has no `Option`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct Capability {
    pub name: String,
    pub availability: String,
    pub reason: String,
}

impl Capability {
    pub fn available(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            availability: availability::AVAILABLE.to_string(),
            reason: String::new(),
        }
    }

    pub fn unavailable(
        name: impl Into<String>,
        availability: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            availability: availability.into(),
            reason: reason.into(),
        }
    }
}
