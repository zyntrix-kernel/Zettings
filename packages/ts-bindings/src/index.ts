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
export type { DisplayModeDto } from "./generated/display_mode";
export type { DisplayApplyModeRequest } from "./generated/display_apply_mode_request";
export type { DisplayApplyModeResult } from "./generated/display_apply_mode_result";
export type { AudioSetVolumeRequest } from "./generated/audio_set_volume_request";
export type { AudioSetVolumeResult } from "./generated/audio_set_volume_result";
export type { AccessPointDto } from "./generated/access_point";
export type { NetworkScanWifiResult } from "./generated/network_scan_wifi_result";
export type { PowerProfileDto } from "./generated/power_profile";
export type { PowerSetProfileRequest } from "./generated/power_set_profile_request";
export type { PowerSetProfileResult } from "./generated/power_set_profile_result";
