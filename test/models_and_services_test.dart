import 'package:dbus/dbus.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:zettings/core/errors/app_error.dart';
import 'package:zettings/core/models/audio_status.dart';
import 'package:zettings/core/models/bluetooth_status.dart';
import 'package:zettings/core/models/network_status.dart';
import 'package:zettings/core/models/power_status.dart';
import 'package:zettings/core/models/user_info.dart';
import 'package:zettings/core/platform/capability.dart';
import 'package:zettings/core/services/accounts_service.dart';
import 'package:zettings/core/services/audio_service.dart';
import 'package:zettings/core/services/bluetooth_service.dart';
import 'package:zettings/core/services/network_service.dart';
import 'package:zettings/core/services/power_service.dart';

void main() {
  group('NetworkStatus.fromDbus', () {
    test('parses valid struct positionally', () {
      final struct = DBusStruct([
        DBusBoolean(true),
        DBusBoolean(true),
        DBusString('Zyntrix-5G'),
        DBusString('192.168.1.142'),
        DBusString('wlan0'),
        DBusByte(92),
        DBusArray(DBusSignature('(sybs)'), [
          DBusStruct([
            DBusString('Zyntrix-5G'),
            DBusByte(92),
            DBusBoolean(true),
            DBusString('WPA3'),
          ]),
        ]),
      ]);

      final status = NetworkStatus.fromDbus(struct);
      expect(status.wifiEnabled, isTrue);
      expect(status.connected, isTrue);
      expect(status.activeSsid, 'Zyntrix-5G');
      expect(status.ipv4Address, '192.168.1.142');
      expect(status.interfaceName, 'wlan0');
      expect(status.signalStrength, 92);
      expect(status.wifiNetworks.length, 1);
      expect(status.wifiNetworks.first.ssid, 'Zyntrix-5G');
      expect(status.wifiNetworks.first.secured, isTrue);
      expect(status.wifiNetworks.first.securityType, 'WPA3');
    });

    test('rejects malformed network structs', () {
      expect(
        () => NetworkStatus.fromDbus(DBusString('not-a-struct')),
        throwsA(isA<AppError>()),
      );
      expect(
        () => WifiNetwork.fromDbus(DBusString('not-a-struct')),
        throwsA(isA<AppError>()),
      );
    });
  });

  group('BluetoothStatus.fromDbus', () {
    test('parses valid struct positionally', () {
      final struct = DBusStruct([
        DBusBoolean(true),
        DBusString('Intel AX210'),
        DBusBoolean(false),
        DBusUint32(1),
        DBusArray(DBusSignature('(ssbbis)'), [
          DBusStruct([
            DBusString('Sony WH-1000XM5'),
            DBusString('AA:BB:CC:DD:EE:FF'),
            DBusBoolean(true),
            DBusBoolean(true),
            DBusInt32(78),
            DBusString('Headphones'),
          ]),
        ]),
      ]);

      final status = BluetoothStatus.fromDbus(struct);
      expect(status.enabled, isTrue);
      expect(status.adapterName, 'Intel AX210');
      expect(status.discovering, isFalse);
      expect(status.connectedDevicesCount, 1);
      expect(status.devices.length, 1);
      expect(status.devices.first.name, 'Sony WH-1000XM5');
      expect(status.devices.first.batteryPercentage, 78);
      expect(status.devices.first.deviceType, 'Headphones');
    });

    test('rejects malformed bluetooth structs', () {
      expect(
        () => BluetoothStatus.fromDbus(DBusString('wrong')),
        throwsA(isA<AppError>()),
      );
      expect(
        () => BluetoothDevice.fromDbus(DBusString('wrong')),
        throwsA(isA<AppError>()),
      );
    });
  });

  group('PowerStatus.fromDbus', () {
    test('parses valid struct positionally', () {
      final struct = DBusStruct([
        DBusBoolean(false),
        DBusDouble(100.0),
        DBusString('Full'),
        DBusUint64(0),
        DBusString('balanced'),
        DBusArray(DBusSignature('s'), [
          DBusString('performance'),
          DBusString('balanced'),
          DBusString('power-saver'),
        ]),
      ]);

      final status = PowerStatus.fromDbus(struct);
      expect(status.onBattery, isFalse);
      expect(status.batteryPercentage, 100.0);
      expect(status.chargingState, 'Full');
      expect(status.timeToEmptySecs, 0);
      expect(status.activeProfile, 'balanced');
      expect(status.availableProfiles, [
        'performance',
        'balanced',
        'power-saver',
      ]);
    });

    test('rejects malformed power structs', () {
      expect(
        () => PowerStatus.fromDbus(DBusString('wrong')),
        throwsA(isA<AppError>()),
      );
    });
  });

  group('AudioStatus.fromDbus', () {
    test('parses valid struct positionally', () {
      final struct = DBusStruct([
        DBusDouble(0.65),
        DBusBoolean(false),
        DBusString('alsa_output.pci'),
        DBusString('alsa_input.pci'),
      ]);

      final status = AudioStatus.fromDbus(struct);
      expect((status.masterVolume - 0.65).abs() < 0.001, isTrue);
      expect(status.muted, isFalse);
      expect(status.defaultSink, 'alsa_output.pci');
      expect(status.defaultSource, 'alsa_input.pci');
    });

    test('rejects malformed audio structs', () {
      expect(
        () => AudioStatus.fromDbus(DBusString('wrong')),
        throwsA(isA<AppError>()),
      );
    });
  });

  group('UserInfo.fromDbus', () {
    test('parses valid struct positionally', () {
      final struct = DBusStruct([
        DBusString('zyntrix'),
        DBusString('Zyntrix Administrator'),
        DBusBoolean(true),
        DBusString('/home/zyntrix'),
        DBusString('/bin/bash'),
      ]);

      final user = UserInfo.fromDbus(struct);
      expect(user.username, 'zyntrix');
      expect(user.realName, 'Zyntrix Administrator');
      expect(user.isAdmin, isTrue);
      expect(user.homeDir, '/home/zyntrix');
      expect(user.shell, '/bin/bash');
    });

    test('rejects malformed user structs', () {
      expect(
        () => UserInfo.fromDbus(DBusString('wrong')),
        throwsA(isA<AppError>()),
      );
    });
  });

  group('Mock services behavior', () {
    test('MockNetworkService toggles Wi-Fi state', () async {
      final service = MockNetworkService();
      var status = await service.getStatus();
      expect(status.wifiEnabled, isTrue);

      await service.setWifiEnabled(false);
      status = await service.getStatus();
      expect(status.wifiEnabled, isFalse);
      expect(status.wifiNetworks, isEmpty);

      final cap = await service.probe();
      expect(cap.availability, ServiceAvailability.available);
    });

    test('MockBluetoothService toggles adapter state', () async {
      final service = MockBluetoothService();
      var status = await service.getStatus();
      expect(status.enabled, isTrue);

      await service.setEnabled(false);
      status = await service.getStatus();
      expect(status.enabled, isFalse);
      expect(status.devices, isEmpty);

      final cap = await service.probe();
      expect(cap.availability, ServiceAvailability.available);
    });

    test('MockPowerService updates active profile', () async {
      final service = MockPowerService();
      var status = await service.getStatus();
      expect(status.activeProfile, 'balanced');

      await service.setProfile('performance');
      status = await service.getStatus();
      expect(status.activeProfile, 'performance');

      final cap = await service.probe();
      expect(cap.availability, ServiceAvailability.available);
    });

    test('MockAudioService clamps volume and toggles mute', () async {
      final service = MockAudioService();
      var status = await service.getStatus();
      expect(status.muted, isFalse);

      await service.setMuted(true);
      await service.setVolume(1.5);
      status = await service.getStatus();
      expect(status.muted, isTrue);
      expect(status.masterVolume, 1.0);

      final cap = await service.probe();
      expect(cap.availability, ServiceAvailability.available);
    });

    test('MockAccountsService reports users and probe availability', () async {
      const service = MockAccountsService();
      final users = await service.getUsers();
      expect(users.length, greaterThanOrEqualTo(1));
      expect(users.first.isAdmin, isTrue);

      final cap = await service.probe();
      expect(cap.availability, ServiceAvailability.available);
    });
  });
}
