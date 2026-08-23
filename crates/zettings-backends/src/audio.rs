//! Audio output control via the `PulseAudio` client library (`PipeWire`'s
//! pulse compatibility layer on modern KDE systems).

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use zettings_core::CapabilityState;

use crate::error::BackendError;

#[cfg(target_os = "linux")]
const SERVICE: &str = "pulseaudio";

/// One output device (sink).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioSink {
    /// Stable sink name (e.g. `alsa_output.pci-0000_00_1f.3.analog-stereo`).
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Mute state.
    pub muted: bool,
    /// Average channel volume, 0–150 (% where 100 = 0 dB reference).
    pub volume_percent: u32,
    /// True for the server's default sink.
    pub is_default: bool,
}

/// Audio output surface.
#[async_trait]
pub trait AudioAdapter: Send + Sync {
    /// Detects whether a PulseAudio-compatible server is reachable.
    async fn capability(&self) -> CapabilityState;
    /// Lists output sinks with volume/mute state.
    ///
    /// # Errors
    /// [`BackendError::Service`] when the server cannot be consulted.
    async fn sinks(&self) -> Result<Vec<AudioSink>, BackendError>;
    /// Sets mute on one sink by name.
    ///
    /// # Errors
    /// [`BackendError::InvalidValue`] for unknown sinks,
    /// [`BackendError::Service`] on failure.
    async fn set_sink_mute(&self, sink: &str, muted: bool) -> Result<(), BackendError>;
    /// Sets average volume (0–150%) on one sink by name.
    ///
    /// # Errors
    /// [`BackendError::InvalidValue`] for unknown sinks or out-of-range
    /// volume, [`BackendError::Service`] on failure.
    async fn set_sink_volume(&self, sink: &str, percent: u32) -> Result<(), BackendError>;
}

/// Deterministic mock with set→get round-trip semantics.
#[derive(Debug, Default)]
pub struct MockAudio {
    state: tokio::sync::Mutex<Vec<AudioSink>>,
}

impl MockAudio {
    fn seed() -> Vec<AudioSink> {
        vec![AudioSink {
            name: "mock-output.analog-stereo".to_owned(),
            description: "Mock Built-in Audio Analog Stereo".to_owned(),
            muted: false,
            volume_percent: 80,
            is_default: true,
        }]
    }
}

#[async_trait]
impl AudioAdapter for MockAudio {
    async fn capability(&self) -> CapabilityState {
        CapabilityState::Available
    }

    async fn sinks(&self) -> Result<Vec<AudioSink>, BackendError> {
        let mut guard = self.state.lock().await;
        if guard.is_empty() {
            *guard = Self::seed();
        }
        Ok(guard.clone())
    }

    async fn set_sink_mute(&self, sink: &str, muted: bool) -> Result<(), BackendError> {
        let mut guard = self.state.lock().await;
        match guard.iter_mut().find(|s| s.name == sink) {
            Some(entry) => {
                entry.muted = muted;
                Ok(())
            }
            None => Err(BackendError::InvalidValue {
                value: sink.to_owned(),
                reason: "unknown sink".to_owned(),
            }),
        }
    }

    async fn set_sink_volume(&self, sink: &str, percent: u32) -> Result<(), BackendError> {
        if percent > 150 {
            return Err(BackendError::InvalidValue {
                value: percent.to_string(),
                reason: "volume must be 0–150%".to_owned(),
            });
        }
        let mut guard = self.state.lock().await;
        match guard.iter_mut().find(|s| s.name == sink) {
            Some(entry) => {
                entry.volume_percent = percent;
                Ok(())
            }
            None => Err(BackendError::InvalidValue {
                value: sink.to_owned(),
                reason: "unknown sink".to_owned(),
            }),
        }
    }
}

/// Real adapter using the `PulseAudio` simple blocking pattern (Linux only).
///
/// Operations run on the blocking pool because libpulse drives its own
/// mainloop; the async surface stays non-blocking for Tokio workers
/// (rust-concurrency skill: blocking FFI → `spawn_blocking`).
#[cfg(target_os = "linux")]
pub struct LinuxAudio;

#[cfg(target_os = "linux")]
impl LinuxAudio {
    /// Constructs the adapter (stateless; each operation opens a short-lived
    /// context, matching pactl-style tooling semantics).
    pub const fn new() -> Self {
        Self
    }

    async fn run_blocking<T>(
        f: impl FnOnce(
            &mut libpulse_binding::context::Context,
            &mut libpulse_binding::mainloop::standard::Mainloop,
        ) -> Result<T, BackendError>
        + Send
        + 'static,
    ) -> Result<T, BackendError>
    where
        T: Send + 'static,
    {
        tokio::task::spawn_blocking(move || {
            use libpulse_binding::context::{Context, FlagSet};
            use libpulse_binding::mainloop::standard::{IterateResult, Mainloop};

            let mut mainloop = Mainloop::new()
                .ok_or_else(|| BackendError::service(SERVICE, "failed to allocate mainloop"))?;
            let mut context = Context::new(&mainloop, "org.zyntrix.zettings")
                .ok_or_else(|| BackendError::service(SERVICE, "failed to allocate context"))?;
            context
                .connect(None, FlagSet::NOFLAGS, None)
                .map_err(|e| BackendError::service(SERVICE, e))?;

            // Pump until the context is ready or fails.
            loop {
                match context.get_state() {
                    libpulse_binding::context::State::Ready => break,
                    libpulse_binding::context::State::Failed
                    | libpulse_binding::context::State::Terminated => {
                        return Err(BackendError::service(SERVICE, "server connection failed"));
                    }
                    _ => match mainloop.iterate(false) {
                        IterateResult::Quit(_) | IterateResult::Err(_) => {
                            return Err(BackendError::service(
                                SERVICE,
                                "mainloop terminated while connecting",
                            ));
                        }
                        IterateResult::Success(_) => {}
                    },
                }
            }

            let outcome = f(&mut context, &mut mainloop);
            context.disconnect();
            outcome
        })
        .await
        .map_err(|e| BackendError::service(SERVICE, e))?
    }
}

#[cfg(target_os = "linux")]
const PULSE_NORM: u32 = libpulse_binding::volume::Volume::NORMAL.0;

#[cfg(target_os = "linux")]
#[async_trait]
impl AudioAdapter for LinuxAudio {
    async fn capability(&self) -> CapabilityState {
        let probe = Self::run_blocking(|ctx, ml| {
            use std::cell::RefCell;
            use std::rc::Rc;

            let found: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
            let sink = found.clone();
            let op = ctx.introspect().get_server_info(move |info| {
                *sink.borrow_mut() = info
                    .default_sink_name
                    .as_ref()
                    .map(std::string::ToString::to_string);
            });
            while op.get_state() == libpulse_binding::operation::State::Running {
                if !matches!(
                    ml.iterate(false),
                    libpulse_binding::mainloop::standard::IterateResult::Success(_)
                ) {
                    break;
                }
            }
            drop(op);
            let default = found.borrow().clone();
            Ok(match default {
                Some(_) => crate::service_available(true, SERVICE),
                None => CapabilityState::Unavailable {
                    reason: format!("{SERVICE} server did not answer"),
                },
            })
        });
        match probe.await {
            Ok(state) => state,
            Err(e) => CapabilityState::Unavailable {
                reason: format!("{SERVICE} unavailable: {e}"),
            },
        }
    }

    async fn sinks(&self) -> Result<Vec<AudioSink>, BackendError> {
        Self::run_blocking(|ctx, ml| {
            use libpulse_binding::mainloop::standard::IterateResult;
            use libpulse_binding::operation::State;
            use std::cell::RefCell;
            use std::rc::Rc;

            let collected: Rc<RefCell<Vec<(String, String, bool, u32)>>> =
                Rc::new(RefCell::new(Vec::new()));
            let done = Rc::new(RefCell::new(false));
            let collector = collected.clone();
            let finish = done.clone();
            let default_name: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
            let dn = default_name.clone();

            let op_list = ctx.introspect().get_sink_info_list(move |result| {
                if let libpulse_binding::callbacks::ListResult::Item(info) = result {
                    let desc = info
                        .description
                        .as_ref()
                        .map(std::string::ToString::to_string)
                        .unwrap_or_default();
                    let vol = info.volume.avg().0;
                    collector.borrow_mut().push((
                        info.name
                            .as_ref()
                            .map(std::string::ToString::to_string)
                            .unwrap_or_default(),
                        desc,
                        info.mute,
                        vol,
                    ));
                } else if let libpulse_binding::callbacks::ListResult::End = result {
                    *finish.borrow_mut() = true;
                } else {
                    *finish.borrow_mut() = true;
                }
            });
            let op_default = ctx.introspect().get_server_info(move |info| {
                *dn.borrow_mut() = info
                    .default_sink_name
                    .as_ref()
                    .map(std::string::ToString::to_string);
            });

            while !*done.borrow() || op_default.get_state() == State::Running {
                match ml.iterate(false) {
                    IterateResult::Success(_) => {}
                    _ => break,
                }
            }
            drop(op_list);

            let default = default_name.borrow().clone();
            let sinks = collected
                .borrow()
                .iter()
                .map(|(name, desc, muted, vol)| {
                    let raw = u64::from(*vol) * 100 / u64::from(PULSE_NORM);
                    // The clamp bounds the value to u32 range before conversion.
                    let percent = u32::try_from(raw.min(150)).unwrap_or(150);
                    AudioSink {
                        name: name.clone(),
                        description: desc.clone(),
                        muted: *muted,
                        volume_percent: percent,
                        is_default: default.as_deref() == Some(name.as_str()),
                    }
                })
                .collect();
            Ok(sinks)
        })
        .await
    }

    async fn set_sink_mute(&self, sink: &str, muted: bool) -> Result<(), BackendError> {
        let sink = sink.to_owned();
        Self::run_blocking(move |ctx, ml| {
            use libpulse_binding::mainloop::standard::IterateResult;
            use libpulse_binding::operation::State;
            use std::cell::Cell;
            use std::rc::Rc;

            let ok = Rc::new(Cell::new(false));
            let done = Rc::new(Cell::new(false));
            let ok2 = ok.clone();
            let done2 = done.clone();
            let name = sink.clone();
            let op = ctx.introspect().set_sink_mute_by_name(
                &name,
                muted,
                Some(Box::new(move |success| {
                    ok2.set(success);
                    done2.set(true);
                })),
            );
            while !done.get() || op.get_state() == State::Running {
                match ml.iterate(false) {
                    IterateResult::Success(_) => {}
                    _ => break,
                }
            }
            if ok.get() {
                Ok(())
            } else {
                Err(BackendError::InvalidValue {
                    value: sink,
                    reason: "set_sink_mute failed or sink unknown".to_owned(),
                })
            }
        })
        .await
    }

    async fn set_sink_volume(&self, sink: &str, percent: u32) -> Result<(), BackendError> {
        if percent > 150 {
            return Err(BackendError::InvalidValue {
                value: percent.to_string(),
                reason: "volume must be 0–150%".to_owned(),
            });
        }
        let sink = sink.to_owned();
        Self::run_blocking(move |ctx, ml| {
            use libpulse_binding::mainloop::standard::IterateResult;
            use libpulse_binding::operation::State;
            use libpulse_binding::volume::{ChannelVolumes, Volume};
            use std::cell::Cell;
            use std::rc::Rc;

            let raw =
                (u64::from(percent) * u64::from(PULSE_NORM) / 100).min(u64::from(Volume::MAX.0));
            // The clamp above bounds the value to Volume::MAX.0 (u32 range).
            let target = Volume(u32::try_from(raw).unwrap_or(Volume::MAX.0));
            let mut volumes = ChannelVolumes::default();
            volumes.set(2, target);

            let ok = Rc::new(Cell::new(false));
            let done = Rc::new(Cell::new(false));
            let ok2 = ok.clone();
            let done2 = done.clone();
            let name = sink.clone();
            let op = ctx.introspect().set_sink_volume_by_name(
                &name,
                &volumes,
                Some(Box::new(move |success| {
                    ok2.set(success);
                    done2.set(true);
                })),
            );
            while !done.get() || op.get_state() == State::Running {
                match ml.iterate(false) {
                    IterateResult::Success(_) => {}
                    _ => break,
                }
            }
            if ok.get() {
                Ok(())
            } else {
                Err(BackendError::InvalidValue {
                    value: sink,
                    reason: "set_sink_volume failed or sink unknown".to_owned(),
                })
            }
        })
        .await
    }
}
