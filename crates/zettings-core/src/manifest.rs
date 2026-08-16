//! Module manifest types — the `module.toml` schema.
//!
//! Every settings module ships a `module.toml` declaring identity, version,
//! polkit actions it can trigger, search keywords, the React route, the
//! icon (lucide name), and the capability set it needs.

use serde::{Deserialize, Serialize};

/// A Zettings module manifest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ModuleManifest {
    /// Globally unique module id, e.g. `org.zyntrix.zettings.display`.
    pub id: ModuleId,
    /// `SemVer` version, e.g. `0.1.0`.
    pub version: String,
    /// Human-readable name shown in the sidebar.
    pub name: String,
    /// Lucide icon name, e.g. `monitor`.
    pub icon: String,
    /// Frontend route under the shell, e.g. `/display`.
    pub route: String,
    /// Polkit action IDs this module may request authorization for.
    #[serde(default)]
    pub polkit_actions: Vec<String>,
    /// User-facing search keywords.
    #[serde(default)]
    pub search_keywords: Vec<String>,
    /// Capability set required by the module. Replaces blanket root.
    #[serde(default)]
    pub capabilities: Vec<Capability>,
}

/// A capability a module may request. Defined in `zettings-plugin-sdk`.
pub use zettings_plugin_sdk::Capability;

/// Opaque newtype for module identifiers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct ModuleId(pub String);

impl ModuleId {
    /// Construct a new module id. Not validated as a DNS-name here; signal
    /// `Registry::register` does it.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl std::fmt::Display for ModuleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_manifest() {
        let toml = r#"
            id = "org.zyntrix.zettings.display"
            version = "0.1.0"
            name = "Display"
            icon = "monitor"
            route = "/display"
            search_keywords = ["resolution", "scaling", "hdr", "night light"]
        "#;
        let m: ModuleManifest = toml::from_str(toml).expect("valid manifest");
        assert_eq!(m.id.0, "org.zyntrix.zettings.display");
        assert!(m.polkit_actions.is_empty());
        assert!(m.capabilities.is_empty());
    }
}
