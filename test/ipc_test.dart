import 'package:dbus/dbus.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:zettings/core/errors/app_error.dart';
import 'package:zettings/core/ipc/zettings_ipc_client.dart';

void main() {
  group('ipcErrorToAppError', () {
    test('recovers every backend code', () {
      const codes = {
        'unavailable': AppErrorType.unavailable,
        'unsupported': AppErrorType.unsupported,
        'permissionDenied': AppErrorType.permissionDenied,
        'invalidInput': AppErrorType.invalidInput,
        'timeout': AppErrorType.timeout,
        'serviceUnavailable': AppErrorType.serviceUnavailable,
        'hardwareUnavailable': AppErrorType.hardwareUnavailable,
        'systemError': AppErrorType.systemError,
      };
      for (final entry in codes.entries) {
        final error = ipcErrorToAppError(
          dbusErrorName: 'org.freedesktop.DBus.Error.Failed',
          message: '${entry.key}: some detail',
        );
        expect(error.type, entry.value, reason: entry.key);
        expect(error.message, 'some detail');
        expect(error.recovery, isNotNull);
      }
    });

    test('standard names map without a code prefix', () {
      final unknown = ipcErrorToAppError(
        dbusErrorName: 'org.freedesktop.DBus.Error.ServiceUnknown',
        message: '',
      );
      expect(unknown.type, AppErrorType.serviceUnavailable);

      final denied = ipcErrorToAppError(
        dbusErrorName: 'org.freedesktop.DBus.Error.AccessDenied',
        message: '',
      );
      expect(denied.type, AppErrorType.permissionDenied);

      final skew = ipcErrorToAppError(
        dbusErrorName: 'org.freedesktop.DBus.Error.UnknownMethod',
        message: 'No such method',
      );
      expect(skew.type, AppErrorType.systemError);
    });

    test('unknown errors stay diagnosable', () {
      final error = ipcErrorToAppError(
        dbusErrorName: 'org.example.Wat',
        message: 'boom',
      );
      expect(error.type, AppErrorType.systemError);
      expect(error.message, 'boom');
      expect(error.details, contains('org.example.Wat'));
    });
  });

  group('RemoteCapability.fromDbus', () {
    test('parses a valid struct', () {
      final capability = RemoteCapability.fromDbus(
        DBusStruct([
          DBusString('network'),
          DBusString('available'),
          DBusString(''),
        ]),
      );
      expect(capability.name, 'network');
      expect(capability.availability, 'available');
      expect(capability.reason, isEmpty);
    });

    test('rejects malformed values with a typed error', () {
      expect(
        () => RemoteCapability.fromDbus(DBusString('nope')),
        throwsA(
          isA<AppError>().having(
            (e) => e.type,
            'type',
            AppErrorType.systemError,
          ),
        ),
      );
      expect(
        () => RemoteCapability.fromDbus(DBusStruct([DBusString('only-one')])),
        throwsA(isA<AppError>()),
      );
    });
  });
}
