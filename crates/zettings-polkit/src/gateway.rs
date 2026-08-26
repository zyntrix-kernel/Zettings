use zettings_core::PolkitActionId;

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "kebab-case")]
pub enum Decision {
    Denied,
    AuthenticationRequired,
    Allowed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthorizationQuery<'a> {
    pub actor: &'a str,
    pub action: &'a PolkitActionId,
}

/// Contract for the single component allowed to consult `PolicyKit1`.
///
/// Implementations must never cache an `Allowed` across process restarts and
/// must surface `AuthenticationRequired` so the desktop polkit agent dialog
/// can run before the caller retries once.
pub trait Gateway: Send + Sync {
    /// # Errors
    ///
    /// Returns [`ZettingsError::InvalidAction`] for actions outside the
    /// Zettings namespace and transport errors from the authority backend.
    fn authorize(
        &self,
        query: &AuthorizationQuery<'_>,
    ) -> Result<Decision, zettings_core::ZettingsError>;
}

/// Development-sandbox gateway mirroring the WSL2 rule documented in
/// docs/setup/wsl2.md. NEVER ship this in production builds: it authorizes a
/// fixed allow-list without consulting `PolicyKit1`.
pub struct AllowListGateway {
    allowed_actions: Vec<PolkitActionId>,
}

impl AllowListGateway {
    #[must_use]
    pub fn new(allowed_actions: &[PolkitActionId]) -> Self {
        Self {
            allowed_actions: allowed_actions.to_vec(),
        }
    }
}

impl Gateway for AllowListGateway {
    fn authorize(
        &self,
        query: &AuthorizationQuery<'_>,
    ) -> Result<Decision, zettings_core::ZettingsError> {
        query.action.validate()?;
        if self.allowed_actions.iter().any(|a| a.0 == query.action.0) {
            Ok(Decision::Allowed)
        } else {
            Ok(Decision::Denied)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ntp() -> PolkitActionId {
        PolkitActionId(String::from("org.zyntrix.zettings.timedate.set-ntp"))
    }

    fn foreign() -> PolkitActionId {
        PolkitActionId(String::from("org.freedesktop.timedate1.set-ntp"))
    }

    fn valid_but_unlisted() -> PolkitActionId {
        PolkitActionId(String::from("org.zyntrix.zettings.timedate.set-timezone"))
    }

    #[test]
    fn sandbox_gateway_authorizes_only_listed_zettings_actions() {
        let gw = AllowListGateway::new(&[ntp()]);

        let ok = gw
            .authorize(&AuthorizationQuery {
                actor: "zyntrix",
                action: &ntp(),
            })
            .unwrap();
        assert_eq!(ok, Decision::Allowed);

        let unlisted = gw
            .authorize(&AuthorizationQuery {
                actor: "zyntrix",
                action: &valid_but_unlisted(),
            })
            .unwrap();
        assert_eq!(unlisted, Decision::Denied);
    }

    #[test]
    fn namespace_validation_rejects_foreign_actions_even_if_listed() {
        let gw = AllowListGateway::new(&[foreign()]);
        assert!(
            gw.authorize(&AuthorizationQuery {
                actor: "x",
                action: &foreign(),
            })
            .is_err()
        );
    }
}
