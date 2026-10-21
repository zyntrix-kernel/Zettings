//! D-Bus interface served by the daemon (`org.zyntrix.Zettings`).
//!
//! Every method is typed: structured inputs and outputs, [`BackendError`]
//! failures. There is intentionally no generic "run anything" method.

use zbus::{fdo, interface};

use super::validation;
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
    pub fn ping(&self) -> String {
        format!("zettings-backend {}", self.version)
    }

    /// Crate version string.
    pub fn get_version(&self) -> String {
        self.version.clone()
    }

    /// Availability of every registered integration.
    pub fn get_capabilities(&self) -> Vec<Capability> {
        self.registry.capabilities()
    }

    /// Full system snapshot (OS, CPU, memory, disks, GPUs, session).
    pub fn get_system_info(&self) -> fdo::Result<SystemInfo> {
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
    pub fn get_network_status(&self) -> fdo::Result<NetworkStatus> {
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
    pub fn set_wifi_enabled(&self, enabled: bool) -> fdo::Result<()> {
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
    pub fn get_bluetooth_status(&self) -> fdo::Result<BluetoothStatus> {
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
    pub fn set_bluetooth_enabled(&self, enabled: bool) -> fdo::Result<()> {
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
    pub fn get_power_status(&self) -> fdo::Result<PowerStatus> {
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
    pub fn set_power_profile(&self, profile: String) -> fdo::Result<()> {
        validation::validate_power_profile(&profile)?;
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
    pub fn get_audio_status(&self) -> fdo::Result<AudioStatus> {
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
    pub fn set_master_volume(&self, volume: f64) -> fdo::Result<()> {
        validation::validate_master_volume(volume)?;
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
    pub fn set_master_mute(&self, muted: bool) -> fdo::Result<()> {
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
    pub fn get_users(&self) -> fdo::Result<Vec<UserInfo>> {
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
    /// unsupported, exercising the [`BackendError`] -> D-Bus mapping.
    pub fn unsupported_operation(&self) -> zbus::fdo::Result<String> {
        Err(BackendError::new(
            crate::error::ErrorKind::Unsupported,
            "unsupported_operation is a mapping probe, not a real operation",
        )
        .into())
    }
}
