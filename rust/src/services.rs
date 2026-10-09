//! System-service registry: the typed home of every backend integration.
//!
//! Each Linux/KDE integration (system info, display, network, …) implements
//! [`ZettingsService`] and registers here. [`ServiceRegistry::capabilities`]
//! is what `GetCapabilities` serves. Probing must never panic: a service that
//! cannot determine its state reports `unavailable`/`unknown` with a reason.

use std::any::Any;

use crate::models::{Capability, availability};

pub mod accounts;
pub mod audio;
pub mod bluetooth;
pub mod network;
pub mod power;
pub mod system_info;

/// Contract for one system integration.
pub trait ZettingsService: Send + Sync {
    /// Stable name, e.g. `"network"`. Matches the Flutter side's
    /// `requiredCapabilities` entries.
    fn name(&self) -> &'static str;

    /// Current availability. Infallible by contract: encode failures as
    /// [`Capability`] values, never as panics.
    fn probe(&self) -> Capability;

    /// Downcast hook so typed backends (e.g. `GetSystemInfo`) can reach the
    /// concrete service behind the registry's trait objects.
    fn as_any(&self) -> &dyn Any;
}

/// Ordered registry of integrations.
#[derive(Default)]
pub struct ServiceRegistry {
    services: Vec<Box<dyn ZettingsService>>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<S: ZettingsService + 'static>(&mut self, service: S) {
        self.services.push(Box::new(service));
    }

    pub fn capabilities(&self) -> Vec<Capability> {
        self.services
            .iter()
            .map(|service| service.probe())
            .collect()
    }

    /// Finds a registered service by name for typed downcasting.
    pub fn get(&self, name: &str) -> Option<&dyn ZettingsService> {
        self.services
            .iter()
            .find(|service| service.name() == name)
            .map(|service| service.as_ref())
    }

    pub fn len(&self) -> usize {
        self.services.len()
    }

    pub fn is_empty(&self) -> bool {
        self.services.is_empty()
    }
}

/// Built-in liveness service. Always present; proves the daemon is up and the
/// capability pipeline works end to end before any hardware integration lands.
pub struct BuiltinService {
    version: String,
}

impl BuiltinService {
    pub fn new(version: impl Into<String>) -> Self {
        Self {
            version: version.into(),
        }
    }
}

impl ZettingsService for BuiltinService {
    fn name(&self) -> &'static str {
        "backend"
    }

    fn probe(&self) -> Capability {
        Capability {
            name: self.name().to_string(),
            availability: availability::AVAILABLE.to_string(),
            reason: format!("zettings-backend {}", self.version),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeService {
        capability: Capability,
    }

    impl ZettingsService for FakeService {
        fn name(&self) -> &'static str {
            "fake"
        }

        fn probe(&self) -> Capability {
            self.capability.clone()
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[test]
    fn empty_registry_reports_no_capabilities() {
        let registry = ServiceRegistry::new();
        assert!(registry.is_empty());
        assert!(registry.capabilities().is_empty());
    }

    #[test]
    fn registered_services_are_probed_in_order() {
        let mut registry = ServiceRegistry::new();
        registry.register(FakeService {
            capability: Capability::available("a"),
        });
        registry.register(FakeService {
            capability: Capability::unavailable("b", availability::UNSUPPORTED, "no hardware"),
        });
        let capabilities = registry.capabilities();
        assert_eq!(capabilities.len(), 2);
        assert_eq!(capabilities[0].name, "a");
        assert_eq!(capabilities[1].availability, availability::UNSUPPORTED);
    }

    #[test]
    fn builtin_service_reports_available() {
        let service = BuiltinService::new("0.1.0");
        let capability = service.probe();
        assert_eq!(capability.name, "backend");
        assert_eq!(capability.availability, availability::AVAILABLE);
    }
}
