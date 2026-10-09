/// Window-width breakpoints for adaptive navigation (logical pixels).
abstract final class ZdlBreakpoints {
  /// Below this: modal navigation drawer.
  static const double compactMax = 600;

  /// compactMax..largeMin: collapsed navigation rail.
  /// At/above largeMin: expanded navigation rail.
  static const double largeMin = 1000;
}
