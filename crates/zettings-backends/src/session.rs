//! Session power actions via `org.freedesktop.login1` (systemd-logind).

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use zettings_core::CapabilityState;

use crate::error::BackendError;

#[cfg(target_os = "linux")]
const SERVICE: &str = "systemd-logind";
#[cfg(target_os = "linux")]
const BUS_NAME: &str = "org.freedesktop.login1";
#[cfg(target_os = "linux")]
const OBJECT_PATH: &str = "/org/freedesktop/login1";
#[cfg(target_os = "linux")]
const IFACE: &str = "org.freedesktop.login1.Manager";

/// Read-only session power-action availability (logind's `yes/no/challenge`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionCapabilities {
    /// Honest capability of the adapter.
    pub capability: CapabilityState,
    /// Power off permitted now? (`yes`/`no`/`challenge`)
    pub can_power_off: String,
    /// Reboot permitted now?
    pub can_reboot: String,
    /// Suspend permitted now?
    pub can_suspend: String,
}

/// Session power-action surface (read-only in this phase; triggering actions
/// lands with the System page and polkit wiring).
#[async_trait]
pub trait SessionAdapter: Send + Sync {
    /// Reads logind capability strings.
    ///
    /// # Errors
    /// [`BackendError::Service`] when logind cannot be consulted.
    async fn capabilities(&self) -> Result<SessionCapabilities, BackendError>;
}

/// Deterministic mock.
#[derive(Debug, Default)]
pub struct MockSession;

#[async_trait]
impl SessionAdapter for MockSession {
    async fn capabilities(&self) -> Result<SessionCapabilities, BackendError> {
        Ok(SessionCapabilities {
            capability: CapabilityState::Available,
            can_power_off: "yes".to_owned(),
            can_reboot: "yes".to_owned(),
            can_suspend: "yes".to_owned(),
        })
    }
}

/// Real adapter over the system bus (Linux only).
#[cfg(target_os = "linux")]
pub struct LinuxSession {
    conn: zbus::Connection,
}

#[cfg(target_os = "linux")]
impl LinuxSession {
    /// Binds to an existing system-bus connection.
    pub const fn new(conn: zbus::Connection) -> Self {
        Self { conn }
    }

    async fn proxy(&self) -> zbus::Result<zbus::Proxy<'_>> {
        zbus::Proxy::new(&self.conn, BUS_NAME, OBJECT_PATH, IFACE).await
    }

    async fn string_property(&self, name: &str) -> Result<String, BackendError> {
        let proxy = self
            .proxy()
            .await
            .map_err(|e| BackendError::service(SERVICE, e))?;
        proxy
            .get_property(name)
            .await
            .map_err(|e| BackendError::service(SERVICE, e))
    }
}

#[cfg(target_os = "linux")]
#[async_trait]
impl SessionAdapter for LinuxSession {
    async fn capabilities(&self) -> Result<SessionCapabilities, BackendError> {
        let can_power_off = self.string_property("CanPowerOff").await?;
        let can_reboot = self.string_property("CanReboot").await?;
        let can_suspend = self.string_property("CanSuspend").await?;
        Ok(SessionCapabilities {
            capability: crate::service_available(true, SERVICE),
            can_power_off,
            can_reboot,
            can_suspend,
        })
    }
}
