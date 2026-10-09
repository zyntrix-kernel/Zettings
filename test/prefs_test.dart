import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:zettings/core/services/prefs_service.dart';
import 'package:zettings/core/theme/appearance.dart';

void main() {
  test('defaults when nothing is stored', () async {
    SharedPreferences.setMockInitialValues({});
    final prefs = await PrefsService.load();
    expect(prefs.appearance(), AppearanceMode.system);
    expect(prefs.accent(), isNull);
    expect(prefs.recents(), isEmpty);
  });

  test('appearance round-trips', () async {
    SharedPreferences.setMockInitialValues({});
    final prefs = await PrefsService.load();
    await prefs.setAppearance(AppearanceMode.oled);
    expect(prefs.appearance(), AppearanceMode.oled);
  });

  test('corrupt appearance index falls back to system', () async {
    SharedPreferences.setMockInitialValues({'appearance': 99});
    final prefs = await PrefsService.load();
    expect(prefs.appearance(), AppearanceMode.system);
  });

  test('accent round-trips including reset', () async {
    SharedPreferences.setMockInitialValues({});
    final prefs = await PrefsService.load();
    await prefs.setAccent(const Color(0xFF00FF00));
    expect(prefs.accent(), const Color(0xFF00FF00));
    await prefs.setAccent(null);
    expect(prefs.accent(), isNull);
  });

  test('recents round-trip', () async {
    SharedPreferences.setMockInitialValues({});
    final prefs = await PrefsService.load();
    await prefs.setRecents(['wifi', 'display']);
    expect(prefs.recents(), ['wifi', 'display']);
  });

  test('accessibility preferences round-trip', () async {
    SharedPreferences.setMockInitialValues({});
    final prefs = await PrefsService.load();
    expect(prefs.reducedMotion(), isFalse);
    expect(prefs.highContrast(), isFalse);
    expect(prefs.largeText(), isFalse);

    await prefs.setReducedMotion(true);
    await prefs.setHighContrast(true);
    await prefs.setLargeText(true);

    expect(prefs.reducedMotion(), isTrue);
    expect(prefs.highContrast(), isTrue);
    expect(prefs.largeText(), isTrue);
  });
}
