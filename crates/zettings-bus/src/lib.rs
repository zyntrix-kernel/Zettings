//! Tokio-based typed message bus for Zettings.
//!
//! Each topic is a typed `broadcast` channel. Publishers obtain a `BusHandle`
//! and call `publish`; subscribers call `subscribe` to receive a `Receiver`.
//! All events must implement `serde::Serialize + Clone`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod events;

use parking_lot::RwLock;
use std::any::TypeId;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::sync::broadcast;

/// Default per-topic channel capacity. Tuned for ~120 FPS UI event bursts.
const DEFAULT_CAPACITY: usize = 256;

/// Type-erased bundle of broadcast channels keyed by `TypeId`.
/// Cheap to clone via `Arc`. Internally a `RwLock<HashMap>`.
#[derive(Clone, Default)]
pub struct Bus {
    inner: Arc<RwLock<HashMap<TypeId, ErasedChannel>>>,
}

type BoxedSend = Box<dyn std::any::Any + Send + Sync>;

/// Opaque boxed channel storing the typed sender + slot for receivers.
/// Receivers are NOT stored here — they are returned to callers from
/// `Bus::subscribe`. The boxed sender is `tokio::sync::broadcast::Sender<T>`.
struct ErasedChannel {
    sender: BoxedSend,
}

impl Bus {
    /// Construct a new empty bus.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Publish an event. Spawns a new channel if the topic is unknown.
    ///
    /// Returns the number of active receivers reached, or `0` if there are none.
    #[must_use]
    pub fn publish<E: BusEvent>(&self, event: E) -> usize {
        let mut guard = self.inner.write();
        let entry = guard
            .entry(TypeId::of::<E>())
            .or_insert_with(|| ErasedChannel {
                sender: Box::new(broadcast::channel::<E>(DEFAULT_CAPACITY).0),
            });
        let sender = entry
            .sender
            .downcast_ref::<broadcast::Sender<E>>()
            .expect("type-erased channel holds a broadcast::Sender<E>");
        let _ = sender.send(event);
        sender.receiver_count()
    }

    /// Subscribe to events of type `E`. Returns a fresh `broadcast::Receiver`
    /// starting from the next published event.
    #[must_use]
    pub fn subscribe<E: BusEvent>(&self) -> broadcast::Receiver<E> {
        let mut guard = self.inner.write();
        let entry = guard
            .entry(TypeId::of::<E>())
            .or_insert_with(|| ErasedChannel {
                sender: Box::new(broadcast::channel::<E>(DEFAULT_CAPACITY).0),
            });
        let sender = entry
            .sender
            .downcast_ref::<broadcast::Sender<E>>()
            .expect("type-erased channel holds a broadcast::Sender<E>");
        sender.subscribe()
    }
}

/// Marker trait for events flowing over the bus. Auto-implemented for any
/// `Clone + Send + Sync + 'static + serde::Serialize` type.
pub trait BusEvent: Clone + Send + Sync + 'static + serde::Serialize {}

impl<T> BusEvent for T where T: Clone + Send + Sync + 'static + serde::Serialize {}

/// Phantom-data helper for crates that need a static handle to a `Bus`
/// parameterized by an event type without binding a concrete channel.
#[allow(dead_code)]
pub struct Topic<E> {
    _phantom: PhantomData<E>,
}

impl<E> Topic<E> {
    /// Construct a new `Topic` marker.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<E> Default for Topic<E> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Debug, Clone, Serialize, PartialEq, Eq)]
    struct DisplayChanged {
        output: u32,
    }

    #[tokio::test]
    async fn publish_subscribe_roundtrip() {
        let bus = Bus::new();
        let mut rx = bus.subscribe::<DisplayChanged>();
        let reached = bus.publish(DisplayChanged { output: 1 });
        assert_eq!(reached, 1);
        let event = rx.recv().await.expect("event");
        assert_eq!(event.output, 1);
    }

    #[tokio::test]
    async fn multiple_receivers() {
        let bus = Bus::new();
        let mut a = bus.subscribe::<DisplayChanged>();
        let mut b = bus.subscribe::<DisplayChanged>();
        let _ = bus.publish(DisplayChanged { output: 7 });
        assert_eq!(a.recv().await.unwrap().output, 7);
        assert_eq!(b.recv().await.unwrap().output, 7);
    }
}
