//! Zettings system backend daemon.
//!
//! Typed D-Bus service (`org.zyntrix.Zettings`) fronting Linux/KDE system
//! APIs. Flutter talks to this daemon over the session bus; the daemon owns
//! every privileged or system-facing operation. No generic command execution
//! exists anywhere in this boundary.

mod error;
pub mod ipc;
mod models;
pub mod services;

pub use error::{BackendError, ErrorKind};
pub use ipc::Backend;
pub use models::{
    AudioStatus, BUS_NAME, BluetoothDevice, BluetoothStatus, Capability, DiskInfo, GpuInfo,
    INTERFACE, NetworkStatus, OBJECT_PATH, PowerStatus, SystemInfo, UserInfo, WifiNetwork,
    availability,
};
pub use services::accounts::AccountsService;
pub use services::audio::AudioService;
pub use services::bluetooth::BluetoothService;
pub use services::network::NetworkService;
pub use services::power::PowerService;
pub use services::system_info::SystemInfoService;
pub use services::{ServiceRegistry, ZettingsService};
