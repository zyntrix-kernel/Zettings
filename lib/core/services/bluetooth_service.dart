import 'package:zettings/core/errors/app_error.dart';
import 'package:zettings/core/ipc/zettings_ipc_client.dart';
import 'package:zettings/core/models/bluetooth_status.dart';
import 'package:zettings/core/platform/capability.dart';
import 'package:zettings/core/services/system_service.dart';

abstract class BluetoothServiceProvider {
  Future<BluetoothStatus> getStatus();
  Future<void> setEnabled(bool enabled);
}

class IpcBluetoothService implements BluetoothServiceProvider, SystemService {
  IpcBluetoothService(this._client);

  final ZettingsIpcClient _client;

  @override
  String get name => 'bluetooth';

  @override
  Future<SystemCapability> probe() async {
    try {
      final caps = await _client.getCapabilities();
      final match = caps.where((c) => c.name == name);
      if (match.isEmpty) {
        return const SystemCapability(
          name: 'bluetooth',
          availability: ServiceAvailability.unavailable,
          reason: 'Bluetooth service not registered in backend',
        );
      }
      final raw = match.first;
      final availability = switch (raw.availability) {
        'available' => ServiceAvailability.available,
        'unsupported' => ServiceAvailability.unsupported,
        'permissionDenied' => ServiceAvailability.permissionDenied,
        _ => ServiceAvailability.unavailable,
      };
      return SystemCapability(
        name: name,
        availability: availability,
        reason: raw.reason.isEmpty ? 'BlueZ subsystem active' : raw.reason,
      );
    } on AppError catch (e) {
      return SystemCapability(
        name: name,
        availability: e.type == AppErrorType.permissionDenied
            ? ServiceAvailability.permissionDenied
            : ServiceAvailability.unavailable,
        reason: e.message,
      );
    } catch (e) {
      return SystemCapability(
        name: name,
        availability: ServiceAvailability.unavailable,
        reason: '$e',
      );
    }
  }

  @override
  Future<BluetoothStatus> getStatus() => _client.getBluetoothStatus();

  @override
  Future<void> setEnabled(bool enabled) => _client.setBluetoothEnabled(enabled);
}

class MockBluetoothService implements BluetoothServiceProvider, SystemService {
  MockBluetoothService({this._enabled = true});

  bool _enabled;

  @override
  String get name => 'bluetooth';

  @override
  Future<SystemCapability> probe() async => const SystemCapability(
    name: 'bluetooth',
    availability: ServiceAvailability.available,
    reason: 'Mock Development Backend (Preview)',
  );

  @override
  Future<BluetoothStatus> getStatus() async {
    return BluetoothStatus(
      enabled: _enabled,
      adapterName: 'Intel AX210',
      discovering: false,
      connectedDevicesCount: _enabled ? 2 : 0,
      devices: _enabled
          ? const [
              BluetoothDevice(
                name: 'Sony WH-1000XM5',
                address: 'AA:BB:CC:DD:EE:01',
                connected: true,
                paired: true,
                batteryPercentage: 78,
                deviceType: 'Headphones',
              ),
              BluetoothDevice(
                name: 'Logitech MX Master 3S',
                address: 'AA:BB:CC:DD:EE:02',
                connected: true,
                paired: true,
                batteryPercentage: 90,
                deviceType: 'Mouse',
              ),
              BluetoothDevice(
                name: 'Keychron Q1 Pro',
                address: 'AA:BB:CC:DD:EE:03',
                connected: true,
                paired: true,
                batteryPercentage: 100,
                deviceType: 'Keyboard',
              ),
            ]
          : const [],
    );
  }

  @override
  Future<void> setEnabled(bool enabled) async {
    _enabled = enabled;
  }
}
