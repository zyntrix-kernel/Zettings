import 'package:flutter/material.dart';

/// Appearance modes supported by ZETTINGS.
///
/// OLED is a first-class mode (true-black surfaces), not a contrast hack.
/// Widgets that need a framework [ThemeMode] use [themeMode]; the app shell
/// selects the matching [ThemeData] (see ZettingsTheme).
enum AppearanceMode { system, light, dark, oled }

extension AppearanceModeX on AppearanceMode {
  ThemeMode get themeMode => switch (this) {
    AppearanceMode.system => ThemeMode.system,
    AppearanceMode.light => ThemeMode.light,
    AppearanceMode.dark || AppearanceMode.oled => ThemeMode.dark,
  };

  String get title => switch (this) {
    AppearanceMode.system => 'System default',
    AppearanceMode.light => 'Light',
    AppearanceMode.dark => 'Dark',
    AppearanceMode.oled => 'OLED black',
  };
}
