//! D-Bus interface served by the daemon (`org.zyntrix.Zettings`).
//!
//! Every method is typed: structured inputs and outputs, [`BackendError`]
//! failures. There is intentionally no generic "run anything" method.

use zbus::{fdo, interface, proxy};

use crate::error::{BackendError, ErrorKind};
use crate::models::{
    AudioStatus, BluetoothStatus, Capability, NetworkStatus, PowerStatus, SystemInfo, UserInfo,
};
use crate::services::accounts::AccountsService;
use crate::services::audio::AudioService;
use crate::services::bluetooth::BluetoothService;
use crate::services::network::NetworkService;
use crate::services::power::PowerService;
use crate::services::system_info::SystemInfoService;
use crate::services::{BuiltinService, ServiceRegistry};

/// Object served at [`OBJECT_PATH`] on [`BUS_NAME`].
pub struct Backend {
    version: String,
    registry: ServiceRegistry,
}

impl Backend {
    pub fn new(version: impl Into<String>, mut registry: ServiceRegistry) -> Self {
        let version = version.into();
        registry.register(BuiltinService::new(version.clone()));
        Self { version, registry }
    }

    /// Crate version baked in at compile time.
    pub fn version(&self) -> &str {
        &self.version
    }
}

#[interface(name = "org.zyntrix.Zettings")]
impl Backend {
    /// Liveness check. Returns e.g. `"zettings-backend 0.1.0"`.
    fn ping(&self) -> String {
        format!("zettings-backend {}", self.version)
    }

    /// Crate version string.
    fn get_version(&self) -> String {
        self.version.clone()
    }

    /// Availability of every registered integration.
    fn get_capabilities(&self) -> Vec<Capability> {
        self.registry.capabilities()
    }

    /// Full system snapshot (OS, CPU, memory, disks, GPUs, session).
    fn get_system_info(&self) -> fdo::Result<SystemInfo> {
        let service = self
            .registry
            .get("system")
            .and_then(|service| service.as_any().downcast_ref::<SystemInfoService>())
            .ok_or_else(|| {
                BackendError::new(ErrorKind::SystemError, "system service is not registered")
            })?;
        Ok(service.get())
    }

    /// Full network status and connection info.
    fn get_network_status(&self) -> fdo::Result<NetworkStatus> {
        let service = self
            .registry
            .get("network")
            .and_then(|s| s.as_any().downcast_ref::<NetworkService>())
            .ok_or_else(|| {
                BackendError::new(ErrorKind::SystemError, "network service is not registered")
            })?;
        Ok(service.get_status())
    }

    /// Enable or disable Wi-Fi subsystem.
    fn set_wifi_enabled(&self, enabled: bool) -> fdo::Result<()> {
        let service = self
            .registry
            .get("network")
            .and_then(|s| s.as_any().downcast_ref::<NetworkService>())
            .ok_or_else(|| {
                BackendError::new(ErrorKind::SystemError, "network service is not registered")
            })?;
        service.set_wifi_enabled(enabled);
        Ok(())
    }

    /// Bluetooth adapter and peripheral status.
    fn get_bluetooth_status(&self) -> fdo::Result<BluetoothStatus> {
        let service = self
            .registry
            .get("bluetooth")
            .and_then(|s| s.as_any().downcast_ref::<BluetoothService>())
            .ok_or_else(|| {
                BackendError::new(
                    ErrorKind::SystemError,
                    "bluetooth service is not registered",
                )
            })?;
        Ok(service.get_status())
    }

    /// Enable or disable Bluetooth adapter.
    fn set_bluetooth_enabled(&self, enabled: bool) -> fdo::Result<()> {
        let service = self
            .registry
            .get("bluetooth")
            .and_then(|s| s.as_any().downcast_ref::<BluetoothService>())
            .ok_or_else(|| {
                BackendError::new(
                    ErrorKind::SystemError,
                    "bluetooth service is not registered",
                )
            })?;
        service.set_enabled(enabled);
        Ok(())
    }

    /// System power, battery, and power-profile status.
    fn get_power_status(&self) -> fdo::Result<PowerStatus> {
        let service = self
            .registry
            .get("power")
            .and_then(|s| s.as_any().downcast_ref::<PowerService>())
            .ok_or_else(|| {
                BackendError::new(ErrorKind::SystemError, "power service is not registered")
            })?;
        Ok(service.get_status())
    }

    /// Switch active power profile (e.g. "performance", "balanced", "power-saver").
    fn set_power_profile(&self, profile: String) -> fdo::Result<()> {
        let service = self
            .registry
            .get("power")
            .and_then(|s| s.as_any().downcast_ref::<PowerService>())
            .ok_or_else(|| {
                BackendError::new(ErrorKind::SystemError, "power service is not registered")
            })?;
        service
            .set_profile(&profile)
            .map_err(|e| BackendError::new(ErrorKind::InvalidInput, e))?;
        Ok(())
    }

    /// Master audio routing and volume status.
    fn get_audio_status(&self) -> fdo::Result<AudioStatus> {
        let service = self
            .registry
            .get("audio")
            .and_then(|s| s.as_any().downcast_ref::<AudioService>())
            .ok_or_else(|| {
                BackendError::new(ErrorKind::SystemError, "audio service is not registered")
            })?;
        Ok(service.get_status())
    }

    /// Set master audio volume (0.0 .. 1.0).
    fn set_master_volume(&self, volume: f64) -> fdo::Result<()> {
        let service = self
            .registry
            .get("audio")
            .and_then(|s| s.as_any().downcast_ref::<AudioService>())
            .ok_or_else(|| {
                BackendError::new(ErrorKind::SystemError, "audio service is not registered")
            })?;
        service.set_volume(volume);
        Ok(())
    }

    /// Set master audio mute state.
    fn set_master_mute(&self, muted: bool) -> fdo::Result<()> {
        let service = self
            .registry
            .get("audio")
            .and_then(|s| s.as_any().downcast_ref::<AudioService>())
            .ok_or_else(|| {
                BackendError::new(ErrorKind::SystemError, "audio service is not registered")
            })?;
        service.set_muted(muted);
        Ok(())
    }

    /// Local human user accounts.
    fn get_users(&self) -> fdo::Result<Vec<UserInfo>> {
        let service = self
            .registry
            .get("accounts")
            .and_then(|s| s.as_any().downcast_ref::<AccountsService>())
            .ok_or_else(|| {
                BackendError::new(ErrorKind::SystemError, "accounts service is not registered")
            })?;
        Ok(service.get_users())
    }

    /// Demonstrates typed failure: always reports this operation as
    /// unsupported, exercising the [`BackendError`] → D-Bus mapping.
    fn unsupported_operation(&self) -> zbus::fdo::Result<String> {
        Err(BackendError::new(
            crate::error::ErrorKind::Unsupported,
            "unsupported_operation is a mapping probe, not a real operation",
        )
        .into())
    }
}

/// Typed client used by integration tests and Rust tooling.
#[proxy(
    interface = "org.zyntrix.Zettings",
    default_service = "org.zyntrix.Zettings",
    default_path = "/org/zyntrix/Zettings"
)]
pub trait ZettingsProxy {
    async fn ping(&self) -> Result<String, zbus::Error>;
    async fn get_version(&self) -> Result<String, zbus::Error>;
    async fn get_capabilities(&self) -> Result<Vec<Capability>, zbus::Error>;
    async fn get_system_info(&self) -> Result<SystemInfo, zbus::Error>;
    async fn get_network_status(&self) -> Result<NetworkStatus, zbus::Error>;
    async fn set_wifi_enabled(&self, enabled: bool) -> Result<(), zbus::Error>;
    async fn get_bluetooth_status(&self) -> Result<BluetoothStatus, zbus::Error>;
    async fn set_bluetooth_enabled(&self, enabled: bool) -> Result<(), zbus::Error>;
    async fn get_power_status(&self) -> Result<PowerStatus, zbus::Error>;
    async fn set_power_profile(&self, profile: String) -> Result<(), zbus::Error>;
    async fn get_audio_status(&self) -> Result<AudioStatus, zbus::Error>;
    async fn set_master_volume(&self, volume: f64) -> Result<(), zbus::Error>;
    async fn set_master_mute(&self, muted: bool) -> Result<(), zbus::Error>;
    async fn get_users(&self) -> Result<Vec<UserInfo>, zbus::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn backend_with_all_services() -> Backend {
        let mut registry = ServiceRegistry::new();
        registry.register(SystemInfoService::new());
        registry.register(NetworkService::new());
        registry.register(BluetoothService::new());
        registry.register(PowerService::new());
        registry.register(AudioService::new());
        registry.register(AccountsService::new());
        Backend::new("9.9.9-test", registry)
    }

    #[test]
    fn ping_reports_version() {
        let backend = Backend::new("9.9.9-test", ServiceRegistry::new());
        assert_eq!(backend.ping(), "zettings-backend 9.9.9-test");
    }

    #[test]
    fn capabilities_include_registered_services() {
        let backend = backend_with_all_services();
        let capabilities = backend.get_capabilities();
        assert!(capabilities.iter().any(|c| c.name == "backend"));
        assert!(capabilities.iter().any(|c| c.name == "system"));
        assert!(capabilities.iter().any(|c| c.name == "network"));
        assert!(capabilities.iter().any(|c| c.name == "bluetooth"));
        assert!(capabilities.iter().any(|c| c.name == "power"));
        assert!(capabilities.iter().any(|c| c.name == "audio"));
        assert!(capabilities.iter().any(|c| c.name == "accounts"));
    }

    #[test]
    fn network_methods_work_through_backend() {
        let backend = backend_with_all_services();
        let status = backend.get_network_status().expect("must succeed");
        assert!(status.wifi_enabled);

        backend.set_wifi_enabled(false).expect("must succeed");
        let status = backend.get_network_status().expect("must succeed");
        assert!(!status.wifi_enabled);
    }

    #[test]
    fn bluetooth_methods_work_through_backend() {
        let backend = backend_with_all_services();
        let status = backend.get_bluetooth_status().expect("must succeed");
        assert!(status.enabled);

        backend.set_bluetooth_enabled(false).expect("must succeed");
        let status = backend.get_bluetooth_status().expect("must succeed");
        assert!(!status.enabled);
    }

    #[test]
    fn power_and_audio_methods_work_through_backend() {
        let backend = backend_with_all_services();
        let power = backend.get_power_status().expect("must succeed");
        assert_eq!(power.active_profile, "balanced");

        backend
            .set_power_profile("performance".to_string())
            .expect("must succeed");
        let power = backend.get_power_status().expect("must succeed");
        assert_eq!(power.active_profile, "performance");

        let audio = backend.get_audio_status().expect("must succeed");
        assert!(!audio.muted);

        backend.set_master_mute(true).expect("must succeed");
        backend.set_master_volume(0.42).expect("must succeed");
        let audio = backend.get_audio_status().expect("must succeed");
        assert!(audio.muted);
        assert!((audio.master_volume - 0.42).abs() < f64::EPSILON);
    }

    #[test]
    fn user_accounts_work_through_backend() {
        let backend = backend_with_all_services();
        let users = backend.get_users().expect("must succeed");
        assert!(!users.is_empty());
    }

    #[test]
    fn unsupported_operation_fails_typed() {
        let backend = backend_with_all_services();
        let error = backend.unsupported_operation().unwrap_err();
        assert!(matches!(error, zbus::fdo::Error::NotSupported(_)));
        assert!(error.to_string().contains("unsupported"));
    }
}
