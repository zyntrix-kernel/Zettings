import 'package:flutter/material.dart';

/// Top-level settings categories. Stable by design: adding a category must not
/// renumber or rename existing ones (routes and deep links depend on them).
enum SettingCategory {
  system,
  network,
  bluetoothDevices,
  personalization,
  accessibility,
  accounts,
  privacySecurity,
  applications,
  updates,
  developer,
}

/// Display metadata for a [SettingCategory].
extension SettingCategoryMeta on SettingCategory {
  String get title => switch (this) {
    SettingCategory.system => 'System',
    SettingCategory.network => 'Network',
    SettingCategory.bluetoothDevices => 'Bluetooth & devices',
    SettingCategory.personalization => 'Personalization',
    SettingCategory.accessibility => 'Accessibility',
    SettingCategory.accounts => 'Accounts',
    SettingCategory.privacySecurity => 'Privacy & security',
    SettingCategory.applications => 'Applications',
    SettingCategory.updates => 'Updates',
    SettingCategory.developer => 'Developer',
  };

  IconData get icon => switch (this) {
    SettingCategory.system => Icons.computer,
    SettingCategory.network => Icons.wifi,
    SettingCategory.bluetoothDevices => Icons.bluetooth,
    SettingCategory.personalization => Icons.palette,
    SettingCategory.accessibility => Icons.accessibility,
    SettingCategory.accounts => Icons.people,
    SettingCategory.privacySecurity => Icons.shield,
    SettingCategory.applications => Icons.apps,
    SettingCategory.updates => Icons.system_update,
    SettingCategory.developer => Icons.terminal,
  };

  /// Base route path for the category, e.g. '/network'.
  String get routePath => switch (this) {
    SettingCategory.bluetoothDevices => '/bluetooth-devices',
    SettingCategory.privacySecurity => '/privacy-security',
    _ => '/${name.toLowerCase()}',
  };
}

/// Structured metadata for one settings page.
///
/// This is the schema of the central registry (Phase 6 turns the static table
/// in the router into a full indexed registry; the fields stay the same).
class SettingDefinition {
  const SettingDefinition({
    required this.id,
    required this.title,
    required this.description,
    required this.category,
    required this.route,
    required this.icon,
    this.keywords = const [],
    this.synonyms = const [],
    this.searchWeight = 1.0,
    this.requiredCapabilities = const [],
  });

  final String id;
  final String title;
  final String description;
  final SettingCategory category;

  /// Stable route path, e.g. '/network/wifi'.
  final String route;

  final IconData icon;
  final List<String> keywords;
  final List<String> synonyms;

  /// Relative search ranking weight. Higher surfaces earlier on ties.
  final double searchWeight;

  /// Capability names (see [SystemCapability]) required to use this page.
  final List<String> requiredCapabilities;
}
