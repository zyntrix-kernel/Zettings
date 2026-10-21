//! System power, battery charge, and active power profile models.

use serde::{Deserialize, Serialize};
use zvariant::Type;

/// System power, battery, and power-profile status.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct PowerStatus {
    pub on_battery: bool,
    pub battery_percentage: f64,
    pub charging_state: String,
    pub time_to_empty_secs: u64,
    pub active_profile: String,
    pub available_profiles: Vec<String>,
}
