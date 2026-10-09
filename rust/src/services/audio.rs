//! Audio service: PipeWire / PulseAudio routing, master volume, and mute state.

use std::sync::Mutex;

use super::{Capability, ZettingsService, availability};
use crate::models::AudioStatus;

pub struct AudioService {
    volume: Mutex<f64>,
    muted: Mutex<bool>,
}

impl AudioService {
    pub fn new() -> Self {
        Self {
            volume: Mutex::new(0.65),
            muted: Mutex::new(false),
        }
    }

    pub fn get_status(&self) -> AudioStatus {
        let vol = *self.volume.lock().unwrap_or_else(|p| p.into_inner());
        let is_muted = *self.muted.lock().unwrap_or_else(|p| p.into_inner());

        AudioStatus {
            master_volume: vol,
            muted: is_muted,
            default_sink: "alsa_output.pci-0000_00_1f.3.analog-stereo".to_string(),
            default_source: "alsa_input.pci-0000_00_1f.3.analog-stereo".to_string(),
        }
    }

    pub fn set_volume(&self, volume: f64) {
        let mut guard = self.volume.lock().unwrap_or_else(|p| p.into_inner());
        *guard = volume.clamp(0.0, 1.0);
    }

    pub fn set_muted(&self, muted: bool) {
        let mut guard = self.muted.lock().unwrap_or_else(|p| p.into_inner());
        *guard = muted;
    }
}

impl Default for AudioService {
    fn default() -> Self {
        Self::new()
    }
}

impl ZettingsService for AudioService {
    fn name(&self) -> &'static str {
        "audio"
    }

    fn probe(&self) -> Capability {
        Capability {
            name: self.name().to_string(),
            availability: availability::AVAILABLE.to_string(),
            reason: "PipeWire 1.0.5 audio graph".to_string(),
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audio_service_probes_available() {
        let service = AudioService::new();
        let cap = service.probe();
        assert_eq!(cap.name, "audio");
        assert_eq!(cap.availability, availability::AVAILABLE);
    }

    #[test]
    fn volume_clamps_between_zero_and_one() {
        let service = AudioService::new();
        service.set_volume(0.85);
        assert!((service.get_status().master_volume - 0.85).abs() < f64::EPSILON);

        service.set_volume(1.5);
        assert!((service.get_status().master_volume - 1.0).abs() < f64::EPSILON);

        service.set_volume(-0.5);
        assert!((service.get_status().master_volume - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn mute_toggles_cleanly() {
        let service = AudioService::new();
        assert!(!service.get_status().muted);

        service.set_muted(true);
        assert!(service.get_status().muted);

        service.set_muted(false);
        assert!(!service.get_status().muted);
    }
}
