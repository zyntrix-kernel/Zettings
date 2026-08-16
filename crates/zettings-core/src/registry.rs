//! Plugin registry — the in-memory table of all mounted modules.

use crate::manifest::ModuleManifest;
use dashmap::DashMap;
use std::path::PathBuf;
use thiserror::Error;

/// Errors surfaced by the registry.
#[derive(Debug, Error)]
pub enum RegistryError {
    /// A module with this id is already registered.
    #[error("duplicate module id: {0}")]
    Duplicate(String),
    /// The module id is not a valid DNS-style reverse name.
    #[error("invalid module id: {0}")]
    InvalidId(String),
    /// Manifest parsing failed.
    #[error("manifest parse error: {0}")]
    Manifest(#[from] toml::de::Error),
}

/// In-memory registry of mounted modules. Cheap to clone; backed by `DashMap`.
#[derive(Default)]
pub struct Registry {
    modules: DashMap<String, RegisteredModule>,
}

/// A registered module. Holds the validated manifest plus the filesystem
/// path the module was loaded from.
#[derive(Debug, Clone)]
pub struct RegisteredModule {
    /// Validated manifest.
    pub manifest: ModuleManifest,
    /// Where on disk the module lives.
    pub path: PathBuf,
}

impl Registry {
    /// Construct an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a module. Rejects duplicates and invalid ids.
    ///
    /// # Errors
    /// - `Duplicate` if the id is already registered.
    /// - `InvalidId` if the id is not a non-empty `<word>.<word>...` string.
    pub fn register(&self, module: RegisteredModule) -> Result<(), RegistryError> {
        let id = module.manifest.id.0.clone();
        if id.split('.').any(str::is_empty) || id.is_empty() {
            return Err(RegistryError::InvalidId(id));
        }
        if self.modules.contains_key(&id) {
            return Err(RegistryError::Duplicate(id));
        }
        self.modules.insert(id, module);
        Ok(())
    }

    /// Look up a module by id.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<RegisteredModule> {
        self.modules.get(id).map(|r| r.clone())
    }

    /// Iterate over all registered modules, in unspecified order.
    pub fn iter(&self) -> impl Iterator<Item = RegisteredModule> + '_ {
        self.modules.iter().map(|r| RegisteredModule {
            manifest: r.manifest.clone(),
            path: r.path.clone(),
        })
    }

    /// Number of registered modules.
    #[must_use]
    pub fn len(&self) -> usize {
        self.modules.len()
    }

    /// True when zero modules are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.modules.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{Capability, ModuleId, ModuleManifest};

    fn sample(id: &str) -> ModuleManifest {
        ModuleManifest {
            id: ModuleId::new(id),
            version: "0.1.0".into(),
            name: "Sample".into(),
            icon: "monitor".into(),
            route: "/x".into(),
            polkit_actions: Vec::new(),
            search_keywords: Vec::new(),
            capabilities: Vec::<Capability>::new(),
        }
    }

    #[test]
    fn registers_a_module() {
        let r = Registry::new();
        r.register(RegisteredModule {
            manifest: sample("org.zyntrix.zettings.display"),
            path: PathBuf::from("/tmp"),
        })
        .expect("ok");
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn rejects_duplicate() {
        let r = Registry::new();
        let m = RegisteredModule {
            manifest: sample("org.zyntrix.zettings.display"),
            path: PathBuf::from("/tmp"),
        };
        r.register(m.clone()).expect("ok");
        let err = r.register(m).unwrap_err();
        assert!(matches!(err, RegistryError::Duplicate(_)));
    }

    #[test]
    fn rejects_empty_id() {
        let r = Registry::new();
        let err = r
            .register(RegisteredModule {
                manifest: sample(""),
                path: PathBuf::from("/tmp"),
            })
            .unwrap_err();
        assert!(matches!(err, RegistryError::InvalidId(_)));
    }
}
