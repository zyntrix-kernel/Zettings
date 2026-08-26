use crate::BackendId;
use crate::capability::{HardwareProbe, PolkitActionId};
use crate::registry::SettingId;

#[derive(Debug, thiserror::Error)]
pub enum ZettingsError {
    #[error("backend {backend} is unreachable: {detail}")]
    BackendUnreachable { backend: BackendId, detail: String },

    #[error("authorization denied for {action}")]
    PolkitDenied { action: PolkitActionId },

    #[error("authorization dialog dismissed for {action}")]
    PolkitDismissed { action: PolkitActionId },

    #[error("validation failed for {setting}: {violation:?}")]
    Validation {
        setting: SettingId,
        violation: ConstraintViolation,
    },

    #[error("hardware requirement not met: {probe:?}")]
    HardwareMissing { probe: HardwareProbe },

    #[error("unsupported on this platform: {reason:?}")]
    NotSupported { reason: UnsupportedReason },

    #[error("invalid route {route}")]
    InvalidRoute { route: String },

    #[error("invalid polkit action {action}")]
    InvalidAction { action: String },

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl ZettingsError {
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::BackendUnreachable { .. } => "backend-unreachable",
            Self::PolkitDenied { .. } => "polkit-denied",
            Self::PolkitDismissed { .. } => "polkit-dismissed",
            Self::Validation { .. } => "validation",
            Self::HardwareMissing { .. } => "hardware-missing",
            Self::NotSupported { .. } => "not-supported",
            Self::InvalidRoute { .. } => "invalid-route",
            Self::InvalidAction { .. } => "invalid-action",
            Self::Io(_) => "io",
        }
    }

    #[must_use]
    pub fn to_bridge(&self) -> BridgeError {
        BridgeError {
            code: self.code().to_owned(),
            message: self.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConstraintViolation {
    OutOfRange { min: i64, max: i64 },
    NotInSet { allowed: Vec<String> },
    Malformed { expected: &'static str },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnsupportedReason {
    NoBluetoothAdapter,
    NoWirelessAdapter,
    CompositorUnsupported,
    FeatureNotCompiled,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct BridgeError {
    pub code: String,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_codes_are_stable_machine_strings() {
        let err = ZettingsError::InvalidRoute {
            route: String::from("nope"),
        };
        assert_eq!(err.code(), "invalid-route");
        assert_eq!(err.to_bridge().code, "invalid-route");
    }

    #[test]
    fn bridge_error_serializes_fields() {
        let err = BridgeError {
            code: String::from("polkit-denied"),
            message: String::from("denied"),
        };
        let json = serde_json::to_value(&err).unwrap();
        assert_eq!(json["code"], "polkit-denied");
        assert_eq!(json["message"], "denied");
    }
}
