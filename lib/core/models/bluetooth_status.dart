import 'package:dbus/dbus.dart';
import 'package:zettings/core/errors/app_error.dart';

/// One Bluetooth peripheral device.
/// Mirrors Rust `BluetoothDevice` struct `(ssbbis)`.
class BluetoothDevice {
  const BluetoothDevice({
    required this.name,
    required this.address,
    required this.connected,
    required this.paired,
    required this.batteryPercentage,
    required this.deviceType,
  });

  final String name;
  final String address;
  final bool connected;
  final bool paired;
  final int batteryPercentage;
  final String deviceType;

  factory BluetoothDevice.fromDbus(DBusValue value) {
    if (value is DBusStruct &&
        value.children.length == 6 &&
        value.children[0] is DBusString &&
        value.children[1] is DBusString &&
        value.children[2] is DBusBoolean &&
        value.children[3] is DBusBoolean &&
        (value.children[4] is DBusInt32 || value.children[4] is DBusUint32) &&
        value.children[5] is DBusString) {
      final battery = value.children[4] is DBusInt32
          ? (value.children[4] as DBusInt32).value
          : (value.children[4] as DBusUint32).value;
      return BluetoothDevice(
        name: (value.children[0] as DBusString).value,
        address: (value.children[1] as DBusString).value,
        connected: (value.children[2] as DBusBoolean).value,
        paired: (value.children[3] as DBusBoolean).value,
        batteryPercentage: battery,
        deviceType: (value.children[5] as DBusString).value,
      );
    }
    throw AppError(
      AppErrorType.systemError,
      'Backend sent a malformed Bluetooth device entry.',
      details: 'Expected struct(ssbbis), got: $value',
    );
  }
}

/// Bluetooth adapter and peripheral status.
/// Mirrors Rust `BluetoothStatus` struct `(bsbua(ssbbis))`.
class BluetoothStatus {
  const BluetoothStatus({
    required this.enabled,
    required this.adapterName,
    required this.discovering,
    required this.connectedDevicesCount,
    required this.devices,
  });

  final bool enabled;
  final String adapterName;
  final bool discovering;
  final int connectedDevicesCount;
  final List<BluetoothDevice> devices;

  factory BluetoothStatus.fromDbus(DBusValue value) {
    if (value is! DBusStruct || value.children.length != 5) {
      throw AppError(
        AppErrorType.systemError,
        'Backend sent a malformed Bluetooth status reply.',
        details: 'Expected 5-field struct (bsbua(ssbbis)), got: $value',
      );
    }
    final c = value.children;
    if (c[0] is! DBusBoolean ||
        c[1] is! DBusString ||
        c[2] is! DBusBoolean ||
        (c[3] is! DBusUint32 && c[3] is! DBusUint64) ||
        c[4] is! DBusArray) {
      throw AppError(
        AppErrorType.systemError,
        'Backend sent invalid field types for Bluetooth status.',
        details: 'Expected struct(bsbua(ssbbis)), got: $value',
      );
    }

    final count = c[3] is DBusUint32
        ? (c[3] as DBusUint32).value
        : (c[3] as DBusUint64).value;

    return BluetoothStatus(
      enabled: (c[0] as DBusBoolean).value,
      adapterName: (c[1] as DBusString).value,
      discovering: (c[2] as DBusBoolean).value,
      connectedDevicesCount: count,
      devices: (c[4] as DBusArray).children
          .map(BluetoothDevice.fromDbus)
          .toList(),
    );
  }
}
