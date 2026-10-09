import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:zettings/app/app.dart';
import 'package:zettings/core/ipc/zettings_ipc_client.dart';
import 'package:zettings/core/search/search.dart';
import 'package:zettings/core/services/accounts_service.dart';
import 'package:zettings/core/services/audio_service.dart';
import 'package:zettings/core/services/bluetooth_service.dart';
import 'package:zettings/core/services/mock_system_info_service.dart';
import 'package:zettings/core/services/network_service.dart';
import 'package:zettings/core/services/power_service.dart';
import 'package:zettings/core/services/prefs_service.dart';
import 'package:zettings/core/services/service_locator.dart';
import 'package:zettings/core/services/system_info_service.dart';
import 'package:zettings/core/services/system_service.dart';
import 'package:zettings/core/state/app_state.dart';
import 'package:zettings/core/state/registry.dart';
import 'package:zettings/core/utils/log.dart';

/// Application entry point.
///
/// Bootstrap installs the production service graph (prefs, registry, search,
/// history, IPC client, system services), restores persisted appearance, then
/// hands off to [ZettingsApp].
Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();

  FlutterError.onError = (details) {
    ZettingsLog.error('Flutter error', details.exception);
    FlutterError.presentError(details);
  };
  PlatformDispatcher.instance.onError = (error, stack) {
    ZettingsLog.error('Uncaught async error: $error');
    return true;
  };

  final prefs = await PrefsService.load();
  final registry = SettingsRegistry();
  ServiceLocator.register<PrefsService>(prefs);
  ServiceLocator.register<SettingsRegistry>(registry);
  ServiceLocator.register<SettingsSearchEngine>(SettingsSearchEngine(registry));
  final history = SearchHistory();
  for (final query in prefs.recents().reversed) {
    history.record(query);
  }
  ServiceLocator.register<SearchHistory>(history);

  final ipc = ZettingsIpcClient();
  ServiceLocator.register<ZettingsIpcClient>(ipc);

  // On Linux (the production target), wire the live D-Bus IPC daemon.
  // On Windows/macOS/web or when explicitly specified via MOCK_BACKEND=true,
  // register mock development services so developers can run and interact with
  // the full Material 3 UI.
  final isLinux = !kIsWeb && defaultTargetPlatform == TargetPlatform.linux;
  final useMock =
      const bool.fromEnvironment('MOCK_BACKEND', defaultValue: false) ||
      !isLinux;

  if (useMock) {
    ZettingsLog.info(
      'Development environment detected: registering mock services',
    );
    const mockSystem = MockSystemInfoService();
    final mockNetwork = MockNetworkService();
    final mockBluetooth = MockBluetoothService();
    final mockPower = MockPowerService();
    final mockAudio = MockAudioService();
    const mockAccounts = MockAccountsService();

    ServiceLocator.register<SystemInfoProvider>(mockSystem);
    ServiceLocator.register<NetworkServiceProvider>(mockNetwork);
    ServiceLocator.register<BluetoothServiceProvider>(mockBluetooth);
    ServiceLocator.register<PowerServiceProvider>(mockPower);
    ServiceLocator.register<AudioServiceProvider>(mockAudio);
    ServiceLocator.register<AccountsServiceProvider>(mockAccounts);
    ServiceLocator.register<SystemService>(mockSystem);
  } else {
    final systemInfo = IpcSystemInfoService(ipc);
    final network = IpcNetworkService(ipc);
    final bluetooth = IpcBluetoothService(ipc);
    final power = IpcPowerService(ipc);
    final audio = IpcAudioService(ipc);
    final accounts = IpcAccountsService(ipc);

    ServiceLocator.register<SystemInfoProvider>(systemInfo);
    ServiceLocator.register<NetworkServiceProvider>(network);
    ServiceLocator.register<BluetoothServiceProvider>(bluetooth);
    ServiceLocator.register<PowerServiceProvider>(power);
    ServiceLocator.register<AudioServiceProvider>(audio);
    ServiceLocator.register<AccountsServiceProvider>(accounts);
    ServiceLocator.register<SystemService>(systemInfo);
  }

  final state = ZettingsState(
    initialAppearance: prefs.appearance(),
    initialAccent: prefs.accent(),
    initialReducedMotion: prefs.reducedMotion(),
    initialHighContrast: prefs.highContrast(),
    initialLargeText: prefs.largeText(),
  );
  runApp(ZettingsApp(state: state));
}
