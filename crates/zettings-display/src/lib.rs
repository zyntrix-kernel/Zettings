//! `zettings-display` — `KScreen` display management.
//!
//! Resolution, refresh rate, scaling, and night color management through
//! `org.kde.KScreen` over `zbus` on Linux. The Windows dev loop uses an
//! in-memory [`MockBackend`] that exposes the same trait surface so the
//! frontend can be exercised without a real `KScreen` daemon.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use serde::{Deserialize, Serialize};
use thiserror::Error;
use zettings_bus::Bus;
use zettings_bus::events::DisplayReplug;

/// A `KScreen` output id, e.g. `HDMI-A-1`. Stable across daemon restarts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct OutputId(pub String);

impl OutputId {
    /// Construct an output id from a bare string.
    #[must_use]
    pub fn new<S: Into<String>>(s: S) -> Self {
        Self(s.into())
    }
}

/// A display mode offered by an output (resolution + refresh).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct DisplayMode {
    /// Width in physical pixels.
    pub width: u32,
    /// Height in physical pixels.
    pub height: u32,
    /// Vertical refresh rate in Hz.
    pub refresh_hz: f32,
}

/// Output configuration applied via [`Backend::apply_mode`].
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct OutputConfig {
    /// Mode to switch to.
    pub mode: DisplayMode,
    /// Logical scale factor (1.0 = native, 2.0 = 200% `HiDPI`).
    pub scale: f32,
}

/// Errors surfaced by the display backend.
#[derive(Debug, Error)]
pub enum DisplayError {
    /// The requested output does not exist.
    #[error("display output not found: {0}")]
    OutputNotFound(String),
    /// The requested mode is not offered by the output.
    #[error("mode not available for output {output}: {width}x{height}@{refresh_hz}Hz")]
    ModeNotAvailable {
        /// Output id that rejected the mode.
        output: String,
        /// Width in pixels.
        width: u32,
        /// Height in pixels.
        height: u32,
        /// Refresh rate in Hz.
        refresh_hz: f32,
    },
    /// The underlying `zbus`/`KScreen` service was unreachable.
    #[error("KScreen service unavailable: {0}")]
    ServiceUnavailable(String),
}

/// Trait surface implemented by both the mock state machine and the real
/// `zbus` Linux backend. The IPC layer calls through this trait so it can be
/// swapped per-target without changing command code.
pub trait Backend: Send + Sync {
    /// List currently connected outputs and the modes each supports.
    ///
    /// # Errors
    /// Returns [`DisplayError::ServiceUnavailable`] when the underlying
    /// `KScreen` service is unreachable on the real target.
    fn list_outputs(&self) -> Result<Vec<(OutputId, Vec<DisplayMode>)>, DisplayError>;

    /// Apply `config` to `output`. Emits a [`DisplayReplug`] event over
    /// `bus` whenever the change is reflected back from `KScreen`.
    ///
    /// # Errors
    /// - [`DisplayError::OutputNotFound`] when `output` is not present.
    /// - [`DisplayError::ModeNotAvailable`] when the mode is not offered.
    fn apply_mode(
        &self,
        output: &OutputId,
        config: OutputConfig,
        bus: &Bus,
    ) -> Result<(), DisplayError>;
}

/// In-memory mock backend. Always compiled; the `zettings-mock` feature only
/// suppresses the `LinuxBackend` compile path on Linux.
///
/// Holds a single virtual `HDMI-A-1` output offering 1920x1080@60 and
/// 3840x2160@60 modes, plus a `DP-1` output at 2560x1440@144. State is mutated
/// in place; every successful `apply_mode` publishes a [`DisplayReplug`].
pub struct MockBackend {
    state: parking_lot::Mutex<Vec<(OutputId, Vec<DisplayMode>)>>,
}

impl MockBackend {
    /// Construct a mock backend pre-populated with two outputs.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: parking_lot::Mutex::new(vec![
                (
                    OutputId::new("HDMI-A-1"),
                    vec![
                        DisplayMode {
                            width: 1920,
                            height: 1080,
                            refresh_hz: 60.0,
                        },
                        DisplayMode {
                            width: 3840,
                            height: 2160,
                            refresh_hz: 60.0,
                        },
                    ],
                ),
                (
                    OutputId::new("DP-1"),
                    vec![DisplayMode {
                        width: 2560,
                        height: 1440,
                        refresh_hz: 144.0,
                    }],
                ),
            ]),
        }
    }
}

impl Default for MockBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl Backend for MockBackend {
    fn list_outputs(&self) -> Result<Vec<(OutputId, Vec<DisplayMode>)>, DisplayError> {
        Ok(self.state.lock().clone())
    }

    fn apply_mode(
        &self,
        output: &OutputId,
        config: OutputConfig,
        bus: &Bus,
    ) -> Result<(), DisplayError> {
        let state = self.state.lock();
        let pair = state
            .iter()
            .find(|(id, _)| id == output)
            .ok_or_else(|| DisplayError::OutputNotFound(output.0.clone()))?;
        let modes: &[DisplayMode] = &pair.1;
        if !modes.contains(&config.mode) {
            return Err(DisplayError::ModeNotAvailable {
                output: output.0.clone(),
                width: config.mode.width,
                height: config.mode.height,
                refresh_hz: config.mode.refresh_hz,
            });
        }
        let _ = bus.publish(DisplayReplug {
            output_id: crc32_of(output.0.as_bytes()),
            connected: true,
        });
        Ok(())
    }
}

/// Cheap deterministic u32 from a string, used as the `output_id` on the bus
/// when the `KScreen` integer id is not available (mock path).
fn crc32_of(bytes: &[u8]) -> u32 {
    // Minimal CRC-32 (IEEE 802.3) — avoids pulling a `crc32fast` dep just for
    // the mock. Polynomial 0xEDB88320 reflected.
    let mut crc: u32 = 0xFFFF_FFFF;
    for &b in bytes {
        crc ^= u32::from(b);
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

/// Real Linux backend. Connects to `org.kde.KScreen` over `zbus`.
#[cfg(all(target_os = "linux", not(feature = "zettings-mock")))]
pub struct LinuxBackend;

#[cfg(all(target_os = "linux", not(feature = "zettings-mock")))]
impl Backend for LinuxBackend {
    fn list_outputs(&self) -> Result<Vec<(OutputId, Vec<DisplayMode>)>, DisplayError> {
        // Phase 5+: real `zbus` call to `org.kde.KScreen.GetOutputs` with
        // mode introspection. Until the typed `KScreen` proxy lands, surface
        // a deterministic error so the frontend can render the failure cleanly.
        Err(DisplayError::ServiceUnavailable(
            "KScreen zbus integration pending".into(),
        ))
    }

    fn apply_mode(
        &self,
        _output: &OutputId,
        _config: OutputConfig,
        _bus: &Bus,
    ) -> Result<(), DisplayError> {
        Err(DisplayError::ServiceUnavailable(
            "KScreen zbus integration pending".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_lists_two_outputs() {
        let b = MockBackend::new();
        let outs = b.list_outputs().expect("list");
        assert_eq!(outs.len(), 2);
    }

    #[tokio::test]
    async fn mock_apply_publishes_replug() {
        let b = MockBackend::new();
        let bus = Bus::new();
        let mut rx = bus.subscribe::<DisplayReplug>();
        let out = OutputId::new("HDMI-A-1");
        b.apply_mode(
            &out,
            OutputConfig {
                mode: DisplayMode {
                    width: 1920,
                    height: 1080,
                    refresh_hz: 60.0,
                },
                scale: 1.0,
            },
            &bus,
        )
        .expect("apply");
        let evt = rx.recv().await.expect("event");
        assert!(evt.connected);
    }

    #[tokio::test]
    async fn mock_rejects_unknown_output() {
        let b = MockBackend::new();
        let bus = Bus::new();
        let err = b
            .apply_mode(
                &OutputId::new("VGA-1"),
                OutputConfig {
                    mode: DisplayMode {
                        width: 1024,
                        height: 768,
                        refresh_hz: 60.0,
                    },
                    scale: 1.0,
                },
                &bus,
            )
            .unwrap_err();
        assert!(matches!(err, DisplayError::OutputNotFound(_)));
    }

    #[tokio::test]
    async fn mock_rejects_unsupported_mode() {
        let b = MockBackend::new();
        let bus = Bus::new();
        let err = b
            .apply_mode(
                &OutputId::new("HDMI-A-1"),
                OutputConfig {
                    mode: DisplayMode {
                        width: 800,
                        height: 600,
                        refresh_hz: 60.0,
                    },
                    scale: 1.0,
                },
                &bus,
            )
            .unwrap_err();
        assert!(matches!(err, DisplayError::ModeNotAvailable { .. }));
    }
}
