//! Domain layer for Zettings: registry schema, routes, capabilities, values,
//! backend adapter contract, and the shared error taxonomy.

pub mod backend;
pub mod capability;
pub mod error;
pub mod registry;
pub mod route;
pub mod value;

pub use backend::{
    BackendAdapter, BackendId, ConfirmationPolicy, Health, OperationKey, PollPolicy, ReadBinding,
    ValueKey, WriteAction,
};
pub use capability::{
    AvailabilityReason, CapabilityRequirement, CapabilityState, EvalContext, HardwareProbe,
    PolkitActionId,
};
pub use error::{BridgeError, ConstraintViolation, UnsupportedReason, ZettingsError};
pub use registry::{
    Category, ControlKind, I18nKey, IconToken, Page, PageTemplate, REGISTRY_SCHEMA_VERSION,
    RegistryBuilder, RegistrySnapshot, Section, SettingDefinition, SettingId, SettingMetadata,
};
pub use route::{CategoryId, PageId, Route, RouteError};
pub use value::SettingValue;

pub const SCHEME_NAME: &str = "zyntrix-settings";

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RebootHint {
    None,
    Recommended,
    Required,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scheme_matches_route_documentation() {
        assert_eq!(SCHEME_NAME, "zyntrix-settings");
    }

    #[test]
    fn reboot_hint_serializes_kebab_case() {
        let json = serde_json::to_string(&RebootHint::Recommended).unwrap();
        assert_eq!(json, r#""recommended""#);
    }
}
