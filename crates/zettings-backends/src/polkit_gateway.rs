//! Real PolicyKit gateway over the system bus (Linux only).
//!
//! Implements the crate-agnostic [`AuthorizationGateway`] seam defined by
//! `zettings-polkit` using `org.freedesktop.PolicyKit1.Authority`.
//! Interactive authentication is requested (`allow_interaction = 1`); results
//! are fail-closed exactly like the mock.

use async_trait::async_trait;
use std::collections::HashMap;
use zettings_polkit::{AuthorizationGateway, Decision, PolkitAction, PolkitError};

const SERVICE: &str = "polkit";
const BUS_NAME: &str = "org.freedesktop.PolicyKit1";
const OBJECT_PATH: &str = "/org/freedesktop/PolicyKit1/Authority";
const IFACE: &str = "org.freedesktop.PolicyKit1.Authority";
/// `Allowed = 1 | Interaction = 2` — we request interactive authorization.
const FLAGS_ALLOW_INTERACTION: u32 = 1;

/// Gateway backed by the system PolicyKit authority.
pub struct PolkitGateway {
    conn: zbus::Connection,
}

impl PolkitGateway {
    /// Binds to an existing system-bus connection.
    pub const fn new(conn: zbus::Connection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl AuthorizationGateway for PolkitGateway {
    async fn authorize(&self, action: &PolkitAction) -> Result<Decision, PolkitError> {
        let proxy = zbus::Proxy::new(&self.conn, BUS_NAME, OBJECT_PATH, IFACE)
            .map_err(|_| PolkitError::AuthorityUnavailable)?;

        let subject: (String, (zbus::zvariant::Value<'_>,)) = (
            "system-bus-name".to_owned(),
            (zbus::zvariant::Value::new(
                self.conn
                    .unique_name()
                    .ok_or(PolkitError::AuthorityUnavailable)?
                    .as_str(),
            ),),
        );
        let details: HashMap<String, String> = HashMap::new();
        let cancel_id: String = String::new();

        let result: (bool, bool, HashMap<String, String>) = proxy
            .call(
                "CheckAuthorization",
                &(
                    subject,
                    action.as_str(),
                    details,
                    FLAGS_ALLOW_INTERACTION,
                    cancel_id,
                ),
            )
            .await
            .map_err(|e| {
                tracing::warn!(service = SERVICE, error = %e, "CheckAuthorization failed");
                PolkitError::AuthorityUnavailable
            })?;

        let (is_authorized, is_challenge, _details) = result;
        if is_authorized {
            Ok(Decision::AuthorizedAfterPrompt)
        } else if is_challenge {
            // The user dismissed or failed the dialog.
            Ok(Decision::Denied)
        } else {
            Ok(Decision::Denied)
        }
    }
}
