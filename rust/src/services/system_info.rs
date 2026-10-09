//! System information: OS, kernel, CPU, memory, disks, GPUs, session.
//!
//! Sources, in order of preference:
//! * [`sysinfo`] crate — cross-platform CPU/memory/disks, units verified
//!   against upstream docs (bytes for memory and disk sizes).
//! * `/etc/os-release` — distribution name/version on Linux.
//! * `XDG_CURRENT_DESKTOP` / `XDG_SESSION_TYPE` — desktop environment.
//! * `lspci -mm` — GPU names; strictly parsed, absence means "no GPUs
//!   reported", never an error.
//!
//! Snapshots are cached for [`CACHE_TTL`] — hardware inventory does not
//! change second to second, and refresh walks /proc + /sys.

use std::collections::HashMap;
use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use sysinfo::{Disks, System};

use super::{Capability, ZettingsService, availability};
use crate::models::{DiskInfo, GpuInfo, SystemInfo};

/// Freshness horizon for cached snapshots.
const CACHE_TTL: Duration = Duration::from_secs(5);

pub struct SystemInfoService {
    cache: Mutex<Option<(SystemInfo, Instant)>>,
}

impl SystemInfoService {
    pub fn new() -> Self {
        Self {
            cache: Mutex::new(None),
        }
    }

    /// Current snapshot, cached for [`CACHE_TTL`].
    pub fn get(&self) -> SystemInfo {
        // Poison-tolerant: a poisoned mutex still yields its last snapshot
        // instead of panicking the D-Bus handler.
        let mut guard = self
            .cache
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some((info, at)) = guard.as_ref()
            && at.elapsed() < CACHE_TTL
        {
            return info.clone();
        }
        let info = collect();
        *guard = Some((info.clone(), Instant::now()));
        info
    }
}

impl Default for SystemInfoService {
    fn default() -> Self {
        Self::new()
    }
}

impl ZettingsService for SystemInfoService {
    fn name(&self) -> &'static str {
        "system"
    }

    fn probe(&self) -> Capability {
        // Local, unprivileged data collection: always available.
        Capability {
            name: self.name().to_string(),
            availability: availability::AVAILABLE.to_string(),
            reason: "sysinfo + os-release".to_string(),
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// One full collection pass. Infallible: every source degrades to empty or
/// `"Unknown"` rather than failing the call.
fn collect() -> SystemInfo {
    let os_release = read_os_release();
    let mut sys = System::new_all();
    sys.refresh_memory();
    sys.refresh_cpu_all();

    let cpus = sys.cpus();
    let (cpu_brand, cpu_freq_mhz) = cpus
        .first()
        .map(|cpu| (cpu.brand().to_string(), cpu.frequency()))
        .unwrap_or_else(|| ("Unknown".to_string(), 0));

    let disks = Disks::new_with_refreshed_list();
    let disk_infos = disks
        .iter()
        .map(|disk| DiskInfo {
            mount: disk.mount_point().to_string_lossy().into_owned(),
            total_bytes: disk.total_space(),
            avail_bytes: disk.available_space(),
            fs: disk.file_system().to_string_lossy().into_owned(),
            removable: disk.is_removable(),
        })
        .collect();

    SystemInfo {
        os_name: os_release
            .get("NAME")
            .cloned()
            .or_else(System::name)
            .unwrap_or_else(|| "Unknown".to_string()),
        os_version: os_release
            .get("VERSION")
            .or_else(|| os_release.get("VERSION_ID"))
            .cloned()
            .or_else(System::os_version)
            .unwrap_or_default(),
        kernel: System::kernel_version().unwrap_or_else(|| "Unknown".to_string()),
        hostname: System::host_name().unwrap_or_else(|| "Unknown".to_string()),
        desktop: std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default(),
        session_type: std::env::var("XDG_SESSION_TYPE").unwrap_or_default(),
        cpu_brand,
        cpu_count: cpus.len() as u32,
        cpu_freq_mhz,
        mem_total_bytes: sys.total_memory(),
        mem_avail_bytes: sys.available_memory(),
        uptime_secs: System::uptime(),
        gpus: detect_gpus(),
        disks: disk_infos,
    }
}

/// Parses `/etc/os-release` into KEY → value. Missing file → empty map.
fn read_os_release() -> HashMap<String, String> {
    let text = std::fs::read_to_string("/etc/os-release").unwrap_or_default();
    parse_os_release(&text)
}

/// Pure parser for os-release format. Unit-tested below.
fn parse_os_release(text: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            let value = value.trim().trim_matches('"').trim_matches('\'');
            map.insert(key.trim().to_string(), value.to_string());
        }
    }
    map
}

/// GPU inventory via `lspci -mm`. Fixed binary, fixed argument, strictly
/// parsed output; any failure (missing binary included) yields an empty list.
fn detect_gpus() -> Vec<GpuInfo> {
    let output = Command::new("lspci").arg("-mm").output();
    let Ok(output) = output else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }
    let text = String::from_utf8_lossy(&output.stdout);
    parse_lspci_mm(&text)
}

/// Pure parser for `lspci -mm` lines. Keeps VGA/3D/Display controllers.
///
/// Sample line:
/// `00:02.0 "VGA compatible controller" "Intel Corporation" "Alder Lake-P"`
fn parse_lspci_mm(text: &str) -> Vec<GpuInfo> {
    let mut gpus = Vec::new();
    for line in text.lines() {
        let fields = split_quoted(line);
        if fields.len() < 4 {
            continue;
        }
        let class = fields[1].to_lowercase();
        if class.contains("vga") || class.contains("3d") || class.contains("display") {
            gpus.push(GpuInfo {
                vendor: fields[2].clone(),
                name: fields[3].clone(),
            });
        }
    }
    gpus
}

/// Splits a `-mm` line into its slot plus quoted fields.
fn split_quoted(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    for ch in line.chars() {
        match ch {
            '"' => {
                if in_quotes {
                    fields.push(std::mem::take(&mut current));
                }
                in_quotes = !in_quotes;
            }
            _ if in_quotes => current.push(ch),
            _ if !current.is_empty() || !fields.is_empty() => {
                if ch.is_whitespace() {
                    if !current.is_empty() {
                        fields.push(std::mem::take(&mut current));
                    }
                } else {
                    current.push(ch);
                }
            }
            _ => current.push(ch),
        }
    }
    if !current.is_empty() {
        fields.push(current);
    }
    fields
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn os_release_parses_quoted_values_and_skips_comments() {
        let map = parse_os_release(
            "# comment\nNAME=\"Kubuntu\"\nVERSION='24.04 LTS'\nID=kubuntu\nEMPTY=\n",
        );
        assert_eq!(map.get("NAME").map(String::as_str), Some("Kubuntu"));
        assert_eq!(map.get("VERSION").map(String::as_str), Some("24.04 LTS"));
        assert_eq!(map.get("ID").map(String::as_str), Some("kubuntu"));
        assert_eq!(map.get("EMPTY").map(String::as_str), Some(""));
        assert_eq!(map.len(), 4);
    }

    #[test]
    fn lspci_keeps_only_display_controllers() {
        let text = "00:02.0 \"VGA compatible controller\" \"Intel Corporation\" \"Alder Lake-P GT2\"\n\
             00:14.0 \"USB controller\" \"Intel Corporation\" \"Alder Lake USB\"\n\
             01:00.0 \"3D controller\" \"NVIDIA Corporation\" \"GA107M [GeForce RTX 3050]\"\n";
        let gpus = parse_lspci_mm(text);
        assert_eq!(gpus.len(), 2);
        assert_eq!(gpus[0].vendor, "Intel Corporation");
        assert_eq!(gpus[0].name, "Alder Lake-P GT2");
        assert_eq!(gpus[1].vendor, "NVIDIA Corporation");
    }

    #[test]
    fn lspci_garbage_yields_no_gpus() {
        assert!(parse_lspci_mm("").is_empty());
        assert!(parse_lspci_mm("not a pci line\n").is_empty());
    }

    #[test]
    fn collect_smoke_reports_sane_values_on_any_machine() {
        // Runs on every platform sysinfo supports (incl. Windows CI).
        let info = collect();
        assert!(!info.kernel.is_empty());
        assert!(info.mem_total_bytes > 0, "machine must report RAM");
        // Cache path returns the same snapshot within the TTL.
        let service = SystemInfoService::new();
        let first = service.get();
        let second = service.get();
        assert_eq!(first, second);
    }
}
