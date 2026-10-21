//! Network status and scanned Wi-Fi access points.

use serde::{Deserialize, Serialize};
use zvariant::Type;

/// Network status and active connection details.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct NetworkStatus {
    pub wifi_enabled: bool,
    pub connected: bool,
    pub active_ssid: String,
    pub ipv4_address: String,
    pub interface_name: String,
    pub signal_strength: u8,
    pub wifi_networks: Vec<WifiNetwork>,
}

/// Scanned Wi-Fi access point.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct WifiNetwork {
    pub ssid: String,
    pub signal_strength: u8,
    pub secured: bool,
    pub security_type: String,
}
