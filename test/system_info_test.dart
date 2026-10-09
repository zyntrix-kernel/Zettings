import 'package:dbus/dbus.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:zettings/core/errors/app_error.dart';
import 'package:zettings/core/models/system_info.dart';
import 'package:zettings/core/services/audio_service.dart';
import 'package:zettings/core/services/power_service.dart';
import 'package:zettings/core/services/system_info_service.dart';
import 'package:zettings/features/system/system_page.dart';

DBusStruct _validSnapshot() => DBusStruct([
  DBusString('Zyntrix OS'),
  DBusString('1.0'),
  DBusString('6.8.0-test'),
  DBusString('zyntrix'),
  DBusString('KDE'),
  DBusString('wayland'),
  DBusString('Test CPU 9000'),
  DBusUint32(8),
  DBusUint64(3200),
  DBusUint64(16000000000),
  DBusUint64(8000000000),
  DBusUint64(3661),
  DBusArray(DBusSignature('(ss)'), [
    DBusStruct([DBusString('Test GPU'), DBusString('Test Vendor')]),
  ]),
  DBusArray(DBusSignature('(sttsb)'), [
    DBusStruct([
      DBusString('/'),
      DBusUint64(500000000000),
      DBusUint64(250000000000),
      DBusString('ext4'),
      DBusBoolean(false),
    ]),
  ]),
]);

SystemInfo _canned() => SystemInfo.fromDbus(_validSnapshot());

class FakeSystemInfoProvider implements SystemInfoProvider {
  FakeSystemInfoProvider({SystemInfo? info, this.error})
    : info = info ?? SystemInfo.fromDbus(_validSnapshot());

  final SystemInfo info;
  final AppError? error;

  @override
  Future<SystemInfo> getSystemInfo() async {
    final failure = error;
    if (failure != null) throw failure;
    return info;
  }
}

void main() {
  group('SystemInfo.fromDbus', () {
    test('parses a valid snapshot positionally', () {
      final info = SystemInfo.fromDbus(_validSnapshot());
      expect(info.osName, 'Zyntrix OS');
      expect(info.cpuCount, 8);
      expect(info.cpuFreqMhz, 3200);
      expect(info.memTotalBytes, 16000000000);
      expect(info.uptimeSecs, 3661);
      expect(info.gpus.single.name, 'Test GPU');
      expect(info.disks.single.mount, '/');
      expect(info.disks.single.removable, isFalse);
    });

    test('rejects malformed snapshots with typed errors', () {
      expect(
        () => SystemInfo.fromDbus(DBusString('nope')),
        throwsA(
          isA<AppError>().having(
            (e) => e.type,
            'type',
            AppErrorType.systemError,
          ),
        ),
      );
      expect(
        () => SystemInfo.fromDbus(DBusStruct([DBusString('short')])),
        throwsA(isA<AppError>()),
      );
      expect(
        () => GpuInfo.fromDbus(DBusString('nope')),
        throwsA(isA<AppError>()),
      );
      expect(
        () => DiskInfo.fromDbus(DBusString('nope')),
        throwsA(isA<AppError>()),
      );
    });
  });

  group('formatters', () {
    test('formatBytes scales cleanly', () {
      expect(formatBytes(512), '512 B');
      expect(formatBytes(2048), '2.0 KB');
      expect(formatBytes(5 * 1024 * 1024), '5.0 MB');
      expect(formatBytes(16 * 1024 * 1024 * 1024), '16.0 GB');
    });

    test('formatUptime picks the two largest units', () {
      expect(formatUptime(45), '45s');
      expect(formatUptime(90), '1m 30s');
      expect(formatUptime(3661), '1h 1m');
      expect(formatUptime(90061), '1d 1h');
    });

    test('valueOrDash never blanks a row', () {
      expect(valueOrDash(''), '—');
      expect(valueOrDash('KDE'), 'KDE');
    });
  });

  group('SystemPage', () {
    testWidgets('renders live data rows', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(home: SystemPage(provider: FakeSystemInfoProvider())),
      );
      await tester.pumpAndSettle();

      expect(find.text('Zyntrix OS'), findsOneWidget);
      expect(find.textContaining('Test CPU 9000'), findsOneWidget);
      expect(find.textContaining('14.9 GB'), findsOneWidget);
      await tester.scrollUntilVisible(find.text('Test GPU'), 300);
      expect(find.text('Test GPU'), findsOneWidget);
      await tester.scrollUntilVisible(find.textContaining('/ · ext4'), 300);
      expect(find.textContaining('/ · ext4'), findsOneWidget);
    });

    testWidgets('renders an empty-GPU note honestly', (
      WidgetTester tester,
    ) async {
      final info = _canned();
      await tester.pumpWidget(
        MaterialApp(
          home: SystemPage(
            provider: FakeSystemInfoProvider(
              info: SystemInfo(
                osName: info.osName,
                osVersion: info.osVersion,
                kernel: info.kernel,
                hostname: info.hostname,
                desktop: info.desktop,
                sessionType: info.sessionType,
                cpuBrand: info.cpuBrand,
                cpuCount: info.cpuCount,
                cpuFreqMhz: info.cpuFreqMhz,
                memTotalBytes: info.memTotalBytes,
                memAvailBytes: info.memAvailBytes,
                uptimeSecs: info.uptimeSecs,
                gpus: const [],
                disks: const [],
              ),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();

      await tester.scrollUntilVisible(
        find.text('No graphics devices reported'),
        300,
      );
      expect(find.text('No graphics devices reported'), findsOneWidget);
      expect(find.text('No disks reported'), findsOneWidget);
    });

    testWidgets('backend failure shows error view with retry', (
      WidgetTester tester,
    ) async {
      await tester.pumpWidget(
        MaterialApp(
          home: SystemPage(
            provider: FakeSystemInfoProvider(
              error: const AppError(
                AppErrorType.serviceUnavailable,
                'Backend is not running.',
              ),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.text('Backend is not running.'), findsOneWidget);
      expect(find.text('Retry'), findsOneWidget);
      await tester.tap(find.text('Retry'));
      await tester.pumpAndSettle();
      expect(find.text('Backend is not running.'), findsOneWidget);
    });

    testWidgets('renders audio and power controls and responds to input', (
      WidgetTester tester,
    ) async {
      final mockPower = MockPowerService();
      final mockAudio = MockAudioService(
        initialVolume: 0.70,
        initialMuted: false,
      );

      await tester.pumpWidget(
        MaterialApp(
          home: SystemPage(
            provider: FakeSystemInfoProvider(),
            powerProvider: mockPower,
            audioProvider: mockAudio,
          ),
        ),
      );
      await tester.pumpAndSettle();

      // Scroll to sound section
      await tester.scrollUntilVisible(find.text('Master volume'), 300);
      expect(find.text('Master volume'), findsOneWidget);
      expect(find.text('70% volume'), findsOneWidget);

      // Ensure mute switch is on-screen and toggle it
      await tester.ensureVisible(find.text('Mute master audio'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Mute master audio'));
      await tester.pumpAndSettle();
      expect(find.text('Muted'), findsOneWidget);

      // Ensure power mode is on-screen
      await tester.scrollUntilVisible(find.text('Power mode'), 300);
      expect(find.text('100% · Full'), findsOneWidget);

      // Ensure power-saver button is on-screen and tap it
      await tester.ensureVisible(find.text('Power saver'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Power saver'));
      await tester.pumpAndSettle();
      final powerStatus = await mockPower.getStatus();
      expect(powerStatus.activeProfile, 'power-saver');
    });
  });
}
