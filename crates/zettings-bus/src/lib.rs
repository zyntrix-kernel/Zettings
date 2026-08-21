//! Typed publish/subscribe message bus for ZETTINGS.
//!
//! Backend adapters publish state changes (a setting changed, a capability
//! appeared/disappeared, a backend failed) and any number of subscribers —
//! the UI bridge, the audit logger, the search re-indexer — observe them.
//!
//! Implementation: a [`tokio::sync::broadcast`] channel with a bounded
//! history of one per sender call. Slow consumers are detected and reported
//! through [`BusError::Lagged`] instead of blocking publishers (PLAN §10:
//! no blocking IPC).

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::broadcast;
use zettings_core::CapabilityState;

/// Capacity of the broadcast channel. Sized generously; events are small.
const CHANNEL_CAPACITY: usize = 256;

/// Events published on the bus.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "kind")]
#[non_exhaustive]
pub enum BusEvent {
    /// A setting value changed in a backend adapter.
    SettingChanged {
        /// Stable identifier of the changed setting, e.g. `system.display.scale`.
        setting_id: String,
    },
    /// An adapter's capability availability changed.
    CapabilityChanged {
        /// Adapter identifier, e.g. `display`.
        adapter: String,
        /// New honest capability state.
        state: CapabilityState,
    },
    /// The settings graph itself changed (plugin loaded/unloaded).
    RegistryChanged {
        /// Human-readable summary of what changed.
        detail: String,
    },
}

/// Errors returned by bus operations.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BusError {
    /// This receiver missed messages because it lagged behind the channel
    /// capacity. Contains the number of skipped events; receivers should
    /// resynchronize from authoritative state.
    #[error("receiver lagged; {0} event(s) skipped")]
    Lagged(u64),
    /// No active receivers were present when publishing.
    #[error("no active receivers")]
    NoReceivers,
}

/// Handle used by adapters to publish events.
#[derive(Debug, Clone)]
pub struct EventBus {
    tx: broadcast::Sender<BusEvent>,
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl EventBus {
    /// Creates an unshared bus. Prefer [`EventBus::shared`] for application
    /// state so multiple owners can publish.
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(CHANNEL_CAPACITY);
        Self { tx }
    }

    /// Creates a shared bus wrapped in an `Arc` for cheap cloning into
    /// adapters and command handlers.
    pub fn shared() -> std::sync::Arc<Self> {
        std::sync::Arc::new(Self::new())
    }

    /// Publishes `event` to all current subscribers.
    ///
    /// Publishing is non-blocking; delivery failures surface on the receiving
    /// side as [`BusError::Lagged`].
    pub fn publish(&self, event: &BusEvent) {
        match self.tx.send(event.clone()) {
            Ok(_) => {}
            Err(broadcast::error::SendError(_)) => {
                tracing::debug!(?event, "bus publish had no active receivers");
            }
        }
    }

    /// Subscribes to the event stream.
    pub fn subscribe(&self) -> EventReceiver {
        EventReceiver {
            rx: self.tx.subscribe(),
        }
    }

    /// Returns the number of active receivers (diagnostics/tests).
    pub fn receiver_count(&self) -> usize {
        self.tx.receiver_count()
    }
}

/// Receiving half of the bus.
#[derive(Debug)]
pub struct EventReceiver {
    rx: broadcast::Receiver<BusEvent>,
}

impl EventReceiver {
    /// Receives the next event, translating lag into [`BusError::Lagged`].
    ///
    /// # Errors
    /// Returns [`BusError::Lagged`] when the receiver fell behind, which is
    /// recoverable — simply await again to resume at the newest events.
    /// Returns [`BusError::NoReceivers`] when every sender has been dropped.
    pub async fn recv(&mut self) -> Result<BusEvent, BusError> {
        match self.rx.recv().await {
            Ok(event) => Ok(event),
            Err(broadcast::error::RecvError::Lagged(skipped)) => Err(BusError::Lagged(skipped)),
            Err(broadcast::error::RecvError::Closed) => Err(BusError::NoReceivers),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn publishes_to_subscribers() {
        let bus = EventBus::new();
        let mut sub = bus.subscribe();
        assert_eq!(bus.receiver_count(), 1);
        bus.publish(&BusEvent::SettingChanged {
            setting_id: "system.display.scale".into(),
        });
        let got = sub.recv().await.expect("event");
        assert_eq!(
            got,
            BusEvent::SettingChanged {
                setting_id: "system.display.scale".into()
            }
        );
    }

    #[tokio::test]
    async fn serializes_round_trip() {
        let event = BusEvent::CapabilityChanged {
            adapter: "audio".into(),
            state: CapabilityState::Unavailable {
                reason: "PipeWire not running".into(),
            },
        };
        let json = serde_json::to_string(&event).expect("serialize");
        let back: BusEvent = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, event);
    }
}
