import 'package:flutter/material.dart';
import 'package:zettings/core/theme/appearance.dart';

/// Material Design 3 Shape Tokens (m3.material.io/styles/shape/overview).
abstract final class M3Shape {
  /// Corner radius: 0dp
  static const double none = 0;

  /// Corner radius: 4dp (snackbars, badges)
  static const double extraSmall = 4;

  /// Corner radius: 8dp (chips, small buttons, text fields)
  static const double small = 8;

  /// Corner radius: 12dp (menus, squircle icons, dropdowns)
  static const double medium = 12;

  /// Corner radius: 16dp (cards, dialogs, setting groups)
  static const double large = 16;

  /// Corner radius: 28dp (search bars, floating action buttons, hero banners)
  static const double extraLarge = 28;

  /// Full pill shape (stadium border)
  static const double full = 9999;
}

/// Material Design 3 Motion Tokens (m3.material.io/styles/motion/overview).
abstract final class M3Motion {
  /// Emphasized easing curve (standard entering / container transform)
  static const Curve emphasized = Cubic(0.2, 0.0, 0.0, 1.0);

  /// Emphasized decelerate curve (incoming elements)
  static const Curve emphasizedDecelerate = Cubic(0.05, 0.7, 0.1, 1.0);

  /// Emphasized accelerate curve (exiting elements)
  static const Curve emphasizedAccelerate = Cubic(0.3, 0.0, 0.8, 0.15);

  /// Standard easing curve
  static const Curve standard = Cubic(0.2, 0.0, 0.0, 1.0);

  /// Standard decelerate curve
  static const Curve standardDecelerate = Cubic(0.0, 0.0, 0.2, 1.0);

  /// Standard accelerate curve
  static const Curve standardAccelerate = Cubic(0.3, 0.0, 1.0, 1.0);

  // Duration tokens
  static const Duration durationShort1 = Duration(milliseconds: 50);
  static const Duration durationShort2 = Duration(milliseconds: 100);
  static const Duration durationShort3 = Duration(milliseconds: 150);
  static const Duration durationShort4 = Duration(milliseconds: 200);
  static const Duration durationMedium1 = Duration(milliseconds: 250);
  static const Duration durationMedium2 = Duration(milliseconds: 300);
  static const Duration durationMedium3 = Duration(milliseconds: 350);
  static const Duration durationMedium4 = Duration(milliseconds: 400);
  static const Duration durationLong1 = Duration(milliseconds: 450);
  static const Duration durationLong2 = Duration(milliseconds: 500);
}

/// Predefined accent colors available in Zettings for dynamic theme seeding.
class AccentOption {
  const AccentOption(this.label, this.color);
  final String label;
  final Color color;
}

/// Centralized Material 3 theme factory. The single place color schemes are
/// built; widgets must use `Theme.of(context).colorScheme`, never raw colors.
abstract final class ZettingsTheme {
  /// Default accent seed: Zyntrix Blue.
  static const Color defaultSeed = Color(0xFF4F6BED);

  /// Palette of available dynamic accents.
  static const List<AccentOption> accentPalette = [
    AccentOption('Zyntrix Blue', Color(0xFF4F6BED)),
    AccentOption('Electric Violet', Color(0xFF7C4DFF)),
    AccentOption('Emerald Teal', Color(0xFF00BFA5)),
    AccentOption('Sunset Coral', Color(0xFFFF5722)),
    AccentOption('Rose Crimson', Color(0xFFF50057)),
    AccentOption('Amber Gold', Color(0xFFFF9100)),
    AccentOption('Sky Cyan', Color(0xFF00B0FF)),
  ];

  static ThemeData light([Color seed = defaultSeed]) {
    final scheme = ColorScheme.fromSeed(seedColor: seed);
    return _buildTheme(scheme, scaffoldBg: scheme.surfaceContainerLowest);
  }

  static ThemeData dark([Color seed = defaultSeed]) {
    final scheme = ColorScheme.fromSeed(
      seedColor: seed,
      brightness: Brightness.dark,
    );
    return _buildTheme(scheme, scaffoldBg: scheme.surfaceContainerLowest);
  }

  /// True-black variant of [dark] for OLED displays.
  static ThemeData oled([Color seed = defaultSeed]) {
    final baseScheme = ColorScheme.fromSeed(
      seedColor: seed,
      brightness: Brightness.dark,
    );
    final scheme = baseScheme.copyWith(
      surface: Colors.black,
      surfaceContainerLowest: Colors.black,
      surfaceContainerLow: const Color(0xFF0B0B0C),
      surfaceContainer: const Color(0xFF121314),
      surfaceContainerHigh: const Color(0xFF1A1B1E),
      surfaceContainerHighest: const Color(0xFF24262B),
    );
    return _buildTheme(
      scheme,
      scaffoldBg: Colors.black,
    ).copyWith(colorScheme: scheme, scaffoldBackgroundColor: Colors.black);
  }

  static ThemeData _buildTheme(
    ColorScheme scheme, {
    required Color scaffoldBg,
  }) {
    return ThemeData(
      useMaterial3: true,
      colorScheme: scheme,
      scaffoldBackgroundColor: scaffoldBg,
      cardTheme: CardThemeData(
        elevation: 0,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(M3Shape.large),
        ),
        clipBehavior: Clip.antiAlias,
      ),
      listTileTheme: ListTileThemeData(
        contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(M3Shape.medium),
        ),
      ),
      dividerTheme: DividerThemeData(
        color: scheme.outlineVariant.withValues(alpha: 0.35),
        thickness: 1,
        space: 1,
      ),
      navigationRailTheme: NavigationRailThemeData(
        backgroundColor: scaffoldBg,
        elevation: 0,
        indicatorColor: scheme.secondaryContainer,
        indicatorShape: const StadiumBorder(),
        selectedIconTheme: IconThemeData(color: scheme.onSecondaryContainer),
        unselectedIconTheme: IconThemeData(color: scheme.onSurfaceVariant),
        selectedLabelTextStyle: TextStyle(
          color: scheme.onSurface,
          fontWeight: FontWeight.w600,
        ),
        unselectedLabelTextStyle: TextStyle(
          color: scheme.onSurfaceVariant,
          fontWeight: FontWeight.w500,
        ),
      ),
      navigationDrawerTheme: NavigationDrawerThemeData(
        backgroundColor: scaffoldBg,
        elevation: 0,
        indicatorColor: scheme.secondaryContainer,
        indicatorShape: const StadiumBorder(),
      ),
      searchBarTheme: SearchBarThemeData(
        elevation: const WidgetStatePropertyAll(0),
        backgroundColor: WidgetStatePropertyAll(scheme.surfaceContainerHigh),
        shape: WidgetStatePropertyAll(
          RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(M3Shape.extraLarge),
          ),
        ),
      ),
      searchViewTheme: SearchViewThemeData(
        backgroundColor: scheme.surfaceContainerHigh,
        elevation: 2,
        headerHintStyle: TextStyle(color: scheme.onSurfaceVariant),
      ),
      switchTheme: SwitchThemeData(
        thumbIcon: WidgetStateProperty.resolveWith<Icon?>((states) {
          if (states.contains(WidgetState.selected)) {
            return Icon(Icons.check, size: 14, color: scheme.onPrimary);
          }
          return null;
        }),
      ),
      sliderTheme: SliderThemeData(
        trackHeight: 6,
        thumbShape: const RoundSliderThumbShape(enabledThumbRadius: 10),
        overlayShape: const RoundSliderOverlayShape(overlayRadius: 20),
        activeTrackColor: scheme.primary,
        inactiveTrackColor: scheme.surfaceContainerHighest,
        thumbColor: scheme.primary,
      ),
      chipTheme: ChipThemeData(
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(M3Shape.small),
        ),
        side: BorderSide(
          color: scheme.outlineVariant.withValues(alpha: 0.5),
          width: 1,
        ),
        backgroundColor: scheme.surfaceContainerLow,
        labelStyle: TextStyle(
          fontSize: 12,
          fontWeight: FontWeight.w500,
          color: scheme.onSurface,
        ),
      ),
      segmentedButtonTheme: SegmentedButtonThemeData(
        style: ButtonStyle(
          shape: WidgetStatePropertyAll(
            RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(M3Shape.full),
            ),
          ),
        ),
      ),
      filledButtonTheme: FilledButtonThemeData(
        style: ButtonStyle(
          shape: WidgetStatePropertyAll(
            RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(M3Shape.full),
            ),
          ),
        ),
      ),
      outlinedButtonTheme: OutlinedButtonThemeData(
        style: ButtonStyle(
          shape: WidgetStatePropertyAll(
            RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(M3Shape.full),
            ),
          ),
          side: WidgetStatePropertyAll(
            BorderSide(color: scheme.outlineVariant, width: 1),
          ),
        ),
      ),
      tooltipTheme: TooltipThemeData(
        decoration: BoxDecoration(
          color: scheme.inverseSurface,
          borderRadius: BorderRadius.circular(M3Shape.small),
        ),
        textStyle: TextStyle(color: scheme.onInverseSurface, fontSize: 12),
        waitDuration: const Duration(milliseconds: 400),
      ),
      snackBarTheme: SnackBarThemeData(
        behavior: SnackBarBehavior.floating,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(M3Shape.medium),
        ),
      ),
      badgeTheme: BadgeThemeData(
        backgroundColor: scheme.error,
        textColor: scheme.onError,
      ),
    );
  }

  /// Resolves the [ThemeData] for an appearance [mode] and optional accent.
  static ThemeData resolve(AppearanceMode mode, [Color? seed]) {
    final accent = seed ?? defaultSeed;
    return switch (mode) {
      AppearanceMode.system || AppearanceMode.light => light(accent),
      AppearanceMode.dark => dark(accent),
      AppearanceMode.oled => oled(accent),
    };
  }
}
