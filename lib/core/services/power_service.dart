import 'package:zettings/core/errors/app_error.dart';
import 'package:zettings/core/ipc/zettings_ipc_client.dart';
import 'package:zettings/core/models/power_status.dart';
import 'package:zettings/core/platform/capability.dart';
import 'package:zettings/core/services/system_service.dart';

abstract class PowerServiceProvider {
  Future<PowerStatus> getStatus();
  Future<void> setProfile(String profile);
}

class IpcPowerService implements PowerServiceProvider, SystemService {
  IpcPowerService(this._client);

  final ZettingsIpcClient _client;

  @override
  String get name => 'power';

  @override
  Future<SystemCapability> probe() async {
    try {
      final caps = await _client.getCapabilities();
      final match = caps.where((c) => c.name == name);
      if (match.isEmpty) {
        return const SystemCapability(
          name: 'power',
          availability: ServiceAvailability.unavailable,
          reason: 'Power service not registered in backend',
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
        reason: raw.reason.isEmpty ? 'Power subsystem active' : raw.reason,
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
  Future<PowerStatus> getStatus() => _client.getPowerStatus();

  @override
  Future<void> setProfile(String profile) => _client.setPowerProfile(profile);
}

class MockPowerService implements PowerServiceProvider, SystemService {
  MockPowerService({String initialProfile = 'balanced'})
    : _profile = initialProfile;

  String _profile;

  @override
  String get name => 'power';

  @override
  Future<SystemCapability> probe() async => const SystemCapability(
    name: 'power',
    availability: ServiceAvailability.available,
    reason: 'Mock Development Backend (Preview)',
  );

  @override
  Future<PowerStatus> getStatus() async {
    return PowerStatus(
      onBattery: false,
      batteryPercentage: 100.0,
      chargingState: 'Full',
      timeToEmptySecs: 0,
      activeProfile: _profile,
      availableProfiles: const ['performance', 'balanced', 'power-saver'],
    );
  }

  @override
  Future<void> setProfile(String profile) async {
    _profile = profile;
  }
}
