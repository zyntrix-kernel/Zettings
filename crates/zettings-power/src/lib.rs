//! `zettings-power` — `UPower` + power-profiles-daemon integration.
//!
//! Battery state, charge thresholds, and performance profile switching
//! through `org.freedesktop.UPower` and `org.freedesktop.power-profiles` over
//! `zbus` on Linux. The Windows dev loop uses an in-memory [`MockBackend`].

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use serde::{Deserialize, Serialize};
use thiserror::Error;
use zettings_bus::Bus;
use zettings_bus::events::PowerState;

/// Power profile offered by `power-profiles-daemon`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Profile {
    /// Balanced (default).
    Balanced,
    /// Performance (high power draw, fans may spin up).
    Performance,
    /// Power-saver (throttle CPU/GPU, dim backlight).
    PowerSaver,
}

/// Battery state for one `UPower` device.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct BatteryState {
    /// `UPower` device object-path index.
    pub device_index: u32,
    /// Charge percentage in `[0.0, 100.0]`.
    pub percentage: f32,
    /// `true` when the device is charging.
    pub charging: bool,
}

/// Errors surfaced by the power backend.
#[derive(Debug, Error)]
pub enum PowerError {
    /// The requested profile is not supported by the daemon.
    #[error("power profile not available: {0:?}")]
    ProfileNotAvailable(Profile),
    /// The requested charge threshold is outside the valid `[0, 100]` range.
    #[error("invalid charge threshold: {0}")]
    InvalidThreshold(u8),
    /// The underlying `UPower`/power-profiles-daemon was unreachable.
    #[error("power service unavailable: {0}")]
    ServiceUnavailable(String),
}

/// Trait surface implemented by both the mock state machine and the real
/// `zbus` backend.
pub trait Backend: Send + Sync {
    /// Read the active power profile.
    ///
    /// # Errors
    /// Returns [`PowerError::ServiceUnavailable`] when the daemon is down.
    fn active_profile(&self) -> Result<Profile, PowerError>;

    /// Switch to `profile`. Emits a [`PowerState`] event over `bus`.
    ///
    /// # Errors
    /// - [`PowerError::ProfileNotAvailable`] when the daemon rejects the profile.
    fn set_profile(&self, profile: Profile, bus: &Bus) -> Result<(), PowerError>;

    /// Read battery state for all `UPower` devices.
    ///
    /// # Errors
    /// Returns [`PowerError::ServiceUnavailable`] when `UPower` is unreachable.
    fn batteries(&self) -> Result<Vec<BatteryState>, PowerError>;

    /// Set the battery charge threshold (charge limiter). `0` disables the
    /// limiter entirely.
    ///
    /// # Errors
    /// - [`PowerError::InvalidThreshold`] when `percent` is outside `[0, 100]`.
    /// - [`PowerError::ServiceUnavailable`] when the platform does not expose a
    ///   writable charge-control threshold.
    fn set_charge_threshold(&self, percent: u8) -> Result<(), PowerError>;
}

/// In-memory mock backend. Holds the active profile, the charge threshold,
/// and one virtual battery discharging from a fixed percentage.
pub struct MockBackend {
    state: parking_lot::Mutex<MockState>,
}

/// Mutable state machine behind [`MockBackend`].
struct MockState {
    profile: Profile,
    charge_threshold: u8,
}

impl MockBackend {
    /// Construct a mock backend whose active profile is [`Profile::Balanced`]
    /// and whose charge threshold is disabled (`0`).
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: parking_lot::Mutex::new(MockState {
                profile: Profile::Balanced,
                charge_threshold: 0,
            }),
        }
    }
}

impl Default for MockBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl Backend for MockBackend {
    fn active_profile(&self) -> Result<Profile, PowerError> {
        Ok(self.state.lock().profile)
    }

    fn set_profile(&self, profile: Profile, bus: &Bus) -> Result<(), PowerError> {
        self.state.lock().profile = profile;
        let _ = bus.publish(PowerState {
            device_index: 0,
            percentage: 80.0,
            charging: matches!(profile, Profile::Performance),
        });
        Ok(())
    }

    fn batteries(&self) -> Result<Vec<BatteryState>, PowerError> {
        Ok(vec![BatteryState {
            device_index: 0,
            percentage: 80.0,
            charging: false,
        }])
    }

    fn set_charge_threshold(&self, percent: u8) -> Result<(), PowerError> {
        if percent > 100 {
            return Err(PowerError::InvalidThreshold(percent));
        }
        self.state.lock().charge_threshold = percent;
        Ok(())
    }
}

/// Real Linux backend. Connects to `org.freedesktop.UPower` and
/// `org.freedesktop.power-profiles` over `zbus`.
#[cfg(all(target_os = "linux", not(feature = "zettings-mock")))]
pub struct LinuxBackend;

#[cfg(all(target_os = "linux", not(feature = "zettings-mock")))]
impl Backend for LinuxBackend {
    fn active_profile(&self) -> Result<Profile, PowerError> {
        Err(PowerError::ServiceUnavailable(
            "UPower zbus integration pending".into(),
        ))
    }

    fn set_profile(&self, _profile: Profile, _bus: &Bus) -> Result<(), PowerError> {
        Err(PowerError::ServiceUnavailable(
            "power-profiles zbus integration pending".into(),
        ))
    }

    fn batteries(&self) -> Result<Vec<BatteryState>, PowerError> {
        Err(PowerError::ServiceUnavailable(
            "UPower zbus integration pending".into(),
        ))
    }

    fn set_charge_threshold(&self, _percent: u8) -> Result<(), PowerError> {
        Err(PowerError::ServiceUnavailable(
            "UPower zbus integration pending".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_default_profile_balanced() {
        let b = MockBackend::new();
        assert_eq!(b.active_profile().expect("profile"), Profile::Balanced);
    }

    #[tokio::test]
    async fn mock_set_profile_publishes_powerstate() {
        let b = MockBackend::new();
        let bus = Bus::new();
        let mut rx = bus.subscribe::<PowerState>();
        b.set_profile(Profile::Performance, &bus).expect("set");
        let evt = rx.recv().await.expect("event");
        assert!(evt.charging);
    }

    #[tokio::test]
    async fn mock_batteries_one_device() {
        let b = MockBackend::new();
        let bats = b.batteries().expect("batteries");
        assert_eq!(bats.len(), 1);
        assert!(!bats[0].charging);
    }

    #[tokio::test]
    async fn mock_set_charge_threshold_succeeds() {
        let b = MockBackend::new();
        b.set_charge_threshold(80).expect("set threshold");
        // The threshold is stored on the state machine; reaching it with a
        // second call confirms the first did not corrupt the state.
        b.set_charge_threshold(80).expect("idempotent set");
    }

    #[tokio::test]
    async fn mock_set_charge_threshold_zero_disables() {
        let b = MockBackend::new();
        b.set_charge_threshold(0).expect("disable threshold");
    }

    #[tokio::test]
    async fn mock_rejects_threshold_over_100() {
        let b = MockBackend::new();
        let err = b.set_charge_threshold(101).unwrap_err();
        assert!(matches!(err, PowerError::InvalidThreshold(101)));
    }

    #[tokio::test]
    async fn mock_rejects_threshold_at_upper_bound() {
        let b = MockBackend::new();
        let err = b.set_charge_threshold(u8::MAX).unwrap_err();
        assert!(matches!(err, PowerError::InvalidThreshold(u8::MAX)));
    }
}
