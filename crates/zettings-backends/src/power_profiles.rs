use async_trait::async_trait;
use zbus::Proxy;
use zettings_core::{
    BackendAdapter, BackendId, Health, OperationKey, SettingValue, ValueKey, ZettingsError,
};

use crate::error::to_zettings_error;

pub const DESTINATION: &str = "org.freedesktop.UPower.PowerProfiles";
pub const PATH: &str = "/net/hadess/PowerProfiles";
pub const INTERFACE: &str = "net.hadess.PowerProfiles";

pub struct PowerProfilesAdapter {
    proxy: Proxy<'static>,
}

impl PowerProfilesAdapter {
    /// # Errors
    ///
    /// Returns [`ZettingsError::BackendUnreachable`] when power-profiles-daemon
    /// is unavailable at construction time.
    pub async fn connect(connection: zbus::Connection) -> Result<Self, ZettingsError> {
        let proxy = Proxy::new(&connection, DESTINATION, PATH, INTERFACE)
            .await
            .map_err(|e| to_zettings_error(&BackendId::power(), &e))?;
        Ok(Self { proxy })
    }
}

#[async_trait]
impl BackendAdapter for PowerProfilesAdapter {
    fn id(&self) -> BackendId {
        BackendId::power()
    }

    async fn health(&self) -> Health {
        match self.proxy.get_property::<String>("ActiveProfile").await {
            Ok(_) => Health::Up,
            Err(_) => Health::Down,
        }
    }

    async fn read(&self, key: &ValueKey) -> Result<SettingValue, ZettingsError> {
        let backend = self.id();
        match key.as_str() {
            "active-profile" => {
                let v = self
                    .proxy
                    .get_property::<String>("ActiveProfile")
                    .await
                    .map_err(|e| to_zettings_error(&backend, &e))?;
                Ok(SettingValue::Text(v))
            }
            other => Err(ZettingsError::Validation {
                setting: zettings_core::SettingId(format!("power.{other}")),
                violation: zettings_core::ConstraintViolation::Malformed {
                    expected: "power read key",
                },
            }),
        }
    }

    async fn write(
        &self,
        operation: &OperationKey,
        value: &SettingValue,
    ) -> Result<(), ZettingsError> {
        let backend = self.id();
        match (operation.as_str(), value) {
            ("set-active-profile", SettingValue::Text(profile)) => self
                .proxy
                .set_property("ActiveProfile", &profile.as_str())
                .await
                .map_err(|e| to_zettings_error(&backend, &zbus::Error::from(e))),
            _ => Err(ZettingsError::NotSupported {
                reason: zettings_core::UnsupportedReason::FeatureNotCompiled,
            }),
        }
    }
}
