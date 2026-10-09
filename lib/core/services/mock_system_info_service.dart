import 'package:zettings/core/models/system_info.dart';
import 'package:zettings/core/platform/capability.dart';
import 'package:zettings/core/services/system_info_service.dart';
import 'package:zettings/core/services/system_service.dart';

/// Development and preview provider for non-Linux platforms (e.g. Windows).
///
/// Provides a realistic Zyntrix OS system snapshot so developers can inspect,
/// verify, and interact with the Material 3 UI on Windows without requiring
/// a running Linux D-Bus session bus.
class MockSystemInfoService implements SystemInfoProvider, SystemService {
  const MockSystemInfoService();

  @override
  String get name => 'system';

  @override
  Future<SystemCapability> probe() async {
    return const SystemCapability(
      name: 'system',
      availability: ServiceAvailability.available,
      reason: 'Mock Development Backend (Windows Preview)',
    );
  }

  @override
  Future<SystemInfo> getSystemInfo() async {
    return const SystemInfo(
      osName: 'Zyntrix OS',
      osVersion: '24.04 LTS (Kubuntu Edition)',
      kernel: 'Linux 6.8.0-45-generic (x86_64)',
      hostname: 'zyntrix-workstation',
      desktop: 'KDE Plasma 6.1.5',
      sessionType: 'Wayland (KWin)',
      cpuBrand: 'AMD Ryzen 9 7950X 16-Core Processor',
      cpuCount: 16,
      cpuFreqMhz: 4500,
      memTotalBytes: 32 * 1024 * 1024 * 1024,
      memAvailBytes: 21 * 1024 * 1024 * 1024,
      uptimeSecs: 14400,
      gpus: [
        GpuInfo(
          name: 'NVIDIA GeForce RTX 4080 (16 GB)',
          vendor: 'NVIDIA Corporation',
        ),
      ],
      disks: [
        DiskInfo(
          mount: '/',
          totalBytes: 1024 * 1024 * 1024 * 1024,
          availBytes: 680 * 1024 * 1024 * 1024,
          fs: 'btrfs',
          removable: false,
        ),
        DiskInfo(
          mount: '/home',
          totalBytes: 2048 * 1024 * 1024 * 1024,
          availBytes: 1420 * 1024 * 1024 * 1024,
          fs: 'btrfs',
          removable: false,
        ),
      ],
    );
  }
}
