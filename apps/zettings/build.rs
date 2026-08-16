//! Build script for the Zettings Tauri shell. Delegates to `tauri-build`
//! which renders `tauri.conf.json` (located at the crate root) into the
//! `cargo:` directives the `tauri::generate_context!()` macro needs.

fn main() {
    tauri_build::build();
}
