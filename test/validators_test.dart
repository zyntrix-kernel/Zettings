import 'package:flutter_test/flutter_test.dart';
import 'package:zettings/core/utils/validators.dart';

void main() {
  group('Validators', () {
    test('audio volume validation', () {
      expect(Validators.validateAudioVolume(0.0), isNull);
      expect(Validators.validateAudioVolume(0.5), isNull);
      expect(Validators.validateAudioVolume(1.0), isNull);
      expect(Validators.validateAudioVolume(null), isNotNull);
      expect(Validators.validateAudioVolume(-0.01), isNotNull);
      expect(Validators.validateAudioVolume(1.01), isNotNull);
      expect(Validators.validateAudioVolume(double.nan), isNotNull);
      expect(Validators.validateAudioVolume(double.infinity), isNotNull);
    });

    test('display scaling validation', () {
      expect(Validators.validateDisplayScaling(1.0), isNull);
      expect(Validators.validateDisplayScaling(1.5), isNull);
      expect(Validators.validateDisplayScaling(2.0), isNull);
      expect(Validators.validateDisplayScaling(null), isNotNull);
      expect(Validators.validateDisplayScaling(0.9), isNotNull);
      expect(Validators.validateDisplayScaling(2.6), isNotNull);
    });

    test('color temperature validation', () {
      expect(Validators.validateColorTemperature(4500), isNull);
      expect(Validators.validateColorTemperature(1000), isNull);
      expect(Validators.validateColorTemperature(10000), isNull);
      expect(Validators.validateColorTemperature(null), isNotNull);
      expect(Validators.validateColorTemperature(999), isNotNull);
      expect(Validators.validateColorTemperature(10001), isNotNull);
    });

    test('wifi ssid validation', () {
      expect(Validators.validateWifiSsid('Zyntrix-Home'), isNull);
      expect(Validators.validateWifiSsid('A'), isNull);
      expect(Validators.validateWifiSsid(null), isNotNull);
      expect(Validators.validateWifiSsid(''), isNotNull);
      expect(Validators.validateWifiSsid('A' * 33), isNotNull);
    });

    test('wifi password validation', () {
      expect(Validators.validateWifiPassword('correct-horse-battery'), isNull);
      expect(Validators.validateWifiPassword('12345678'), isNull);
      expect(Validators.validateWifiPassword(null), isNotNull);
      expect(Validators.validateWifiPassword(''), isNotNull);
      expect(Validators.validateWifiPassword('short'), isNotNull);
      expect(Validators.validateWifiPassword('A' * 64), isNotNull);
    });

    test('ipv4 address validation', () {
      expect(Validators.validateIpv4Address('192.168.1.1'), isNull);
      expect(Validators.validateIpv4Address('10.0.0.1'), isNull);
      expect(Validators.validateIpv4Address('127.0.0.1'), isNull);
      expect(Validators.validateIpv4Address(null), isNotNull);
      expect(Validators.validateIpv4Address(''), isNotNull);
      expect(Validators.validateIpv4Address('256.1.1.1'), isNotNull);
      expect(Validators.validateIpv4Address('192.168.1'), isNotNull);
      expect(Validators.validateIpv4Address('192.168.01.1'), isNotNull);
      expect(Validators.validateIpv4Address('abc.def.ghi.jkl'), isNotNull);
    });

    test('username validation', () {
      expect(Validators.validateUsername('zyntrix'), isNull);
      expect(Validators.validateUsername('_daemon'), isNull);
      expect(Validators.validateUsername('user-01'), isNull);
      expect(Validators.validateUsername(null), isNotNull);
      expect(Validators.validateUsername(''), isNotNull);
      expect(Validators.validateUsername('1user'), isNotNull);
      expect(Validators.validateUsername('User'), isNotNull);
      expect(Validators.validateUsername('user space'), isNotNull);
      expect(Validators.validateUsername('user@domain'), isNotNull);
    });

    test('power profile validation', () {
      expect(Validators.validatePowerProfile('balanced'), isNull);
      expect(Validators.validatePowerProfile('performance'), isNull);
      expect(Validators.validatePowerProfile('power-saver'), isNull);
      expect(Validators.validatePowerProfile(null), isNotNull);
      expect(Validators.validatePowerProfile(''), isNotNull);
      expect(Validators.validatePowerProfile('overclock'), isNotNull);
    });
  });
}
