import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:zettings/core/ipc/zettings_ipc_client.dart';

/// Live Dart → Rust round trip. Runs only on Linux with a session bus and a
/// freshly built daemon (`cargo build` in `rust/`); everywhere else it
/// reports SKIP and passes. The Rust side owns the mirror test.
void main() {
  test('live daemon round trip', () async {
    if (!Platform.isLinux) {
      // ignore: avoid_print
      print('SKIP ipc_live: not Linux');
      return;
    }
    // `flutter test` runs with the package root as cwd; fall back to a
    // script-relative lookup for other runners.
    var binary = File('rust/target/debug/zettings-backend');
    if (!binary.existsSync()) {
      binary = File.fromUri(
        Platform.script.resolve('../../rust/target/debug/zettings-backend'),
      );
    }
    if (!binary.existsSync()) {
      // ignore: avoid_print
      print('SKIP ipc_live: daemon not built (run cargo build in rust/)');
      return;
    }
    if (Platform.environment['DBUS_SESSION_BUS_ADDRESS'] == null) {
      // ignore: avoid_print
      print('SKIP ipc_live: no session bus');
      return;
    }

    final daemon = await Process.start(binary.path, []);
    try {
      final client = ZettingsIpcClient();
      String? ping;
      for (var i = 0; i < 50; i++) {
        try {
          ping = await client.ping();
          break;
        } catch (_) {
          await Future<void>.delayed(const Duration(milliseconds: 200));
        }
      }
      expect(ping, startsWith('zettings-backend '));

      final version = await client.getVersion();
      expect(version, isNotEmpty);

      final capabilities = await client.getCapabilities();
      expect(
        capabilities.any((c) => c.name == 'backend'),
        isTrue,
        reason: 'builtin capability must be present',
      );
      expect(
        capabilities.any((c) => c.name == 'system'),
        isTrue,
        reason: 'system capability must be present',
      );

      final info = await client.getSystemInfo();
      expect(info.osName, isNotEmpty);
      expect(info.kernel, isNotEmpty);
      // Unit discriminator: sysinfo reports bytes. A KB mix-up would read
      // ~1000x smaller and fail this on any machine with >100MB of RAM.
      expect(info.memTotalBytes, greaterThan(100000000));
      await client.close();
    } finally {
      daemon.kill();
    }
  }, timeout: const Timeout(Duration(minutes: 2)));
}
