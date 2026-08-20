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
export type { SettingsEntry } from "./generated/search_settings_entry";
export type { SearchHit } from "./generated/search_hit";
export type { SearchRegisterEntriesRequest } from "./generated/search_register_entries_request";
export type { SearchRegisterEntriesResult } from "./generated/search_register_entries_result";
export type { SearchQueryRequest } from "./generated/search_query_request";
// Phase 7 read-side DTOs
export type { DisplayOutputDto } from "./generated/display_output";
export type { DisplayListOutputsResult } from "./generated/display_list_outputs_result";
export type { AudioStreamDto } from "./generated/audio_stream";
export type { AudioListStreamsResult } from "./generated/audio_list_streams_result";
export type { PairedDeviceDto } from "./generated/paired_device";
export type { BluetoothListPairedResult } from "./generated/bluetooth_list_paired_result";
export type { PowerActiveProfileResult } from "./generated/power_active_profile_result";
export type { BatteryStateDto } from "./generated/battery_state";
export type { PowerBatteriesResult } from "./generated/power_batteries_result";
export type { AccentPaletteDto } from "./generated/accent_palette";
export type { PaletteExtractRequest } from "./generated/palette_extract_request";
export type { PaletteExtractResult } from "./generated/palette_extract_result";
export type { PerfStatsDto } from "./generated/perf_stats";
