import 'package:dbus/dbus.dart';
import 'package:zettings/core/errors/app_error.dart';

/// System power, battery, and power-profile status.
/// Mirrors Rust `PowerStatus` struct `(bdstsas)`.
class PowerStatus {
  const PowerStatus({
    required this.onBattery,
    required this.batteryPercentage,
    required this.chargingState,
    required this.timeToEmptySecs,
    required this.activeProfile,
    required this.availableProfiles,
  });

  final bool onBattery;
  final double batteryPercentage;
  final String chargingState;
  final int timeToEmptySecs;
  final String activeProfile;
  final List<String> availableProfiles;

  factory PowerStatus.fromDbus(DBusValue value) {
    if (value is! DBusStruct || value.children.length != 6) {
      throw AppError(
        AppErrorType.systemError,
        'Backend sent a malformed power status reply.',
        details: 'Expected 6-field struct (bdstsas), got: $value',
      );
    }
    final c = value.children;
    if (c[0] is! DBusBoolean ||
        c[1] is! DBusDouble ||
        c[2] is! DBusString ||
        (c[3] is! DBusUint64 && c[3] is! DBusUint32) ||
        c[4] is! DBusString ||
        c[5] is! DBusArray) {
      throw AppError(
        AppErrorType.systemError,
        'Backend sent invalid field types for power status.',
        details: 'Expected struct(bdstsas), got: $value',
      );
    }

    final time = c[3] is DBusUint64
        ? (c[3] as DBusUint64).value
        : (c[3] as DBusUint32).value;

    return PowerStatus(
      onBattery: (c[0] as DBusBoolean).value,
      batteryPercentage: (c[1] as DBusDouble).value,
      chargingState: (c[2] as DBusString).value,
      timeToEmptySecs: time,
      activeProfile: (c[4] as DBusString).value,
      availableProfiles: (c[5] as DBusArray).children
          .whereType<DBusString>()
          .map((s) => s.value)
          .toList(),
    );
  }
}
