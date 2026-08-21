//! `PolicyKit` authorization gateway.
//!
//! Every privileged mutation in ZETTINGS flows through this gateway
//! (threat-model.md T2). The trait is the security seam: adapters must never
//! bypass it, and the real zbus-backed implementation (Phase 5) plus the
//! deterministic mock used on Windows hosts both satisfy the same contract.

use async_trait::async_trait;
use thiserror::Error;

/// Outcome of an authorization check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Decision {
    /// The subject is authorized; proceed without prompting.
    Authorized,
    /// The user granted authorization after interactive authentication.
    AuthorizedAfterPrompt,
    /// The user denied or dismissed the authentication dialog.
    Denied,
}

/// A registered `PolicyKit` action identifier, e.g.
/// `org.zyntrix.zettings.modify-network`.
///
/// Construct via [`PolkitAction::parse`]; free-form strings are rejected so a
/// typo cannot silently authorize the wrong action namespace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolkitAction(String);

impl PolkitAction {
    /// Validates and constructs an action id under the ZETTINGS namespace
    /// (`org.zyntrix.zettings.<area>[.<verb>]`, lowercase segments).
    ///
    /// # Errors
    /// Returns [`PolkitError::InvalidAction`] when the grammar is violated.
    pub fn parse(raw: &str) -> Result<Self, PolkitError> {
        let Some(rest) = raw.strip_prefix("org.zyntrix.zettings.") else {
            return Err(PolkitError::InvalidAction(raw.to_owned()));
        };
        let segments_ok = rest.split('.').all(|seg| {
            !seg.is_empty()
                && seg
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        });
        if segments_ok && !rest.is_empty() {
            Ok(Self(raw.to_owned()))
        } else {
            Err(PolkitError::InvalidAction(raw.to_owned()))
        }
    }

    /// The action identifier string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Errors produced by the authorization layer.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PolkitError {
    /// The action identifier failed validation.
    #[error("invalid polkit action {0:?}")]
    InvalidAction(String),
    /// The authority could not be reached (service absent/bus error).
    #[error("polkit authority unavailable")]
    AuthorityUnavailable,
}

/// Gateway for privileged operations.
#[async_trait]
pub trait AuthorizationGateway: Send + Sync {
    /// Checks whether the current subject may perform `action`, triggering
    /// the `PolicyKit` authentication dialog when required.
    ///
    /// Implementations MUST NOT cache decisions beyond `PolicyKit`'s own
    /// session semantics and MUST fail closed ([`Decision::Denied`] or
    /// [`PolkitError::AuthorityUnavailable`], never implicit allow).
    ///
    /// # Errors
    /// [`PolkitError::AuthorityUnavailable`] when the authority cannot be
    /// consulted — callers must treat the operation as not permitted.
    async fn authorize(&self, action: &PolkitAction) -> Result<Decision, PolkitError>;
}

/// Deterministic gateway for development and tests (Windows host builds).
///
/// Behavior is scriptable per action; unlisted actions **fail closed**.
#[derive(Debug, Clone, Default)]
pub struct MockGateway {
    scripted: Vec<(PolkitAction, Decision)>,
}

impl MockGateway {
    /// Creates an empty mock; all checks deny until scripted otherwise.
    pub fn new() -> Self {
        Self::default()
    }

    /// Scripts `decision` for `action` (ignored when `action` is invalid).
    pub fn allow(&mut self, action: &str, decision: Decision) -> &mut Self {
        if let Ok(parsed) = PolkitAction::parse(action) {
            self.scripted.push((parsed, decision));
        }
        self
    }

    /// Resolves the scripted decision for `action` (latest wins), else deny.
    fn resolve(&self, action: &PolkitAction) -> Decision {
        self.scripted
            .iter()
            .rev()
            .find(|(a, _)| a == action)
            .map_or(Decision::Denied, |(_, d)| *d)
    }
}

#[async_trait]
impl AuthorizationGateway for MockGateway {
    async fn authorize(&self, action: &PolkitAction) -> Result<Decision, PolkitError> {
        Ok(self.resolve(action))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_fails_closed_for_unknown_actions() {
        let gw = MockGateway::new();
        let action = PolkitAction::parse("org.zyntrix.zettings.modify-network").expect("valid");
        assert_eq!(gw.authorize(&action).await, Ok(Decision::Denied));
    }

    #[tokio::test]
    async fn scripted_decisions_apply_latest_first() {
        let mut gw = MockGateway::new();
        let id = "org.zyntrix.zettings.modify-network";
        gw.allow(id, Decision::Authorized);
        gw.allow(id, Decision::AuthorizedAfterPrompt);
        let action = PolkitAction::parse(id).expect("valid");
        assert_eq!(
            gw.authorize(&action).await,
            Ok(Decision::AuthorizedAfterPrompt)
        );
    }

    #[test]
    fn rejects_foreign_namespaces_and_malformed_ids() {
        assert!(PolkitAction::parse("org.freedesktop.network1").is_err());
        assert!(PolkitAction::parse("org.zyntrix.zettings.Bad-Segment").is_err());
        assert!(PolkitAction::parse("org.zyntrix.zettings.").is_err());
    }
}
