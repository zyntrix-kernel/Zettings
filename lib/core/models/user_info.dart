import 'package:dbus/dbus.dart';
import 'package:zettings/core/errors/app_error.dart';

/// Local human user account info.
/// Mirrors Rust `UserInfo` struct `(ssbss)`.
class UserInfo {
  const UserInfo({
    required this.username,
    required this.realName,
    required this.isAdmin,
    required this.homeDir,
    required this.shell,
  });

  final String username;
  final String realName;
  final bool isAdmin;
  final String homeDir;
  final String shell;

  factory UserInfo.fromDbus(DBusValue value) {
    if (value is! DBusStruct || value.children.length != 5) {
      throw AppError(
        AppErrorType.systemError,
        'Backend sent a malformed user account entry.',
        details: 'Expected 5-field struct (ssbss), got: $value',
      );
    }
    final c = value.children;
    if (c[0] is! DBusString ||
        c[1] is! DBusString ||
        c[2] is! DBusBoolean ||
        c[3] is! DBusString ||
        c[4] is! DBusString) {
      throw AppError(
        AppErrorType.systemError,
        'Backend sent invalid field types for user account.',
        details: 'Expected struct(ssbss), got: $value',
      );
    }

    return UserInfo(
      username: (c[0] as DBusString).value,
      realName: (c[1] as DBusString).value,
      isAdmin: (c[2] as DBusBoolean).value,
      homeDir: (c[3] as DBusString).value,
      shell: (c[4] as DBusString).value,
    );
  }
}
