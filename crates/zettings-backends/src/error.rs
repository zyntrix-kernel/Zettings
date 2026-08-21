//! Shared backend error type.

/// Errors surfaced by system adapters.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum BackendError {
    /// The backing D-Bus service could not be reached or answered.
    #[error("{service} call failed: {detail}")]
    Service {
        /// Human-readable service name for diagnostics.
        service: &'static str,
        /// What went wrong.
        detail: String,
    },
    /// The requested value is outside the service's accepted set.
    #[error("invalid value {value:?}: {reason}")]
    InvalidValue {
        /// The rejected value.
        value: String,
        /// Why it was rejected.
        reason: String,
    },
}

impl BackendError {
    /// Builds a [`BackendError::Service`] from any displayable cause.
    pub fn service(service: &'static str, cause: impl std::fmt::Display) -> Self {
        Self::Service {
            service,
            detail: cause.to_string(),
        }
    }
}
