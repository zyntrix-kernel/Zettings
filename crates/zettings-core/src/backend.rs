use crate::value::SettingValue;

#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct BackendId(pub String);

impl BackendId {
    #[must_use]
    pub fn timedate() -> Self {
        Self(String::from("timedate"))
    }

    #[must_use]
    pub fn network() -> Self {
        Self(String::from("network"))
    }

    #[must_use]
    pub fn audio() -> Self {
        Self(String::from("audio"))
    }

    #[must_use]
    pub fn power() -> Self {
        Self(String::from("power"))
    }

    #[must_use]
    pub fn bluetooth() -> Self {
        Self(String::from("bluetooth"))
    }

    #[must_use]
    pub fn display() -> Self {
        Self(String::from("display"))
    }

    #[must_use]
    pub fn accounts() -> Self {
        Self(String::from("accounts"))
    }

    #[must_use]
    pub fn personalization() -> Self {
        Self(String::from("personalization"))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for BackendId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct ValueKey(pub String);

impl ValueKey {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct OperationKey(pub String);

impl OperationKey {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Health {
    Up,
    Degraded,
    Down,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "policy", rename_all = "kebab-case")]
pub enum PollPolicy {
    OnDemand,
    Interval(std::time::Duration),
    OnSignal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConfirmationPolicy {
    None,
    Preview,
    ExplicitConfirm,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ReadBinding {
    pub backend: BackendId,
    pub key: ValueKey,
    pub poll: PollPolicy,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WriteAction {
    pub backend: BackendId,
    pub operation: OperationKey,
    pub confirmation: ConfirmationPolicy,
}

#[async_trait::async_trait]
pub trait BackendAdapter: Send + Sync {
    fn id(&self) -> BackendId;

    async fn health(&self) -> Health;

    async fn read(&self, key: &ValueKey) -> Result<SettingValue, crate::ZettingsError>;

    async fn write(
        &self,
        operation: &OperationKey,
        value: &SettingValue,
    ) -> Result<(), crate::ZettingsError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poll_policy_serializes_interval() {
        let policy = PollPolicy::Interval(std::time::Duration::from_secs(5));
        let json = serde_json::to_string(&policy).unwrap();
        assert!(json.contains("\"interval\""));
        assert!(json.contains("\"secs\":5"));
    }

    #[test]
    fn confirmation_policy_round_trips() {
        for policy in [
            ConfirmationPolicy::None,
            ConfirmationPolicy::Preview,
            ConfirmationPolicy::ExplicitConfirm,
        ] {
            let json = serde_json::to_string(&policy).unwrap();
            let back: ConfirmationPolicy = serde_json::from_str(&json).unwrap();
            assert_eq!(back, policy);
        }
    }

    #[test]
    fn known_backend_ids_display() {
        assert_eq!(BackendId::timedate().to_string(), "timedate");
        assert_eq!(BackendId::audio().as_str(), "audio");
    }
}
