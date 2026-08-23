//! Bluetooth control via `org.bluez` (`BlueZ`).

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use zettings_core::CapabilityState;

use crate::error::BackendError;

#[cfg(target_os = "linux")]
const SERVICE: &str = "bluetoothd";
#[cfg(target_os = "linux")]
const BUS_NAME: &str = "org.bluez";
#[cfg(target_os = "linux")]
const ROOT_PATH: &str = "/";
#[cfg(target_os = "linux")]
const ADAPTER_IFACE: &str = "org.bluez.Adapter1";
#[cfg(target_os = "linux")]
const DEVICE_IFACE: &str = "org.bluez.Device1";

/// A paired/known Bluetooth device.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BluetoothDevice {
    /// D-Bus object path (stable identity).
    pub path: String,
    /// Remote device name.
    pub alias: String,
    /// Paired to the local adapter.
    pub paired: bool,
    /// Currently connected.
    pub connected: bool,
}

/// Bluetooth radio + device surface.
#[async_trait]
pub trait BluetoothAdapter: Send + Sync {
    /// Detects whether `BlueZ` is running and an adapter exists.
    async fn capability(&self) -> CapabilityState;
    /// Reads adapter power state and known devices.
    ///
    /// # Errors
    /// [`BackendError::Service`] when `BlueZ` cannot be consulted.
    async fn status(&self) -> Result<BluetoothStatus, BackendError>;
    /// Powers the default adapter on/off. `BlueZ` enforces its own polkit
    /// policy for privileged changes.
    ///
    /// # Errors
    /// [`BackendError::Service`] on failure.
    async fn set_powered(&self, powered: bool) -> Result<(), BackendError>;
}

/// Adapter power + device snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BluetoothStatus {
    /// Honest capability of the adapter.
    pub capability: CapabilityState,
    /// Default adapter powered state; `None` when no adapter exists.
    pub powered: Option<bool>,
    /// Known devices across all adapters.
    pub devices: Vec<BluetoothDevice>,
}

/// Deterministic mock with set→get round-trip semantics.
#[derive(Debug)]
pub struct MockBluetooth {
    state: tokio::sync::Mutex<BluetoothStatus>,
}

impl Default for MockBluetooth {
    fn default() -> Self {
        Self {
            state: tokio::sync::Mutex::new(BluetoothStatus {
                capability: CapabilityState::Available,
                powered: Some(true),
                devices: vec![BluetoothDevice {
                    path: "/org/bluez/hci0/dev_Mock_Mouse".to_owned(),
                    alias: "Mock Mouse".to_owned(),
                    paired: true,
                    connected: true,
                }],
            }),
        }
    }
}

#[async_trait]
impl BluetoothAdapter for MockBluetooth {
    async fn capability(&self) -> CapabilityState {
        CapabilityState::Available
    }

    async fn status(&self) -> Result<BluetoothStatus, BackendError> {
        Ok(self.state.lock().await.clone())
    }

    async fn set_powered(&self, powered: bool) -> Result<(), BackendError> {
        self.state.lock().await.powered = Some(powered);
        Ok(())
    }
}

/// Real adapter over the system bus (Linux only).
#[cfg(target_os = "linux")]
pub struct LinuxBluetooth {
    conn: zbus::Connection,
}

#[cfg(target_os = "linux")]
impl LinuxBluetooth {
    /// Binds to an existing system-bus connection.
    pub const fn new(conn: zbus::Connection) -> Self {
        Self { conn }
    }

    async fn object_manager(&self) -> zbus::Result<zbus::fdo::ObjectManagerProxy<'_>> {
        zbus::fdo::ObjectManagerProxy::builder(&self.conn)
            .destination(BUS_NAME)?
            .path(ROOT_PATH)?
            .build()
            .await
    }
}

#[cfg(target_os = "linux")]
#[async_trait]
impl BluetoothAdapter for LinuxBluetooth {
    async fn capability(&self) -> CapabilityState {
        match self.object_manager().await {
            Ok(proxy) => match proxy.get_managed_objects().await {
                Ok(map) => {
                    let has_adapter = map
                        .values()
                        .any(|ifaces| ifaces.keys().any(|k| k.as_str() == ADAPTER_IFACE));
                    crate::service_available(has_adapter, SERVICE)
                }
                Err(e) => CapabilityState::Unavailable {
                    reason: format!("{SERVICE} enumeration failed: {e}"),
                },
            },
            Err(e) => CapabilityState::Unavailable {
                reason: format!("{SERVICE} proxy failed: {e}"),
            },
        }
    }

    async fn status(&self) -> Result<BluetoothStatus, BackendError> {
        let objects = self
            .object_manager()
            .await
            .map_err(|e| BackendError::service(SERVICE, e))?
            .get_managed_objects()
            .await
            .map_err(|e| BackendError::service(SERVICE, e))?;

        let mut powered = None;
        let mut devices = Vec::new();
        for (path, ifaces) in objects {
            for (iface, props) in &ifaces {
                match iface.as_str() {
                    ADAPTER_IFACE => {
                        if powered.is_none() {
                            powered = props
                                .get("Powered")
                                .and_then(|v| <bool>::try_from(v.clone()).ok());
                        }
                    }
                    DEVICE_IFACE => {
                        let alias = props
                            .get("Alias")
                            .and_then(|v| <String>::try_from(v.clone()).ok())
                            .unwrap_or_default();
                        let paired = props
                            .get("Paired")
                            .and_then(|v| <bool>::try_from(v.clone()).ok())
                            .unwrap_or(false);
                        let connected = props
                            .get("Connected")
                            .and_then(|v| <bool>::try_from(v.clone()).ok())
                            .unwrap_or(false);
                        devices.push(BluetoothDevice {
                            path: path.to_string(),
                            alias,
                            paired,
                            connected,
                        });
                    }
                    _ => {}
                }
            }
        }
        Ok(BluetoothStatus {
            capability: crate::service_available(powered.is_some(), SERVICE),
            powered,
            devices,
        })
    }

    async fn set_powered(&self, powered: bool) -> Result<(), BackendError> {
        // First adapter path from the object tree.
        let objects = self
            .object_manager()
            .await
            .map_err(|e| BackendError::service(SERVICE, e))?
            .get_managed_objects()
            .await
            .map_err(|e| BackendError::service(SERVICE, e))?;
        let adapter_path = objects
            .into_iter()
            .find(|(_, ifaces)| ifaces.keys().any(|k| k.as_str() == ADAPTER_IFACE))
            .map(|(path, _)| path)
            .ok_or_else(|| BackendError::service(SERVICE, "no bluetooth adapter present"))?;

        let adapter = zbus::Proxy::new(&self.conn, BUS_NAME, adapter_path.as_str(), ADAPTER_IFACE)
            .await
            .map_err(|e| BackendError::service(SERVICE, e))?;
        adapter
            .set_property("Powered", zbus::zvariant::Value::from(powered))
            .await
            .map_err(|e| BackendError::service(SERVICE, e))
    }
}
