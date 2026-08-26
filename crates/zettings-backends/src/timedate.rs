use async_trait::async_trait;
use zbus::Proxy;
use zettings_core::{
    BackendAdapter, BackendId, Health, OperationKey, SettingValue, ValueKey, ZettingsError,
};

use crate::error::to_zettings_error;

pub const DESTINATION: &str = "org.freedesktop.timedate1";
pub const PATH: &str = "/org/freedesktop/timedate1";
pub const INTERFACE: &str = "org.freedesktop.timedate1";

pub struct TimedateAdapter {
    proxy: Proxy<'static>,
}

impl TimedateAdapter {
    /// # Errors
    ///
    /// Returns [`ZettingsError::BackendUnreachable`] when the system bus or
    /// the timedate1 service is unavailable at construction time.
    pub async fn connect(connection: zbus::Connection) -> Result<Self, ZettingsError> {
        let proxy = Proxy::new(&connection, DESTINATION, PATH, INTERFACE)
            .await
            .map_err(|e| to_zettings_error(&BackendId::timedate(), &e))?;
        Ok(Self { proxy })
    }
}

#[async_trait]
impl BackendAdapter for TimedateAdapter {
    fn id(&self) -> BackendId {
        BackendId::timedate()
    }

    async fn health(&self) -> Health {
        match self.proxy.get_property::<bool>("CanNTP").await {
            Ok(_) => Health::Up,
            Err(_) => Health::Down,
        }
    }

    async fn read(&self, key: &ValueKey) -> Result<SettingValue, ZettingsError> {
        let backend = self.id();
        match key.as_str() {
            "ntp" => {
                let v = self
                    .proxy
                    .get_property::<bool>("NTP")
                    .await
                    .map_err(|e| to_zettings_error(&backend, &e))?;
                Ok(SettingValue::Bool(v))
            }
            "timezone" => {
                let v = self
                    .proxy
                    .get_property::<String>("Timezone")
                    .await
                    .map_err(|e| to_zettings_error(&backend, &e))?;
                Ok(SettingValue::Text(v))
            }
            "local-rtc" => {
                let v = self
                    .proxy
                    .get_property::<bool>("LocalRTC")
                    .await
                    .map_err(|e| to_zettings_error(&backend, &e))?;
                Ok(SettingValue::Bool(v))
            }
            other => Err(ZettingsError::Validation {
                setting: zettings_core::SettingId(format!("timedate.{other}")),
                violation: unknown_key_violation(),
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
            ("set-ntp", SettingValue::Bool(enable)) => {
                call_unit(&self.proxy, "SetNTP", (*enable, false), &backend).await
            }
            ("set-timezone", SettingValue::Text(tz)) => {
                call_unit(&self.proxy, "SetTimezone", (tz.as_str(), false), &backend).await
            }
            ("set-local-rtc", SettingValue::Bool(local)) => {
                call_unit(&self.proxy, "SetLocalRTC", (*local, false, false), &backend).await
            }
            _ => Err(ZettingsError::NotSupported {
                reason: zettings_core::UnsupportedReason::FeatureNotCompiled,
            }),
        }
    }
}

async fn call_unit<A>(
    proxy: &Proxy<'_>,
    method: &str,
    args: A,
    backend: &BackendId,
) -> Result<(), ZettingsError>
where
    A: serde::ser::Serialize + zbus::zvariant::Type + zbus::zvariant::DynamicType,
{
    proxy
        .call_method(method, &(args,))
        .await
        .map_err(|e| to_zettings_error(backend, &e))?;
    Ok(())
}

fn unknown_key_violation() -> zettings_core::ConstraintViolation {
    zettings_core::ConstraintViolation::Malformed {
        expected: "timedate read key",
    }
}
