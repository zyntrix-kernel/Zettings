import 'package:zettings/core/errors/app_error.dart';

/// Availability of a system service on the current machine.
enum ServiceAvailability {
  /// Fully usable.
  available,

  /// Temporarily unreachable (service down, device missing, offline).
  unavailable,

  /// This machine can never provide it (wrong platform, no hardware, no daemon).
  unsupported,

  /// Exists, but the caller is not authorized.
  permissionDenied,

  /// Not probed yet.
  unknown,
}

/// Authorization state for an operation.
enum PermissionState { granted, denied, requiresAuth, unknown }

/// Result of probing one system capability.
///
/// The UI renders exclusively from this: anything that is not [available]
/// must be shown as disabled/with explanation — never as a fake working control.
class SystemCapability {
  const SystemCapability({
    required this.name,
    required this.availability,
    this.permission = PermissionState.unknown,
    this.reason,
  });

  /// Stable identifier, e.g. 'network', 'bluetooth'.
  final String name;

  final ServiceAvailability availability;

  final PermissionState permission;

  /// Human-readable reason shown in the UI when not available.
  final String? reason;

  /// Convenience: throws a typed [AppError] when this capability is required
  /// but not currently usable.
  void require() {
    switch (availability) {
      case ServiceAvailability.available:
        return;
      case ServiceAvailability.unsupported:
        throw AppError(
          AppErrorType.unsupported,
          '$name is not supported on this system.',
          recovery:
              'This feature needs hardware or a service this machine '
              'does not provide.',
        );
      case ServiceAvailability.permissionDenied:
        throw AppError(
          AppErrorType.permissionDenied,
          'Permission denied for $name.',
          recovery:
              'Authorize the request or run with an account that '
              'is allowed to change this setting.',
        );
      case ServiceAvailability.unavailable:
        throw AppError(
          AppErrorType.unavailable,
          '$name is currently unavailable.',
          recovery: reason ?? 'Try again later.',
        );
      case ServiceAvailability.unknown:
        throw AppError(
          AppErrorType.unavailable,
          '$name has not been probed yet.',
        );
    }
  }
}
