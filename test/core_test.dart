import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:zettings/app/router.dart';
import 'package:zettings/core/errors/app_error.dart';
import 'package:zettings/core/models/setting.dart';
import 'package:zettings/core/platform/capability.dart';
import 'package:zettings/core/services/service_locator.dart';
import 'package:zettings/core/services/system_service.dart';
import 'package:zettings/core/state/app_state.dart';
import 'package:zettings/core/theme/appearance.dart';

class _FakeService implements SystemService {
  @override
  String get name => 'fake';

  @override
  Future<SystemCapability> probe() async => const SystemCapability(
    name: 'fake',
    availability: ServiceAvailability.available,
  );
}

void main() {
  setUp(ServiceLocator.reset);

  group('SettingsRoute', () {
    test('resolves every known path', () {
      for (final route in SettingsRoute.values) {
        expect(SettingsRoute.resolve(route.path), isNotNull);
      }
    });

    test('returns null for unknown paths', () {
      expect(SettingsRoute.resolve('/nope'), isNull);
      expect(SettingsRoute.resolve(''), isNull);
    });

    test('table covers all ten categories', () {
      final categories = SettingsRoute.values.map((r) => r.category).toSet();
      expect(categories, hasLength(SettingCategory.values.length));
    });
  });

  group('parseDeepLink', () {
    test('maps zettings:// links to route paths', () {
      expect(SettingsRoute.parseDeepLink('zettings://network'), '/network');
      expect(
        SettingsRoute.parseDeepLink('zettings://network/wifi'),
        '/network/wifi',
      );
      expect(SettingsRoute.parseDeepLink('zettings://'), '/');
    });

    test('rejects non-zettings input', () {
      expect(SettingsRoute.parseDeepLink('https://example.com'), isNull);
      expect(SettingsRoute.parseDeepLink('not a uri at all %%%'), isNull);
      expect(SettingsRoute.parseDeepLink(''), isNull);
    });
  });

  group('AppError', () {
    test('carries type, message, recovery', () {
      const error = AppError(
        AppErrorType.permissionDenied,
        'Denied.',
        recovery: 'Ask an admin.',
      );
      expect(error.toString(), contains('permissionDenied'));
      expect(error.recovery, 'Ask an admin.');
    });
  });

  group('SystemCapability.require', () {
    test('passes when available', () {
      const capability = SystemCapability(
        name: 'x',
        availability: ServiceAvailability.available,
      );
      expect(capability.require, returnsNormally);
    });

    test('throws typed errors per availability', () {
      expect(
        const SystemCapability(
          name: 'x',
          availability: ServiceAvailability.unsupported,
        ).require,
        throwsA(
          isA<AppError>().having(
            (e) => e.type,
            'type',
            AppErrorType.unsupported,
          ),
        ),
      );
      expect(
        const SystemCapability(
          name: 'x',
          availability: ServiceAvailability.permissionDenied,
        ).require,
        throwsA(
          isA<AppError>().having(
            (e) => e.type,
            'type',
            AppErrorType.permissionDenied,
          ),
        ),
      );
      expect(
        const SystemCapability(
          name: 'x',
          availability: ServiceAvailability.unknown,
        ).require,
        throwsA(isA<AppError>()),
      );
    });
  });

  group('ServiceLocator', () {
    test('throws a typed error when missing', () {
      expect(
        ServiceLocator.get<String>,
        throwsA(
          isA<AppError>().having(
            (e) => e.type,
            'type',
            AppErrorType.systemError,
          ),
        ),
      );
    });

    test('round-trips registrations', () {
      ServiceLocator.register<String>('hello');
      expect(ServiceLocator.get<String>(), 'hello');
      expect(ServiceLocator.isRegistered<String>(), isTrue);
      expect(ServiceLocator.isRegistered<int>(), isFalse);
    });
  });

  group('SystemService contract', () {
    test('fake probe reports available', () async {
      final capability = await _FakeService().probe();
      expect(capability.availability, ServiceAvailability.available);
    });
  });

  group('SettingCategoryMeta', () {
    test('every category has title, icon path', () {
      for (final category in SettingCategory.values) {
        expect(category.title, isNotEmpty);
        expect(category.routePath, startsWith('/'));
      }
    });
  });

  group('ZettingsState', () {
    test('has sane defaults', () {
      final state = ZettingsState();
      expect(state.appearance, AppearanceMode.system);
      expect(state.accent, isNull);
      expect(state.selectedRoute, '/');
      expect(
        state.capabilityFor('nope').availability,
        ServiceAvailability.unknown,
      );
      state.dispose();
    });

    test('notifies on appearance, accent and route changes', () {
      final state = ZettingsState();
      var notifications = 0;
      state.addListener(() => notifications++);
      state.setAppearance(AppearanceMode.dark);
      state.setAccent(const Color(0xFF00FF00));
      state.selectRoute('/network');
      expect(state.appearance, AppearanceMode.dark);
      expect(state.accent, const Color(0xFF00FF00));
      expect(state.selectedRoute, '/network');
      expect(notifications, 3);
      // No-op updates stay silent.
      state.setAppearance(AppearanceMode.dark);
      state.setAccent(const Color(0xFF00FF00));
      state.selectRoute('/network');
      expect(notifications, 3);
      state.dispose();
    });

    test('caches capabilities', () {
      final state = ZettingsState();
      state.updateCapability(
        const SystemCapability(
          name: 'network',
          availability: ServiceAvailability.available,
        ),
      );
      expect(
        state.capabilityFor('network').availability,
        ServiceAvailability.available,
      );
      state.dispose();
    });
  });
}
