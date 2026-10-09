import 'package:dbus/dbus.dart';
import 'package:zettings/core/errors/app_error.dart';

/// System master audio routing and volume status.
/// Mirrors Rust `AudioStatus` struct `(dbss)`.
class AudioStatus {
  const AudioStatus({
    required this.masterVolume,
    required this.muted,
    required this.defaultSink,
    required this.defaultSource,
  });

  final double masterVolume;
  final bool muted;
  final String defaultSink;
  final String defaultSource;

  factory AudioStatus.fromDbus(DBusValue value) {
    if (value is! DBusStruct || value.children.length != 4) {
      throw AppError(
        AppErrorType.systemError,
        'Backend sent a malformed audio status reply.',
        details: 'Expected 4-field struct (dbss), got: $value',
      );
    }
    final c = value.children;
    if (c[0] is! DBusDouble ||
        c[1] is! DBusBoolean ||
        c[2] is! DBusString ||
        c[3] is! DBusString) {
      throw AppError(
        AppErrorType.systemError,
        'Backend sent invalid field types for audio status.',
        details: 'Expected struct(dbss), got: $value',
      );
    }

    return AudioStatus(
      masterVolume: (c[0] as DBusDouble).value,
      muted: (c[1] as DBusBoolean).value,
      defaultSink: (c[2] as DBusString).value,
      defaultSource: (c[3] as DBusString).value,
    );
  }
}
