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

#[cfg(test)]
mod tests {
    //! Cross-target correctness checks for the IPC command surface.
    //! All cases run under the `zettings-mock` feature (the Windows dev loop
    //! default), so they exercise the mock-state backend paths.

    use super::*;

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
    }
}
