import 'package:zettings/core/errors/app_error.dart';
import 'package:zettings/core/ipc/zettings_ipc_client.dart';
import 'package:zettings/core/models/system_info.dart';
import 'package:zettings/core/platform/capability.dart';
import 'package:zettings/core/services/system_service.dart';

/// Data source for the system page. Production implementation talks to the
/// daemon; tests inject fakes. Keeps the page deterministic without a bus.
abstract class SystemInfoProvider {
  Future<SystemInfo> getSystemInfo();
}

/// Live provider: daemon-backed system snapshots.
///
/// `probe` is infallible by contract — every transport failure becomes an
/// `unavailable` capability with the backend's own message as the reason.
class IpcSystemInfoService implements SystemInfoProvider, SystemService {
  IpcSystemInfoService(this._client);

  final ZettingsIpcClient _client;

  @override
  String get name => 'system';

  @override
  Future<SystemCapability> probe() async {
    try {
      await _client.ping();
      return SystemCapability(
        name: name,
        availability: ServiceAvailability.available,
        reason: 'Backend reachable',
      );
    } on AppError catch (e) {
      switch (e.type) {
        case AppErrorType.permissionDenied:
          return SystemCapability(
            name: name,
            availability: ServiceAvailability.permissionDenied,
            reason: e.message,
          );
        default:
          return SystemCapability(
            name: name,
            availability: ServiceAvailability.unavailable,
            reason: e.message,
          );
      }
    } catch (e) {
      return SystemCapability(
        name: name,
        availability: ServiceAvailability.unavailable,
        reason: '$e',
      );
    }
  }

  @override
  Future<SystemInfo> getSystemInfo() => _client.getSystemInfo();
}
