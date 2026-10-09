//! `zettings-backend`: D-Bus daemon fronting Linux/KDE system services.
//!
//! Serves `org.zyntrix.Zettings` on the session bus. Never run as root: the
//! daemon holds no privileges itself; privileged operations authorize per call
//! through PolicyKit (from Phase 9 onward).

use std::{error::Error, future::pending};
use zbus::connection;
use zettings_backend::{
    AccountsService, AudioService, BUS_NAME, Backend, BluetoothService, NetworkService,
    OBJECT_PATH, PowerService, ServiceRegistry, SystemInfoService,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let version = env!("CARGO_PKG_VERSION");
    let mut registry = ServiceRegistry::new();
    registry.register(SystemInfoService::new());
    registry.register(NetworkService::new());
    registry.register(BluetoothService::new());
    registry.register(PowerService::new());
    registry.register(AudioService::new());
    registry.register(AccountsService::new());
    let backend = Backend::new(version, registry);

    let _connection = connection::Builder::session()?
        .name(BUS_NAME)?
        .serve_at(OBJECT_PATH, backend)?
        .build()
        .await?;

    eprintln!("zettings-backend {version} serving {BUS_NAME} ({OBJECT_PATH})");
    pending::<()>().await;
    // `pending` never resolves; this satisfies the `Result` return type.
    Ok(())
}
