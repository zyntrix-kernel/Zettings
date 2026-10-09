import 'package:zettings/core/platform/capability.dart';

/// Contract every system-facing service implements.
///
/// The UI never calls Linux/D-Bus APIs directly; it calls services, and
/// services expose their usability through [probe]. Widgets render from the
/// probed [SystemCapability] and call [SystemCapability.require] before acting.
abstract class SystemService {
  /// Stable capability name, e.g. 'network'. Matches
  /// [SettingDefinition.requiredCapabilities] entries.
  String get name;

  /// Determines current availability and permission state. Must never throw:
  /// probe failures are reported as [ServiceAvailability.unavailable] with a
  /// [SystemCapability.reason], because probing itself must be infallible.
  Future<SystemCapability> probe();
}
