//! Power profile control via `net.hadess.PowerProfiles` (power-profiles-daemon).

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use zettings_core::CapabilityState;

use crate::error::BackendError;

#[cfg(target_os = "linux")]
const SERVICE: &str = "power-profiles-daemon";
#[cfg(target_os = "linux")]
const BUS_NAME: &str = "net.hadess.PowerProfiles";
#[cfg(target_os = "linux")]
const OBJECT_PATH: &str = "/net/hadess/PowerProfiles";

/// A snapshot of the power-profiles state for IPC.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PowerProfileSnapshot {
    /// Honest capability of the adapter.
    pub capability: CapabilityState,
    /// Profiles the daemon offers (e.g. power-saver/balanced/performance).
    pub available: Vec<String>,
    /// Currently active profile; empty when unavailable.
    pub active: String,
}

/// Control surface for system power profiles.
#[async_trait]
pub trait PowerProfilesAdapter: Send + Sync {
    /// Detects whether the daemon is present.
    async fn capability(&self) -> CapabilityState;
    /// Returns available + active profiles.
    ///
    /// # Errors
    /// [`BackendError::Service`] when the daemon cannot be consulted.
    async fn snapshot(&self) -> Result<PowerProfileSnapshot, BackendError>;
    /// Activates `profile`; rejects values outside the daemon's list.
    ///
    /// # Errors
    /// [`BackendError::InvalidValue`] for unknown profiles,
    /// [`BackendError::Service`] on failure.
    async fn set_active_profile(&self, profile: &str) -> Result<(), BackendError>;
}

/// Deterministic mock with set→get round-trip semantics.
#[derive(Debug, Default)]
pub struct MockPowerProfiles {
    state: tokio::sync::Mutex<MockState>,
}

#[derive(Debug, Default)]
struct MockState {
    active: Option<String>,
}

const MOCK_PROFILES: [&str; 3] = ["power-saver", "balanced", "performance"];

#[async_trait]
impl PowerProfilesAdapter for MockPowerProfiles {
    async fn capability(&self) -> CapabilityState {
        CapabilityState::Available
    }

    async fn snapshot(&self) -> Result<PowerProfileSnapshot, BackendError> {
        let state = self.state.lock().await;
        Ok(PowerProfileSnapshot {
            capability: CapabilityState::Available,
            available: MOCK_PROFILES.iter().map(|s| (*s).to_owned()).collect(),
            active: state
                .active
                .clone()
                .unwrap_or_else(|| "balanced".to_owned()),
        })
    }

    async fn set_active_profile(&self, profile: &str) -> Result<(), BackendError> {
        if !MOCK_PROFILES.contains(&profile) {
            return Err(BackendError::InvalidValue {
                value: profile.to_owned(),
                reason: "unknown power profile".to_owned(),
            });
        }
        self.state.lock().await.active = Some(profile.to_owned());
        Ok(())
    }
}

/// Real adapter over the system bus (Linux only).
#[cfg(target_os = "linux")]
pub struct LinuxPowerProfiles {
    conn: zbus::Connection,
}

#[cfg(target_os = "linux")]
impl LinuxPowerProfiles {
    /// Binds to an existing system-bus connection.
    pub const fn new(conn: zbus::Connection) -> Self {
        Self { conn }
    }

    fn proxy(&self) -> zbus::Result<zbus::Proxy<'_>> {
        zbus::Proxy::new(
            &self.conn,
            BUS_NAME,
            OBJECT_PATH,
            "net.hadess.PowerProfiles",
        )
    }
}

#[cfg(target_os = "linux")]
#[async_trait]
impl PowerProfilesAdapter for LinuxPowerProfiles {
    async fn capability(&self) -> CapabilityState {
        let probe = self
            .proxy()
            .and_then(|proxy| proxy.get_property::<String>("ActiveProfile").map(|_| ()));
        match probe.await {
            Ok(()) => crate::service_available(true, SERVICE),
            Err(e) => CapabilityState::Unavailable {
                reason: format!("{SERVICE} probe failed: {e}"),
            },
        }
    }

    async fn snapshot(&self) -> Result<PowerProfileSnapshot, BackendError> {
        let proxy = self
            .proxy()
            .map_err(|e| BackendError::service(SERVICE, e))?;
        let active: String = proxy
            .get_property("ActiveProfile")
            .await
            .map_err(|e| BackendError::service(SERVICE, e))?;
        let profiles_zv: Vec<zbus::zvariant::OwnedValue> = proxy
            .get_property("Profiles")
            .await
            .map_err(|e| BackendError::service(SERVICE, e))?;
        let available = profiles_zv
            .iter()
            .filter_map(|entry| {
                let map =
                    <std::collections::HashMap<String, zbus::zvariant::OwnedValue>>::try_from(
                        entry.clone(),
                    )
                    .ok()?;
                map.get("Profile").and_then(|v| v.try_clone().ok())
            })
            .filter_map(|v| <String>::try_from(v).ok())
            .collect();
        Ok(PowerProfileSnapshot {
            capability: crate::service_available(true, SERVICE),
            available,
            active,
        })
    }

    async fn set_active_profile(&self, profile: &str) -> Result<(), BackendError> {
        let snapshot = self.snapshot().await?;
        if !snapshot.available.iter().any(|p| p == profile) {
            return Err(BackendError::InvalidValue {
                value: profile.to_owned(),
                reason: format!("profiles offered by {SERVICE}: {:?}", snapshot.available),
            });
        }
        let proxy = self
            .proxy()
            .map_err(|e| BackendError::service(SERVICE, e))?;
        proxy
            .set_property("ActiveProfile", zbus::zvariant::Value::from(profile))
            .await
            .map_err(|e| BackendError::service(SERVICE, e))
    }
}
