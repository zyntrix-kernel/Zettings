import 'dart:io';

import 'package:zettings/core/models/system_info.dart';
import 'package:zettings/core/platform/capability.dart';
import 'package:zettings/core/services/system_info_service.dart';
import 'package:zettings/core/services/system_service.dart';

/// Production Linux provider for the system page.
///
/// Reads directly from /proc and /sys (sysfs/procfs) — no D-Bus needed for
/// system info, per the architecture decision in RESEARCH.md §6. Falls back
/// gracefully when any file is unreadable so probe() is always infallible and
/// getSystemInfo() never returns partial garbage.
class LinuxSystemInfoService implements SystemInfoProvider, SystemService {
  const LinuxSystemInfoService();

  @override
  String get name => 'system';

  @override
  Future<SystemCapability> probe() async {
    // System info is readable on any Linux host; only a missing /proc makes
    // it unsupported. Probe never throws.
    try {
      if (!File('/proc/version').existsSync()) {
        return const SystemCapability(
          name: 'system',
          availability: ServiceAvailability.unsupported,
          reason: 'This host is not Linux (no /proc filesystem).',
        );
      }
      return const SystemCapability(
        name: 'system',
        availability: ServiceAvailability.available,
        reason: 'Linux /proc + /sys readable.',
      );
    } catch (e) {
      return SystemCapability(
        name: 'system',
        availability: ServiceAvailability.unavailable,
        reason: '$e',
      );
    }
  }

  @override
  Future<SystemInfo> getSystemInfo() async {
    final os = await _readOsRelease();
    final kernel = await _readString('/proc/version').then((v) => v.trim());
    final hostname = await _readString('/proc/sys/kernel/hostname')
        .then((v) => v.trim());
    final desktop = Platform.environment['XDG_CURRENT_DESKTOP'] ??
        Platform.environment['DESKTOP_SESSION'] ??
        'Unknown';
    final sessionType =
        Platform.environment['XDG_SESSION_TYPE'] ?? 'Unknown';

    final cpu = await _readCpu();
    final mem = await _readMem();
    final uptime = await _readUptime();

    return SystemInfo(
      osName: os['name'] ?? 'Linux',
      osVersion: os['version'] ?? 'Unknown',
      kernel: kernel.isEmpty ? 'Unknown' : kernel,
      hostname: hostname.isEmpty ? 'localhost' : hostname,
      desktop: desktop,
      sessionType: sessionType,
      cpuBrand: cpu['brand'] ?? 'Unknown CPU',
      cpuCount: cpu['count'] ?? 0,
      cpuFreqMhz: cpu['freqMhz'] ?? 0,
      memTotalBytes: mem['totalBytes'] ?? 0,
      memAvailBytes: mem['availBytes'] ?? 0,
      uptimeSecs: uptime,
      gpus: const [],
      disks: const [],
    );
  }

  /// Reads /etc/os-release into {name, version}.
  Future<Map<String, String>> _readOsRelease() async {
    final result = <String, String>{};
    try {
      final text = await File('/etc/os-release').readAsString();
      for (final line in text.split('\n')) {
        final eq = line.indexOf('=');
        if (eq < 0) continue;
        final key = line.substring(0, eq).trim();
        var value = line.substring(eq + 1).trim();
        // Strip surrounding quotes.
        if (value.startsWith('"') && value.endsWith('"') && value.length > 1) {
          value = value.substring(1, value.length - 1);
        }
        if (key == 'PRETTY_NAME') {
          result['name'] = value;
          result['version'] = value;
        } else if (key == 'NAME' && result['name'] == null) {
          result['name'] = value;
        } else if (key == 'VERSION_ID' && result['version'] == null) {
          result['version'] = value;
        } else if (key == 'VERSION' && result['version'] == null) {
          result['version'] = value;
        }
      }
    } catch (_) {
      // /etc/os-release missing — keep defaults.
    }
    return result;
  }

  Future<String> _readString(String path) async {
    try {
      return await File(path).readAsString();
    } catch (_) {
      return '';
    }
  }

  /// Parses /proc/stat: counts "processor N" lines, grabs the first "cpu MHz".
  Future<Map<String, int>> _readCpu() async {
    var count = 0;
    var freqMhz = 0;
    var brand = 'Unknown CPU';
    try {
      final text = await File('/proc/stat').readAsString();
      for (final line in text.split('\n')) {
        if (line.startsWith('processor')) count++;
        if (freqMhz == 0 && line.startsWith('cpu MHz')) {
          // "cpu MHz   : 4500.000"
          final colon = line.indexOf(':');
          if (colon > 0) {
            final num = line.substring(colon + 1).trim();
            freqMhz = num.split('.').first.isNotEmpty
                ? int.tryParse(num.split('.').first) ?? 0
                : 0;
          }
        }
      }
    } catch (_) {}
    return {'count': count, 'freqMhz': freqMhz, 'brand': brand};
  }

  /// Parses /proc/meminfo: MemTotal and MemAvailable (kB).
  Future<Map<String, int>> _readMem() async {
    var totalBytes = 0;
    var availBytes = 0;
    try {
      final text = await File('/proc/meminfo').readAsString();
      for (final line in text.split('\n')) {
        if (line.startsWith('MemTotal:')) {
          totalBytes = _memKbToBytes(line);
        } else if (line.startsWith('MemAvailable:')) {
          availBytes = _memKbToBytes(line);
        }
      }
    } catch (_) {}
    return {'totalBytes': totalBytes, 'availBytes': availBytes};
  }

  static int _memKbToBytes(String line) {
    final digits = line.replaceAll(RegExp(r'[^0-9]'), '');
    final kb = int.tryParse(digits) ?? 0;
    return kb * 1024;
  }

  /// Parses /proc/uptime: first field is seconds (may be fractional).
  Future<int> _readUptime() async {
    try {
      final text = await File('/proc/uptime').readAsString();
      final first = text.trim().split(' ').first;
      final seconds = double.tryParse(first) ?? 0;
      return seconds.round();
    } catch (_) {
      return 0;
    }
  }
}
