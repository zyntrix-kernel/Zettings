//! `zettings-audio` — `PipeWire`/`PulseAudio` mixer, device routing.
//!
//! Per-application volume sliders, stream routing, and device selection
//! through `pipewire`/`libpulse-binding` on Linux. The Windows dev loop uses
//! an in-memory [`MockBackend`] that exposes the same trait surface.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use zettings_bus::Bus;
use zettings_bus::events::AudioVolume;

/// A `PulseAudio`/`PipeWire` sink or source id.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct StreamId(pub u32);

/// Normalized volume in `[0.0, 1.0]`. `1.0` == 0 dB unity gain.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Volume(pub f32);

impl Volume {
    /// Clamp a raw float into the valid `[0.0, 1.0]` range.
    #[must_use]
    pub fn clamp(raw: f32) -> Self {
        Self(raw.clamp(0.0, 1.0))
    }
}

/// A named app/output stream with live volume state, for the mixer UI.
/// The simpler [`StreamId`] only conveys identity; this DTO adds the human
/// label, current volume and mute flag so the frontend mixer panel can render
/// without a follow-up per-stream query.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct AudioStream {
    /// Application or sink id matching the [`StreamId`] registry.
    pub stream_id: u32,
    /// Human-readable label, e.g. "Master Output", "Zyntrix Aurora", "Voice Call".
    pub label_id: u8,
    /// Normalized volume in `[0.0, 1.0]`.
    pub volume: f32,
    /// `true` when the stream is muted.
    pub muted: bool,
}

// Stable label table for mock streams; the frontend translates the `label_id`
// index into a localized display string (kept Rust-side so the schema is
// explicit and unit-testable without bundling string resources here).
impl AudioStream {
    /// Label id for the system master sink (index 0).
    pub const LABEL_MASTER: u8 = 0;
    /// Label id for the Zyntrix Aurora app sink (index 1).
    pub const LABEL_AURORA: u8 = 1;
    /// Label id for a voice/video call sink (index 2).
    pub const LABEL_CALL: u8 = 2;

    /// English fallback label — the frontend overrides with localized names.
    #[must_use]
    pub fn label_text(&self) -> &'static str {
        match self.label_id {
            Self::LABEL_MASTER => "Master Output",
            Self::LABEL_AURORA => "Zyntrix Aurora",
            Self::LABEL_CALL => "Voice Call",
            _ => "Unknown Stream",
        }
    }
}

/// Errors surfaced by the audio backend.
#[derive(Debug, Error)]
pub enum AudioError {
    /// The stream id does not exist.
    #[error("audio stream not found: {0}")]
    StreamNotFound(u32),
    /// The underlying `PipeWire`/`PulseAudio` daemon was unreachable.
    #[error("audio daemon unavailable: {0}")]
    ServiceUnavailable(String),
}

/// Trait surface implemented by both the mock state machine and the real
/// `pipewire`/`libpulse` backend. The IPC layer calls through this trait so
/// the implementation can be swapped per-target without touching commands.
pub trait Backend: Send + Sync {
    /// List currently active stream ids (sinks + applications).
    ///
    /// # Errors
    /// Returns [`AudioError::ServiceUnavailable`] when the daemon is down.
    fn list_streams(&self) -> Result<Vec<StreamId>, AudioError>;

    /// List streams with their live volume state (label + volume + muted).
    /// Drives the mixer panel. The mock returns the same stream set as
    /// [`Backend::list_streams`] but enriched with current state from the
    /// mock store so the frontend can render cards without a second round-trip.
    ///
    /// # Errors
    /// Returns [`AudioError::ServiceUnavailable`] when the daemon is down.
    fn list_streams_detailed(&self) -> Result<Vec<AudioStream>, AudioError>;

    /// Apply `volume` (and `muted`) to `stream`. Emits an [`AudioVolume`]
    /// event over `bus` reflecting the new state.
    ///
    /// # Errors
    /// - [`AudioError::StreamNotFound`] when `stream` is not registered.
    fn set_volume(
        &self,
        stream: StreamId,
        volume: Volume,
        muted: bool,
        bus: &Bus,
    ) -> Result<(), AudioError>;
}

/// In-memory mock backend. Holds a fixed set of stream ids and tracks
/// per-stream volume in a hash; every successful `set_volume` publishes an
/// [`AudioVolume`].
pub struct MockBackend {
    state: parking_lot::Mutex<HashMap<u32, AudioVolume>>,
}

impl MockBackend {
    /// Construct a mock backend pre-populated with three stream ids 0, 1, 2
    /// all at unity volume and unmuted.
    #[must_use]
    pub fn new() -> Self {
        let mut state = HashMap::new();
        for id in 0..3_u32 {
            state.insert(
                id,
                AudioVolume {
                    stream_id: id,
                    volume: 1.0,
                    muted: false,
                },
            );
        }
        Self {
            state: parking_lot::Mutex::new(state),
        }
    }
}

impl Default for MockBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl Backend for MockBackend {
    fn list_streams(&self) -> Result<Vec<StreamId>, AudioError> {
        Ok(self.state.lock().keys().copied().map(StreamId).collect())
    }

    fn list_streams_detailed(&self) -> Result<Vec<AudioStream>, AudioError> {
        // The mock stores three streams keyed 0,1,2 with stable labels; match
        // stream_id → label_id directly (they coincide by construction).
        let state = self.state.lock();
        let mut streams = state
            .values()
            .map(|v| AudioStream {
                stream_id: v.stream_id,
                label_id: match v.stream_id {
                    0 => AudioStream::LABEL_MASTER,
                    1 => AudioStream::LABEL_AURORA,
                    2 => AudioStream::LABEL_CALL,
                    other => other.try_into().unwrap_or(u8::MAX),
                },
                volume: v.volume,
                muted: v.muted,
            })
            .collect::<Vec<_>>();
        streams.sort_by_key(|s| s.stream_id);
        Ok(streams)
    }

    fn set_volume(
        &self,
        stream: StreamId,
        volume: Volume,
        muted: bool,
        bus: &Bus,
    ) -> Result<(), AudioError> {
        let mut state = self.state.lock();
        if !state.contains_key(&stream.0) {
            return Err(AudioError::StreamNotFound(stream.0));
        }
        let evt = AudioVolume {
            stream_id: stream.0,
            volume: volume.0,
            muted,
        };
        state.insert(stream.0, evt);
        drop(state);
        let _ = bus.publish(evt);
        Ok(())
    }
}

/// Real Linux backend. Connects to `pipewire` + `libpulse-binding`.
#[cfg(all(target_os = "linux", not(feature = "zettings-mock")))]
pub struct LinuxBackend;

#[cfg(all(target_os = "linux", not(feature = "zettings-mock")))]
impl Backend for LinuxBackend {
    fn list_streams(&self) -> Result<Vec<StreamId>, AudioError> {
        Err(AudioError::ServiceUnavailable(
            "pipewire integration pending".into(),
        ))
    }

    fn list_streams_detailed(&self) -> Result<Vec<AudioStream>, AudioError> {
        Err(AudioError::ServiceUnavailable(
            "pipewire integration pending".into(),
        ))
    }

    fn set_volume(
        &self,
        _stream: StreamId,
        _volume: Volume,
        _muted: bool,
        _bus: &Bus,
    ) -> Result<(), AudioError> {
        Err(AudioError::ServiceUnavailable(
            "pipewire integration pending".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_lists_three_streams() {
        let b = MockBackend::new();
        let streams = b.list_streams().expect("list");
        assert_eq!(streams.len(), 3);
    }

    #[tokio::test]
    async fn mock_list_streams_detailed_carries_labels_and_volume() {
        let b = MockBackend::new();
        let detailed = b.list_streams_detailed().expect("detailed");
        assert_eq!(detailed.len(), 3);
        // Master sink is id 0, label MASTER, full volume, unmuted.
        let master = detailed.iter().find(|s| s.stream_id == 0).expect("master");
        assert_eq!(master.label_id, AudioStream::LABEL_MASTER);
        assert_eq!(master.label_text(), "Master Output");
        assert!((master.volume - 1.0).abs() < f32::EPSILON);
        assert!(!master.muted);
    }

    #[tokio::test]
    async fn mock_set_volume_publishes_event() {
        let b = MockBackend::new();
        let bus = Bus::new();
        let mut rx = bus.subscribe::<AudioVolume>();
        b.set_volume(StreamId(1), Volume::clamp(0.5), false, &bus)
            .expect("set");
        let evt = rx.recv().await.expect("event");
        assert!((evt.volume - 0.5).abs() < f32::EPSILON);
        assert!(!evt.muted);
    }

    #[tokio::test]
    async fn mock_rejects_unknown_stream() {
        let b = MockBackend::new();
        let bus = Bus::new();
        let err = b
            .set_volume(StreamId(99), Volume::clamp(0.3), true, &bus)
            .unwrap_err();
        assert!(matches!(err, AudioError::StreamNotFound(99)));
    }
}
