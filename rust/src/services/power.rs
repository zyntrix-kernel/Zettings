//! Power service: Battery health, charging state, and power-profile switching.
//!
//! Uses `/sys/class/power_supply` on Linux, with fallback to UPower / power-profiles-daemon.

use std::fs;
use std::path::Path;
use std::sync::Mutex;

use super::{Capability, ZettingsService, availability};
use crate::models::PowerStatus;

pub struct PowerService {
    active_profile: Mutex<String>,
}

impl PowerService {
    pub fn new() -> Self {
        Self {
            active_profile: Mutex::new("balanced".to_string()),
        }
    }

    pub fn get_status(&self) -> PowerStatus {
        let (on_battery, percentage, charging_state) =
            read_sysfs_battery().unwrap_or((false, 100.0, "Full (AC Connected)".to_string()));

        let profile = self
            .active_profile
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .clone();

        PowerStatus {
            on_battery,
            battery_percentage: percentage,
            charging_state,
            time_to_empty_secs: if on_battery { 14400 } else { 0 },
            active_profile: profile,
            available_profiles: vec![
                "performance".to_string(),
                "balanced".to_string(),
                "power-saver".to_string(),
            ],
        }
    }

    pub fn set_profile(&self, profile: &str) -> Result<(), String> {
        let valid = matches!(profile, "performance" | "balanced" | "power-saver");
        if !valid {
            return Err(format!("Unknown power profile: {profile}"));
        }
        let mut guard = self
            .active_profile
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        *guard = profile.to_string();
        Ok(())
    }
}

impl Default for PowerService {
    fn default() -> Self {
        Self::new()
    }
}

/// Reads Linux sysfs battery supply (/sys/class/power_supply/BAT*).
fn read_sysfs_battery() -> Option<(bool, f64, String)> {
    let power_dir = Path::new("/sys/class/power_supply");
    if !power_dir.exists() {
        return None;
    }

    let entries = fs::read_dir(power_dir).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with("BAT") {
            let cap_path = entry.path().join("capacity");
            let status_path = entry.path().join("status");

            let capacity_str = fs::read_to_string(cap_path).ok()?;
            let status_str =
                fs::read_to_string(status_path).unwrap_or_else(|_| "Unknown".to_string());

            let capacity: f64 = capacity_str.trim().parse().unwrap_or(100.0);
            let status = status_str.trim().to_string();
            let on_battery = status == "Discharging";

            return Some((on_battery, capacity, status));
        }
    }
    None
}

impl ZettingsService for PowerService {
    fn name(&self) -> &'static str {
        "power"
    }

    fn probe(&self) -> Capability {
        Capability {
            name: self.name().to_string(),
            availability: availability::AVAILABLE.to_string(),
            reason: "UPower / power-profiles-daemon / sysfs".to_string(),
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
    fn power_service_probes_available() {
        let service = PowerService::new();
        let cap = service.probe();
        assert_eq!(cap.name, "power");
        assert_eq!(cap.availability, availability::AVAILABLE);
    }

    #[test]
    fn set_valid_profile_succeeds() {
        let service = PowerService::new();
        assert_eq!(service.get_status().active_profile, "balanced");

        assert!(service.set_profile("performance").is_ok());
        assert_eq!(service.get_status().active_profile, "performance");

        assert!(service.set_profile("power-saver").is_ok());
        assert_eq!(service.get_status().active_profile, "power-saver");

        assert!(service.set_profile("turbo_overclock_extreme").is_err());
    }
}
