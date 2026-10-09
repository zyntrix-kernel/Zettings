import 'package:shared_preferences/shared_preferences.dart';
import 'package:zettings/core/ipc/zettings_ipc_client.dart';
import 'package:zettings/core/search/search.dart';
import 'package:zettings/core/services/accounts_service.dart';
import 'package:zettings/core/services/audio_service.dart';
import 'package:zettings/core/services/bluetooth_service.dart';
import 'package:zettings/core/services/network_service.dart';
import 'package:zettings/core/services/power_service.dart';
import 'package:zettings/core/services/prefs_service.dart';
import 'package:zettings/core/services/service_locator.dart';
import 'package:zettings/core/services/system_info_service.dart';
import 'package:zettings/core/services/system_service.dart';
import 'package:zettings/core/state/registry.dart';

/// Registers the production service graph for tests and tooling.
///
/// Widget tests run without `main()`, so they call this to get the same
/// registry/engine/history/prefs/IPC wiring the real bootstrap installs.
/// The IPC client is lazy: constructing it opens no connection, so this is
/// safe without a bus (calls fail with typed errors until a daemon runs).
Future<void> registerTestServices() async {
  SharedPreferences.setMockInitialValues({});
  ServiceLocator.reset();
  final prefs = await PrefsService.load();
  ServiceLocator.register<PrefsService>(prefs);
  final registry = SettingsRegistry();
  ServiceLocator.register<SettingsRegistry>(registry);
  ServiceLocator.register<SettingsSearchEngine>(SettingsSearchEngine(registry));
  ServiceLocator.register<SearchHistory>(SearchHistory());
  final ipc = ZettingsIpcClient();
  ServiceLocator.register<ZettingsIpcClient>(ipc);
  final systemInfo = IpcSystemInfoService(ipc);
  ServiceLocator.register<SystemInfoProvider>(systemInfo);
  ServiceLocator.register<NetworkServiceProvider>(MockNetworkService());
  ServiceLocator.register<BluetoothServiceProvider>(MockBluetoothService());
  ServiceLocator.register<PowerServiceProvider>(MockPowerService());
  ServiceLocator.register<AudioServiceProvider>(MockAudioService());
  ServiceLocator.register<AccountsServiceProvider>(const MockAccountsService());
  ServiceLocator.register<SystemService>(systemInfo);
}
