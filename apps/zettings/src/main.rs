//! Zettings Rust entry — Phase 1 payload is a minimal app that opens a
//! window pointing at the React webview and exposes a typed health command.
//! Phase 4 wires the real plugin loader, search index, and bus.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::sync::OnceLock;
use std::time::Instant;

use tauri::Manager;
use tauri_plugin_decorum::WebviewWindowExt;
use tracing_subscriber::EnvFilter;
use zettings_bus::Bus;
use zettings_bus::events::{AudioVolume, DisplayReplug, LinkState, PowerState};
use zettings_ipc::{
    AudioListStreamsResult, BluetoothListPairedResult, DisplayListOutputsResult, Health,
    ModuleInfo, PaletteExtractRequest, PaletteExtractResult, PerfStatsDto,
    PowerActiveProfileResult, PowerBatteriesResult, SearchQueryRequest,
    SearchRegisterEntriesRequest, SearchRegisterEntriesResult,
};
use zettings_search::SearchHit;

/// Process boot anchor. Captured on the first line of `main()` before the
/// tracing subscriber initializes so `zettings_perf_stats` can report true
/// backend uptime and startup latency (Phase 9 launch-time audit).
static PROCESS_START: OnceLock<Instant> = OnceLock::new();

/// Set at the end of the Tauri `setup` closure (after the titlebar overlay).
/// The gap between `PROCESS_START` and this instant is the backend cold-start
/// window that `zettings_perf_stats` reports as `backend_startup_ms`.
static SETUP_DONE: OnceLock<Instant> = OnceLock::new();

/// `zettings_perf_stats` — Phase 9 process-telemetry surface for the frontend
/// `PerfMonitor` overlay. Reports backend uptime, startup latency, resident set
/// size (Linux only; `None` on the Windows mock dev loop), and the mock flag.
/// The overlay samples this on an interval to validate the PLAN.md Phase 9
/// budgets (`<500ms` cold start, `<150MB` idle RAM).
#[tauri::command]
fn zettings_perf_stats() -> PerfStatsDto {
    let start = PROCESS_START.get().copied().unwrap_or_else(Instant::now);
    let startup = SETUP_DONE.get().map_or(0, |done| {
        u32::try_from(done.duration_since(start).as_millis()).unwrap_or(u32::MAX)
    });
    let uptime = u32::try_from(start.elapsed().as_millis()).unwrap_or(u32::MAX);
    zettings_ipc::perf_stats(uptime, startup)
}

/// `zettings_health` — frontend ping that returns backend version + mock flag.
#[tauri::command]
fn zettings_health() -> Health {
    Health {
        version: env!("CARGO_PKG_VERSION").to_string(),
        is_mock: cfg!(feature = "zettings-mock"),
    }
}

/// `zettings_modules` — returns mounted modules. Phase 1: empty placeholder.
#[tauri::command]
fn zettings_modules() -> Vec<ModuleInfo> {
    Vec::new()
}

/// `zettings_search_register` — bulk-upserts settings entries into the global
/// in-process Tantivy index. Modules call this once on mount so their
/// sub-pages, sub-menus, and toggle controls become discoverable from the
/// Spotlight modal (Phase 6.3). The search index is feature-flag-free — it
/// touches no system resources — so this command is safe to invoke on the
/// Windows dev loop and on real Linux targets without `PolicyKit` authorization.
#[tauri::command]
fn zettings_search_register(
    request: SearchRegisterEntriesRequest,
) -> Result<SearchRegisterEntriesResult, zettings_ipc::IpcError> {
    zettings_ipc::search_register_entries(request)
}

/// `zettings_search_query` — fuzzy + typo-tolerant search over the Spotlight
/// index. The frontend Spotlight modal debounces the user input and invokes
/// this command via `@tauri-apps/api/core`'s `invoke`; `keepPreviousData`
/// keeps stale hits visible during the IPC round-trip to preserve the
/// <5ms perceived-latency budget (see `apps/zettings/web` Spotlight modal).
#[tauri::command]
fn zettings_search_query(
    request: SearchQueryRequest,
) -> Result<Vec<SearchHit>, zettings_ipc::IpcError> {
    zettings_ipc::search_query(request)
}

/// `zettings_display_list_outputs` — Phase 7 read-side surface returning the
/// connected monitor outputs and their available modes. Mock backend returns
/// two sample outputs; the real `KScreen` `DBus` integration lands in Phase 5.
#[tauri::command]
fn zettings_display_list_outputs() -> Result<DisplayListOutputsResult, zettings_ipc::IpcError> {
    zettings_ipc::display_list_outputs()
}

/// `zettings_audio_list_streams` — Phase 7 read-side surface returning the
/// per-stream volume/mute state plus human-readable labels. Mock backend
/// returns three sample streams (Master, Aurora, Voice Call); real `PipeWire`
/// integration lands in Phase 5.
#[tauri::command]
fn zettings_audio_list_streams() -> Result<AudioListStreamsResult, zettings_ipc::IpcError> {
    zettings_ipc::audio_list_streams()
}

/// `zettings_bluetooth_list_paired` — Phase 7 read-side surface returning the
/// paired Bluetooth/peripheral devices with battery indicators. Mock backend
/// returns three sample devices; real `BlueZ` zbus integration lands in Phase 5.
#[tauri::command]
fn zettings_bluetooth_list_paired() -> Result<BluetoothListPairedResult, zettings_ipc::IpcError> {
    zettings_ipc::bluetooth_list_paired()
}

/// `zettings_power_active_profile` — Phase 7 read-side surface returning the
/// currently active power profile (`Balanced`/`Performance`/`PowerSaver`).
/// Mock backend defaults to `Balanced`; real `power-profiles-daemon` `DBus`
/// integration lands in Phase 5.
#[tauri::command]
fn zettings_power_active_profile() -> Result<PowerActiveProfileResult, zettings_ipc::IpcError> {
    zettings_ipc::power_active_profile()
}

/// `zettings_power_batteries` — Phase 7 read-side surface returning the
/// battery state (percentage + charging flag) for each UPower-registered
/// battery. Mock backend returns a single sample battery; real `UPower` zbus
/// integration lands in Phase 5.
#[tauri::command]
fn zettings_power_batteries() -> Result<PowerBatteriesResult, zettings_ipc::IpcError> {
    zettings_ipc::power_batteries()
}

/// `zettings_palette_extract` — Phase 7 accent-palette extraction surface.
/// Pure in-process operation (image decode + median-cut quantizer) — touches
/// no system resources and requires no `PolicyKit` authorization. The frontend
/// Personalization panel invokes this with the wallpaper bytes and applies the
/// returned accent/secondary colors to the ZDL theme tokens.
#[tauri::command]
fn zettings_palette_extract(
    request: PaletteExtractRequest,
) -> Result<PaletteExtractResult, zettings_ipc::IpcError> {
    zettings_ipc::palette_extract(request)
}

/// Headless daemon entry point for `zettings-daemon.service`.
///
/// Runs the Zettings backend without a webview window. It warms the in-process
/// Tantivy search index and subscribes to the typed broadcast bus so the
/// daemon process holds live state-sync subscriptions (`LinkState`,
/// `AudioVolume`, `DisplayReplug`, `PowerState`) for the lifetime of the user session. This gives
/// the GUI a shared process that owns the search corpus and bus before the
/// window mounts, so a subsequently launched `zettings` GUI instance starts
/// with an already-warmed index.
///
/// Blocks until SIGTERM/SIGINT (systemd `KillSignal=SIGINT`), then exits 0.
fn run_daemon() {
    tracing::info!(
        event = "daemon.start",
        "zettings daemon starting (headless)"
    );
    // Warm the search index synchronously so the first Spotlight query in a
    // later GUI process never blocks on index construction. The index lives
    // for the whole daemon lifetime (future cross-process index hosting).
    let _index = zettings_search::Index::new();
    tracing::info!(event = "daemon.index.warm", "in-memory search index ready");
    let bus = Bus::new();
    let mut link_rx = bus.subscribe::<LinkState>();
    let mut audio_rx = bus.subscribe::<AudioVolume>();
    let mut display_rx = bus.subscribe::<DisplayReplug>();
    let mut power_rx = bus.subscribe::<PowerState>();

    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(error) => {
            tracing::error!(event = "daemon.runtime.failed", error = %error, "failed to start tokio runtime");
            std::process::exit(1);
        }
    };
    runtime.block_on(async {
        loop {
            tokio::select! {
                Ok(state) = link_rx.recv() => {
                    tracing::debug!(event = "bus.link", interface = state.interface_index, up = state.link_up);
                }
                Ok(state) = audio_rx.recv() => {
                    tracing::debug!(event = "bus.audio", stream = state.stream_id, volume = state.volume, muted = state.muted);
                }
                Ok(state) = display_rx.recv() => {
                    tracing::debug!(event = "bus.display", output = state.output_id, connected = state.connected);
                }
                Ok(state) = power_rx.recv() => {
                    tracing::debug!(event = "bus.power", device = state.device_index, percent = state.percentage, charging = state.charging);
                }
                _ = tokio::signal::ctrl_c() => {
                    tracing::info!(event = "daemon.stop", "zettings daemon stopping");
                    break;
                }
            }
        }
    });
}

/// Zettings application entry point.
///
/// Initializes the tracing subscriber, then either enters headless daemon mode
/// (`zettings --daemon`, launched by `zettings-daemon.service`) or starts the
/// full GUI runtime: registers the Tauri plugins (log, shell, window-state,
/// deep-link, decorum), mounts the IPC command surface, and opens the main
/// webview window pointing at the React 19 frontend.
///
/// Phase 9 launch-time instrumentation: `PROCESS_START` is captured before
/// anything else runs and `SETUP_DONE` right after the `setup` closure
/// finishes. `zettings_perf_stats` turns that gap into `backend_startup_ms`
/// for the `PerfMonitor` overlay, and `tracing::info!` emits the same numbers
/// to the log channel for the WSL2 audit commands in
/// `docs/performance/audit.md`.
fn main() {
    PROCESS_START.get_or_init(Instant::now);
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    tracing::info!(event = "main.enter", "zettings backend starting");

    // `zettings-daemon.service` launches `zettings --daemon` headless. The
    // service owns the search index warm-up + bus subscriptions; the GUI path
    // below is skipped entirely so no webview/polkit surface is created.
    if std::env::args().any(|arg| arg == "--daemon") {
        run_daemon();
        std::process::exit(0);
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_decorum::init())
        .invoke_handler(tauri::generate_handler![
            zettings_health,
            zettings_modules,
            zettings_search_register,
            zettings_search_query,
            zettings_display_list_outputs,
            zettings_audio_list_streams,
            zettings_bluetooth_list_paired,
            zettings_power_active_profile,
            zettings_power_batteries,
            zettings_palette_extract,
            zettings_perf_stats,
        ])
        .setup(|app| {
            let window = app.get_webview_window("main").expect("main window");
            // Install the decorum titlebar overlay: removes the native frame
            // (decorations:false is also set in tauri.conf.json), creates the
            // drag region, and injects platform-native window controls into
            // [data-tauri-decorum-tb]. The frontend ShellFrame component
            // provides that container and a drag-region element so the
            // user can drag the window; decorum auto-appends the buttons.
            window.create_overlay_titlebar()?;
            SETUP_DONE.get_or_init(Instant::now);
            let startup_ms = SETUP_DONE
                .get()
                .expect("SETUP_DONE set above")
                .duration_since(*PROCESS_START.get().expect("PROCESS_START set in main"))
                .as_millis();
            tracing::info!(
                event = "setup.complete",
                backend_startup_ms = u32::try_from(startup_ms).unwrap_or(u32::MAX),
                "zettings backend ready"
            );
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("zettings: tauri::run failed");
}
