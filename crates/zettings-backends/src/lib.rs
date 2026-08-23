//! Linux/KDE system adapters for ZETTINGS.
//!
//! Layering (PLAN §13): adapters expose typed async APIs and honest
//! [`CapabilityState`] reporting. Real implementations use zbus on Linux;
//! deterministic state-machine mocks exist everywhere so the Windows host
//! runs the full shell without Linux services, and so tests are reproducible.
//!
//! Security contract (threat-model.md): privileged mutations flow through the
//! [`zettings_polkit::AuthorizationGateway`]; adapters never spawn shells.

mod audio;
mod bluetooth;
mod datetime;
mod display;
mod error;
mod network;
mod power;
mod session;

#[cfg(target_os = "linux")]
mod polkit_gateway;

pub use audio::{AudioAdapter, AudioSink, MockAudio};
pub use bluetooth::{BluetoothAdapter, BluetoothDevice, BluetoothStatus, MockBluetooth};
pub use datetime::{MockTimedate, TimedateAdapter, TimedateSnapshot};
pub use display::{DisplayAdapter, DisplayOutput, DisplayStatus, MockDisplay};
pub use error::BackendError;
pub use network::{MockNetwork, NetworkAdapter, NetworkDevice, NetworkDeviceKind, NetworkStatus};
pub use power::{MockPowerProfiles, PowerProfileSnapshot, PowerProfilesAdapter};
pub use session::{MockSession, SessionAdapter, SessionCapabilities};

/// Real `PolicyKit` gateway (Linux builds only).
#[cfg(target_os = "linux")]
pub use polkit_gateway::PolkitGateway;

use std::sync::Arc;
use zettings_core::CapabilityState;

/// The adapter set owned by the application shell.
///
/// Construction is asynchronous because real adapters open a D-Bus
/// connection; mocks construct instantly.
pub struct BackendSet {
    /// Power profile control (power-profiles-daemon).
    pub power: Arc<dyn PowerProfilesAdapter>,
    /// Network status and radios (`NetworkManager`).
    pub network: Arc<dyn NetworkAdapter>,
    /// Session power actions (systemd login1).
    pub session: Arc<dyn SessionAdapter>,
    /// Audio output control (PulseAudio/PipeWire).
    pub audio: Arc<dyn AudioAdapter>,
    /// Bluetooth radio and devices (`BlueZ`).
    pub bluetooth: Arc<dyn BluetoothAdapter>,
    /// Display topology (kernel DRM; read-only in this phase).
    pub display: Arc<dyn DisplayAdapter>,
    /// System date & time (systemd `timedated`).
    pub datetime: Arc<dyn TimedateAdapter>,
    /// Authorization seam for privileged mutations.
    pub auth: Arc<dyn zettings_polkit::AuthorizationGateway>,
}

impl BackendSet {
    /// Builds the platform-appropriate set: real adapters on Linux unless
    /// `force_mock`, otherwise deterministic mocks.
    ///
    /// # Errors
    /// [`BackendError::Bus`] when the Linux system bus cannot be reached —
    /// callers then fall back to [`BackendSet::mocks`] and surface the honest
    /// capability states to the UI.
    #[cfg_attr(not(target_os = "linux"), allow(clippy::unused_async))]
    pub async fn detect(force_mock: bool) -> Result<Self, BackendError> {
        if force_mock || !cfg!(target_os = "linux") {
            return Ok(Self::mocks());
        }
        #[cfg(target_os = "linux")]
        {
            let conn = zbus::Connection::system()
                .await
                .map_err(|e| BackendError::service("system bus", e))?;
            Ok(Self {
                power: Arc::new(power::LinuxPowerProfiles::new(conn.clone())),
                network: Arc::new(network::LinuxNetwork::new(conn.clone())),
                session: Arc::new(session::LinuxSession::new(conn.clone())),
                audio: Arc::new(audio::LinuxAudio::new()),
                bluetooth: Arc::new(bluetooth::LinuxBluetooth::new(conn.clone())),
                display: Arc::new(display::LinuxDisplay::default()),
                datetime: Arc::new(datetime::LinuxTimedate::new(conn.clone())),
                auth: Arc::new(polkit_gateway::PolkitGateway::new(conn)),
            })
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = force_mock; // unreachable; kept for signature symmetry
            Ok(Self::mocks())
        }
    }

    /// Deterministic mock set (Windows hosts, tests, forced UI development).
    pub fn mocks() -> Self {
        Self {
            power: Arc::new(power::MockPowerProfiles::default()),
            network: Arc::new(network::MockNetwork::default()),
            session: Arc::new(session::MockSession),
            audio: Arc::new(audio::MockAudio::default()),
            bluetooth: Arc::new(bluetooth::MockBluetooth::default()),
            display: Arc::new(display::MockDisplay::default()),
            datetime: Arc::new(datetime::MockTimedate::default()),
            auth: Arc::new(zettings_polkit::MockGateway::new()),
        }
    }
}

/// Convenience for capability probes shared by adapters: maps "service
/// answered" into an available/degraded decision with a stable reason string.
pub fn service_available(present: bool, service: &str) -> CapabilityState {
    if present {
        CapabilityState::Available
    } else {
        CapabilityState::Unavailable {
            reason: format!("{service} is not running on this system"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zettings_polkit::PolkitAction;

    #[tokio::test]
    async fn mock_power_round_trips_and_rejects_unknown_profiles() {
        let set = BackendSet::mocks();
        let before = set.power.snapshot().await.expect("snapshot");
        assert_eq!(before.active, "balanced");
        assert_eq!(before.available.len(), 3);

        set.power
            .set_active_profile("performance")
            .await
            .expect("set");
        let after = set.power.snapshot().await.expect("snapshot");
        assert_eq!(after.active, "performance");

        let invalid = set.power.set_active_profile("turbo").await;
        assert!(matches!(invalid, Err(BackendError::InvalidValue { .. })));
    }

    #[tokio::test]
    async fn mock_network_toggles_wireless() {
        let set = BackendSet::mocks();
        assert!(set.network.status().await.expect("status").wireless_enabled);
        set.network.set_wireless_enabled(false).await.expect("set");
        let status = set.network.status().await.expect("status");
        assert!(!status.wireless_enabled);
        assert_eq!(status.devices.len(), 2);
    }

    #[tokio::test]
    async fn mock_timedate_round_trips_and_rejects_unknown_zones() {
        let set = BackendSet::mocks();
        let before = set.datetime.snapshot().await.expect("snapshot");
        assert_eq!(before.timezone, "Etc/UTC");
        assert!(before.ntp_enabled);
        assert!(!before.available_timezones.is_empty());

        set.datetime.set_ntp(false).await.expect("set ntp");
        set.datetime
            .set_timezone("Asia/Kolkata")
            .await
            .expect("set zone");
        let after = set.datetime.snapshot().await.expect("snapshot");
        assert_eq!(after.timezone, "Asia/Kolkata");
        assert!(!after.ntp_enabled);
        assert!(!after.ntp_synchronized);

        let invalid = set.datetime.set_timezone("Mars/Olympus").await;
        assert!(matches!(invalid, Err(BackendError::InvalidValue { .. })));
    }

    #[tokio::test]
    async fn mock_auth_fails_closed() {
        let set = BackendSet::mocks();
        let action = PolkitAction::parse("org.zyntrix.zettings.test").expect("valid");
        assert_eq!(
            set.auth.authorize(&action).await,
            Ok(zettings_polkit::Decision::Denied)
        );
    }
}
