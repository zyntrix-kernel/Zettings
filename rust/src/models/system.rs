//! System hardware, OS snapshot, GPU, and disk mount models.

use serde::{Deserialize, Serialize};
use zvariant::Type;

/// Full system snapshot served by `GetSystemInfo`.
///
/// Field order is the D-Bus wire order; the Dart parser mirrors it
/// positionally (see `SystemInfo.fromDbus`). Only append new fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct SystemInfo {
    pub os_name: String,
    pub os_version: String,
    pub kernel: String,
    pub hostname: String,
    pub desktop: String,
    pub session_type: String,
    pub cpu_brand: String,
    pub cpu_count: u32,
    pub cpu_freq_mhz: u64,
    pub mem_total_bytes: u64,
    pub mem_avail_bytes: u64,
    pub uptime_secs: u64,
    pub gpus: Vec<GpuInfo>,
    pub disks: Vec<DiskInfo>,
}

/// One graphics device.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct GpuInfo {
    pub name: String,
    pub vendor: String,
}

/// One mounted filesystem.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct DiskInfo {
    pub mount: String,
    pub total_bytes: u64,
    pub avail_bytes: u64,
    pub fs: String,
    pub removable: bool,
}
