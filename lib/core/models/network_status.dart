import 'package:dbus/dbus.dart';
import 'package:zettings/core/errors/app_error.dart';

/// One scanned Wi-Fi access point reported by the backend.
/// Mirrors Rust `WifiNetwork` struct `(sybs)`.
class WifiNetwork {
  const WifiNetwork({
    required this.ssid,
    required this.signalStrength,
    required this.secured,
    required this.securityType,
  });

  final String ssid;
  final int signalStrength;
  final bool secured;
  final String securityType;

  factory WifiNetwork.fromDbus(DBusValue value) {
    if (value is DBusStruct &&
        value.children.length == 4 &&
        value.children[0] is DBusString &&
        (value.children[1] is DBusByte || value.children[1] is DBusUint32) &&
        value.children[2] is DBusBoolean &&
        value.children[3] is DBusString) {
      final strength = value.children[1] is DBusByte
          ? (value.children[1] as DBusByte).value
          : (value.children[1] as DBusUint32).value;
      return WifiNetwork(
        ssid: (value.children[0] as DBusString).value,
        signalStrength: strength,
        secured: (value.children[2] as DBusBoolean).value,
        securityType: (value.children[3] as DBusString).value,
      );
    }
    throw AppError(
      AppErrorType.systemError,
      'Backend sent a malformed Wi-Fi network entry.',
      details: 'Expected struct(sybs), got: $value',
    );
  }
}

/// Full network status and connection info.
/// Mirrors Rust `NetworkStatus` struct `(bbsssya(sybs))`.
class NetworkStatus {
  const NetworkStatus({
    required this.wifiEnabled,
    required this.connected,
    required this.activeSsid,
    required this.ipv4Address,
    required this.interfaceName,
    required this.signalStrength,
    required this.wifiNetworks,
  });

  final bool wifiEnabled;
  final bool connected;
  final String activeSsid;
  final String ipv4Address;
  final String interfaceName;
  final int signalStrength;
  final List<WifiNetwork> wifiNetworks;

  factory NetworkStatus.fromDbus(DBusValue value) {
    if (value is! DBusStruct || value.children.length != 7) {
      throw AppError(
        AppErrorType.systemError,
        'Backend sent a malformed network status reply.',
        details: 'Expected 7-field struct (bbsssya(sybs)), got: $value',
      );
    }
    final c = value.children;
    if (c[0] is! DBusBoolean ||
        c[1] is! DBusBoolean ||
        c[2] is! DBusString ||
        c[3] is! DBusString ||
        c[4] is! DBusString ||
        (c[5] is! DBusByte && c[5] is! DBusUint32) ||
        c[6] is! DBusArray) {
      throw AppError(
        AppErrorType.systemError,
        'Backend sent invalid field types for network status.',
        details: 'Expected struct(bbsssya(sybs)), got: $value',
      );
    }

    final strength = c[5] is DBusByte
        ? (c[5] as DBusByte).value
        : (c[5] as DBusUint32).value;

    return NetworkStatus(
      wifiEnabled: (c[0] as DBusBoolean).value,
      connected: (c[1] as DBusBoolean).value,
      activeSsid: (c[2] as DBusString).value,
      ipv4Address: (c[3] as DBusString).value,
      interfaceName: (c[4] as DBusString).value,
      signalStrength: strength,
      wifiNetworks: (c[6] as DBusArray).children
          .map(WifiNetwork.fromDbus)
          .toList(),
    );
  }
}
