//! D-Bus backend adapters implementing the [`zettings_core::BackendAdapter`]
//! contract against freedesktop system daemons.
//!
//! Change-notification streaming (D-Bus property-change signals mapped to a
//! broadcast channel) is the extension surface this crate owns; bridges
//! consume it from here, never from concrete adapter types.

pub mod error;
pub mod mock;
pub mod power_profiles;
pub mod registry;
pub mod timedate;

pub use error::to_zettings_error;
pub use mock::MockAdapter;
pub use power_profiles::PowerProfilesAdapter;
pub use registry::AdapterRegistry;
pub use timedate::TimedateAdapter;

use zettings_core::BackendId;

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(tag = "event", rename_all = "kebab-case")]
pub enum BackendEvent {
    PropertyChanged { backend: String, key: String },
}

impl BackendEvent {
    #[must_use]
    pub fn backend(&self) -> &str {
        match self {
            Self::PropertyChanged { backend, .. } => backend,
        }
    }
}

#[async_trait::async_trait]
pub trait EventSource: Send + Sync {
    /// Subscribe to change events for this backend. Dropping the receiver
    /// unsubscribes. Bounded channel applies backpressure; a lagging bridge
    /// skips stale events rather than blocking D-Bus dispatch.
    async fn subscribe(&self) -> tokio::sync::mpsc::Receiver<BackendEvent>;

    fn backend_id(&self) -> BackendId;
}
