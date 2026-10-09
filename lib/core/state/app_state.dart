import 'package:flutter/material.dart';
import 'package:zettings/core/platform/capability.dart';
import 'package:zettings/core/theme/appearance.dart';

/// Application-level state: navigation selection, appearance, capability cache.
///
/// Widgets observe this (via [AppScope]); they never perform system I/O.
/// System results flow service → [updateCapability]/selection → notify → UI.
class ZettingsState extends ChangeNotifier {
  ZettingsState({
    AppearanceMode initialAppearance = AppearanceMode.system,
    Color? initialAccent,
    bool initialReducedMotion = false,
    bool initialHighContrast = false,
    bool initialLargeText = false,
  }) : _appearance = initialAppearance,
       _accent = initialAccent,
       _reducedMotion = initialReducedMotion,
       _highContrast = initialHighContrast,
       _largeText = initialLargeText;

  AppearanceMode _appearance;
  Color? _accent;
  bool _reducedMotion;
  bool _highContrast;
  bool _largeText;
  String _selectedRoute = '/';
  final Map<String, SystemCapability> _capabilities = {};

  AppearanceMode get appearance => _appearance;

  /// User-selected accent seed. Null means the default seed.
  Color? get accent => _accent;

  bool get reducedMotion => _reducedMotion;
  bool get highContrast => _highContrast;
  bool get largeText => _largeText;

  String get selectedRoute => _selectedRoute;

  /// Capability snapshot by service name. Missing entry means "not probed".
  Map<String, SystemCapability> get capabilities =>
      Map.unmodifiable(_capabilities);

  SystemCapability capabilityFor(String name) =>
      _capabilities[name] ??
      SystemCapability(name: name, availability: ServiceAvailability.unknown);

  void setAppearance(AppearanceMode mode) {
    if (_appearance == mode) return;
    _appearance = mode;
    notifyListeners();
  }

  void setAccent(Color? accent) {
    if (_accent == accent) return;
    _accent = accent;
    notifyListeners();
  }

  void setReducedMotion(bool value) {
    if (_reducedMotion == value) return;
    _reducedMotion = value;
    notifyListeners();
  }

  void setHighContrast(bool value) {
    if (_highContrast == value) return;
    _highContrast = value;
    notifyListeners();
  }

  void setLargeText(bool value) {
    if (_largeText == value) return;
    _largeText = value;
    notifyListeners();
  }

  void selectRoute(String route) {
    if (_selectedRoute == route) return;
    _selectedRoute = route;
    notifyListeners();
  }

  void updateCapability(SystemCapability capability) {
    _capabilities[capability.name] = capability;
    notifyListeners();
  }
}

/// Exposes [ZettingsState] down the widget tree.
class AppScope extends InheritedNotifier<ZettingsState> {
  const AppScope({
    super.key,
    required ZettingsState state,
    required super.child,
  }) : super(notifier: state);

  static ZettingsState of(BuildContext context) {
    final scope = context.dependOnInheritedWidgetOfExactType<AppScope>();
    assert(scope != null, 'AppScope.of() called outside of AppScope');
    return scope!.notifier!;
  }
}
