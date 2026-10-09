/// Typed error for every fallible operation in ZETTINGS.
///
/// UI shows [message] plus [recovery] where present; [details] is reserved
/// for the expandable diagnostic section and logs — never shown by default.
enum AppErrorType {
  unavailable,
  unsupported,
  permissionDenied,
  invalidInput,
  timeout,
  serviceUnavailable,
  hardwareUnavailable,
  systemError,
  notFound,
}

class AppError implements Exception {
  const AppError(this.type, this.message, {this.recovery, this.details});

  final AppErrorType type;

  /// Human-readable, shown in the UI.
  final String message;

  /// What the user can do next. Shown when present.
  final String? recovery;

  /// Technical diagnostics. Logged, shown only in expandable UI.
  final String? details;

  @override
  String toString() => 'AppError($type): $message';
}
