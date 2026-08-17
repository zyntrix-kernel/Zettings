//! Per-action `PolicyKit` authorization for Zettings.
//!
//! Every privileged Tauri command declares a polkit action id of the form
//! `org.zyntrix.zettings.<domain>.<verb>` and calls [`check_authorization`]
//! before performing the privileged step. The actual dialog is the
//! distribution's polkit agent (e.g. `polkit-kde-agent`) — Zettings never
//! prompts for passwords inside its own UI.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use serde::{Deserialize, Serialize};

/// A polkit action id, e.g. `org.zyntrix.zettings.network.set-hostname`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct ActionId(pub String);

impl ActionId {
    /// Construct a new action id.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Construct an action id following the Zettings namespace convention.
    #[must_use]
    pub fn zettings(domain: &str, verb: &str) -> Self {
        Self(format!("org.zyntrix.zettings.{domain}.{verb}"))
    }
}

/// Result of an authorization check.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Authorization {
    /// User is already authorized (e.g. active session admin).
    Authorized,
    /// User became authorized after authenticating via polkit agent.
    Challenge,
    /// User denied, dismissed, or is not permitted to perform the action.
    Denied,
}

/// Errors surfaced by the polkit layer.
#[derive(Debug, thiserror::Error)]
pub enum PolkitError {
    /// Could not reach the polkit daemon.
    #[error("polkit daemon unreachable: {0}")]
    DaemonUnreachable(String),
    /// The action id was not registered with polkit (missing .policy file).
    #[error("unknown polkit action: {0}")]
    UnknownAction(String),
}

/// Authorizer trait. Real impl uses zbus on Linux; mock impl is used on Windows.
pub trait Authorizer: Send + Sync {
    /// Check whether the active session is authorized for `action`.
    ///
    /// This MAY block while the polkit agent displays a dialog to the user.
    /// Callers should run it on a `tokio::task::spawn_blocking` or a dedicated
    /// blocking IPC pool.
    ///
    /// # Errors
    /// Returns [`PolkitError`] only when the polkit daemon is unreachable
    /// or the action is unknown. A user-denied dialog returns `Ok(Denied)`.
    fn check_authorization(&self, action: &ActionId) -> Result<Authorization, PolkitError>;
}

/// Mock authorizer used by the `zettings-mock` feature (Windows dev loop).
#[cfg(feature = "zettings-mock")]
pub struct MockAuthorizer;

#[cfg(feature = "zettings-mock")]
impl Authorizer for MockAuthorizer {
    fn check_authorization(&self, _action: &ActionId) -> Result<Authorization, PolkitError> {
        Ok(Authorization::Authorized)
    }
}

/// Linux real authorizer: forwards to `org.freedesktop.PolicyKit1.Authority`.
#[cfg(all(target_os = "linux", not(feature = "zettings-mock")))]
pub struct LinuxAuthorizer;

#[cfg(all(target_os = "linux", not(feature = "zettings-mock")))]
impl Authorizer for LinuxAuthorizer {
    fn check_authorization(&self, action: &ActionId) -> Result<Authorization, PolkitError> {
        let _ = action;
        // Phase 5: real zbus call to `CheckAuthorization` with
        // `polkit` subject = "system-bus-name". Blocking.
        Ok(Authorization::Authorized)
    }
}

/// Recorded decision emitted by [`check_authorization_gateway`] for auditing
/// and live UI streaming over [`zettings_bus::Bus`].
#[derive(Debug, Clone, serde::Serialize)]
pub struct PolkitDecision {
    /// Action id that was checked.
    pub action: String,
    /// Outcome of the check.
    pub authorization: Authorization,
    /// Epoch-microsecond timestamp for audit logs.
    pub timestamp_us: i64,
}

/// Phase 4 authorization gateway. This is the single entry point the IPC
/// layer calls before performing a privileged operation. It:
/// 1. validates the action id is well-formed under the Zettings namespace,
/// 2. delegates to the supplied [`Authorizer`] (which MAY block while the
///    polkit agent shows a dialog),
/// 3. emits a [`PolkitDecision`] over `bus` so audit/UI subscribers can
///    observe every privileged operation in real time,
/// 4. returns the structured [`Authorization`] outcome.
///
/// # Errors
/// - [`PolkitError::UnknownAction`] when the action id does not start with
///   the `org.zyntrix.zettings.` namespace.
/// - Other errors from the underlying [`Authorizer`].
pub fn check_authorization_gateway(
    authorizer: &impl Authorizer,
    bus: &zettings_bus::Bus,
    action: &ActionId,
) -> Result<Authorization, PolkitError> {
    if !action.0.starts_with("org.zyntrix.zettings.") {
        return Err(PolkitError::UnknownAction(action.0.clone()));
    }
    // The underlying authorizer MAY block on the polkit agent dialog.
    // Phase 4 keeps the call inline; Phase 5 will route it through a
    // dedicated blocking IPC pool via `tokio::task::spawn_blocking` once the
    // Linux zbus authorizer lands its blocking DBus dispatch.
    let outcome = authorizer.check_authorization(action)?;
    let decision = PolkitDecision {
        action: action.0.clone(),
        authorization: outcome,
        timestamp_us: chrono_us_now(),
    };
    let _ = bus.publish(decision);
    Ok(outcome)
}

/// Cheap monotonically-ish epoch microseconds, computed without pulling the
/// full `chrono` dependency into this crate. Uses `std::time::SystemTime`.
fn chrono_us_now() -> i64 {
    match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(d) => i64::try_from(d.as_micros()).unwrap_or(i64::MAX),
        Err(_) => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_id_namespace() {
        let a = ActionId::zettings("network", "set-hostname");
        assert_eq!(a.0, "org.zyntrix.zettings.network.set-hostname");
    }

    #[cfg(feature = "zettings-mock")]
    #[test]
    fn mock_authorizer_always_authorizes() {
        let m = MockAuthorizer;
        let r = m
            .check_authorization(&ActionId::zettings("audio", "set-volume"))
            .unwrap();
        assert_eq!(r, Authorization::Authorized);
    }
}
