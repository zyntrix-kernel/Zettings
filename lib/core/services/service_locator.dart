import 'package:zettings/core/errors/app_error.dart';

/// Manual dependency-injection registry (stdlib only).
///
/// A package like get_it would add API surface for zero current benefit:
/// registration, lookup, and test reset are all covered below.
class ServiceLocator {
  static final Map<Type, Object> _services = {};

  /// Registers [instance] for type [T]. Overwrites any previous registration.
  static void register<T extends Object>(T instance) {
    _services[T] = instance;
  }

  /// Returns the registered instance for [T].
  ///
  /// Throws [AppError] with [AppErrorType.systemError] when missing, so a
  /// missing registration is an explicit diagnostic instead of a null crash.
  static T get<T extends Object>() {
    final service = _services[T];
    if (service == null) {
      throw AppError(
        AppErrorType.systemError,
        'Service $T is not registered.',
        recovery:
            'This is an application bug. Restart Zettings; if it '
            'persists, report it with the diagnostics below.',
        details: 'ServiceLocator.get<$T> found no registration.',
      );
    }
    return service as T;
  }

  /// True when a registration for [T] exists.
  static bool isRegistered<T extends Object>() => _services.containsKey(T);

  /// All registered instances implementing [T]. Used to discover system
  /// services (e.g. every [SystemService]) without naming each one.
  static List<T> all<T extends Object>() =>
      _services.values.whereType<T>().toList();

  /// Clears all registrations. Intended for tests.
  static void reset() => _services.clear();
}
