//! Typed input validation for incoming D-Bus IPC requests.
//!
//! Validates ranges, lengths, and formats before passing values down to system services,
//! adhering to the least-privilege and strict-input-validation mandate of AGENTS.md.

use crate::error::{BackendError, ErrorKind};

/// Validate master audio volume level. Must be a finite number in [0.0, 1.0].
pub fn validate_master_volume(volume: f64) -> Result<(), BackendError> {
    if !volume.is_finite() {
        return Err(BackendError::new(
            ErrorKind::InvalidInput,
            "volume must be a finite number",
        ));
    }
    if !(0.0..=1.0).contains(&volume) {
        return Err(BackendError::new(
            ErrorKind::InvalidInput,
            format!("volume {volume} is out of bounds; must be between 0.0 and 1.0"),
        ));
    }
    Ok(())
}

/// Validate power profile identifier.
pub fn validate_power_profile(profile: &str) -> Result<(), BackendError> {
    let trimmed = profile.trim();
    if trimmed.is_empty() {
        return Err(BackendError::new(
            ErrorKind::InvalidInput,
            "power profile identifier cannot be empty",
        ));
    }
    if trimmed.len() > 64 {
        return Err(BackendError::new(
            ErrorKind::InvalidInput,
            "power profile identifier exceeds maximum length (64)",
        ));
    }
    if !trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(BackendError::new(
            ErrorKind::InvalidInput,
            "power profile contains invalid characters; only alphanumeric, '-' and '_' allowed",
        ));
    }
    Ok(())
}

/// Validate Wi-Fi network SSID (1 to 32 octets per IEEE 802.11).
pub fn validate_wifi_ssid(ssid: &str) -> Result<(), BackendError> {
    if ssid.is_empty() {
        return Err(BackendError::new(
            ErrorKind::InvalidInput,
            "SSID cannot be empty",
        ));
    }
    if ssid.len() > 32 {
        return Err(BackendError::new(
            ErrorKind::InvalidInput,
            "SSID exceeds IEEE 802.11 maximum length of 32 bytes",
        ));
    }
    Ok(())
}

/// Validate username format according to POSIX / shadow standards.
pub fn validate_username(username: &str) -> Result<(), BackendError> {
    if username.is_empty() {
        return Err(BackendError::new(
            ErrorKind::InvalidInput,
            "username cannot be empty",
        ));
    }
    if username.len() > 32 {
        return Err(BackendError::new(
            ErrorKind::InvalidInput,
            "username exceeds maximum length of 32 characters",
        ));
    }
    let mut chars = username.chars();
    let first = chars.next().unwrap();
    if !(first.is_ascii_lowercase() || first == '_') {
        return Err(BackendError::new(
            ErrorKind::InvalidInput,
            "username must start with a lowercase letter or underscore",
        ));
    }
    if !chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-') {
        return Err(BackendError::new(
            ErrorKind::InvalidInput,
            "username contains invalid characters",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volume_validation() {
        assert!(validate_master_volume(0.0).is_ok());
        assert!(validate_master_volume(0.5).is_ok());
        assert!(validate_master_volume(1.0).is_ok());
        assert!(validate_master_volume(-0.01).is_err());
        assert!(validate_master_volume(1.01).is_err());
        assert!(validate_master_volume(f64::NAN).is_err());
        assert!(validate_master_volume(f64::INFINITY).is_err());
    }

    #[test]
    fn test_power_profile_validation() {
        assert!(validate_power_profile("balanced").is_ok());
        assert!(validate_power_profile("performance").is_ok());
        assert!(validate_power_profile("power-saver").is_ok());
        assert!(validate_power_profile("").is_err());
        assert!(validate_power_profile("   ").is_err());
        assert!(validate_power_profile("invalid profile; rm -rf").is_err());
    }

    #[test]
    fn test_wifi_ssid_validation() {
        assert!(validate_wifi_ssid("Zyntrix-5G").is_ok());
        assert!(validate_wifi_ssid("").is_err());
        assert!(validate_wifi_ssid(&"A".repeat(33)).is_err());
    }

    #[test]
    fn test_username_validation() {
        assert!(validate_username("zyntrix").is_ok());
        assert!(validate_username("_zyntrix").is_ok());
        assert!(validate_username("zyntrix-user").is_ok());
        assert!(validate_username("").is_err());
        assert!(validate_username("1admin").is_err());
        assert!(validate_username("Admin").is_err());
        assert!(validate_username("user;evil").is_err());
    }
}
