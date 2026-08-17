//! Zettings Rust entry — Phase 1 payload is a minimal app that opens a
//! window pointing at the React webview and exposes a typed health command.
//! Phase 4 wires the real plugin loader, search index, and bus.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use tauri::Manager;
use tauri_plugin_decorum::WebviewWindowExt;
use tracing_subscriber::EnvFilter;
use zettings_ipc::{
    Health, ModuleInfo, SearchQueryRequest, SearchRegisterEntriesRequest,
    SearchRegisterEntriesResult,
};
use zettings_search::SearchHit;

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

/// Zettings application entry point.
///
/// Initializes the tracing subscriber, registers the Tauri plugins
/// (log, shell, window-state, deep-link, decorum), mounts the Phase 1
/// IPC command surface, and opens the main webview window pointing at
/// the React 19 frontend.
fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

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
            zettings_search_query
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
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("zettings: tauri::run failed");
}
