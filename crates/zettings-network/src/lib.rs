//! `zettings-network` — `NetworkManager`/`BlueZ` integration.
//!
//! Hostname management, Wi-Fi scanning, VPN activation, and Bluetooth device
//! pairing through `org.freedesktop.NetworkManager` and `org.bluez` over
//! `zbus` on Linux. The Windows dev loop uses an in-memory [`MockBackend`].

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use serde::{Deserialize, Serialize};
use thiserror::Error;
use zettings_bus::Bus;
use zettings_bus::events::LinkState;

/// A network interface id assigned by `NetworkManager`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct InterfaceIndex(pub u32);

/// A scanned Wi-Fi access point.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct AccessPoint {
    /// SSID as a fixed-size byte array (truncated/padded to 32 bytes per
    /// IEEE 802.11). Real impls surface it as a string when UTF-8 valid.
    pub ssid: [u8; 32],
    /// Length of the SSID in bytes (the rest of the array is padding).
    pub ssid_len: u8,
    /// Signal strength in dBm, typically in `[-100, 0]`.
    pub signal_dbm: i8,
    /// `true` when the AP requires authentication.
    pub secured: bool,
}

impl AccessPoint {
    /// Construct an AP from an SSID byte slice, padding to 32 bytes.
    /// SSIDs longer than 32 bytes (the IEEE 802.11 maximum) are truncated.
    #[must_use]
    pub fn from_ssid(ssid: &[u8], signal_dbm: i8, secured: bool) -> Self {
        let mut buf = [0u8; 32];
        let len = ssid.len().min(32);
        buf[..len].copy_from_slice(&ssid[..len]);
        Self {
            ssid: buf,
            ssid_len: u8::try_from(len).unwrap_or(u8::MAX),
            signal_dbm,
            secured,
        }
    }

    /// The active SSID bytes (no padding).
    #[must_use]
    pub fn ssid_bytes(&self) -> &[u8] {
        &self.ssid[..usize::from(self.ssid_len)]
    }
}

/// A paired `BlueZ` peripheral. Surfaced to the frontend for the Bluetooth
/// panel's paired-device list with battery level (when the device advertises
/// the `org.bluez.Battery1` interface).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PairedDevice {
    /// `BlueZ` object-path-leaf identifier, stable across daemon restarts.
    pub address: String,
    /// Human-readable device name (model / manufacturer / friendly alias).
    pub name: String,
    /// `true` when the device is currently connected (not just paired).
    pub connected: bool,
    /// Battery charge in `[0, 100]`. `None` for devices without a Battery1
    /// interface — the UI then hides the battery indicator rather than
    /// showing a 0% "empty" glyph (ui-ux-pro-max/ Accessibility/Color-Only:
    /// never imply low battery purely by an absent number).
    pub battery_percent: Option<u8>,
    /// Major device class as reported by `BlueZ` (`Audio`, `Peripheral`,
    /// `Phone`, etc.).
    pub device_class: String,
}

/// Errors surfaced by the network backend.
#[derive(Debug, Error)]
pub enum NetworkError {
    /// Hostname validation failed (length, charset).
    #[error("invalid hostname: {0}")]
    InvalidHostname(String),
    /// The underlying `NetworkManager`/`BlueZ` service was unreachable.
    #[error("network service unavailable: {0}")]
    ServiceUnavailable(String),
}

/// Trait surface implemented by both the mock state machine and the real
/// `zbus` backend.
pub trait Backend: Send + Sync {
    /// Read the current system hostname.
    ///
    /// # Errors
    /// Returns [`NetworkError::ServiceUnavailable`] when `NetworkManager` is
    /// unreachable on the real target.
    fn hostname(&self) -> Result<String, NetworkError>;

    /// Apply a new hostname. Emits a [`LinkState`] event with the host
    /// loopback flag set to `true` to signal that identity changed.
    ///
    /// # Errors
    /// - [`NetworkError::InvalidHostname`] when validation fails.
    fn set_hostname(&self, hostname: &str, bus: &Bus) -> Result<(), NetworkError>;

    /// Scan for nearby Wi-Fi access points.
    ///
    /// # Errors
    /// Returns [`NetworkError::ServiceUnavailable`] on the real target until
    /// the `zbus` Wi-Fi scan lands.
    fn scan_wifi(&self) -> Result<Vec<AccessPoint>, NetworkError>;

    /// List paired (and optionally connected) `BlueZ` peripherals.
    ///
    /// # Errors
    /// Returns [`NetworkError::ServiceUnavailable`] when `BlueZ` is
    /// unreachable on the real target.
    fn list_paired_devices(&self) -> Result<Vec<PairedDevice>, NetworkError>;
}

/// In-memory mock backend. Holds the current hostname and a fixed set of
/// simulated access points so the frontend rendering paths can be exercised.
pub struct MockBackend {
    state: parking_lot::Mutex<String>,
}

impl MockBackend {
    /// Construct a mock backend whose hostname is `zettings-mock`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: parking_lot::Mutex::new("zettings-mock".to_string()),
        }
    }
}

impl Default for MockBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl Backend for MockBackend {
    fn hostname(&self) -> Result<String, NetworkError> {
        Ok(self.state.lock().clone())
    }

    fn set_hostname(&self, hostname: &str, bus: &Bus) -> Result<(), NetworkError> {
        if !is_valid_hostname(hostname) {
            return Err(NetworkError::InvalidHostname(hostname.to_string()));
        }
        *self.state.lock() = hostname.to_string();
        let _ = bus.publish(LinkState {
            interface_index: 0,
            link_up: true,
        });
        Ok(())
    }

    fn scan_wifi(&self) -> Result<Vec<AccessPoint>, NetworkError> {
        Ok(vec![
            AccessPoint::from_ssid(b"Zyntrix-Aurora", -42, true),
            AccessPoint::from_ssid(b"open-guest", -67, false),
            AccessPoint::from_ssid(b"Hidden", -88, true),
        ])
    }

    fn list_paired_devices(&self) -> Result<Vec<PairedDevice>, NetworkError> {
        Ok(vec![
            PairedDevice {
                address: "AA:BB:CC:11:22:33".into(),
                name: "Zyntrix Aurora Buds".into(),
                connected: true,
                battery_percent: Some(78),
                device_class: "Audio".into(),
            },
            PairedDevice {
                address: "DD:EE:FF:44:55:66".into(),
                name: "Zyntrix Trackpad".into(),
                connected: true,
                battery_percent: Some(42),
                device_class: "Peripheral".into(),
            },
            PairedDevice {
                address: "11:22:33:AA:BB:CC".into(),
                name: "Snowpeak Keyboard".into(),
                connected: false,
                battery_percent: None,
                device_class: "Peripheral".into(),
            },
        ])
    }
}

/// RFC 1123 hostname validity: 1-63 chars, `[A-Za-z0-9-]`, no leading/trailing
/// `-`. Strict enough for `NetworkManager` without pulling a regex dep.
fn is_valid_hostname(s: &str) -> bool {
    if s.is_empty() || s.len() > 63 {
        return false;
    }
    if s.starts_with('-') || s.ends_with('-') {
        return false;
    }
    s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

/// Real Linux backend. Connects to `org.freedesktop.NetworkManager` over
/// `zbus`.
#[cfg(all(target_os = "linux", not(feature = "zettings-mock")))]
pub struct LinuxBackend;

#[cfg(all(target_os = "linux", not(feature = "zettings-mock")))]
impl Backend for LinuxBackend {
    fn hostname(&self) -> Result<String, NetworkError> {
        Err(NetworkError::ServiceUnavailable(
            "NetworkManager zbus integration pending".into(),
        ))
    }

    fn set_hostname(&self, _hostname: &str, _bus: &Bus) -> Result<(), NetworkError> {
        Err(NetworkError::ServiceUnavailable(
            "NetworkManager zbus integration pending".into(),
        ))
    }

    fn scan_wifi(&self) -> Result<Vec<AccessPoint>, NetworkError> {
        Err(NetworkError::ServiceUnavailable(
            "NetworkManager zbus integration pending".into(),
        ))
    }

    fn list_paired_devices(&self) -> Result<Vec<PairedDevice>, NetworkError> {
        Err(NetworkError::ServiceUnavailable(
            "BlueZ zbus integration pending".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_hostname_default() {
        let b = MockBackend::new();
        assert_eq!(b.hostname().expect("hostname"), "zettings-mock");
    }

    #[tokio::test]
    async fn mock_set_hostname_publishes_linkstate() {
        let b = MockBackend::new();
        let bus = Bus::new();
        let mut rx = bus.subscribe::<LinkState>();
        b.set_hostname("aurora-dev", &bus).expect("set");
        let evt = rx.recv().await.expect("event");
        assert!(evt.link_up);
    }

    #[tokio::test]
    async fn mock_rejects_invalid_hostname() {
        let b = MockBackend::new();
        let bus = Bus::new();
        let err = b.set_hostname("bad_host!", &bus).unwrap_err();
        assert!(matches!(err, NetworkError::InvalidHostname(_)));
    }

    #[tokio::test]
    async fn mock_rejects_overlong_hostname() {
        let b = MockBackend::new();
        let bus = Bus::new();
        let long = "a".repeat(64);
        let err = b.set_hostname(&long, &bus).unwrap_err();
        assert!(matches!(err, NetworkError::InvalidHostname(_)));
    }

    #[tokio::test]
    async fn mock_scan_returns_three_aps() {
        let b = MockBackend::new();
        let aps = b.scan_wifi().expect("scan");
        assert_eq!(aps.len(), 3);
        assert!(aps[0].secured);
        assert!(!aps[1].secured);
    }

    #[tokio::test]
    async fn mock_paired_returns_three_devices() {
        let b = MockBackend::new();
        let devices = b.list_paired_devices().expect("paired");
        assert_eq!(devices.len(), 3);
        // The connected earbuds report a battery level; the disconnected
        // keyboard (no Battery1 interface) yields `None`.
        let buds = devices
            .iter()
            .find(|d| d.name == "Zyntrix Aurora Buds")
            .expect("buds present");
        assert!(buds.connected);
        assert_eq!(buds.battery_percent, Some(78));
        let kb = devices
            .iter()
            .find(|d| d.name == "Snowpeak Keyboard")
            .expect("keyboard present");
        assert!(!kb.connected);
        assert_eq!(kb.battery_percent, None);
    }
}
