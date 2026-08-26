use async_trait::async_trait;
use std::collections::BTreeMap;
use std::sync::Mutex;
use zettings_core::{
    BackendAdapter, BackendId, Health, OperationKey, SettingValue, ValueKey, ZettingsError,
};

pub struct MockAdapter {
    backend: BackendId,
    values: BTreeMap<String, SettingValue>,
    written: Mutex<Vec<(String, SettingValue)>>,
    up: bool,
}

impl MockAdapter {
    #[must_use]
    pub fn new(backend: BackendId) -> Self {
        Self {
            backend,
            values: BTreeMap::new(),
            written: Mutex::new(Vec::new()),
            up: true,
        }
    }

    #[must_use]
    pub fn with_value(mut self, key: &str, value: SettingValue) -> Self {
        self.values.insert(String::from(key), value);
        self
    }

    #[must_use]
    pub fn down(mut self) -> Self {
        self.up = false;
        self
    }

    #[must_use]
    pub fn written(&self) -> Vec<(String, SettingValue)> {
        self.written.lock().map(|g| g.clone()).unwrap_or_default()
    }
}

#[async_trait]
impl BackendAdapter for MockAdapter {
    fn id(&self) -> BackendId {
        self.backend.clone()
    }

    async fn health(&self) -> Health {
        if self.up { Health::Up } else { Health::Down }
    }

    async fn read(&self, key: &ValueKey) -> Result<SettingValue, ZettingsError> {
        if !self.up {
            return Err(ZettingsError::BackendUnreachable {
                backend: self.backend.clone(),
                detail: String::from("mock marked down"),
            });
        }
        match self.values.get(key.as_str()) {
            Some(v) => Ok(v.clone()),
            None => Err(ZettingsError::Validation {
                setting: zettings_core::SettingId(format!("{}.{}", self.backend.0, key.as_str())),
                violation: zettings_core::ConstraintViolation::Malformed {
                    expected: "known mock key",
                },
            }),
        }
    }

    async fn write(
        &self,
        operation: &OperationKey,
        value: &SettingValue,
    ) -> Result<(), ZettingsError> {
        if !self.up {
            return Err(ZettingsError::BackendUnreachable {
                backend: self.backend.clone(),
                detail: String::from("mock marked down"),
            });
        }
        if let Ok(mut guard) = self.written.lock() {
            guard.push((String::from(operation.as_str()), value.clone()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn routes_reads_writes_and_reports_health() {
        let adapter =
            MockAdapter::new(BackendId::timedate()).with_value("ntp", SettingValue::Bool(true));

        assert_eq!(adapter.health().await, Health::Up);
        assert_eq!(
            adapter.read(&ValueKey(String::from("ntp"))).await.unwrap(),
            SettingValue::Bool(true)
        );

        adapter
            .write(
                &OperationKey(String::from("set-timezone")),
                &SettingValue::Text(String::from("Europe/Oslo")),
            )
            .await
            .unwrap();
        assert_eq!(adapter.written()[0].0, "set-timezone");

        assert!(adapter.read(&ValueKey(String::from("nope"))).await.is_err());
    }

    #[tokio::test]
    async fn down_adapter_fails_closed() {
        let adapter = MockAdapter::new(BackendId::audio()).down();
        assert_eq!(adapter.health().await, Health::Down);
        assert!(matches!(
            adapter.read(&ValueKey(String::from("x"))).await,
            Err(ZettingsError::BackendUnreachable { .. })
        ));
    }
}
