import 'package:dbus/dbus.dart';
import 'package:zettings/core/errors/app_error.dart';

/// Full system snapshot. Mirrors the Rust `SystemInfo` struct field-for-field
/// and in wire order (positional D-Bus parsing).
class SystemInfo {
  const SystemInfo({
    required this.osName,
    required this.osVersion,
    required this.kernel,
    required this.hostname,
    required this.desktop,
    required this.sessionType,
    required this.cpuBrand,
    required this.cpuCount,
    required this.cpuFreqMhz,
    required this.memTotalBytes,
    required this.memAvailBytes,
    required this.uptimeSecs,
    required this.gpus,
    required this.disks,
  });

  final String osName;
  final String osVersion;
  final String kernel;
  final String hostname;
  final String desktop;
  final String sessionType;
  final String cpuBrand;
  final int cpuCount;
  final int cpuFreqMhz;
  final int memTotalBytes;
  final int memAvailBytes;
  final int uptimeSecs;
  final List<GpuInfo> gpus;
  final List<DiskInfo> disks;

  /// Parses the `GetSystemInfo` struct reply. Positional and strict: any
  /// shape or type deviation is a contract violation → typed [AppError].
  factory SystemInfo.fromDbus(DBusValue value) {
    if (value is! DBusStruct || value.children.length != 14) {
      throw _malformed(value);
    }
    final c = value.children;
    String stringAt(int i) {
      final v = c[i];
      if (v is! DBusString) throw _malformed(value);
      return v.value;
    }

    int uintAt(int i) {
      final v = c[i];
      if (v is DBusUint32) return v.value;
      if (v is DBusUint64) return v.value;
      throw _malformed(value);
    }

    List<GpuInfo> gpusAt(int i) {
      final v = c[i];
      if (v is! DBusArray) throw _malformed(value);
      return v.children.map(GpuInfo.fromDbus).toList();
    }

    List<DiskInfo> disksAt(int i) {
      final v = c[i];
      if (v is! DBusArray) throw _malformed(value);
      return v.children.map(DiskInfo.fromDbus).toList();
    }

    return SystemInfo(
      osName: stringAt(0),
      osVersion: stringAt(1),
      kernel: stringAt(2),
      hostname: stringAt(3),
      desktop: stringAt(4),
      sessionType: stringAt(5),
      cpuBrand: stringAt(6),
      cpuCount: uintAt(7),
      cpuFreqMhz: uintAt(8),
      memTotalBytes: uintAt(9),
      memAvailBytes: uintAt(10),
      uptimeSecs: uintAt(11),
      gpus: gpusAt(12),
      disks: disksAt(13),
    );
  }

  static AppError _malformed(DBusValue value) => AppError(
    AppErrorType.systemError,
    'Backend sent a malformed system snapshot.',
    details: 'Expected 14-field struct, got: $value',
  );
}

class GpuInfo {
  const GpuInfo({required this.name, required this.vendor});

  final String name;
  final String vendor;

  factory GpuInfo.fromDbus(DBusValue value) {
    if (value is DBusStruct &&
        value.children.length == 2 &&
        value.children.every((c) => c is DBusString)) {
      return GpuInfo(
        name: (value.children[0] as DBusString).value,
        vendor: (value.children[1] as DBusString).value,
      );
    }
    throw AppError(
      AppErrorType.systemError,
      'Backend sent a malformed GPU entry.',
      details: 'Expected struct(ss), got: $value',
    );
  }
}

class DiskInfo {
  const DiskInfo({
    required this.mount,
    required this.totalBytes,
    required this.availBytes,
    required this.fs,
    required this.removable,
  });

  final String mount;
  final int totalBytes;
  final int availBytes;
  final String fs;
  final bool removable;

  factory DiskInfo.fromDbus(DBusValue value) {
    if (value is DBusStruct &&
        value.children.length == 5 &&
        value.children[0] is DBusString &&
        (value.children[1] is DBusUint32 || value.children[1] is DBusUint64) &&
        (value.children[2] is DBusUint32 || value.children[2] is DBusUint64) &&
        value.children[3] is DBusString &&
        value.children[4] is DBusBoolean) {
      int uint(DBusValue v) =>
          v is DBusUint32 ? v.value : (v as DBusUint64).value;
      return DiskInfo(
        mount: (value.children[0] as DBusString).value,
        totalBytes: uint(value.children[1]),
        availBytes: uint(value.children[2]),
        fs: (value.children[3] as DBusString).value,
        removable: (value.children[4] as DBusBoolean).value,
      );
    }
    throw AppError(
      AppErrorType.systemError,
      'Backend sent a malformed disk entry.',
      details: 'Expected struct(sstsb), got: $value',
    );
  }
}

/// Human-readable byte counts (B/KB/MB/GB/TB, 1024-based).
String formatBytes(int bytes) {
  if (bytes < 1024) return '$bytes B';
  const units = ['KB', 'MB', 'GB', 'TB'];
  var value = bytes / 1024.0;
  var unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit++;
  }
  return '${value.toStringAsFixed(1)} ${units[unit]}';
}

/// Human-readable uptime (`3d 4h`, `5h 12m`, `8m 30s`).
String formatUptime(int totalSeconds) {
  final days = totalSeconds ~/ 86400;
  final hours = (totalSeconds % 86400) ~/ 3600;
  final minutes = (totalSeconds % 3600) ~/ 60;
  final seconds = totalSeconds % 60;
  if (days > 0) return '${days}d ${hours}h';
  if (hours > 0) return '${hours}h ${minutes}m';
  if (minutes > 0) return '${minutes}m ${seconds}s';
  return '${seconds}s';
}

/// Empty strings read as an em dash in the UI (never blank rows).
String valueOrDash(String value) => value.isEmpty ? '—' : value;
