import 'package:zettings/core/errors/app_error.dart';
import 'package:zettings/core/ipc/zettings_ipc_client.dart';
import 'package:zettings/core/models/user_info.dart';
import 'package:zettings/core/platform/capability.dart';
import 'package:zettings/core/services/system_service.dart';

abstract class AccountsServiceProvider {
  Future<List<UserInfo>> getUsers();
}

class IpcAccountsService implements AccountsServiceProvider, SystemService {
  IpcAccountsService(this._client);

  final ZettingsIpcClient _client;

  @override
  String get name => 'accounts';

  @override
  Future<SystemCapability> probe() async {
    try {
      final caps = await _client.getCapabilities();
      final match = caps.where((c) => c.name == name);
      if (match.isEmpty) {
        return const SystemCapability(
          name: 'accounts',
          availability: ServiceAvailability.unavailable,
          reason: 'Accounts service not registered in backend',
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
        reason: raw.reason.isEmpty ? 'Accounts subsystem active' : raw.reason,
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
  Future<List<UserInfo>> getUsers() => _client.getUsers();
}

class MockAccountsService implements AccountsServiceProvider, SystemService {
  const MockAccountsService();

  @override
  String get name => 'accounts';

  @override
  Future<SystemCapability> probe() async => const SystemCapability(
    name: 'accounts',
    availability: ServiceAvailability.available,
    reason: 'Mock Development Backend (Preview)',
  );

  @override
  Future<List<UserInfo>> getUsers() async {
    return const [
      UserInfo(
        username: 'zyntrix',
        realName: 'Zyntrix Admin',
        isAdmin: true,
        homeDir: '/home/zyntrix',
        shell: '/bin/bash',
      ),
      UserInfo(
        username: 'guest',
        realName: 'Guest User',
        isAdmin: false,
        homeDir: '/home/guest',
        shell: '/bin/bash',
      ),
    ];
  }
}
