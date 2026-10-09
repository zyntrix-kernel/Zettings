/// Desktop spacing scale. Single source for padding/gaps so pages share
/// one density instead of inventing their own.
abstract final class ZdlSpacing {
  static const double xs = 4;
  static const double sm = 8;
  static const double md = 12;
  static const double lg = 16;
  static const double xl = 24;
  static const double xxl = 32;

  /// Maximum content width for settings pages on wide windows.
  static const double maxContentWidth = 720;

  /// Horizontal page padding.
  static const double pageHorizontal = 24;
}
