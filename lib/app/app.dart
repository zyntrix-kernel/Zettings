import 'package:flutter/material.dart';
import 'package:zettings/app/router.dart';
import 'package:zettings/core/state/app_state.dart';
import 'package:zettings/core/theme/appearance.dart';
import 'package:zettings/core/theme/zettings_theme.dart';

/// Zettings application root. Themes come from [ZettingsTheme]; appearance
/// (light/dark/OLED) and accent come from [ZettingsState].
class ZettingsApp extends StatefulWidget {
  const ZettingsApp({super.key, this.state});

  /// Externally owned state (production bootstrap). When omitted, the app
  /// owns a default instance (used by tests and tooling).
  final ZettingsState? state;

  @override
  State<ZettingsApp> createState() => _ZettingsAppState();
}

class _ZettingsAppState extends State<ZettingsApp> {
  late final ZettingsState _state = widget.state ?? ZettingsState();
  late final bool _owned = widget.state == null;

  @override
  void dispose() {
    if (_owned) _state.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return AppScope(
      state: _state,
      child: ListenableBuilder(
        listenable: _state,
        builder: (context, _) {
          return MaterialApp(
            title: 'Zettings',
            debugShowCheckedModeBanner: false,
            themeMode: _state.appearance.themeMode,
            theme: ZettingsTheme.light(
              _state.accent ?? ZettingsTheme.defaultSeed,
            ),
            darkTheme: _state.appearance == AppearanceMode.oled
                ? ZettingsTheme.oled(_state.accent ?? ZettingsTheme.defaultSeed)
                : ZettingsTheme.dark(
                    _state.accent ?? ZettingsTheme.defaultSeed,
                  ),
            initialRoute: '/',
            onGenerateRoute: generateSettingsRoute,
            builder: (context, child) {
              final media = MediaQuery.of(context);
              return MediaQuery(
                data: media.copyWith(
                  disableAnimations:
                      _state.reducedMotion || media.disableAnimations,
                  highContrast: _state.highContrast || media.highContrast,
                  textScaler: _state.largeText
                      ? const TextScaler.linear(1.2)
                      : media.textScaler,
                ),
                child: child ?? const SizedBox.shrink(),
              );
            },
          );
        },
      ),
    );
  }
}
