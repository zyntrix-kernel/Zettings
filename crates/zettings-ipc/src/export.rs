//! Binding export test (run via `pnpm bindings` / `cargo test -p zettings-ipc
//! --features export-bindings`).
//!
//! Writes generated TypeScript into the committed bindings package. The
//! absolute output directory is derived from `CARGO_MANIFEST_DIR` so the test
//! works from any checkout location.

use std::path::Path;
use ts_rs::TS as _;

#[test]
fn export_typescript_bindings() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let out_dir = Path::new(manifest_dir).join("../../packages/ts-bindings/src/generated");
    std::fs::create_dir_all(&out_dir).expect("create generated dir");
    // SAFETY: no other threads read this variable; ts-rs consumes it below.
    unsafe {
        std::env::set_var("TS_RS_EXPORT_DIR", &out_dir);
    }
    let config = ts_rs::Config::from_env();
    crate::CategorySummaryDto::export(&config).expect("export CategorySummaryDto");
    crate::RegistrySnapshotDto::export(&config).expect("export RegistrySnapshotDto");
}
