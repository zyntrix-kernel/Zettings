//! ZETTINGS application shell.
//!
//! Thin composition root: initializes logging, builds the backend adapter
//! set, and registers typed IPC commands. All command functions live here so
//! the `generate_handler!` registration and the `#[tauri::command]`
//! annotations stay adjacent (AGENTS.md §8).

#![allow(clippy::missing_docs_in_private_items)]

use zettings_backends::BackendSet;

/// Returns the built-in top-level settings graph snapshot served to the
/// shell at startup.
///
/// The seed graph is compiled into the binary; plugin-provided pages extend
/// it in later phases. This command is intentionally side-effect free so the
/// shell can call it during boot without backend availability.
#[tauri::command]
fn registry_snapshot() -> zettings_ipc::RegistrySnapshotDto {
    zettings_ipc::RegistrySnapshotDto::built_in()
}

/// Ranks built-in registry pages against a raw query using the weighted
/// search kernel (spec §9 weights). Pure and stateless; recency/pinned/freq
/// boosts attach with their persistence layer in Phase 7.
#[tauri::command]
fn search_registry(query: &str) -> zettings_ipc::SearchResponseDto {
    zettings_ipc::SearchResponseDto::built_in_query(query)
}

/// Adapter failures degrade into honest unavailable snapshots instead of
/// failing the whole command — the UI must be able to show *why* an area is
/// dark (spec §15, PLAN §13).
fn power_dto(
    result: &Result<zettings_backends::PowerProfileSnapshot, zettings_backends::BackendError>,
) -> zettings_ipc::PowerProfileSnapshotDto {
    result.as_ref().map_or_else(
        |e| zettings_ipc::PowerProfileSnapshotDto {
            capability: zettings_ipc::CapabilityStateDto::Unavailable {
                reason: e.to_string(),
            },
            available: Vec::new(),
            active: String::new(),
        },
        zettings_ipc::PowerProfileSnapshotDto::from,
    )
}

fn network_dto(
    result: &Result<zettings_backends::NetworkStatus, zettings_backends::BackendError>,
) -> zettings_ipc::NetworkStatusDto {
    result.as_ref().map_or_else(
        |e| zettings_ipc::NetworkStatusDto {
            capability: zettings_ipc::CapabilityStateDto::Unavailable {
                reason: e.to_string(),
            },
            networking_enabled: false,
            wireless_enabled: false,
            devices: Vec::new(),
        },
        zettings_ipc::NetworkStatusDto::from,
    )
}

fn session_dto(
    result: &Result<zettings_backends::SessionCapabilities, zettings_backends::BackendError>,
) -> zettings_ipc::SessionCapabilitiesDto {
    result.as_ref().map_or_else(
        |e| zettings_ipc::SessionCapabilitiesDto {
            capability: zettings_ipc::CapabilityStateDto::Unavailable {
                reason: e.to_string(),
            },
            can_power_off: "unknown".to_owned(),
            can_reboot: "unknown".to_owned(),
            can_suspend: "unknown".to_owned(),
        },
        zettings_ipc::SessionCapabilitiesDto::from,
    )
}

/// Aggregated system snapshot (power/network/session) with honest capability
/// states for each area.
#[tauri::command]
async fn system_snapshot(
    backends: tauri::State<'_, std::sync::Arc<BackendSet>>,
) -> Result<zettings_ipc::SystemSnapshotDto, String> {
    let power = backends.power.snapshot().await;
    let network = backends.network.status().await;
    let session = backends.session.capabilities().await;
    Ok(zettings_ipc::SystemSnapshotDto {
        power: power_dto(&power),
        network: network_dto(&network),
        session: session_dto(&session),
    })
}

/// Activates a power profile after validating it against the daemon's list.
#[tauri::command]
async fn set_power_profile(
    profile: &str,
    backends: tauri::State<'_, std::sync::Arc<BackendSet>>,
) -> Result<(), String> {
    backends
        .power
        .set_active_profile(profile)
        .await
        .map_err(|e| e.to_string())
}

/// Toggles the Wi-Fi radio via `NetworkManager` (NM enforces its own policy).
#[tauri::command]
async fn set_wireless_enabled(
    enabled: bool,
    backends: tauri::State<'_, std::sync::Arc<BackendSet>>,
) -> Result<(), String> {
    backends
        .network
        .set_wireless_enabled(enabled)
        .await
        .map_err(|e| e.to_string())
}

fn init_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}

fn main() {
    init_tracing();
    // Real adapters on Linux; deterministic mocks elsewhere or when the
    // `zettings-mock` feature forces them. Bus failures degrade to mocks so
    // the UI still boots with honest unavailable states.
    let force_mock = cfg!(feature = "zettings-mock") || !cfg!(target_os = "linux");
    let backends = match tauri::async_runtime::block_on(BackendSet::detect(force_mock)) {
        Ok(set) => set,
        Err(error) => {
            tracing::warn!(%error, "system bus unreachable; falling back to mock adapters");
            BackendSet::mocks()
        }
    };

    tauri::Builder::default()
        .manage(std::sync::Arc::new(backends))
        .invoke_handler(tauri::generate_handler![
            registry_snapshot,
            search_registry,
            system_snapshot,
            set_power_profile,
            set_wireless_enabled
        ])
        .run(tauri::generate_context!())
        .expect("zettings runtime failure");
}
