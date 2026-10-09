import 'package:flutter/foundation.dart';

/// Minimal structured logger. stdlib only (debugPrint); no package needed.
enum LogLevel { debug, info, warning, error }

class ZettingsLog {
  static void _log(LogLevel level, String message, [Object? error]) {
    final suffix = error == null ? '' : ' | $error';
    debugPrint('[zettings][${level.name}] $message$suffix');
  }

  static void debug(String message) => _log(LogLevel.debug, message);

  static void info(String message) => _log(LogLevel.info, message);

  static void warning(String message, [Object? error]) =>
      _log(LogLevel.warning, message, error);

  static void error(String message, [Object? error]) =>
      _log(LogLevel.error, message, error);
}
