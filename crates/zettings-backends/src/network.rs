//! Network status and radio control via `org.freedesktop.NetworkManager`.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use zettings_core::CapabilityState;

use crate::error::BackendError;

#[cfg(target_os = "linux")]
const SERVICE: &str = "NetworkManager";
#[cfg(target_os = "linux")]
const BUS_NAME: &str = "org.freedesktop.NetworkManager";
#[cfg(target_os = "linux")]
const OBJECT_PATH: &str = "/org/freedesktop/NetworkManager";

/// Coarse device classification exposed to the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NetworkDeviceKind {
    /// 802.11 wireless adapter.
    Wifi,
    /// Wired ethernet NIC.
    Ethernet,
    /// Modem / WWAN.
    Cellular,
    /// Anything else NM manages (loopback, VPN, bridge…).
    Other,
}

/// A single managed network device.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkDevice {
    /// Kernel interface name.
    pub interface: String,
    /// Device classification.
    pub kind: NetworkDeviceKind,
    /// NM numeric state (100 = activated); raw value keeps mapping honest.
    pub state: u32,
}

/// Connectivity snapshot for the shell.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkStatus {
    /// Honest capability of the adapter.
    pub capability: CapabilityState,
    /// Overall networking (NM manages connections at all).
    pub networking_enabled: bool,
    /// Wi-Fi radio state.
    pub wireless_enabled: bool,
    /// Managed devices.
    pub devices: Vec<NetworkDevice>,
}

/// Network control surface.
#[async_trait]
pub trait NetworkAdapter: Send + Sync {
    /// Detects whether `NetworkManager` is reachable.
    async fn capability(&self) -> CapabilityState;
    /// Reads the connectivity snapshot.
    ///
    /// # Errors
    /// [`BackendError::Service`] when the daemon cannot be consulted.
    async fn status(&self) -> Result<NetworkStatus, BackendError>;
    /// Enables/disables the Wi-Fi radio. NM enforces its own polkit policy.
    ///
    /// # Errors
    /// [`BackendError::Service`] on failure.
    async fn set_wireless_enabled(&self, enabled: bool) -> Result<(), BackendError>;
}

/// Deterministic mock with set→get round-trip semantics.
#[derive(Debug)]
pub struct MockNetwork {
    state: tokio::sync::Mutex<MockState>,
}

#[derive(Debug, Default)]
struct MockState {
    networking_enabled: bool,
    wireless_enabled: bool,
}

impl Default for MockNetwork {
    fn default() -> Self {
        Self {
            state: tokio::sync::Mutex::new(MockState {
                networking_enabled: true,
                wireless_enabled: true,
            }),
        }
    }
}

impl MockNetwork {
    /// Seed devices returned by [`NetworkAdapter::status`].
    pub fn mock_devices() -> Vec<NetworkDevice> {
        vec![
            NetworkDevice {
                interface: "wlan0".to_owned(),
                kind: NetworkDeviceKind::Wifi,
                state: 100,
            },
            NetworkDevice {
                interface: "eth0".to_owned(),
                kind: NetworkDeviceKind::Ethernet,
                state: 100,
            },
        ]
    }
}

#[async_trait]
impl NetworkAdapter for MockNetwork {
    async fn capability(&self) -> CapabilityState {
        CapabilityState::Available
    }

    async fn status(&self) -> Result<NetworkStatus, BackendError> {
        let s = self.state.lock().await;
        Ok(NetworkStatus {
            capability: CapabilityState::Available,
            networking_enabled: s.networking_enabled,
            wireless_enabled: s.wireless_enabled,
            devices: Self::mock_devices(),
        })
    }

    async fn set_wireless_enabled(&self, enabled: bool) -> Result<(), BackendError> {
        self.state.lock().await.wireless_enabled = enabled;
        Ok(())
    }
}

/// Real adapter over the system bus (Linux only).
#[cfg(target_os = "linux")]
pub struct LinuxNetwork {
    conn: zbus::Connection,
}

#[cfg(target_os = "linux")]
const NM_DEVICE_TYPE_WIFI: u32 = 2;
#[cfg(target_os = "linux")]
const NM_DEVICE_TYPE_ETHERNET: u32 = 1;
#[cfg(target_os = "linux")]
const NM_DEVICE_TYPE_MODEM: u32 = 8;

#[cfg(target_os = "linux")]
impl LinuxNetwork {
    /// Binds to an existing system-bus connection.
    pub const fn new(conn: zbus::Connection) -> Self {
        Self { conn }
    }

    fn proxy(&self) -> zbus::Result<zbus::Proxy<'_>> {
        zbus::Proxy::new(
            &self.conn,
            BUS_NAME,
            OBJECT_PATH,
            "org.freedesktop.NetworkManager",
        )
    }

    fn classify(kind: u32) -> NetworkDeviceKind {
        match kind {
            NM_DEVICE_TYPE_WIFI => NetworkDeviceKind::Wifi,
            NM_DEVICE_TYPE_ETHERNET => NetworkDeviceKind::Ethernet,
            NM_DEVICE_TYPE_MODEM => NetworkDeviceKind::Cellular,
            _ => NetworkDeviceKind::Other,
        }
    }
}

#[cfg(target_os = "linux")]
#[async_trait]
impl NetworkAdapter for LinuxNetwork {
    async fn capability(&self) -> CapabilityState {
        let probe = self
            .proxy()
            .and_then(|proxy| proxy.get_property::<bool>("NetworkingEnabled").map(|_| ()));
        match probe.await {
            Ok(()) => crate::service_available(true, SERVICE),
            Err(e) => CapabilityState::Unavailable {
                reason: format!("{SERVICE} probe failed: {e}"),
            },
        }
    }

    async fn status(&self) -> Result<NetworkStatus, BackendError> {
        let proxy = self
            .proxy()
            .map_err(|e| BackendError::service(SERVICE, e))?;
        let networking_enabled: bool = proxy
            .get_property("NetworkingEnabled")
            .await
            .map_err(|e| BackendError::service(SERVICE, e))?;
        let wireless_enabled: bool = proxy
            .get_property("WirelessEnabled")
            .await
            .map_err(|e| BackendError::service(SERVICE, e))?;
        let paths: Vec<zbus::zvariant::OwnedObjectPath> = proxy
            .call("GetDevices", &())
            .await
            .map_err(|e| BackendError::service(SERVICE, e))?;

        let mut devices = Vec::with_capacity(paths.len());
        for path in paths {
            let Ok(device) = zbus::Proxy::new(
                &self.conn,
                BUS_NAME,
                path.as_str(),
                "org.freedesktop.NetworkManager.Device",
            ) else {
                continue;
            };
            let interface: String = match device.get_property("Interface").await {
                Ok(v) => v,
                Err(_) => continue,
            };
            let kind: u32 = device.get_property("DeviceType").await.unwrap_or(0);
            let state: u32 = device.get_property("State").await.unwrap_or(0);
            devices.push(NetworkDevice {
                interface,
                kind: Self::classify(kind),
                state,
            });
        }
        Ok(NetworkStatus {
            capability: crate::service_available(true, SERVICE),
            networking_enabled,
            wireless_enabled,
            devices,
        })
    }

    async fn set_wireless_enabled(&self, enabled: bool) -> Result<(), BackendError> {
        let proxy = self
            .proxy()
            .map_err(|e| BackendError::service(SERVICE, e))?;
        proxy
            .set_property("WirelessEnabled", zbus::zvariant::Value::from(enabled))
            .await
            .map_err(|e| BackendError::service(SERVICE, e))
    }
}
