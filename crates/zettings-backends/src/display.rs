//! Display output enumeration via kernel DRM sysfs (read-only v1).
//!
//! Mode changes require `KScreen`/`KWin` and land with the `KScreen` adapter;
//! this module provides the honest, dependency-free baseline the UI can
//! already render (connector status, modes, current mode).

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use zettings_core::CapabilityState;

use crate::error::BackendError;

#[cfg(target_os = "linux")]
const SERVICE: &str = "drm";

/// One physical connector.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DisplayOutput {
    /// DRM connector name (e.g. `card0-HDMI-A-1`).
    pub name: String,
    /// Whether a display is connected.
    pub connected: bool,
    /// Supported mode strings (e.g. `1920x1080`), best first as reported.
    pub modes: Vec<String>,
    /// Currently active mode; empty when off/disconnected.
    pub current_mode: String,
}

/// Read-only display topology.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DisplayStatus {
    /// Honest capability of the adapter.
    pub capability: CapabilityState,
    /// Detected connectors.
    pub outputs: Vec<DisplayOutput>,
}

/// Display enumeration surface.
#[async_trait]
pub trait DisplayAdapter: Send + Sync {
    /// Detects whether any DRM connectors exist.
    async fn capability(&self) -> CapabilityState;
    /// Reads the connector/mode topology.
    ///
    /// # Errors
    /// [`BackendError::Service`] when the kernel interfaces are unreadable.
    async fn status(&self) -> Result<DisplayStatus, BackendError>;
}

#[cfg(target_os = "linux")]
fn drm_root() -> PathBuf {
    PathBuf::from("/sys/class/drm")
}

#[cfg(target_os = "linux")]
fn read_modes(dir: &std::path::Path) -> Vec<String> {
    let mut modes: Vec<String> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| name.starts_with("mode-"))
        .map(|name| name.trim_start_matches("mode-").replace('_', ":"))
        .collect();
    // Kernel emits preferred/active first via attributes; keep lexical order
    // stable for tests while deduplicating.
    modes.sort();
    modes.dedup();
    modes
}

#[cfg(target_os = "linux")]
fn parse_status(raw: &str) -> bool {
    raw.trim() == "connected"
}

/// Deterministic mock.
#[derive(Debug, Default)]
pub struct MockDisplay {
    state: tokio::sync::Mutex<Option<DisplayStatus>>,
}

#[async_trait]
impl DisplayAdapter for MockDisplay {
    async fn capability(&self) -> CapabilityState {
        CapabilityState::Available
    }

    async fn status(&self) -> Result<DisplayStatus, BackendError> {
        let mut guard = self.state.lock().await;
        if guard.is_none() {
            *guard = Some(DisplayStatus {
                capability: CapabilityState::Available,
                outputs: vec![DisplayOutput {
                    name: "card0-eDP-1".to_owned(),
                    connected: true,
                    modes: vec![
                        "1920x1080".to_owned(),
                        "1600x900".to_owned(),
                        "1366x768".to_owned(),
                    ],
                    current_mode: "1920x1080".to_owned(),
                }],
            });
        }
        Ok(guard.clone().expect("seeded above"))
    }
}

/// Real adapter reading kernel DRM state (Linux only).
#[cfg(target_os = "linux")]
pub struct LinuxDisplay {
    root: PathBuf,
}

#[cfg(target_os = "linux")]
impl Default for LinuxDisplay {
    fn default() -> Self {
        Self { root: drm_root() }
    }
}

#[cfg(target_os = "linux")]
#[async_trait]
impl DisplayAdapter for LinuxDisplay {
    async fn capability(&self) -> CapabilityState {
        let exists = tokio::fs::metadata(&self.root)
            .await
            .map(|m| m.is_dir())
            .unwrap_or(false);
        crate::service_available(exists, SERVICE)
    }

    async fn status(&self) -> Result<DisplayStatus, BackendError> {
        self.status_from(&self.root).await
    }
}

#[cfg(target_os = "linux")]
impl LinuxDisplay {
    /// Binds to an explicit DRM root (tests inject a fixture directory).
    pub fn with_root(root: PathBuf) -> Self {
        Self { root }
    }

    async fn status_from(&self, root: &std::path::Path) -> Result<DisplayStatus, BackendError> {
        let entries = tokio::fs::read_dir(root)
            .await
            .map_err(|e| BackendError::service(SERVICE, e))?
            .entries()
            .collect::<Result<Vec<_>, _>>()
            .await
            .map_err(|e| BackendError::service(SERVICE, e))?;

        let mut outputs = Vec::new();
        for entry in entries {
            let file_name = entry.file_name().into_string().unwrap_or_default();
            if !file_name.contains('-') {
                continue; // skip cardN dirs
            }
            let dir = entry.path();
            let status_raw = tokio::fs::read_to_string(dir.join("status"))
                .await
                .unwrap_or_default();
            let connected = parse_status(&status_raw);
            if !connected && !dir.join("modes").exists() {
                continue; // disconnected and no mode history → not useful yet
            }
            let enabled_raw = tokio::fs::read_to_string(dir.join("enabled"))
                .await
                .unwrap_or_default();
            let mut modes = read_modes(&dir);
            let current_mode = if enabled_raw.trim() == "enabled" {
                let mode_file = tokio::fs::read_to_string(dir.join("modes"))
                    .await
                    .unwrap_or_default();
                mode_file
                    .lines()
                    .next()
                    .map(str::trim)
                    .map(str::to_owned)
                    .unwrap_or_default()
            } else {
                String::new()
            };
            // Preferred mode marker lives in the `modes` attribute's first
            // line when enabled; surface it at position 0 otherwise keep list.
            if !current_mode.is_empty() {
                if let Some(pos) = modes.iter().position(|m| m == &current_mode) {
                    let current = modes.remove(pos);
                    modes.insert(0, current);
                }
            }
            outputs.push(DisplayOutput {
                name: file_name,
                connected,
                modes,
                current_mode,
            });
        }
        Ok(DisplayStatus {
            capability: crate::service_available(!outputs.is_empty(), SERVICE),
            outputs,
        })
    }
}
