//! System adapter snapshot payloads (Phase 5 surface).
use serde::{Deserialize, Serialize};
use zettings_backends::{
    AudioSink, BluetoothDevice, BluetoothStatus, DisplayOutput, DisplayStatus, NetworkDevice,
    NetworkDeviceKind, NetworkStatus, PowerProfileSnapshot, SessionCapabilities,
};
use zettings_core::CapabilityState;

/// Wire mirror of [`CapabilityState`] (honest availability reporting).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "capability-state.ts")]
#[serde(tag = "state", rename_all = "kebab-case")]
pub enum CapabilityStateDto {
    /// Fully functional.
    Available,
    /// Partially functional.
    Degraded {
        /// What is limited and why.
        reason: String,
    },
    /// Cannot be used in this environment.
    Unavailable {
        /// Why.
        reason: String,
    },
}

impl From<&CapabilityState> for CapabilityStateDto {
    fn from(value: &CapabilityState) -> Self {
        // `#[non_exhaustive]` upstream: match through reference with a
        // catch-all so future variants degrade honestly instead of breaking.
        let reason = value.reason().map(str::to_owned).unwrap_or_default();
        match value {
            CapabilityState::Available => Self::Available,
            CapabilityState::Degraded { .. } => Self::Degraded { reason },
            _ => Self::Unavailable { reason },
        }
    }
}

/// Power-profiles snapshot on the wire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "power-snapshot.ts")]
pub struct PowerProfileSnapshotDto {
    /// Adapter capability.
    pub capability: CapabilityStateDto,
    /// Profiles offered by the daemon.
    pub available: Vec<String>,
    /// Active profile; empty when unavailable.
    pub active: String,
}

impl From<&PowerProfileSnapshot> for PowerProfileSnapshotDto {
    fn from(value: &PowerProfileSnapshot) -> Self {
        Self {
            capability: CapabilityStateDto::from(&value.capability),
            available: value.available.clone(),
            active: value.active.clone(),
        }
    }
}

/// Managed device entry on the wire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "network-device.ts")]
pub struct NetworkDeviceDto {
    /// Kernel interface name.
    pub interface: String,
    /// Device kind (kebab-case of the backend enum).
    pub kind: String,
    /// Raw NM state value.
    pub state: u32,
}

impl From<&NetworkDevice> for NetworkDeviceDto {
    fn from(value: &NetworkDevice) -> Self {
        Self {
            interface: value.interface.clone(),
            kind: match value.kind {
                NetworkDeviceKind::Wifi => "wifi",
                NetworkDeviceKind::Ethernet => "ethernet",
                NetworkDeviceKind::Cellular => "cellular",
                NetworkDeviceKind::Other => "other",
            }
            .to_owned(),
            state: value.state,
        }
    }
}

/// Network snapshot on the wire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "network-status.ts")]
pub struct NetworkStatusDto {
    /// Adapter capability.
    pub capability: CapabilityStateDto,
    /// Whether NM manages connections at all.
    pub networking_enabled: bool,
    /// Wi-Fi radio state.
    pub wireless_enabled: bool,
    /// Managed devices.
    pub devices: Vec<NetworkDeviceDto>,
}

impl From<&NetworkStatus> for NetworkStatusDto {
    fn from(value: &NetworkStatus) -> Self {
        Self {
            capability: CapabilityStateDto::from(&value.capability),
            networking_enabled: value.networking_enabled,
            wireless_enabled: value.wireless_enabled,
            devices: value.devices.iter().map(NetworkDeviceDto::from).collect(),
        }
    }
}

/// Session power-action availability on the wire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "session-capabilities.ts")]
pub struct SessionCapabilitiesDto {
    /// Adapter capability.
    pub capability: CapabilityStateDto,
    /// logind verdicts (`yes`/`no`/`challenge`).
    pub can_power_off: String,
    /// Reboot verdict.
    pub can_reboot: String,
    /// Suspend verdict.
    pub can_suspend: String,
}

impl From<&SessionCapabilities> for SessionCapabilitiesDto {
    fn from(value: &SessionCapabilities) -> Self {
        Self {
            capability: CapabilityStateDto::from(&value.capability),
            can_power_off: value.can_power_off.clone(),
            can_reboot: value.can_reboot.clone(),
            can_suspend: value.can_suspend.clone(),
        }
    }
}

/// Aggregate system snapshot served to the shell's System surfaces.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "system-snapshot.ts")]
pub struct SystemSnapshotDto {
    /// Power profiles area.
    pub power: PowerProfileSnapshotDto,
    /// Network area.
    pub network: NetworkStatusDto,
    /// Session actions area.
    pub session: SessionCapabilitiesDto,
    /// Audio output area.
    pub audio: AudioAreaDto,
    /// Bluetooth area.
    pub bluetooth: BluetoothStatusDto,
    /// Display area (read-only in this phase).
    pub display: DisplayStatusDto,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dto_mapping_preserves_honest_states() {
        let unavailable = CapabilityStateDto::from(&CapabilityState::Unavailable {
            reason: "not running".into(),
        });
        assert_eq!(
            unavailable,
            CapabilityStateDto::Unavailable {
                reason: "not running".into()
            }
        );
    }

    #[test]
    fn snapshot_serializes_round_trip() {
        let snap = SystemSnapshotDto {
            power: PowerProfileSnapshotDto {
                capability: CapabilityStateDto::Available,
                available: vec!["balanced".into()],
                active: "balanced".into(),
            },
            network: NetworkStatusDto {
                capability: CapabilityStateDto::Available,
                networking_enabled: true,
                wireless_enabled: false,
                devices: vec![NetworkDeviceDto {
                    interface: "wlan0".into(),
                    kind: "wifi".into(),
                    state: 100,
                }],
            },
            session: SessionCapabilitiesDto {
                capability: CapabilityStateDto::Available,
                can_power_off: "yes".into(),
                can_reboot: "yes".into(),
                can_suspend: "challenge".into(),
            },
            audio: AudioAreaDto {
                capability: CapabilityStateDto::Available,
                sinks: vec![AudioSinkDto {
                    name: "mock-output".into(),
                    description: "Mock output".into(),
                    muted: false,
                    volume_percent: 80,
                    is_default: true,
                }],
            },
            bluetooth: BluetoothStatusDto {
                capability: CapabilityStateDto::Available,
                powered: Some(true),
                devices: vec![],
            },
            display: DisplayStatusDto {
                capability: CapabilityStateDto::Available,
                outputs: vec![],
            },
        };
        let json = serde_json::to_string(&snap).expect("serialize");
        let back: SystemSnapshotDto = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, snap);
    }
}

/// Audio sink on the wire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "audio-sink.ts")]
pub struct AudioSinkDto {
    /// Stable sink name.
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Mute state.
    pub muted: bool,
    /// Average volume 0-150%.
    pub volume_percent: u32,
    /// Default output flag.
    pub is_default: bool,
}

impl From<&AudioSink> for AudioSinkDto {
    fn from(value: &AudioSink) -> Self {
        Self {
            name: value.name.clone(),
            description: value.description.clone(),
            muted: value.muted,
            volume_percent: value.volume_percent,
            is_default: value.is_default,
        }
    }
}

/// Audio area snapshot with its own honest capability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "audio-area.ts")]
pub struct AudioAreaDto {
    /// Adapter capability.
    pub capability: CapabilityStateDto,
    /// Output sinks (empty when unavailable).
    pub sinks: Vec<AudioSinkDto>,
}

/// Bluetooth device on the wire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "bluetooth-device.ts")]
pub struct BluetoothDeviceDto {
    /// Remote device name.
    pub alias: String,
    /// Paired flag.
    pub paired: bool,
    /// Connected flag.
    pub connected: bool,
}

impl From<&BluetoothDevice> for BluetoothDeviceDto {
    fn from(value: &BluetoothDevice) -> Self {
        Self {
            alias: value.alias.clone(),
            paired: value.paired,
            connected: value.connected,
        }
    }
}

/// Bluetooth area snapshot on the wire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "bluetooth-status.ts")]
pub struct BluetoothStatusDto {
    /// Adapter capability.
    pub capability: CapabilityStateDto,
    /// Powered state; null when no adapter exists.
    pub powered: Option<bool>,
    /// Known devices.
    pub devices: Vec<BluetoothDeviceDto>,
}

impl From<&BluetoothStatus> for BluetoothStatusDto {
    fn from(value: &BluetoothStatus) -> Self {
        Self {
            capability: CapabilityStateDto::from(&value.capability),
            powered: value.powered,
            devices: value.devices.iter().map(BluetoothDeviceDto::from).collect(),
        }
    }
}

/// Display connector on the wire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "display-output.ts")]
pub struct DisplayOutputDto {
    /// DRM connector name.
    pub name: String,
    /// Connected flag.
    pub connected: bool,
    /// Supported modes (best first).
    pub modes: Vec<String>,
    /// Active mode; empty when off/disconnected.
    pub current_mode: String,
}

impl From<&DisplayOutput> for DisplayOutputDto {
    fn from(value: &DisplayOutput) -> Self {
        Self {
            name: value.name.clone(),
            connected: value.connected,
            modes: value.modes.clone(),
            current_mode: value.current_mode.clone(),
        }
    }
}

/// Display area snapshot on the wire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "display-status.ts")]
pub struct DisplayStatusDto {
    /// Adapter capability.
    pub capability: CapabilityStateDto,
    /// Detected connectors.
    pub outputs: Vec<DisplayOutputDto>,
}

impl From<&DisplayStatus> for DisplayStatusDto {
    fn from(value: &DisplayStatus) -> Self {
        Self {
            capability: CapabilityStateDto::from(&value.capability),
            outputs: value.outputs.iter().map(DisplayOutputDto::from).collect(),
        }
    }
}
