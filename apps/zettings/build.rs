//! Build script for the ZETTINGS shell: invokes Tauri's codegen for
//! capabilities, permissions, and platform resources.

fn main() {
    tauri_build::build();
}
