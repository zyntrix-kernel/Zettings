//! Shared D-Bus data model: bus identity, capability reporting, and subsystems.

pub mod audio;
pub mod bluetooth;
pub mod capability;
pub mod network;
pub mod power;
pub mod system;
pub mod user;

pub use audio::AudioStatus;
pub use bluetooth::{BluetoothDevice, BluetoothStatus};
pub use capability::{BUS_NAME, Capability, INTERFACE, OBJECT_PATH, availability};
pub use network::{NetworkStatus, WifiNetwork};
pub use power::PowerStatus;
pub use system::{DiskInfo, GpuInfo, SystemInfo};
pub use user::UserInfo;

#[cfg(test)]
mod tests {
    use super::*;
    use zvariant::Type;

    #[test]
    fn capability_json_round_trip() {
        let capability = Capability::available("backend");
        let json = serde_json::to_string(&capability).unwrap();
        let back: Capability = serde_json::from_str(&json).unwrap();
        assert_eq!(capability, back);
    }

    #[test]
    fn availability_constants_match_dart_names() {
        assert_eq!(availability::AVAILABLE, "available");
        assert_eq!(availability::PERMISSION_DENIED, "permissionDenied");
    }

    #[test]
    fn system_info_json_round_trip() {
        let info = SystemInfo {
            os_name: "Zyntrix OS".to_string(),
            os_version: "1.0".to_string(),
            kernel: "6.8.0".to_string(),
            hostname: "zyntrix".to_string(),
            desktop: "KDE".to_string(),
            session_type: "wayland".to_string(),
            cpu_brand: "Test CPU".to_string(),
            cpu_count: 8,
            cpu_freq_mhz: 3200,
            mem_total_bytes: 16_000_000_000,
            mem_avail_bytes: 8_000_000_000,
            uptime_secs: 3600,
            gpus: vec![GpuInfo {
                name: "Test GPU".to_string(),
                vendor: "Test Vendor".to_string(),
            }],
            disks: vec![DiskInfo {
                mount: "/".to_string(),
                total_bytes: 500_000_000_000,
                avail_bytes: 250_000_000_000,
                fs: "ext4".to_string(),
                removable: false,
            }],
        };
        let json = serde_json::to_string(&info).unwrap();
        let back: SystemInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(info, back);
    }

    #[test]
    fn network_status_json_round_trip() {
        let status = NetworkStatus {
            wifi_enabled: true,
            connected: true,
            active_ssid: "Zyntrix-5G".to_string(),
            ipv4_address: "192.168.1.142".to_string(),
            interface_name: "wlan0".to_string(),
            signal_strength: 92,
            wifi_networks: vec![WifiNetwork {
                ssid: "Zyntrix-5G".to_string(),
                signal_strength: 92,
                secured: true,
                security_type: "WPA3".to_string(),
            }],
        };
        let json = serde_json::to_string(&status).unwrap();
        let back: NetworkStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, back);
    }

    #[test]
    fn bluetooth_status_json_round_trip() {
        let status = BluetoothStatus {
            enabled: true,
            adapter_name: "Intel AX210".to_string(),
            discovering: false,
            connected_devices_count: 1,
            devices: vec![BluetoothDevice {
                name: "Sony WH-1000XM5".to_string(),
                address: "AA:BB:CC:DD:EE:FF".to_string(),
                connected: true,
                paired: true,
                battery_percentage: 78,
                device_type: "Headphones".to_string(),
            }],
        };
        let json = serde_json::to_string(&status).unwrap();
        let back: BluetoothStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, back);
    }

    #[test]
    fn power_and_audio_status_json_round_trip() {
        let power = PowerStatus {
            on_battery: false,
            battery_percentage: 100.0,
            charging_state: "Full".to_string(),
            time_to_empty_secs: 0,
            active_profile: "balanced".to_string(),
            available_profiles: vec!["performance".to_string(), "balanced".to_string()],
        };
        let json = serde_json::to_string(&power).unwrap();
        let back_power: PowerStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(power, back_power);

        let audio = AudioStatus {
            master_volume: 0.65,
            muted: false,
            default_sink: "alsa_output.pci".to_string(),
            default_source: "alsa_input.pci".to_string(),
        };
        let json = serde_json::to_string(&audio).unwrap();
        let back_audio: AudioStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(audio, back_audio);
    }

    #[test]
    fn user_info_json_round_trip() {
        let user = UserInfo {
            username: "zyntrix".to_string(),
            real_name: "Zyntrix Admin".to_string(),
            is_admin: true,
            home_dir: "/home/zyntrix".to_string(),
            shell: "/bin/bash".to_string(),
        };
        let json = serde_json::to_string(&user).unwrap();
        let back: UserInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(user, back);
    }

    #[test]
    fn model_dbus_signatures() {
        assert_eq!(<Capability as Type>::SIGNATURE.to_string(), "(sss)");
        assert_eq!(<WifiNetwork as Type>::SIGNATURE.to_string(), "(sybs)");
        assert_eq!(
            <NetworkStatus as Type>::SIGNATURE.to_string(),
            "(bbsssya(sybs))"
        );
        assert_eq!(<BluetoothDevice as Type>::SIGNATURE.to_string(), "(ssbbis)");
        assert_eq!(
            <BluetoothStatus as Type>::SIGNATURE.to_string(),
            "(bsbua(ssbbis))"
        );
        assert_eq!(<PowerStatus as Type>::SIGNATURE.to_string(), "(bdstsas)");
        assert_eq!(<AudioStatus as Type>::SIGNATURE.to_string(), "(dbss)");
        assert_eq!(<UserInfo as Type>::SIGNATURE.to_string(), "(ssbss)");
    }
}
