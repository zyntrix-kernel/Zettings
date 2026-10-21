//! Bluetooth adapter and paired peripheral models.

use serde::{Deserialize, Serialize};
use zvariant::Type;

/// Bluetooth adapter and peripheral status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct BluetoothStatus {
    pub enabled: bool,
    pub adapter_name: String,
    pub discovering: bool,
    pub connected_devices_count: u32,
    pub devices: Vec<BluetoothDevice>,
}

/// One Bluetooth peripheral device.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct BluetoothDevice {
    pub name: String,
    pub address: String,
    pub connected: bool,
    pub paired: bool,
    pub battery_percentage: i32,
    pub device_type: String,
}
