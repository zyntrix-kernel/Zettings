//! Typed proxy client used by integration tests and Rust consumers.

use zbus::proxy;

use crate::models::{
    AudioStatus, BluetoothStatus, Capability, NetworkStatus, PowerStatus, SystemInfo, UserInfo,
};

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
