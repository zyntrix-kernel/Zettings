import 'package:zettings/core/errors/app_error.dart';
import 'package:zettings/core/ipc/zettings_ipc_client.dart';
import 'package:zettings/core/models/network_status.dart';
import 'package:zettings/core/platform/capability.dart';
import 'package:zettings/core/services/system_service.dart';

abstract class NetworkServiceProvider {
  Future<NetworkStatus> getStatus();
  Future<void> setWifiEnabled(bool enabled);
}

class IpcNetworkService implements NetworkServiceProvider, SystemService {
  IpcNetworkService(this._client);

  final ZettingsIpcClient _client;

  @override
  String get name => 'network';

  @override
  Future<SystemCapability> probe() async {
    try {
      final caps = await _client.getCapabilities();
      final match = caps.where((c) => c.name == name);
      if (match.isEmpty) {
        return const SystemCapability(
          name: 'network',
          availability: ServiceAvailability.unavailable,
          reason: 'Network service not registered in backend',
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
        reason: raw.reason.isEmpty ? 'Network subsystem active' : raw.reason,
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
  Future<NetworkStatus> getStatus() => _client.getNetworkStatus();

  @override
  Future<void> setWifiEnabled(bool enabled) => _client.setWifiEnabled(enabled);
}

class MockNetworkService implements NetworkServiceProvider, SystemService {
  MockNetworkService({this._wifiEnabled = true});

  bool _wifiEnabled;

  @override
  String get name => 'network';

  @override
  Future<SystemCapability> probe() async => const SystemCapability(
    name: 'network',
    availability: ServiceAvailability.available,
    reason: 'Mock Development Backend (Preview)',
  );

  @override
  Future<NetworkStatus> getStatus() async {
    return NetworkStatus(
      wifiEnabled: _wifiEnabled,
      connected: _wifiEnabled,
      activeSsid: _wifiEnabled ? 'Zyntrix-5G' : '',
      ipv4Address: _wifiEnabled ? '192.168.1.142' : '',
      interfaceName: 'wlan0',
      signalStrength: _wifiEnabled ? 92 : 0,
      wifiNetworks: _wifiEnabled
          ? const [
              WifiNetwork(
                ssid: 'Zyntrix-5G',
                signalStrength: 92,
                secured: true,
                securityType: 'WPA3',
              ),
              WifiNetwork(
                ssid: 'Studio-Office',
                signalStrength: 78,
                secured: true,
                securityType: 'WPA2/WPA3',
              ),
              WifiNetwork(
                ssid: 'Guest-Network',
                signalStrength: 54,
                secured: true,
                securityType: 'WPA2',
              ),
            ]
          : const [],
    );
  }

  @override
  Future<void> setWifiEnabled(bool enabled) async {
    _wifiEnabled = enabled;
  }
}
