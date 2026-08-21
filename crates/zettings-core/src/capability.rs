//! Honest backend capability reporting.
//!
//! Every adapter (Phase 5) exposes its state through [`CapabilityState`] so the
//! UI can represent missing integrations truthfully instead of faking success
//! (`PLAN.md` §13, prompt.txt "no fake integrations").

use serde::{Deserialize, Serialize};

/// The availability of a backend capability, as reported by an adapter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(tag = "state", rename_all = "kebab-case")]
pub enum CapabilityState {
    /// The capability is fully functional.
    Available,
    /// The capability works partially; `reason` explains what is limited and why.
    Degraded {
        /// Why the capability is degraded (service version, missing plugin, ...).
        reason: String,
    },
    /// The capability cannot be used at all in this environment.
    Unavailable {
        /// Why the capability is unavailable (no hardware, service absent, ...).
        reason: String,
    },
}

impl CapabilityState {
    /// Returns `true` only for [`CapabilityState::Available`].
    pub fn is_available(&self) -> bool {
        matches!(self, Self::Available)
    }

    /// Returns the explanatory reason for non-[`CapabilityState::Available`]
    /// states, or `None` when fully available.
    pub fn reason(&self) -> Option<&str> {
        match self {
            Self::Available => None,
            Self::Degraded { reason } | Self::Unavailable { reason } => Some(reason),
        }
    }
}
