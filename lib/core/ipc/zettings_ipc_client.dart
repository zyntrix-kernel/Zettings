import 'package:dbus/dbus.dart';
import 'package:zettings/core/errors/app_error.dart';
import 'package:zettings/core/models/audio_status.dart';
import 'package:zettings/core/models/bluetooth_status.dart';
import 'package:zettings/core/models/network_status.dart';
import 'package:zettings/core/models/power_status.dart';
import 'package:zettings/core/models/system_info.dart';
import 'package:zettings/core/models/user_info.dart';

/// One capability reported by the backend daemon. Mirrors the Rust
/// `Capability` struct (`name`, `availability`, `reason`).
class RemoteCapability {
  const RemoteCapability({
    required this.name,
    required this.availability,
    required this.reason,
  });

  final String name;
  final String availability;
  final String reason;

  /// Parses one `DBusStruct(DBusString, DBusString, DBusString)`.
  /// Anything else is a contract violation → typed [AppError].
  factory RemoteCapability.fromDbus(DBusValue value) {
    if (value is DBusStruct &&
        value.children.length == 3 &&
        value.children.every((c) => c is DBusString)) {
      final fields = value.children
          .map((c) => (c as DBusString).value)
          .toList();
      return RemoteCapability(
        name: fields[0],
        availability: fields[1],
        reason: fields[2],
      );
    }
    throw AppError(
      AppErrorType.systemError,
      'Backend sent a malformed capability.',
      recovery:
          'The backend may be a different version. '
          'Restart it and try again.',
      details: 'Expected struct(sss), got: $value',
    );
  }
}

/// Typed client for the `org.zyntrix.Zettings` session-bus daemon.
///
/// Methods mirror the Rust `#[interface]` exactly. Transport failures become
/// typed [AppError]s; the `code: detail` wire prefix recovers the exact
/// backend kind (see `rust/src/error.rs`).
class ZettingsIpcClient {
  ZettingsIpcClient({DBusClient? client}) : _injected = client;

  static const busName = 'org.zyntrix.Zettings';
  static final objectPath = DBusObjectPath('/org/zyntrix/Zettings');
  static const interface = 'org.zyntrix.Zettings';

  final DBusClient? _injected;
  DBusClient? _created;

  DBusClient get _effective => _injected ?? (_created ??= DBusClient.session());

  Future<List<DBusValue>> _call(
    String method,
    List<DBusValue> args,
    DBusSignature replySignature,
  ) async {
    final object = DBusRemoteObject(
      _effective,
      name: busName,
      path: objectPath,
    );
    try {
      final reply = await object.callMethod(
        interface,
        method,
        args,
        replySignature: replySignature,
      );
      return reply.returnValues;
    } on DBusServiceUnknownException catch (e) {
      throw AppError(
        AppErrorType.serviceUnavailable,
        'Zettings backend is not running.',
        recovery: 'Start the zettings-backend daemon and try again.',
        details: e.toString(),
      );
    } on DBusMethodResponseException catch (e) {
      throw translateMethodError(e);
    }
  }

  static String _expectString(List<DBusValue> values, String method) {
    if (values.length == 1 && values.single is DBusString) {
      return (values.single as DBusString).value;
    }
    throw AppError(
      AppErrorType.systemError,
      'Backend sent a malformed $method reply.',
      details: 'Expected single string, got: $values',
    );
  }

  /// Liveness check. Returns e.g. `"zettings-backend 0.1.0"`.
  Future<String> ping() async =>
      _expectString(await _call('Ping', [], DBusSignature('s')), 'Ping');

  /// Backend crate version string.
  Future<String> getVersion() async => _expectString(
    await _call('GetVersion', [], DBusSignature('s')),
    'GetVersion',
  );

  /// Availability of every registered backend integration.
  Future<List<RemoteCapability>> getCapabilities() async {
    final values = await _call('GetCapabilities', [], DBusSignature('a(sss)'));
    if (values.length != 1) {
      throw AppError(
        AppErrorType.systemError,
        'Backend sent a malformed GetCapabilities reply.',
        details: 'Expected single array, got: $values',
      );
    }
    final first = values.single;
    if (first is! DBusArray) {
      throw AppError(
        AppErrorType.systemError,
        'Backend sent a malformed GetCapabilities reply.',
        details: 'Expected single array, got: $values',
      );
    }
    return first.children.map(RemoteCapability.fromDbus).toList();
  }

  /// Full system snapshot. The reply signature mirrors the Rust struct:
  /// 7 strings, 5 uints, a GPU array, then a disk array.
  Future<SystemInfo> getSystemInfo() async {
    final values = await _call(
      'GetSystemInfo',
      [],
      DBusSignature('(sssssssutttta(ss)a(sttsb))'),
    );
    if (values.length != 1) {
      throw AppError(
        AppErrorType.systemError,
        'Backend sent a malformed GetSystemInfo reply.',
        details: 'Expected single struct, got: $values',
      );
    }
    return SystemInfo.fromDbus(values.single);
  }

  /// Full network status and connection info.
  Future<NetworkStatus> getNetworkStatus() async {
    final values = await _call(
      'GetNetworkStatus',
      [],
      DBusSignature('(bbsssya(sybs))'),
    );
    if (values.length != 1) {
      throw AppError(
        AppErrorType.systemError,
        'Backend sent a malformed GetNetworkStatus reply.',
        details: 'Expected single struct, got: $values',
      );
    }
    return NetworkStatus.fromDbus(values.single);
  }

  /// Enable or disable Wi-Fi subsystem.
  Future<void> setWifiEnabled(bool enabled) async {
    await _call('SetWifiEnabled', [DBusBoolean(enabled)], DBusSignature(''));
  }

  /// Bluetooth adapter and peripheral status.
  Future<BluetoothStatus> getBluetoothStatus() async {
    final values = await _call(
      'GetBluetoothStatus',
      [],
      DBusSignature('(bsbua(ssbbis))'),
    );
    if (values.length != 1) {
      throw AppError(
        AppErrorType.systemError,
        'Backend sent a malformed GetBluetoothStatus reply.',
        details: 'Expected single struct, got: $values',
      );
    }
    return BluetoothStatus.fromDbus(values.single);
  }

  /// Enable or disable Bluetooth adapter.
  Future<void> setBluetoothEnabled(bool enabled) async {
    await _call('SetBluetoothEnabled', [
      DBusBoolean(enabled),
    ], DBusSignature(''));
  }

  /// System power, battery, and power-profile status.
  Future<PowerStatus> getPowerStatus() async {
    final values = await _call(
      'GetPowerStatus',
      [],
      DBusSignature('(bdstsas)'),
    );
    if (values.length != 1) {
      throw AppError(
        AppErrorType.systemError,
        'Backend sent a malformed GetPowerStatus reply.',
        details: 'Expected single struct, got: $values',
      );
    }
    return PowerStatus.fromDbus(values.single);
  }

  /// Switch active power profile (e.g. "performance", "balanced", "power-saver").
  Future<void> setPowerProfile(String profile) async {
    await _call('SetPowerProfile', [DBusString(profile)], DBusSignature(''));
  }

  /// Master audio routing and volume status.
  Future<AudioStatus> getAudioStatus() async {
    final values = await _call('GetAudioStatus', [], DBusSignature('(dbss)'));
    if (values.length != 1) {
      throw AppError(
        AppErrorType.systemError,
        'Backend sent a malformed GetAudioStatus reply.',
        details: 'Expected single struct, got: $values',
      );
    }
    return AudioStatus.fromDbus(values.single);
  }

  /// Set master audio volume (0.0 .. 1.0).
  Future<void> setMasterVolume(double volume) async {
    await _call('SetMasterVolume', [DBusDouble(volume)], DBusSignature(''));
  }

  /// Set master audio mute state.
  Future<void> setMasterMute(bool muted) async {
    await _call('SetMasterMute', [DBusBoolean(muted)], DBusSignature(''));
  }

  /// Local human user accounts.
  Future<List<UserInfo>> getUsers() async {
    final values = await _call('GetUsers', [], DBusSignature('a(ssbss)'));
    if (values.length != 1 || values.single is! DBusArray) {
      throw AppError(
        AppErrorType.systemError,
        'Backend sent a malformed GetUsers reply.',
        details: 'Expected single array, got: $values',
      );
    }
    return (values.single as DBusArray).children
        .map(UserInfo.fromDbus)
        .toList();
  }

  Future<void> close() async {
    await _created?.close();
    _created = null;
  }
}

/// Translates a D-Bus method error into a typed [AppError], recovering the
/// backend `code: detail` prefix first, then standard D-Bus error names.
AppError translateMethodError(DBusMethodResponseException e) {
  // `response` is statically the error variant; no check needed.
  final response = e.response;
  final message = response.values
      .whereType<DBusString>()
      .map((s) => s.value)
      .join(' ');
  return ipcErrorToAppError(
    dbusErrorName: response.errorName,
    message: message,
  );
}

/// Pure error-code mapping. Unit-tested without a bus.
AppError ipcErrorToAppError({
  required String dbusErrorName,
  required String message,
}) {
  final match = RegExp(
    r'^([A-Za-z]+):\s*(.*)$',
    dotAll: true,
  ).firstMatch(message);
  final code = match?.group(1) ?? '';
  final detail = match?.group(2) ?? '';
  final type = _codeToType(code);
  if (type != null) {
    return AppError(
      type,
      detail.isEmpty ? code : detail,
      recovery: _recoveryFor(type),
      details: dbusErrorName.isEmpty ? null : '$dbusErrorName: $message',
    );
  }
  switch (dbusErrorName) {
    case 'org.freedesktop.DBus.Error.ServiceUnknown':
      return AppError(
        AppErrorType.serviceUnavailable,
        'Zettings backend is not running.',
        recovery: 'Start the zettings-backend daemon and try again.',
        details: message.isEmpty ? null : message,
      );
    case 'org.freedesktop.DBus.Error.AccessDenied':
      return AppError(
        AppErrorType.permissionDenied,
        message.isEmpty ? 'Permission denied.' : message,
        recovery: _recoveryFor(AppErrorType.permissionDenied),
      );
    case 'org.freedesktop.DBus.Error.UnknownMethod':
    case 'org.freedesktop.DBus.Error.UnknownInterface':
    case 'org.freedesktop.DBus.Error.UnknownObject':
      return AppError(
        AppErrorType.systemError,
        'Backend does not implement this call.',
        recovery: 'The backend may be outdated. Restart it and try again.',
        details: '$dbusErrorName $message'.trim(),
      );
    default:
      return AppError(
        AppErrorType.systemError,
        message.isEmpty ? dbusErrorName : message,
        details: dbusErrorName.isEmpty ? null : dbusErrorName,
      );
  }
}

AppErrorType? _codeToType(String code) => switch (code) {
  'unavailable' => AppErrorType.unavailable,
  'unsupported' => AppErrorType.unsupported,
  'permissionDenied' => AppErrorType.permissionDenied,
  'invalidInput' => AppErrorType.invalidInput,
  'timeout' => AppErrorType.timeout,
  'serviceUnavailable' => AppErrorType.serviceUnavailable,
  'hardwareUnavailable' => AppErrorType.hardwareUnavailable,
  'systemError' => AppErrorType.systemError,
  _ => null,
};

String _recoveryFor(AppErrorType type) => switch (type) {
  AppErrorType.permissionDenied =>
    'Authorize the request or use an account allowed to change this.',
  AppErrorType.unsupported || AppErrorType.hardwareUnavailable =>
    'This machine does not provide the needed hardware or service.',
  AppErrorType.unavailable || AppErrorType.serviceUnavailable =>
    'Try again later; restart the backend if it persists.',
  AppErrorType.timeout => 'The operation timed out. Try again.',
  AppErrorType.invalidInput => 'Check the value and try again.',
  AppErrorType.systemError || AppErrorType.notFound =>
    'Restart the backend; report with diagnostics if it persists.',
};
