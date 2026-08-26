//! cxx-qt QObject bridges. One module per domain; the only crate permitted to
//! cross the Rust ↔ QML boundary (AGENTS.md §8).

mod app_info;

pub use app_info::AppInfoBridge;
