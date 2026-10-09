import 'package:flutter/material.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:zettings/core/theme/appearance.dart';

/// Persistent user preferences: appearance, accent, recent searches.
///
/// Thin typed wrapper over [SharedPreferences]; all keys live here.
class PrefsService {
  PrefsService(this._prefs);

  final SharedPreferences _prefs;

  static Future<PrefsService> load() async {
    final prefs = await SharedPreferences.getInstance();
    return PrefsService(prefs);
  }

  static const _appearanceKey = 'appearance';
  static const _accentKey = 'accent_argb';
  static const _recentsKey = 'recent_searches';
  static const _reducedMotionKey = 'reduced_motion';
  static const _highContrastKey = 'high_contrast';
  static const _largeTextKey = 'large_text';

  AppearanceMode appearance() {
    final index = _prefs.getInt(_appearanceKey);
    if (index == null || index < 0 || index >= AppearanceMode.values.length) {
      return AppearanceMode.system;
    }
    return AppearanceMode.values[index];
  }

  Future<void> setAppearance(AppearanceMode mode) =>
      _prefs.setInt(_appearanceKey, mode.index);

  Color? accent() {
    final argb = _prefs.getInt(_accentKey);
    return argb == null ? null : Color(argb);
  }

  Future<void> setAccent(Color? accent) => accent == null
      ? _prefs.remove(_accentKey)
      : _prefs.setInt(_accentKey, accent.toARGB32());

  List<String> recents() => _prefs.getStringList(_recentsKey) ?? [];

  Future<void> setRecents(List<String> recents) =>
      _prefs.setStringList(_recentsKey, recents);

  bool reducedMotion() => _prefs.getBool(_reducedMotionKey) ?? false;

  Future<void> setReducedMotion(bool value) =>
      _prefs.setBool(_reducedMotionKey, value);

  bool highContrast() => _prefs.getBool(_highContrastKey) ?? false;

  Future<void> setHighContrast(bool value) =>
      _prefs.setBool(_highContrastKey, value);

  bool largeText() => _prefs.getBool(_largeTextKey) ?? false;

  Future<void> setLargeText(bool value) => _prefs.setBool(_largeTextKey, value);
}
