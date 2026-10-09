import 'package:flutter/widgets.dart';

/// ZDL motion tokens. Restrained by design: short durations, standard M3
/// curves, and total respect for the platform reduced-motion setting.
///
/// Rule: query [enabled] before running any decorative animation. Functional
/// feedback (selection, focus) stays instant either way.
abstract final class ZdlMotion {
  static const Duration instant = Duration(milliseconds: 80);
  static const Duration fast = Duration(milliseconds: 150);
  static const Duration medium = Duration(milliseconds: 250);

  /// True when decorative motion is allowed in this [context].
  static bool enabled(BuildContext context) =>
      !MediaQuery.disableAnimationsOf(context);

  /// Zero duration when reduced motion is on, [duration] otherwise.
  static Duration resolve(BuildContext context, Duration duration) =>
      enabled(context) ? duration : Duration.zero;
}
