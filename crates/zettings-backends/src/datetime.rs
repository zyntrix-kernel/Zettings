//! Date & time control via systemd `org.freedesktop.timedate1`.
//!
//! Read-only facts (timezone, NTP state/sync) plus the two mutations the
//! daemon itself exposes: enabling network time sync and changing the
//! system timezone. Privileged mutations authenticate through timedated's
//! own `PolicyKit` policy (`org.freedesktop.timedate1.*`), rendered by the
//! desktop agent — same trust path as every other system daemon adapter;
//! this crate never prompts or spawns shells itself.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use zettings_core::CapabilityState;

use crate::error::BackendError;

#[cfg(target_os = "linux")]
const SERVICE: &str = "systemd-timedated";
#[cfg(target_os = "linux")]
const BUS_NAME: &str = "org.freedesktop.timedate1";
#[cfg(target_os = "linux")]
const OBJECT_PATH: &str = "/org/freedesktop/timedate1";

/// A snapshot of the system clock configuration for IPC.
// Wire mirror of timedated's boolean properties; grouping them into enums
// would obscure the 1:1 D-Bus mapping this contract guarantees.
#[expect(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimedateSnapshot {
    /// Honest capability of the adapter.
    pub capability: CapabilityState,
    /// Active IANA timezone identifier (e.g. `Europe/Berlin`).
    pub timezone: String,
    /// Whether NTP-based automatic sync is enabled.
    pub ntp_enabled: bool,
    /// Whether the system can perform NTP sync at all.
    pub ntp_available: bool,
    /// Whether the clock currently reports successful synchronization.
    pub ntp_synchronized: bool,
    /// Whether the hardware RTC keeps local instead of UTC time.
    pub local_rtc: bool,
    /// Timezone identifiers selectable on this system.
    pub available_timezones: Vec<String>,
}

/// Control surface for system date & time.
#[async_trait]
pub trait TimedateAdapter: Send + Sync {
    /// Detects whether the timedated service is reachable.
    async fn capability(&self) -> CapabilityState;

    /// Returns the current clock configuration plus selectable timezones.
    ///
    /// # Errors
    /// [`BackendError::Service`] when the daemon cannot be consulted.
    async fn snapshot(&self) -> Result<TimedateSnapshot, BackendError>;

    /// Enables or disables NTP synchronization.
    ///
    /// # Errors
    /// [`BackendError::Service`] when timedated rejects the change.
    async fn set_ntp(&self, enabled: bool) -> Result<(), BackendError>;

    /// Applies an IANA timezone identifier.
    ///
    /// # Errors
    /// [`BackendError::InvalidValue`] when the identifier is unknown to this
    /// system, [`BackendError::Service`] when timedated rejects it.
    async fn set_timezone(&self, timezone: &str) -> Result<(), BackendError>;
}

/// Deterministic mock with set→get round-trip semantics.
#[derive(Debug, Default)]
pub struct MockTimedate {
    state: tokio::sync::Mutex<MockTimedateState>,
}

#[derive(Debug, Default)]
struct MockTimedateState {
    timezone: Option<String>,
    ntp_enabled: Option<bool>,
}

const MOCK_TIMEZONES: [&str; 8] = [
    "Etc/UTC",
    "Europe/Berlin",
    "Europe/London",
    "America/New_York",
    "America/Los_Angeles",
    "Asia/Kolkata",
    "Asia/Tokyo",
    "Australia/Sydney",
];

impl MockTimedate {
    fn zones() -> Vec<String> {
        MOCK_TIMEZONES.iter().map(|s| (*s).to_owned()).collect()
    }
}

#[async_trait]
impl TimedateAdapter for MockTimedate {
    async fn capability(&self) -> CapabilityState {
        CapabilityState::Available
    }

    async fn snapshot(&self) -> Result<TimedateSnapshot, BackendError> {
        let state = self.state.lock().await;
        Ok(TimedateSnapshot {
            capability: CapabilityState::Available,
            timezone: state
                .timezone
                .clone()
                .unwrap_or_else(|| "Etc/UTC".to_owned()),
            ntp_enabled: state.ntp_enabled.unwrap_or(true),
            ntp_available: true,
            ntp_synchronized: state.ntp_enabled.unwrap_or(true),
            local_rtc: false,
            available_timezones: Self::zones(),
        })
    }

    async fn set_ntp(&self, enabled: bool) -> Result<(), BackendError> {
        self.state.lock().await.ntp_enabled = Some(enabled);
        Ok(())
    }

    async fn set_timezone(&self, timezone: &str) -> Result<(), BackendError> {
        if !MOCK_TIMEZONES.contains(&timezone) {
            return Err(BackendError::InvalidValue {
                value: timezone.to_owned(),
                reason: "unknown timezone identifier".to_owned(),
            });
        }
        self.state.lock().await.timezone = Some(timezone.to_owned());
        Ok(())
    }
}

/// Real adapter over the system bus (Linux only).
#[cfg(target_os = "linux")]
pub struct LinuxTimedate {
    conn: zbus::Connection,
}

#[cfg(target_os = "linux")]
impl LinuxTimedate {
    /// Binds to an existing system-bus connection.
    pub const fn new(conn: zbus::Connection) -> Self {
        Self { conn }
    }

    async fn proxy(&self) -> zbus::Result<zbus::Proxy<'_>> {
        zbus::Proxy::new(
            &self.conn,
            BUS_NAME,
            OBJECT_PATH,
            "org.freedesktop.timedate1",
        )
        .await
    }
}

/// Lists IANA timezone identifiers installed under `/usr/share/zoneinfo`,
/// keeping only `Region/Location` regular files (two segments) so POSIX/
/// right-timezone copies and metadata never leak into user-facing options.
#[cfg(target_os = "linux")]
fn installed_timezones() -> Vec<String> {
    const ZONEINFO: &str = "/usr/share/zoneinfo";
    const EXCLUDED_REGIONS: [&str; 3] = ["posix", "right", "Etc"];

    let Ok(regions) = std::fs::read_dir(ZONEINFO) else {
        return Vec::new();
    };
    let mut zones = Vec::new();
    for region in regions.flatten() {
        let region_name = region.file_name().to_string_lossy().into_owned();
        if region.file_type().is_ok_and(|t| !t.is_dir())
            || EXCLUDED_REGIONS.contains(&region_name.as_str())
        {
            continue;
        }
        let Ok(entries) = std::fs::read_dir(region.path()) else {
            continue;
        };
        for entry in entries.flatten() {
            let location = entry.file_name().to_string_lossy().into_owned();
            if entry.file_type().is_ok_and(|t| t.is_file())
                && !location.contains('.')
                && !location.contains('+')
            {
                zones.push(format!("{region_name}/{location}"));
            }
        }
    }
    zones.sort();
    zones
}

#[cfg(target_os = "linux")]
#[async_trait]
impl TimedateAdapter for LinuxTimedate {
    async fn capability(&self) -> CapabilityState {
        let probe = async {
            let proxy = self.proxy().await?;
            let _: String = proxy.get_property("Timezone").await?;
            Ok::<(), zbus::Error>(())
        };
        match probe.await {
            Ok(()) => crate::service_available(true, SERVICE),
            Err(e) => CapabilityState::Unavailable {
                reason: format!("{SERVICE} probe failed: {e}"),
            },
        }
    }

    async fn snapshot(&self) -> Result<TimedateSnapshot, BackendError> {
        let proxy = self
            .proxy()
            .await
            .map_err(|e| BackendError::service(SERVICE, e))?;
        let timezone: String = proxy
            .get_property("Timezone")
            .await
            .map_err(|e| BackendError::service(SERVICE, e))?;
        let ntp_enabled: bool = proxy
            .get_property("NTP")
            .await
            .map_err(|e| BackendError::service(SERVICE, e))?;
        let ntp_available: bool = proxy
            .get_property("CanNTP")
            .await
            .map_err(|e| BackendError::service(SERVICE, e))?;
        let ntp_synchronized: bool = proxy
            .get_property("NTPSynchronized")
            .await
            .map_err(|e| BackendError::service(SERVICE, e))?;
        let local_rtc: bool = proxy
            .get_property("LocalRTC")
            .await
            .map_err(|e| BackendError::service(SERVICE, e))?;
        Ok(TimedateSnapshot {
            capability: crate::service_available(true, SERVICE),
            timezone,
            ntp_enabled,
            ntp_available,
            ntp_synchronized,
            local_rtc,
            available_timezones: installed_timezones(),
        })
    }

    async fn set_ntp(&self, enabled: bool) -> Result<(), BackendError> {
        let proxy = self
            .proxy()
            .await
            .map_err(|e| BackendError::service(SERVICE, e))?;
        let _: () = proxy
            .call("SetNTP", &(enabled, false))
            .await
            .map_err(|e| BackendError::service(SERVICE, e))?;
        Ok(())
    }

    async fn set_timezone(&self, timezone: &str) -> Result<(), BackendError> {
        if !installed_timezones().iter().any(|z| z == timezone) {
            return Err(BackendError::InvalidValue {
                value: timezone.to_owned(),
                reason: "not present under /usr/share/zoneinfo".to_owned(),
            });
        }
        let proxy = self
            .proxy()
            .await
            .map_err(|e| BackendError::service(SERVICE, e))?;
        let _: () = proxy
            .call("SetTimezone", &(timezone, false))
            .await
            .map_err(|e| BackendError::service(SERVICE, e))?;
        Ok(())
    }
}
