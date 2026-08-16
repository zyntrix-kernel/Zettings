// Re-export generated ts-rs bindings.
// Regenerate via:
//   cargo test -p zettings-ipc --features export-bindings
// The generated files live under ./generated/ and are checked into the repo
// so the frontend can be typechecked without a Rust toolchain present.

export type { Health } from "./generated/health";
export type { ModuleInfo } from "./generated/module_info";
export type { IpcError } from "./generated/ipc_error";
export type { SetHostnameRequest } from "./generated/set_hostname_request";
export type { SetHostnameResult } from "./generated/set_hostname_result";
