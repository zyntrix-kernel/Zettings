//! D-Bus IPC subsystem for `org.zyntrix.Zettings`.
//!
//! Provides the typed D-Bus service, client proxy, and strict input validation.

pub mod proxy;
pub mod service;
pub mod validation;

pub use proxy::*;
pub use service::Backend;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::ServiceRegistry;
    use crate::services::accounts::AccountsService;
    use crate::services::audio::AudioService;
    use crate::services::bluetooth::BluetoothService;
    use crate::services::network::NetworkService;
    use crate::services::power::PowerService;
    use crate::services::system_info::SystemInfoService;

    fn backend_with_all_services() -> Backend {
        let mut registry = ServiceRegistry::new();
        registry.register(SystemInfoService::new());
        registry.register(NetworkService::new());
        registry.register(BluetoothService::new());
        registry.register(PowerService::new());
        registry.register(AudioService::new());
        registry.register(AccountsService::new());
        Backend::new("9.9.9-test", registry)
    }

    #[test]
    fn ping_reports_version() {
        let backend = Backend::new("9.9.9-test", ServiceRegistry::new());
        assert_eq!(backend.ping(), "zettings-backend 9.9.9-test");
    }

    #[test]
    fn capabilities_include_registered_services() {
        let backend = backend_with_all_services();
        let capabilities = backend.get_capabilities();
        assert!(capabilities.iter().any(|c| c.name == "backend"));
        assert!(capabilities.iter().any(|c| c.name == "system"));
        assert!(capabilities.iter().any(|c| c.name == "network"));
        assert!(capabilities.iter().any(|c| c.name == "bluetooth"));
        assert!(capabilities.iter().any(|c| c.name == "power"));
        assert!(capabilities.iter().any(|c| c.name == "audio"));
        assert!(capabilities.iter().any(|c| c.name == "accounts"));
    }

    #[test]
    fn network_methods_work_through_backend() {
        let backend = backend_with_all_services();
        let status = backend.get_network_status().expect("must succeed");
        assert!(status.wifi_enabled);

        backend.set_wifi_enabled(false).expect("must succeed");
        let status = backend.get_network_status().expect("must succeed");
        assert!(!status.wifi_enabled);
    }

    #[test]
    fn bluetooth_methods_work_through_backend() {
        let backend = backend_with_all_services();
        let status = backend.get_bluetooth_status().expect("must succeed");
        assert!(status.enabled);

        backend.set_bluetooth_enabled(false).expect("must succeed");
        let status = backend.get_bluetooth_status().expect("must succeed");
        assert!(!status.enabled);
    }

    #[test]
    fn power_and_audio_methods_work_through_backend() {
        let backend = backend_with_all_services();
        let power = backend.get_power_status().expect("must succeed");
        assert_eq!(power.active_profile, "balanced");

        backend
            .set_power_profile("performance".to_string())
            .expect("must succeed");
        let power = backend.get_power_status().expect("must succeed");
        assert_eq!(power.active_profile, "performance");

        let audio = backend.get_audio_status().expect("must succeed");
        assert!(!audio.muted);

        backend.set_master_mute(true).expect("must succeed");
        backend.set_master_volume(0.42).expect("must succeed");
        let audio = backend.get_audio_status().expect("must succeed");
        assert!(audio.muted);
        assert!((audio.master_volume - 0.42).abs() < f64::EPSILON);
    }

    #[test]
    fn power_and_audio_reject_invalid_inputs() {
        let backend = backend_with_all_services();

        // Volume out of bounds
        let err = backend.set_master_volume(1.5).unwrap_err();
        assert!(matches!(err, zbus::fdo::Error::InvalidArgs(_)));

        // Volume negative
        let err = backend.set_master_volume(-0.1).unwrap_err();
        assert!(matches!(err, zbus::fdo::Error::InvalidArgs(_)));

        // Invalid power profile string
        let err = backend
            .set_power_profile("invalid; command".to_string())
            .unwrap_err();
        assert!(matches!(err, zbus::fdo::Error::InvalidArgs(_)));
    }

    #[test]
    fn user_accounts_work_through_backend() {
        let backend = backend_with_all_services();
        let users = backend.get_users().expect("must succeed");
        assert!(!users.is_empty());
    }

    #[test]
    fn unsupported_operation_fails_typed() {
        let backend = backend_with_all_services();
        let error = backend.unsupported_operation().unwrap_err();
        assert!(matches!(error, zbus::fdo::Error::NotSupported(_)));
        assert!(error.to_string().contains("unsupported"));
    }
}
