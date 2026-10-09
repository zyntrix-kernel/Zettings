import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:zettings/core/theme/appearance.dart';
import 'package:zettings/core/theme/zettings_theme.dart';

void main() {
  group('ZettingsTheme', () {
    test('all factories produce Material 3 themes', () {
      for (final theme in [
        ZettingsTheme.light(),
        ZettingsTheme.dark(),
        ZettingsTheme.oled(),
      ]) {
        expect(theme.useMaterial3, isTrue);
      }
    });

    test('light and dark differ in brightness', () {
      expect(ZettingsTheme.light().colorScheme.brightness, Brightness.light);
      expect(ZettingsTheme.dark().colorScheme.brightness, Brightness.dark);
    });

    test('oled uses true-black surfaces', () {
      final scheme = ZettingsTheme.oled().colorScheme;
      expect(scheme.brightness, Brightness.dark);
      expect(scheme.surface, Colors.black);
      expect(scheme.surfaceContainerLowest, Colors.black);
    });

    test('custom seed changes the scheme', () {
      final a = ZettingsTheme.light(const Color(0xFF0000FF));
      final b = ZettingsTheme.light(const Color(0xFF00FF00));
      expect(a.colorScheme.primary, isNot(b.colorScheme.primary));
    });

    test('resolve maps every appearance mode', () {
      expect(
        ZettingsTheme.resolve(AppearanceMode.light).colorScheme.brightness,
        Brightness.light,
      );
      expect(
        ZettingsTheme.resolve(AppearanceMode.dark).colorScheme.brightness,
        Brightness.dark,
      );
      expect(
        ZettingsTheme.resolve(AppearanceMode.oled).colorScheme.surface,
        Colors.black,
      );
      // System falls back to the light scheme definition.
      expect(
        ZettingsTheme.resolve(AppearanceMode.system).colorScheme.brightness,
        Brightness.light,
      );
    });
  });

  group('AppearanceMode', () {
    test('maps to framework theme modes', () {
      expect(AppearanceMode.system.themeMode, ThemeMode.system);
      expect(AppearanceMode.light.themeMode, ThemeMode.light);
      expect(AppearanceMode.dark.themeMode, ThemeMode.dark);
      expect(AppearanceMode.oled.themeMode, ThemeMode.dark);
    });

    test('every mode has a title', () {
      for (final mode in AppearanceMode.values) {
        expect(mode.title, isNotEmpty);
      }
    });
  });
}
