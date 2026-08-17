//! Standard Zettings event types published over [`crate::Bus`].
//!
//! Domain backend crates (`zettings-display`, `zettings-audio`, ...) emit these
//! events whenever the underlying system service changes state so the
//! frontend can subscribe once and react without polling. Every event is
//! `Clone + Send + Sync + 'static + serde::Serialize` so it satisfies
//! [`crate::BusEvent`] automatically.

use serde::{Deserialize, Serialize};

/// Network link state changed (cable plugged / unplugged / carrier up).
///
/// Emitted by `zettings-network` from a `NetworkManager` `StateChange` signal.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct LinkState {
    /// `NetworkManager` interface index (`org.freedesktop.NetworkManager.Device::Interface`).
    pub interface_index: u32,
    /// `true` when the link has carrier, `false` when down.
    pub link_up: bool,
}

/// Per-stream audio volume changed.
///
/// Emitted by `zettings-audio` from a `PipeWire`/`PulseAudio` volume sink event.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct AudioVolume {
    /// Application or sink id matching the `zettings-audio` registry.
    pub stream_id: u32,
    /// Normalized volume in `[0.0, 1.0]`. `1.0` is unity (0 dB) gain.
    pub volume: f32,
    /// `true` when the stream is muted.
    pub muted: bool,
}

/// A display was connected or disconnected.
///
/// Emitted by `zettings-display` from a `KScreen` output change signal.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct DisplayReplug {
    /// `KScreen` output name, e.g. `"HDMI-A-1"`.
    pub output_id: u32,
    /// `true` when the display is now connected, `false` when disconnected.
    pub connected: bool,
}

/// Battery / power supply state changed.
///
/// Emitted by `zettings-power` from a `UPower` `PropertiesChanged` signal.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct PowerState {
    /// `UPower` device object-path index (stable across runs).
    pub device_index: u32,
    /// Charge percentage in `[0.0, 100.0]`.
    pub percentage: f32,
    /// `true` when the device is charging, `false` when discharging or full.
    pub charging: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Bus;

    #[tokio::test]
    async fn link_state_roundtrip() {
        let bus = Bus::new();
        let mut rx = bus.subscribe::<LinkState>();
        let _ = bus.publish(LinkState {
            interface_index: 2,
            link_up: true,
        });
        let evt = rx.recv().await.expect("event");
        assert!(evt.link_up);
    }

    #[tokio::test]
    async fn audio_volume_roundtrip() {
        let bus = Bus::new();
        let mut rx = bus.subscribe::<AudioVolume>();
        let _ = bus.publish(AudioVolume {
            stream_id: 7,
            volume: 0.5,
            muted: false,
        });
        let evt = rx.recv().await.expect("event");
        assert_eq!(evt.stream_id, 7);
        assert!((evt.volume - 0.5).abs() < f32::EPSILON);
    }

    #[tokio::test]
    async fn display_replug_roundtrip() {
        let bus = Bus::new();
        let mut rx = bus.subscribe::<DisplayReplug>();
        let _ = bus.publish(DisplayReplug {
            output_id: 1,
            connected: true,
        });
        let evt = rx.recv().await.expect("event");
        assert!(evt.connected);
    }

    #[tokio::test]
    async fn power_state_roundtrip() {
        let bus = Bus::new();
        let mut rx = bus.subscribe::<PowerState>();
        let _ = bus.publish(PowerState {
            device_index: 0,
            percentage: 80.0,
            charging: true,
        });
        let evt = rx.recv().await.expect("event");
        assert!(evt.charging);
    }
}
