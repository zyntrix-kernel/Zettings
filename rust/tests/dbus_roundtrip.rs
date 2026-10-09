//! End-to-end round trip against a live daemon over a real session bus.
//!
//! Run under a bus: `dbus-run-session -- cargo test`. Without a bus the test
//! prints SKIP and passes — unit tests in `src/` cover the logic on all
//! platforms, this file covers the wire on machines with D-Bus.

use std::process::{Child, Command};
use std::time::Duration;
use zbus::Connection;
use zettings_backend::availability;
use zettings_backend::ipc::ZettingsProxyProxy;

fn session_bus_available() -> bool {
    std::env::var_os("DBUS_SESSION_BUS_ADDRESS").is_some()
}

struct DaemonGuard {
    child: Child,
}

impl DaemonGuard {
    fn spawn() -> Self {
        let exe = env!("CARGO_BIN_EXE_zettings-backend");
        let child = Command::new(exe)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("daemon binary must start");
        Self { child }
    }
}

impl Drop for DaemonGuard {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[tokio::test]
async fn ping_and_capabilities_round_trip() {
    if !session_bus_available() {
        eprintln!("SKIP dbus_round_trip: no session bus (run under dbus-run-session)");
        return;
    }

    let _daemon = DaemonGuard::spawn();

    // The daemon needs a moment to own its well-known name.
    let mut proxy = None;
    for _ in 0..50 {
        if let Ok(connection) = Connection::session().await
            && let Ok(candidate) = ZettingsProxyProxy::new(&connection).await
            && candidate.ping().await.is_ok()
        {
            proxy = Some(candidate);
            break;
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    let proxy = proxy.expect("daemon must answer Ping on the bus");

    let ping = proxy.ping().await.expect("Ping must succeed");
    assert!(
        ping.starts_with("zettings-backend "),
        "unexpected Ping reply: {ping}"
    );

    let version = proxy.get_version().await.expect("GetVersion must succeed");
    assert!(!version.is_empty());

    let capabilities = proxy
        .get_capabilities()
        .await
        .expect("GetCapabilities must succeed");
    let backend = capabilities
        .iter()
        .find(|c| c.name == "backend")
        .expect("builtin backend capability must be present");
    assert_eq!(backend.availability, availability::AVAILABLE);

    let system = capabilities
        .iter()
        .find(|c| c.name == "system")
        .expect("system capability must be present");
    assert_eq!(system.availability, availability::AVAILABLE);

    let info = proxy
        .get_system_info()
        .await
        .expect("GetSystemInfo must succeed");
    assert!(!info.kernel.is_empty(), "kernel must be reported");
    assert!(info.mem_total_bytes > 0, "memory total must be reported");
}
