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
use std::path::PathBuf;

#[cfg(target_os = "linux")]
fn drm_root() -> PathBuf {
    PathBuf::from("/sys/class/drm")
}

#[cfg(target_os = "linux")]
fn read_modes(dir: &std::path::Path) -> Vec<String> {
    // DRM exposes supported modes as newline-separated entries of the
    // connector's `modes` attribute file.
    let mut modes: Vec<String> = std::fs::read_to_string(dir.join("modes"))
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
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
            .is_ok_and(|m| m.is_dir());
        crate::service_available(exists, SERVICE)
    }

    async fn status(&self) -> Result<DisplayStatus, BackendError> {
        self.status_from(&self.root).await
    }
}

#[cfg(target_os = "linux")]
impl LinuxDisplay {
    /// Binds to an explicit DRM root (tests inject a fixture directory).
    #[cfg(test)]
    pub fn with_root(root: PathBuf) -> Self {
        Self { root }
    }

    async fn status_from(&self, root: &std::path::Path) -> Result<DisplayStatus, BackendError> {
        let mut reader = tokio::fs::read_dir(root)
            .await
            .map_err(|e| BackendError::service(SERVICE, e))?;

        let mut outputs = Vec::new();
        while let Some(entry) = reader
            .next_entry()
            .await
            .map_err(|e| BackendError::service(SERVICE, e))?
        {
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
            if !current_mode.is_empty()
                && let Some(pos) = modes.iter().position(|m| m == &current_mode)
            {
                let current = modes.remove(pos);
                modes.insert(0, current);
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

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn parses_connector_fixture_topology() {
        let fixture = std::env::temp_dir().join(format!("zettings-drm-{}", std::process::id()));
        let card = fixture.join("card0-HDMI-A-1");
        tokio::fs::create_dir_all(&card)
            .await
            .expect("create connector dir");
        tokio::fs::write(card.join("status"), "connected\n")
            .await
            .expect("write status");
        tokio::fs::write(card.join("enabled"), "enabled\n")
            .await
            .expect("write enabled");
        tokio::fs::write(card.join("modes"), "1920x1080\n3840x2160\n1600x900\n")
            .await
            .expect("write modes");

        let display = LinuxDisplay::with_root(fixture.clone());
        let status = display.status().await.expect("status");

        assert_eq!(status.outputs.len(), 1);
        let output = &status.outputs[0];
        assert_eq!(output.name, "card0-HDMI-A-1");
        assert!(output.connected);
        assert_eq!(output.current_mode, "1920x1080");
        // Current mode is surfaced first; the remainder stays lexically sorted.
        assert_eq!(
            output.modes,
            ["1920x1080", "1600x900", "3840x2160"]
                .iter()
                .map(|m| (*m).to_owned())
                .collect::<Vec<String>>()
        );

        tokio::fs::remove_dir_all(&fixture)
            .await
            .expect("cleanup fixture");
    }
}
