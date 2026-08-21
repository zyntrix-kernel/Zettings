//! Shared error type for the ZETTINGS domain layer.

/// Errors produced by registry, route, and domain validation logic.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ZettingsError {
    /// A route string violated the `zettings://` scheme grammar.
    #[error("invalid route {raw:?}: {reason}")]
    InvalidRoute {
        /// The rejected raw route string.
        raw: String,
        /// Human-readable explanation of the violation.
        reason: String,
    },
    /// An identifier violated the slug grammar (`[a-z0-9-]`, non-empty).
    #[error("invalid identifier {raw:?}: {reason}")]
    InvalidId {
        /// The rejected raw identifier.
        raw: String,
        /// Human-readable explanation of the violation.
        reason: String,
    },
    /// A referenced category or setting does not exist in the graph.
    #[error("unknown registry node: {0}")]
    UnknownNode(
        /// The missing node identifier.
        String,
    ),
}

impl ZettingsError {
    /// Convenience constructor for [`ZettingsError::InvalidId`].
    pub fn invalid_id(raw: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidId {
            raw: raw.into(),
            reason: reason.into(),
        }
    }

    /// Convenience constructor for [`ZettingsError::InvalidRoute`].
    pub fn invalid_route(raw: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidRoute {
            raw: raw.into(),
            reason: reason.into(),
        }
    }
}
