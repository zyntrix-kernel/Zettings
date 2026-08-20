//! `zettings-ipc` — Tauri v2 Command Surface and Typed Binding Exporter.
//!
//! Exposes async Tauri commands to the webview shell and enforces `PolicyKit`
//! authorization checks via [`zettings_polkit::Authorizer`] prior to executing
//! privileged actions. All payload types are exported to TypeScript via `ts-rs`
//! into `packages/ts-bindings/src/generated/` so the React frontend invokes the
//! backend with strict typing — hand-typed duplicate frontend payloads are
//! forbidden (see `AGENTS.md`).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;
#[cfg(feature = "zettings-mock")]
use zettings_bus::Bus;
#[cfg(not(feature = "zettings-mock"))]
use zettings_polkit::ActionId;
#[cfg(feature = "zettings-mock")]
use zettings_polkit::{ActionId, Authorization, MockAuthorizer};
// Error + DTO types are always exposed from domain crates (not cfg-gated).
use zettings_audio::AudioError;
use zettings_display::DisplayError;
use zettings_network::{AccessPoint, NetworkError};
use zettings_power::{PowerError, Profile};
// Search types are always available (the Tantivy in-memory index builds on
// every Tauri target — no `zbus` dependency). The DTO shapes come from the
// domain crate directly so the frontend `@zettings/bindings` barrel imports
// the same `search_settings_entry` / `search_hit` files regenerated here.
use zettings_search::{SearchError, SearchHit, SettingsEntry};
// Backend trait impls are gated by the zettings-mock feature on the Windows
// dev loop (mock state machines) and by `target_os = "linux"` on real targets.
#[cfg(feature = "zettings-mock")]
use zettings_audio::Backend as AudioBackend;
#[cfg(feature = "zettings-mock")]
use zettings_display::Backend as DisplayBackend;
#[cfg(feature = "zettings-mock")]
use zettings_display::{DisplayMode, OutputConfig, OutputId};
#[cfg(feature = "zettings-mock")]
use zettings_network::Backend as NetworkBackend;
#[cfg(feature = "zettings-mock")]
use zettings_power::Backend as PowerBackend;
// `StreamId`/`Volume` are only used by the mock audio command path.
#[cfg(feature = "zettings-mock")]
use zettings_audio::{StreamId, Volume};

/// Backend health payload returned by the `zettings_health` command.
///
/// The frontend uses `is_mock` to surface a "mock backend" badge during Windows
/// frontend iteration and to gate UI affordances that require real Linux services.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "health.ts")]
pub struct Health {
    /// Zettings backend version (matches `CARGO_PKG_VERSION`).
    pub version: String,
    /// `true` when running against the `zettings-mock` state-machine backend
    /// (Windows dev loop). `false` on the real WSL2/Linux target.
    pub is_mock: bool,
}

/// Frontend-facing module descriptor returned by the `zettings_modules` command.
///
/// Mirrors the subset of [`zettings_core::manifest::ModuleManifest`] the sidebar
/// navigation needs. The full manifest stays backend-side.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "module_info.ts")]
pub struct ModuleInfo {
    /// Globally unique module id, e.g. `org.zyntrix.zettings.display`.
    pub id: String,
    /// Human-readable name shown in the sidebar.
    pub name: String,
    /// Lucide icon name, e.g. `monitor`.
    pub icon: String,
    /// Frontend route, e.g. `/display`.
    pub route: String,
}

/// Errors surfacing from IPC command execution. Serialized to the frontend as
/// a tagged enum so the React layer can pattern-match on `type`.
#[derive(Debug, Error, Serialize, Deserialize, TS)]
#[serde(tag = "type", content = "message")]
#[ts(export, export_to = "ipc_error.ts")]
pub enum IpcError {
    /// `PolicyKit` authorization was denied or dismissed by the user.
    #[error("PolicyKit authorization denied: {0}")]
    PolkitDenied(String),
    /// The underlying system service (`DBus`/`PipeWire`/etc.) was unreachable.
    #[error("System service unavailable: {0}")]
    ServiceUnavailable(String),
    /// The request payload failed validation.
    #[error("Invalid request payload: {0}")]
    InvalidPayload(String),
    /// An internal execution error not covered by a more specific variant.
    #[error("Internal execution error: {0}")]
    Internal(String),
}

impl From<DisplayError> for IpcError {
    fn from(e: DisplayError) -> Self {
        match e {
            DisplayError::OutputNotFound(msg) => {
                Self::InvalidPayload(format!("display output not found: {msg}"))
            }
            DisplayError::ModeNotAvailable {
                output,
                width,
                height,
                refresh_hz,
            } => Self::InvalidPayload(format!(
                "mode not available for output {output}: {width}x{height}@{refresh_hz}Hz"
            )),
            DisplayError::ServiceUnavailable(msg) => Self::ServiceUnavailable(msg),
        }
    }
}

impl From<AudioError> for IpcError {
    fn from(e: AudioError) -> Self {
        match e {
            AudioError::StreamNotFound(id) => {
                Self::InvalidPayload(format!("audio stream not found: {id}"))
            }
            AudioError::ServiceUnavailable(msg) => Self::ServiceUnavailable(msg),
        }
    }
}

impl From<NetworkError> for IpcError {
    fn from(e: NetworkError) -> Self {
        match e {
            NetworkError::InvalidHostname(msg) => {
                Self::InvalidPayload(format!("invalid hostname: {msg}"))
            }
            NetworkError::ServiceUnavailable(msg) => Self::ServiceUnavailable(msg),
        }
    }
}

impl From<PowerError> for IpcError {
    fn from(e: PowerError) -> Self {
        match e {
            PowerError::ProfileNotAvailable(p) => {
                Self::InvalidPayload(format!("power profile not available: {p:?}"))
            }
            PowerError::ServiceUnavailable(msg) => Self::ServiceUnavailable(msg),
        }
    }
}

impl From<SearchError> for IpcError {
    fn from(e: SearchError) -> Self {
        match e {
            SearchError::Query(msg) => {
                Self::InvalidPayload(format!("search query parse error: {msg}"))
            }
            // The Tantivy RAMDirectory cannot actually fail to open, but we
            // surface any underlying error as a generic service-unavailable so
            // the frontend has a deterministic error variant to render.
            SearchError::Open(msg) => Self::ServiceUnavailable(msg),
        }
    }
}

/// Request payload to modify the system hostname.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "set_hostname_request.ts")]
pub struct SetHostnameRequest {
    /// The new hostname. Validated by `NetworkManager` on the backend.
    pub hostname: String,
}

/// Response payload for a hostname mutation.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "set_hostname_result.ts")]
pub struct SetHostnameResult {
    /// Whether the hostname was successfully applied.
    pub success: bool,
    /// The active hostname after the operation (echo-back for UI confirmation).
    pub active_hostname: String,
}

/// Display mode DTO crossing the IPC boundary (resolution + refresh).
/// Mirrors [`zettings_display::DisplayMode`] with `ts-rs` derives.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, TS)]
#[ts(export, export_to = "display_mode.ts")]
pub struct DisplayModeDto {
    /// Width in physical pixels.
    pub width: u32,
    /// Height in physical pixels.
    pub height: u32,
    /// Vertical refresh rate in Hz.
    pub refresh_hz: f32,
}

/// Power profile DTO mirroring [`zettings_power::Profile`].
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export, export_to = "power_profile.ts")]
pub enum PowerProfileDto {
    /// Balanced (default).
    Balanced,
    /// Performance (high power draw, fans may spin up).
    Performance,
    /// Power-saver (throttle CPU/GPU, dim backlight).
    PowerSaver,
}

impl From<PowerProfileDto> for Profile {
    fn from(dto: PowerProfileDto) -> Self {
        match dto {
            PowerProfileDto::Balanced => Self::Balanced,
            PowerProfileDto::Performance => Self::Performance,
            PowerProfileDto::PowerSaver => Self::PowerSaver,
        }
    }
}

/// Request payload to apply a display mode to an output.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "display_apply_mode_request.ts")]
pub struct DisplayApplyModeRequest {
    /// `KScreen` output id, e.g. `HDMI-A-1`.
    pub output: String,
    /// Target resolution + refresh.
    pub mode: DisplayModeDto,
    /// Logical scale factor (1.0 = native, 2.0 = 200% `HiDPI`).
    pub scale: f32,
}

/// Response payload for a display mode application.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "display_apply_mode_result.ts")]
pub struct DisplayApplyModeResult {
    /// Whether the mode was successfully applied.
    pub applied: bool,
}

/// Request payload to set per-stream audio volume.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "audio_set_volume_request.ts")]
pub struct AudioSetVolumeRequest {
    /// `PulseAudio`/`PipeWire` stream id.
    pub stream_id: u32,
    /// Normalized volume in `[0.0, 1.0]`.
    pub volume: f32,
    /// `true` to mute the stream.
    pub muted: bool,
}

/// Response payload for an audio volume mutation.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "audio_set_volume_result.ts")]
pub struct AudioSetVolumeResult {
    /// Whether the volume change was successfully applied.
    pub applied: bool,
}

/// Frontend-facing Wi-Fi access point descriptor. SSID is exposed as a string
/// (UTF-8 lossy) since the fixed 32-byte raw array in [`AccessPoint`] is not
/// ergonomic over the IPC boundary.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "access_point.ts")]
pub struct AccessPointDto {
    /// SSID decoded lossy-UTF-8. Hidden networks surface as an empty string.
    pub ssid: String,
    /// Signal strength in dBm, typically in `[-100, 0]`.
    pub signal_dbm: i8,
    /// `true` when the AP requires authentication.
    pub secured: bool,
}

impl From<AccessPoint> for AccessPointDto {
    fn from(ap: AccessPoint) -> Self {
        Self {
            ssid: String::from_utf8_lossy(ap.ssid_bytes()).into_owned(),
            signal_dbm: ap.signal_dbm,
            secured: ap.secured,
        }
    }
}

/// Response payload for a Wi-Fi scan.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "network_scan_wifi_result.ts")]
pub struct NetworkScanWifiResult {
    /// Discovered access points, sorted by descending signal strength.
    pub access_points: Vec<AccessPointDto>,
}

/// Request payload to switch the active power profile.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "power_set_profile_request.ts")]
pub struct PowerSetProfileRequest {
    /// Target profile.
    pub profile: PowerProfileDto,
}

/// Response payload for a power profile switch.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "power_set_profile_result.ts")]
pub struct PowerSetProfileResult {
    /// Whether the profile was successfully applied.
    pub applied: bool,
}

/// Request payload for batch-registering settings entries with the search
/// index. Each module emits one of these on mount so its settings surface in
/// the global Spotlight modal.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "search_register_entries_request.ts")]
pub struct SearchRegisterEntriesRequest {
    /// Entries to upsert. Idempotent on `SettingsEntry::id` so re-registration
    /// by hot-reloading modules never duplicates documents.
    pub entries: Vec<SettingsEntry>,
}

/// Response payload for a batch registration.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "search_register_entries_result.ts")]
pub struct SearchRegisterEntriesResult {
    /// Number of entries actually written to the index.
    pub registered: usize,
}

/// Request payload for a Spotlight search query.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "search_query_request.ts")]
pub struct SearchQueryRequest {
    /// Free-form user query. Tantivy tokenizes and fuzzy-matches.
    pub query: String,
}

// ===========================================================================
// Phase 7 read-side DTOs — exposed so the domain panels can render live state
// without bundling syscall-bound writes behind every render. Each panel reads
// its data set on mount via `@tanstack/react-query` and refreshes on focus /
// visibility change. The `zettings-mock` feature gates the backends that back
// these reads on the Windows dev loop; the real `zbus`/`PipeWire`/`BlueZ`
// paths land in Phase 5+ and surface `IpcError::ServiceUnavailable` until then.
// ===========================================================================

/// Display output descriptor returned by `display_list_outputs`. One entry
/// per connected `KScreen` output plus the modes that output advertises.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "display_output.ts")]
pub struct DisplayOutputDto {
    /// `KScreen` output id, e.g. `HDMI-A-1`.
    pub output_id: String,
    /// Modes advertised by this output (resolution + refresh).
    pub modes: Vec<DisplayModeDto>,
}

/// Response payload for `display_list_outputs`.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "display_list_outputs_result.ts")]
pub struct DisplayListOutputsResult {
    /// Connected outputs, in stable `KScreen` order.
    pub outputs: Vec<DisplayOutputDto>,
}

/// Audio stream descriptor for the mixer panel. Combines identity, label,
/// and live volume state so the panel renders one IPC round-trip.
///
/// `label_id` is a stable discriminator the frontend translates into a
/// localized display string (kept Rust-side so the schema is explicit and
/// unit-testable without bundling localizations here).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, TS)]
#[ts(export, export_to = "audio_stream.ts")]
pub struct AudioStreamDto {
    /// Application or sink id matching the audio backend registry.
    pub stream_id: u32,
    /// Stable label discriminator (0 = Master, 1 = Aurora, 2 = Voice Call).
    pub label_id: u8,
    /// Normalized volume in `[0.0, 1.0]`.
    pub volume: f32,
    /// `true` when the stream is muted.
    pub muted: bool,
}

/// Response payload for `audio_list_streams`.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "audio_list_streams_result.ts")]
pub struct AudioListStreamsResult {
    /// Active streams, sorted by `stream_id`.
    pub streams: Vec<AudioStreamDto>,
}

/// Paired `BlueZ` device descriptor returned by `bluetooth_list_paired`.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "paired_device.ts")]
pub struct PairedDeviceDto {
    /// `BlueZ` device address (object-path-leaf identifier).
    pub address: String,
    /// Human-readable device name.
    pub name: String,
    /// `true` when the device is currently connected (not just paired).
    pub connected: bool,
    /// Battery charge in `[0, 100]`. `None` when the device does not advertise
    /// a Battery1 interface — the UI then hides the battery glyph rather than
    /// showing a 0% "empty" affordance (ui-ux-pro-max Accessibility: never
    /// imply low battery purely by an absent number).
    pub battery_percent: Option<u8>,
    /// Major device class (`Audio`, `Peripheral`, `Phone`, ...).
    pub device_class: String,
}

impl From<zettings_network::PairedDevice> for PairedDeviceDto {
    fn from(d: zettings_network::PairedDevice) -> Self {
        Self {
            address: d.address,
            name: d.name,
            connected: d.connected,
            battery_percent: d.battery_percent,
            device_class: d.device_class,
        }
    }
}

/// Response payload for `bluetooth_list_paired`.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bluetooth_list_paired_result.ts")]
pub struct BluetoothListPairedResult {
    /// Paired devices, in stable `BlueZ` enumeration order.
    pub devices: Vec<PairedDeviceDto>,
}

/// Response payload for `power_active_profile`. The current power profile
/// is a small discriminator; we wrap it in a result struct so frontend IPC
/// callers can switch on a stable response shape rather than the bare enum.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
#[ts(export, export_to = "power_active_profile_result.ts")]
pub struct PowerActiveProfileResult {
    /// The currently-active `power-profiles-daemon` profile.
    pub profile: PowerProfileDto,
}

/// Battery state descriptor mirroring [`zettings_power::BatteryState`].
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, TS)]
#[ts(export, export_to = "battery_state.ts")]
pub struct BatteryStateDto {
    /// `UPower` device object-path index.
    pub device_index: u32,
    /// Charge percentage in `[0.0, 100.0]`.
    pub percentage: f32,
    /// `true` when the device is charging.
    pub charging: bool,
}

impl From<zettings_power::BatteryState> for BatteryStateDto {
    fn from(b: zettings_power::BatteryState) -> Self {
        Self {
            device_index: b.device_index,
            percentage: b.percentage,
            charging: b.charging,
        }
    }
}

/// Response payload for `power_batteries`.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "power_batteries_result.ts")]
pub struct PowerBatteriesResult {
    /// `UPower` battery devices, ordered by `device_index`.
    pub batteries: Vec<BatteryStateDto>,
}

/// Accent palette DTO mirroring [`zettings_palette::AccentPalette`] for the
/// personalization panel's wallpaper-accent picker. Each component is `[f32; 3]`
/// in `[0, 1]` so the frontend can drop the values directly into a CSS
/// `rgb()` expression without further conversion.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, TS)]
#[ts(export, export_to = "accent_palette.ts")]
pub struct AccentPaletteDto {
    /// Primary accent (`--accent`).
    pub accent: [f32; 3],
    /// High-contrast text/glyph color placed on `accent` (`--accent-on`).
    pub on_accent: [f32; 3],
    /// Secondary muted accent (`--accent-secondary`).
    pub secondary: [f32; 3],
}

impl From<zettings_palette::AccentPalette> for AccentPaletteDto {
    fn from(p: zettings_palette::AccentPalette) -> Self {
        Self {
            accent: p.accent.0,
            on_accent: p.on_accent.0,
            secondary: p.secondary.0,
        }
    }
}

/// Request payload for the palette accent-extraction command. The webview
/// passes the wallpaper image bytes (PNG/JPEG/WebP); the backend decodes and
/// quantizes them via `zettings-palette::extract`.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "palette_extract_request.ts")]
pub struct PaletteExtractRequest {
    /// Raw image bytes (e.g. PNG, JPEG, WebP) from the webview file picker.
    pub bytes: Vec<u8>,
}

/// Response payload for `palette_extract`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
#[ts(export, export_to = "palette_extract_result.ts")]
pub struct PaletteExtractResult {
    /// The extracted accent palette, ready to drop into CSS `--accent-*`
    /// custom properties.
    pub palette: AccentPaletteDto,
}

/// Process-wide [`zettings_search::Index`] instance. Constructed lazily on
/// first access from any Tauri command surface (register or query). We use
/// `once_cell::sync::OnceCell` rather than `Lazy` so initialization stays
/// explicit and the lockguard lives in `std`.
fn search_index() -> &'static zettings_search::Index {
    static INDEX: once_cell::sync::OnceCell<zettings_search::Index> =
        once_cell::sync::OnceCell::new();
    INDEX.get_or_init(zettings_search::Index::new)
}

/// Sets the system hostname after verifying `PolicyKit` authorization.
///
/// Action ID: `org.zyntrix.zettings.network.set-hostname`.
///
/// Under the `zettings-mock` feature (Windows dev loop) the mock authorizer
/// always returns [`Authorization::Authorized`] and the new hostname is echoed
/// back directly. On the real Linux target the call is forwarded to
/// `org.freedesktop.NetworkManager.SetHostname` over `zbus` (Phase 5).
///
/// The `#[tauri::command]` attribute is applied in `apps/zettings/src/main.rs`
/// when this command is registered with `generate_handler!` (Phase 5). Keeping
/// the attribute off the lib definition avoids Tauri v2 macro-namespace
/// duplicate-definition errors when the lib is compiled independently of the
/// bin's `generate_handler!` invocation.
///
/// # Errors
/// - [`IpcError::PolkitDenied`] if the user dismisses the polkit dialog or is
///   not permitted to change the hostname.
/// - [`IpcError::ServiceUnavailable`] on the non-mock target until Phase 5
///   wires the real `zbus` integration.
//
// `needless_pass_by_value`: Tauri's `generate_handler!` deserializes the
// request payload from the webview into an owned `SetHostnameRequest` before
// calling this function, so the public IPC boundary must accept by value.
#[allow(clippy::needless_pass_by_value)]
pub fn network_set_hostname(request: SetHostnameRequest) -> Result<SetHostnameResult, IpcError> {
    set_hostname_impl(&request)
}

/// Backend implementation of [`network_set_hostname`], split out so the
/// `#[tauri::command]` macro expands over a thin wrapper instead of a body
/// containing `#[cfg]` blocks (which confuse the macro's helper-symbol
/// generation on Tauri v2).
fn set_hostname_impl(request: &SetHostnameRequest) -> Result<SetHostnameResult, IpcError> {
    let action = ActionId::zettings("network", "set-hostname");
    set_hostname_with_authorizer(request, &action)
}

/// Mock-path hostname mutation. The mock authorizer always permits the change.
#[cfg(feature = "zettings-mock")]
fn set_hostname_with_authorizer(
    request: &SetHostnameRequest,
    action: &ActionId,
) -> Result<SetHostnameResult, IpcError> {
    let authorizer = MockAuthorizer;
    let bus = Bus::new();
    let outcome = zettings_polkit::check_authorization_gateway(&authorizer, &bus, action)
        .map_err(|e| IpcError::PolkitDenied(e.to_string()))?;
    if !matches!(
        outcome,
        Authorization::Authorized | Authorization::Challenge
    ) {
        return Err(IpcError::PolkitDenied(
            "User denied authorization to change system hostname".into(),
        ));
    }
    let backend = zettings_network::MockBackend::new();
    backend
        .set_hostname(&request.hostname, &bus)
        .map_err(IpcError::from)?;
    Ok(SetHostnameResult {
        success: true,
        active_hostname: request.hostname.clone(),
    })
}

/// Real-target hostname mutation. The `zbus` call to
/// `org.freedesktop.NetworkManager.SetHostname` lands in Phase 5+; until then
/// this surfaces a deterministic service-unavailable error so the frontend can
/// render a clear state rather than a silent no-op.
#[cfg(not(feature = "zettings-mock"))]
fn set_hostname_with_authorizer(
    _request: &SetHostnameRequest,
    _action: &ActionId,
) -> Result<SetHostnameResult, IpcError> {
    Err(IpcError::ServiceUnavailable(
        "Real DBus integration lands in Phase 5".into(),
    ))
}

/// Applies a display mode to an output after verifying `PolicyKit`.
///
/// Action ID: `org.zyntrix.zettings.display.apply-mode`.
///
/// # Errors
/// - [`IpcError::PolkitDenied`] when authorization is denied.
/// - [`IpcError::InvalidPayload`] when the output or mode is not available.
/// - [`IpcError::ServiceUnavailable`] on the non-mock target until Phase 5+.
#[allow(clippy::needless_pass_by_value)]
pub fn display_apply_mode(
    request: DisplayApplyModeRequest,
) -> Result<DisplayApplyModeResult, IpcError> {
    display_apply_mode_impl(&request)
}

fn display_apply_mode_impl(
    request: &DisplayApplyModeRequest,
) -> Result<DisplayApplyModeResult, IpcError> {
    let action = ActionId::zettings("display", "apply-mode");
    display_apply_mode_with_authorizer(request, &action)
}

#[cfg(feature = "zettings-mock")]
fn display_apply_mode_with_authorizer(
    request: &DisplayApplyModeRequest,
    action: &ActionId,
) -> Result<DisplayApplyModeResult, IpcError> {
    let authorizer = MockAuthorizer;
    let bus = Bus::new();
    let outcome = zettings_polkit::check_authorization_gateway(&authorizer, &bus, action)
        .map_err(|e| IpcError::PolkitDenied(e.to_string()))?;
    if !matches!(
        outcome,
        Authorization::Authorized | Authorization::Challenge
    ) {
        return Err(IpcError::PolkitDenied(
            "User denied authorization to change display mode".into(),
        ));
    }
    let backend = zettings_display::MockBackend::new();
    backend
        .apply_mode(
            &OutputId::new(request.output.clone()),
            OutputConfig {
                mode: DisplayMode {
                    width: request.mode.width,
                    height: request.mode.height,
                    refresh_hz: request.mode.refresh_hz,
                },
                scale: request.scale,
            },
            &bus,
        )
        .map_err(IpcError::from)?;
    Ok(DisplayApplyModeResult { applied: true })
}

#[cfg(not(feature = "zettings-mock"))]
fn display_apply_mode_with_authorizer(
    _request: &DisplayApplyModeRequest,
    _action: &ActionId,
) -> Result<DisplayApplyModeResult, IpcError> {
    Err(IpcError::ServiceUnavailable(
        "Real KScreen DBus integration lands in Phase 5".into(),
    ))
}

/// Sets per-stream audio volume after verifying `PolicyKit`.
///
/// Action ID: `org.zyntrix.zettings.audio.set-volume`.
///
/// # Errors
/// - [`IpcError::PolkitDenied`] when authorization is denied.
/// - [`IpcError::InvalidPayload`] when the stream id is not registered.
/// - [`IpcError::ServiceUnavailable`] on the non-mock target until Phase 5+.
#[allow(clippy::needless_pass_by_value)]
pub fn audio_set_volume(request: AudioSetVolumeRequest) -> Result<AudioSetVolumeResult, IpcError> {
    audio_set_volume_impl(&request)
}

fn audio_set_volume_impl(
    request: &AudioSetVolumeRequest,
) -> Result<AudioSetVolumeResult, IpcError> {
    let action = ActionId::zettings("audio", "set-volume");
    audio_set_volume_with_authorizer(request, &action)
}

#[cfg(feature = "zettings-mock")]
fn audio_set_volume_with_authorizer(
    request: &AudioSetVolumeRequest,
    action: &ActionId,
) -> Result<AudioSetVolumeResult, IpcError> {
    let authorizer = MockAuthorizer;
    let bus = Bus::new();
    let outcome = zettings_polkit::check_authorization_gateway(&authorizer, &bus, action)
        .map_err(|e| IpcError::PolkitDenied(e.to_string()))?;
    if !matches!(
        outcome,
        Authorization::Authorized | Authorization::Challenge
    ) {
        return Err(IpcError::PolkitDenied(
            "User denied authorization to change audio volume".into(),
        ));
    }
    let backend = zettings_audio::MockBackend::new();
    backend
        .set_volume(
            StreamId(request.stream_id),
            Volume::clamp(request.volume),
            request.muted,
            &bus,
        )
        .map_err(IpcError::from)?;
    Ok(AudioSetVolumeResult { applied: true })
}

#[cfg(not(feature = "zettings-mock"))]
fn audio_set_volume_with_authorizer(
    _request: &AudioSetVolumeRequest,
    _action: &ActionId,
) -> Result<AudioSetVolumeResult, IpcError> {
    Err(IpcError::ServiceUnavailable(
        "Real PipeWire/PulseAudio DBus integration lands in Phase 5".into(),
    ))
}

/// Scans for nearby Wi-Fi access points after verifying `PolicyKit`.
///
/// Action ID: `org.zyntrix.zettings.network.scan-wifi`.
///
/// # Errors
/// - [`IpcError::PolkitDenied`] when authorization is denied.
/// - [`IpcError::ServiceUnavailable`] on the non-mock target until Phase 5+.
pub fn network_scan_wifi() -> Result<NetworkScanWifiResult, IpcError> {
    network_scan_wifi_impl()
}

fn network_scan_wifi_impl() -> Result<NetworkScanWifiResult, IpcError> {
    let action = ActionId::zettings("network", "scan-wifi");
    network_scan_wifi_with_authorizer(&action)
}

#[cfg(feature = "zettings-mock")]
fn network_scan_wifi_with_authorizer(action: &ActionId) -> Result<NetworkScanWifiResult, IpcError> {
    let authorizer = MockAuthorizer;
    let bus = Bus::new();
    let outcome = zettings_polkit::check_authorization_gateway(&authorizer, &bus, action)
        .map_err(|e| IpcError::PolkitDenied(e.to_string()))?;
    if !matches!(
        outcome,
        Authorization::Authorized | Authorization::Challenge
    ) {
        return Err(IpcError::PolkitDenied(
            "User denied authorization to scan Wi-Fi".into(),
        ));
    }
    let backend = zettings_network::MockBackend::new();
    let mut aps: Vec<AccessPointDto> = backend
        .scan_wifi()
        .map_err(IpcError::from)?
        .into_iter()
        .map(AccessPointDto::from)
        .collect();
    // Sort by descending signal strength so the strongest network surfaces
    // first. `sort_unstable_by_key` + `reverse` is a stable permutation across
    // runs because ties on `signal_dbm` preserve prior order from the mock.
    aps.sort_unstable_by_key(|ap| ap.signal_dbm);
    aps.reverse();
    Ok(NetworkScanWifiResult { access_points: aps })
}

#[cfg(not(feature = "zettings-mock"))]
fn network_scan_wifi_with_authorizer(
    _action: &ActionId,
) -> Result<NetworkScanWifiResult, IpcError> {
    Err(IpcError::ServiceUnavailable(
        "Real NetworkManager DBus integration lands in Phase 5".into(),
    ))
}

/// Switches the active power profile after verifying `PolicyKit`.
///
/// Action ID: `org.zyntrix.zettings.power.set-profile`.
///
/// # Errors
/// - [`IpcError::PolkitDenied`] when authorization is denied.
/// - [`IpcError::InvalidPayload`] when the profile is not available.
/// - [`IpcError::ServiceUnavailable`] on the non-mock target until Phase 5+.
#[allow(clippy::needless_pass_by_value)]
pub fn power_set_profile(
    request: PowerSetProfileRequest,
) -> Result<PowerSetProfileResult, IpcError> {
    power_set_profile_impl(&request)
}

fn power_set_profile_impl(
    request: &PowerSetProfileRequest,
) -> Result<PowerSetProfileResult, IpcError> {
    let action = ActionId::zettings("power", "set-profile");
    power_set_profile_with_authorizer(request, &action)
}

#[cfg(feature = "zettings-mock")]
fn power_set_profile_with_authorizer(
    request: &PowerSetProfileRequest,
    action: &ActionId,
) -> Result<PowerSetProfileResult, IpcError> {
    let authorizer = MockAuthorizer;
    let bus = Bus::new();
    let outcome = zettings_polkit::check_authorization_gateway(&authorizer, &bus, action)
        .map_err(|e| IpcError::PolkitDenied(e.to_string()))?;
    if !matches!(
        outcome,
        Authorization::Authorized | Authorization::Challenge
    ) {
        return Err(IpcError::PolkitDenied(
            "User denied authorization to change power profile".into(),
        ));
    }
    let backend = zettings_power::MockBackend::new();
    backend
        .set_profile(Profile::from(request.profile), &bus)
        .map_err(IpcError::from)?;
    Ok(PowerSetProfileResult { applied: true })
}

#[cfg(not(feature = "zettings-mock"))]
fn power_set_profile_with_authorizer(
    _request: &PowerSetProfileRequest,
    _action: &ActionId,
) -> Result<PowerSetProfileResult, IpcError> {
    Err(IpcError::ServiceUnavailable(
        "Real UPower/power-profiles DBus integration lands in Phase 5".into(),
    ))
}

/// Batch-registers settings entries in the global Spotlight search index.
/// Modules call this once on mount.
///
/// Unlike the `zbus`-backed command surfaces, the Tantivy in-memory index is
/// purely in-process — it touches no system resources (no `DBus`, no `PipeWire`,
/// no `UPower`), so no `PolicyKit` authorization is required. The search IPC
/// commands are therefore feature-flag-free and behave identically on the
/// Windows dev loop and on the real WSL2/Linux target.
///
/// # Errors
/// - [`IpcError::ServiceUnavailable`] when the underlying Tantivy writer fails
///   (theoretically impossible for a `RAMDirectory` but surfaced for safety).
#[allow(clippy::needless_pass_by_value)]
pub fn search_register_entries(
    request: SearchRegisterEntriesRequest,
) -> Result<SearchRegisterEntriesResult, IpcError> {
    search_register_entries_impl(&request)
}

fn search_register_entries_impl(
    request: &SearchRegisterEntriesRequest,
) -> Result<SearchRegisterEntriesResult, IpcError> {
    let count = request.entries.len();
    search_index()
        .insert_many(&request.entries)
        .map_err(IpcError::from)?;
    Ok(SearchRegisterEntriesResult { registered: count })
}

/// Queries the Spotlight search index.
///
/// Returns up to 20 ranked [`SearchHit`] entries. The frontend Spotlight modal
/// also stages client-side fuzzy results while the IPC round-trip is in flight
/// to keep the <5ms perceived latency budget (see `apps/zettings/web`). Like
/// [`search_register_entries`], the query path touches no system resources and
/// so requires no `PolicyKit` authorization.
///
/// # Errors
/// - [`IpcError::InvalidPayload`] when the query cannot be parsed.
/// - [`IpcError::ServiceUnavailable`] when the underlying Tantivy reader or
///   document fetch fails.
#[allow(clippy::needless_pass_by_value)]
pub fn search_query(request: SearchQueryRequest) -> Result<Vec<SearchHit>, IpcError> {
    search_query_impl(&request)
}

fn search_query_impl(request: &SearchQueryRequest) -> Result<Vec<SearchHit>, IpcError> {
    search_index()
        .search(&request.query)
        .map_err(IpcError::from)
}

// ===========================================================================
// Phase 7 read-side surface — used by the domain feature panel modules on
// mount. The mock backend (zettings-mock feature) returns deterministic state
// so the panel render paths and react-query caches can be exercised on the
// Windows dev loop. The real `zbus`/`PipeWire`/`BlueZ`/`UPower` paths land in
// Phase 5+ and surface `IpcError::ServiceUnavailable` until then.
// ===========================================================================

/// Lists connected display outputs and their advertised modes.
///
/// Action ID: `org.zyntrix.zettings.display.list-outputs`. No `PolicyKit`
/// authorization — read-only introspection of `KScreen` state (the
/// apply-mode mutation is the privileged operation; reads are unprivileged
/// so the panel can populate on mount without a polkit dialog).
///
/// # Errors
/// - [`IpcError::ServiceUnavailable`] on the non-mock target until Phase 5+
///   wires the real `zbus` integration.
pub fn display_list_outputs() -> Result<DisplayListOutputsResult, IpcError> {
    display_list_outputs_impl()
}

fn display_list_outputs_impl() -> Result<DisplayListOutputsResult, IpcError> {
    let action = ActionId::zettings("display", "list-outputs");
    display_list_outputs_with_authorizer(&action)
}

#[cfg(feature = "zettings-mock")]
fn display_list_outputs_with_authorizer(
    _action: &ActionId,
) -> Result<DisplayListOutputsResult, IpcError> {
    let backend = zettings_display::MockBackend::new();
    let outputs = backend.list_outputs().map_err(IpcError::from)?;
    let dtos = outputs
        .into_iter()
        .map(|(id, modes)| DisplayOutputDto {
            output_id: id.0,
            modes: modes
                .into_iter()
                .map(|m| DisplayModeDto {
                    width: m.width,
                    height: m.height,
                    refresh_hz: m.refresh_hz,
                })
                .collect(),
        })
        .collect();
    Ok(DisplayListOutputsResult { outputs: dtos })
}

#[cfg(not(feature = "zettings-mock"))]
fn display_list_outputs_with_authorizer(
    _action: &ActionId,
) -> Result<DisplayListOutputsResult, IpcError> {
    Err(IpcError::ServiceUnavailable(
        "Real KScreen DBus integration lands in Phase 5".into(),
    ))
}

/// Lists active audio streams with live volume state for the mixer panel.
///
/// Action ID: `org.zyntrix.zettings.audio.list-streams`. Read-only; no
/// `PolicyKit` authorization (the set-volume mutation is privileged).
///
/// # Errors
/// - [`IpcError::ServiceUnavailable`] on the non-mock target until Phase 5+.
pub fn audio_list_streams() -> Result<AudioListStreamsResult, IpcError> {
    audio_list_streams_impl()
}

fn audio_list_streams_impl() -> Result<AudioListStreamsResult, IpcError> {
    let action = ActionId::zettings("audio", "list-streams");
    audio_list_streams_with_authorizer(&action)
}

#[cfg(feature = "zettings-mock")]
fn audio_list_streams_with_authorizer(
    _action: &ActionId,
) -> Result<AudioListStreamsResult, IpcError> {
    let backend = zettings_audio::MockBackend::new();
    let streams = backend.list_streams_detailed().map_err(IpcError::from)?;
    let dtos = streams
        .into_iter()
        .map(|s| AudioStreamDto {
            stream_id: s.stream_id,
            label_id: s.label_id,
            volume: s.volume,
            muted: s.muted,
        })
        .collect();
    Ok(AudioListStreamsResult { streams: dtos })
}

#[cfg(not(feature = "zettings-mock"))]
fn audio_list_streams_with_authorizer(
    _action: &ActionId,
) -> Result<AudioListStreamsResult, IpcError> {
    Err(IpcError::ServiceUnavailable(
        "Real PipeWire/PulseAudio DBus integration lands in Phase 5".into(),
    ))
}

/// Lists paired (and optionally connected) `BlueZ` peripherals.
///
/// Action ID: `org.zyntrix.zettings.bluetooth.list-paired`. Read-only, no
/// `PolicyKit` authorization.
///
/// # Errors
/// - [`IpcError::ServiceUnavailable`] on the non-mock target until Phase 5+.
pub fn bluetooth_list_paired() -> Result<BluetoothListPairedResult, IpcError> {
    bluetooth_list_paired_impl()
}

fn bluetooth_list_paired_impl() -> Result<BluetoothListPairedResult, IpcError> {
    let action = ActionId::zettings("bluetooth", "list-paired");
    bluetooth_list_paired_with_authorizer(&action)
}

#[cfg(feature = "zettings-mock")]
fn bluetooth_list_paired_with_authorizer(
    _action: &ActionId,
) -> Result<BluetoothListPairedResult, IpcError> {
    let backend = zettings_network::MockBackend::new();
    let devices = backend.list_paired_devices().map_err(IpcError::from)?;
    let dtos = devices.into_iter().map(PairedDeviceDto::from).collect();
    Ok(BluetoothListPairedResult { devices: dtos })
}

#[cfg(not(feature = "zettings-mock"))]
fn bluetooth_list_paired_with_authorizer(
    _action: &ActionId,
) -> Result<BluetoothListPairedResult, IpcError> {
    Err(IpcError::ServiceUnavailable(
        "Real BlueZ DBus integration lands in Phase 5".into(),
    ))
}

/// Reads the currently-active power profile.
///
/// Action ID: `org.zyntrix.zettings.power.active-profile`. Read-only, no
/// `PolicyKit` authorization (the set-profile mutation is privileged).
///
/// # Errors
/// - [`IpcError::ServiceUnavailable`] on the non-mock target until Phase 5+.
pub fn power_active_profile() -> Result<PowerActiveProfileResult, IpcError> {
    power_active_profile_impl()
}

fn power_active_profile_impl() -> Result<PowerActiveProfileResult, IpcError> {
    let action = ActionId::zettings("power", "active-profile");
    power_active_profile_with_authorizer(&action)
}

#[cfg(feature = "zettings-mock")]
fn power_active_profile_with_authorizer(
    _action: &ActionId,
) -> Result<PowerActiveProfileResult, IpcError> {
    let backend = zettings_power::MockBackend::new();
    let profile = backend.active_profile().map_err(IpcError::from)?;
    Ok(PowerActiveProfileResult {
        profile: match profile {
            Profile::Balanced => PowerProfileDto::Balanced,
            Profile::Performance => PowerProfileDto::Performance,
            Profile::PowerSaver => PowerProfileDto::PowerSaver,
        },
    })
}

#[cfg(not(feature = "zettings-mock"))]
fn power_active_profile_with_authorizer(
    _action: &ActionId,
) -> Result<PowerActiveProfileResult, IpcError> {
    Err(IpcError::ServiceUnavailable(
        "Real UPower/power-profiles-daemon DBus integration lands in Phase 5".into(),
    ))
}

/// Reads battery state for all `UPower` devices.
///
/// Action ID: `org.zyntrix.zettings.power.batteries`. Read-only, no
/// `PolicyKit` authorization.
///
/// # Errors
/// - [`IpcError::ServiceUnavailable`] on the non-mock target until Phase 5+.
pub fn power_batteries() -> Result<PowerBatteriesResult, IpcError> {
    power_batteries_impl()
}

fn power_batteries_impl() -> Result<PowerBatteriesResult, IpcError> {
    let action = ActionId::zettings("power", "batteries");
    power_batteries_with_authorizer(&action)
}

#[cfg(feature = "zettings-mock")]
fn power_batteries_with_authorizer(_action: &ActionId) -> Result<PowerBatteriesResult, IpcError> {
    let backend = zettings_power::MockBackend::new();
    let batteries = backend.batteries().map_err(IpcError::from)?;
    let dtos = batteries.into_iter().map(BatteryStateDto::from).collect();
    Ok(PowerBatteriesResult { batteries: dtos })
}

#[cfg(not(feature = "zettings-mock"))]
fn power_batteries_with_authorizer(_action: &ActionId) -> Result<PowerBatteriesResult, IpcError> {
    Err(IpcError::ServiceUnavailable(
        "Real UPower DBus integration lands in Phase 5".into(),
    ))
}

/// Process performance telemetry returned by the `zettings_perf_stats` command.
///
/// Phase 9 launch-time + memory-footprint audit surface. The frontend
/// `PerfMonitor` overlay (dev-loop tooling) samples this on an interval to
/// validate the `<500ms` cold-start and `<150MB` idle-RAM budgets from
/// `PLAN.md` Phase 9. RSS is only measurable on Linux (via `/proc/self/status`)
/// where real backend integration runs; the Windows mock dev loop reports
/// `None` and relies on the WSL2 audit commands in `docs/performance/audit.md`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export, export_to = "perf_stats.ts")]
pub struct PerfStatsDto {
    /// Backend process uptime in milliseconds (from `main()` entry).
    /// `u32` wraps after ~49 days of uptime — acceptable for an audit surface;
    /// the JSON IPC protocol delivers JS `number` for `u32` (no `bigint`
    /// mismatch like `u64`).
    pub backend_uptime_ms: u32,
    /// Backend startup time in milliseconds — `main()` entry to `setup` done.
    pub backend_startup_ms: u32,
    /// Resident set size in bytes for the current process. `None` on
    /// non-Linux targets where no safe std API exists. `u32` comfortably
    /// bounds the `<150MB` Phase 9 budget (and any leak up to 4GB).
    pub memory_rss_bytes: Option<u32>,
    /// `true` when running against the `zettings-mock` state-machine backend.
    pub is_mock: bool,
}

/// Builds the Phase 9 process-telemetry payload.
///
/// `backend_uptime_ms` and `backend_startup_ms` are measured in
/// `apps/zettings/src/main.rs` (a `OnceLock<Instant>` captured at `main()`
/// entry and after the `setup` closure completes) and passed in so this lib
/// stays a pure, unit-testable snapshot builder. Memory is read from
/// `/proc/self/status` on Linux only — safe std I/O, no syscall bindings.
pub fn perf_stats(backend_uptime_ms: u32, backend_startup_ms: u32) -> PerfStatsDto {
    PerfStatsDto {
        backend_uptime_ms,
        backend_startup_ms,
        memory_rss_bytes: current_rss_bytes(),
        is_mock: cfg!(feature = "zettings-mock"),
    }
}

/// Resident set size in bytes for the current process.
///
/// Linux: reads the `VmRSS` line from `/proc/self/status` (kB → bytes).
/// Other targets: `None` — no safe `std` API exposes process memory there,
/// and the Phase 9 audit targets the Linux/WSL2 backend anyway.
#[cfg(target_os = "linux")]
fn current_rss_bytes() -> Option<u32> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    let line = status.lines().find(|l| l.starts_with("VmRSS:"))?;
    let kb: u32 = line.split_whitespace().nth(1)?.parse().ok()?;
    Some(kb.saturating_mul(1024))
}

/// Non-Linux placeholder — the Windows mock dev loop cannot query RSS safely.
#[cfg(not(target_os = "linux"))]
fn current_rss_bytes() -> Option<u32> {
    None
}

/// Extracts an accent palette from wallpaper image bytes via `zettings-palette`.
///
/// Feature-flag-FREE — touches no system resources (the `image` decoder + the
/// in-process Median-cut quantizer live entirely in `zettings-palette`). No
/// `PolicyKit` authorization needed, exactly like the search commands.
///
/// # Errors
/// - [`IpcError::InvalidPayload`] when the bytes cannot be decoded or the
///   image is too small to extract a palette.
#[allow(clippy::needless_pass_by_value)]
pub fn palette_extract(request: PaletteExtractRequest) -> Result<PaletteExtractResult, IpcError> {
    palette_extract_impl(&request)
}

fn palette_extract_impl(request: &PaletteExtractRequest) -> Result<PaletteExtractResult, IpcError> {
    let palette = zettings_palette::extract(&request.bytes)
        .map_err(|e| IpcError::InvalidPayload(format!("palette extraction failed: {e}")))?;
    Ok(PaletteExtractResult {
        palette: AccentPaletteDto::from(palette),
    })
}

#[cfg(test)]
mod tests {
    //! Cross-target correctness checks for the IPC command surface.
    //! All cases run under the `zettings-mock` feature (the Windows dev loop
    //! default), so they exercise the mock-state backend paths.

    use super::*;

    #[test]
    fn perf_stats_roundtrips_uptime_and_startup() {
        let dto = perf_stats(12_345, 432);
        assert_eq!(dto.backend_uptime_ms, 12_345);
        assert_eq!(dto.backend_startup_ms, 432);
        assert_eq!(dto.is_mock, cfg!(feature = "zettings-mock"));
    }

    #[test]
    fn set_hostname_mock_succeeds() {
        let result = network_set_hostname(SetHostnameRequest {
            hostname: "aurora-dev".into(),
        })
        .expect("mock set-hostname");
        assert!(result.success);
        assert_eq!(result.active_hostname, "aurora-dev");
    }

    #[test]
    fn set_hostname_rejects_invalid_payload() {
        let err = network_set_hostname(SetHostnameRequest {
            hostname: "bad_host!".into(),
        })
        .unwrap_err();
        assert!(matches!(err, IpcError::InvalidPayload(_)));
    }

    #[test]
    fn display_apply_mode_mock_succeeds() {
        let result = display_apply_mode(DisplayApplyModeRequest {
            output: "HDMI-A-1".into(),
            mode: DisplayModeDto {
                width: 1920,
                height: 1080,
                refresh_hz: 60.0,
            },
            scale: 1.0,
        })
        .expect("mock display apply");
        assert!(result.applied);
    }

    #[test]
    fn display_apply_mode_rejects_unknown_output() {
        let err = display_apply_mode(DisplayApplyModeRequest {
            output: "VGA-1".into(),
            mode: DisplayModeDto {
                width: 1024,
                height: 768,
                refresh_hz: 60.0,
            },
            scale: 1.0,
        })
        .unwrap_err();
        assert!(matches!(err, IpcError::InvalidPayload(_)));
    }

    #[test]
    fn audio_set_volume_mock_succeeds() {
        let result = audio_set_volume(AudioSetVolumeRequest {
            stream_id: 1,
            volume: 0.5,
            muted: false,
        })
        .expect("mock audio set");
        assert!(result.applied);
    }

    #[test]
    fn audio_set_volume_clamps_out_of_range() {
        // volume > 1.0 should clamp, not error.
        let result = audio_set_volume(AudioSetVolumeRequest {
            stream_id: 0,
            volume: 2.0,
            muted: true,
        })
        .expect("clamped mock audio set");
        assert!(result.applied);
    }

    #[test]
    fn audio_set_volume_rejects_unknown_stream() {
        let err = audio_set_volume(AudioSetVolumeRequest {
            stream_id: 99,
            volume: 0.3,
            muted: false,
        })
        .unwrap_err();
        assert!(matches!(err, IpcError::InvalidPayload(_)));
    }

    #[test]
    fn network_scan_wifi_returns_sorted_aps() {
        let result = network_scan_wifi().expect("mock scan");
        assert!(!result.access_points.is_empty());
        // Sorted by descending signal_dbm.
        let mut prev = i8::MAX;
        for ap in &result.access_points {
            assert!(ap.signal_dbm <= prev);
            prev = ap.signal_dbm;
        }
    }

    #[test]
    fn power_set_profile_mock_succeeds() {
        let result = power_set_profile(PowerSetProfileRequest {
            profile: PowerProfileDto::Performance,
        })
        .expect("mock power set");
        assert!(result.applied);
    }

    #[test]
    fn search_register_then_query_roundtrips() {
        // The global Index is shared across all tests (and across the Tauri
        // app's lifetime), so we register an entry with a unique id + label
        // and query a unique keyword that only this test inserts. That keeps
        // the assertion robust to test execution order. Note: Tantivy's
        // default tokenizer splits on hyphens, so we use an underscore-free
        // camelPascal keyword to avoid spurious token boundary effects.
        let entry = SettingsEntry {
            id: "org.zyntrix.zettings.display:phasesixroundtrip".into(),
            module_id: "org.zyntrix.zettings.display".into(),
            label: "PhaseSix Roundtrip".into(),
            category: "Display".into(),
            route: "/display/phase-six-roundtrip".into(),
            keywords: vec!["phasesixroundtrip".into()],
        };
        let reg = search_register_entries(SearchRegisterEntriesRequest {
            entries: vec![entry.clone()],
        })
        .expect("mock search register");
        assert_eq!(reg.registered, 1);
        let hits = search_query(SearchQueryRequest {
            query: "phasesixroundtrip".into(),
        })
        .expect("mock search query");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].entry.route, "/display/phase-six-roundtrip");
    }

    #[test]
    fn search_query_tolerates_typos() {
        // Register and query with a near-miss typo ("nite"); the fuzzy-prefix
        // Tantivy query + strsim re-rank should still surface the entry.
        search_register_entries(SearchRegisterEntriesRequest {
            entries: vec![SettingsEntry {
                id: "display:night-light".into(),
                module_id: "display".into(),
                label: "Night Light".into(),
                category: "Display".into(),
                route: "/display/night-light".into(),
                keywords: vec!["night".into()],
            }],
        })
        .expect("register");
        let hits = search_query(SearchQueryRequest {
            query: "nite".into(),
        })
        .expect("query");
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn search_register_is_idempotent_on_id() {
        let entry = SettingsEntry {
            id: "audio:volume".into(),
            module_id: "audio".into(),
            label: "Master Volume".into(),
            category: "Audio".into(),
            route: "/audio/volume".into(),
            keywords: vec!["loudness".into()],
        };
        search_register_entries(SearchRegisterEntriesRequest {
            entries: vec![entry.clone()],
        })
        .expect("first register");
        search_register_entries(SearchRegisterEntriesRequest {
            entries: vec![entry],
        })
        .expect("second register");
        let hits = search_query(SearchQueryRequest {
            query: "volume".into(),
        })
        .expect("query");
        assert_eq!(hits.len(), 1, "re-registering must not duplicate entries");
    }
}

#[cfg(test)]
mod bindings_export {
    //! Regenerates the TypeScript bindings under
    //! `packages/ts-bindings/src/generated/` when this crate is tested with the
    //! `export-bindings` feature.
    //!
    //! Run via:
    //! ```sh
    //! cargo test -p zettings-ipc --features export-bindings
    //! ```
    //! No environment variables required — the test computes the absolute
    //! path to the workspace `packages/ts-bindings/src/generated/` directory
    //! from `CARGO_MANIFEST_DIR`.

    use super::*;
    use std::path::PathBuf;

    /// Absolute path to `packages/ts-bindings/src/generated/` relative to
    /// this crate's `CARGO_MANIFEST_DIR` (`crates/zettings-ipc/`).
    fn generated_dir() -> PathBuf {
        let manifest_dir =
            std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is always set by cargo");
        PathBuf::from(manifest_dir)
            // crates/zettings-ipc -> crates/
            .parent()
            .expect("crate manifest dir has a parent")
            // crates/ -> workspace root
            .parent()
            .expect("crates dir has a parent (workspace root)")
            .join("packages")
            .join("ts-bindings")
            .join("src")
            .join("generated")
    }

    #[test]
    fn export_all_bindings() {
        // ts-rs 12 API: `TS::export_all(&cfg)` writes each type to
        // `cfg.export_dir.join(output_path)` where `output_path` is the
        // per-type `#[ts(export_to = "..")]` value (a filename like
        // "health.ts"). We compute the absolute workspace path so the
        // bindings always land in `packages/ts-bindings/src/generated/`
        // regardless of the caller's working directory.
        let cfg = ts_rs::Config::default().with_out_dir(generated_dir());
        Health::export_all(&cfg).expect("export Health bindings");
        ModuleInfo::export_all(&cfg).expect("export ModuleInfo bindings");
        IpcError::export_all(&cfg).expect("export IpcError bindings");
        SetHostnameRequest::export_all(&cfg).expect("export SetHostnameRequest bindings");
        SetHostnameResult::export_all(&cfg).expect("export SetHostnameResult bindings");
        DisplayModeDto::export_all(&cfg).expect("export DisplayModeDto bindings");
        DisplayApplyModeRequest::export_all(&cfg).expect("export DisplayApplyModeRequest bindings");
        DisplayApplyModeResult::export_all(&cfg).expect("export DisplayApplyModeResult bindings");
        AudioSetVolumeRequest::export_all(&cfg).expect("export AudioSetVolumeRequest bindings");
        AudioSetVolumeResult::export_all(&cfg).expect("export AudioSetVolumeResult bindings");
        AccessPointDto::export_all(&cfg).expect("export AccessPointDto bindings");
        NetworkScanWifiResult::export_all(&cfg).expect("export NetworkScanWifiResult bindings");
        PowerProfileDto::export_all(&cfg).expect("export PowerProfileDto bindings");
        PowerSetProfileRequest::export_all(&cfg).expect("export PowerSetProfileRequest bindings");
        PowerSetProfileResult::export_all(&cfg).expect("export PowerSetProfileResult bindings");
        SearchRegisterEntriesRequest::export_all(&cfg)
            .expect("export SearchRegisterEntriesRequest bindings");
        SearchRegisterEntriesResult::export_all(&cfg)
            .expect("export SearchRegisterEntriesResult bindings");
        SearchQueryRequest::export_all(&cfg).expect("export SearchQueryRequest bindings");
        // The SearchHit / SettingsEntry derives live in the `zettings-search`
        // crate; export them from there via the search crate's own test mod so
        // they land in this same `generated/` directory.
        SettingsEntry::export_all(&cfg).expect("export SettingsEntry bindings");
        SearchHit::export_all(&cfg).expect("export SearchHit bindings");
        // Phase 7 read-side DTOs
        DisplayOutputDto::export_all(&cfg).expect("export DisplayOutputDto bindings");
        DisplayListOutputsResult::export_all(&cfg)
            .expect("export DisplayListOutputsResult bindings");
        AudioStreamDto::export_all(&cfg).expect("export AudioStreamDto bindings");
        AudioListStreamsResult::export_all(&cfg).expect("export AudioListStreamsResult bindings");
        PairedDeviceDto::export_all(&cfg).expect("export PairedDeviceDto bindings");
        BluetoothListPairedResult::export_all(&cfg)
            .expect("export BluetoothListPairedResult bindings");
        PowerActiveProfileResult::export_all(&cfg)
            .expect("export PowerActiveProfileResult bindings");
        BatteryStateDto::export_all(&cfg).expect("export BatteryStateDto bindings");
        PowerBatteriesResult::export_all(&cfg).expect("export PowerBatteriesResult bindings");
        AccentPaletteDto::export_all(&cfg).expect("export AccentPaletteDto bindings");
        PaletteExtractRequest::export_all(&cfg).expect("export PaletteExtractRequest bindings");
        PaletteExtractResult::export_all(&cfg).expect("export PaletteExtractResult bindings");
        PerfStatsDto::export_all(&cfg).expect("export PerfStatsDto bindings");
    }
}
