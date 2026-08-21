//! ZETTINGS application shell.
//!
//! Thin composition root: initializes logging, builds application state, and
//! registers typed IPC commands. All command functions live here so the
//! `generate_handler!` registration and the `#[tauri::command]` annotations
//! stay adjacent (AGENTS.md §8).

#![allow(clippy::missing_docs_in_private_items)]

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

fn init_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}

fn main() {
    init_tracing();
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![registry_snapshot])
        .run(tauri::generate_context!())
        .expect("zettings runtime failure");
}
