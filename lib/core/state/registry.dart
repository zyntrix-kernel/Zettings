import 'package:flutter/material.dart';
import 'package:zettings/core/models/setting.dart';

/// Central settings registry: the single source of truth for navigation,
/// search, deep links, and breadcrumbs.
///
/// The catalog holds exactly the pages that exist. Feature phases extend it
/// via [register]; nothing here may reference a route with no page.
class SettingsRegistry {
  SettingsRegistry([List<SettingDefinition>? seed])
    : _entries = List.of(seed ?? buildDefaultCatalog());

  final List<SettingDefinition> _entries;

  List<SettingDefinition> get entries => List.unmodifiable(_entries);

  /// Adds a definition. Duplicate ids or routes are programming errors.
  void register(SettingDefinition definition) {
    assert(
      _entries.every(
        (e) => e.id != definition.id && e.route != definition.route,
      ),
      'Duplicate setting id/route: ${definition.id} ${definition.route}',
    );
    _entries.add(definition);
  }

  SettingDefinition? byId(String id) {
    for (final entry in _entries) {
      if (entry.id == id) return entry;
    }
    return null;
  }

  SettingDefinition? byRoute(String route) {
    for (final entry in _entries) {
      if (entry.route == route) return entry;
    }
    return null;
  }

  List<SettingDefinition> forCategory(SettingCategory category) =>
      _entries.where((e) => e.category == category).toList();
}

/// Initial catalog: the ten category landing pages. Sub-pages arrive with
/// their feature phases (each registers its own entries + routes together).
List<SettingDefinition> buildDefaultCatalog() => const [
  SettingDefinition(
    id: 'system',
    title: 'System',
    description: 'About this device, display, sound, power and storage.',
    category: SettingCategory.system,
    route: '/system',
    icon: Icons.computer,
    keywords: [
      'about',
      'info',
      'device',
      'display',
      'screen',
      'sound',
      'audio',
      'volume',
      'power',
      'battery',
      'storage',
      'ram',
      'memory',
      'cpu',
      'multitasking',
      'notifications',
      'night light',
      'scaling',
    ],
    synonyms: [
      'system info',
      'about this pc',
      'sound settings',
      'battery life',
      'power plan',
      'volume control',
    ],
    searchWeight: 2,
  ),
  SettingDefinition(
    id: 'network',
    title: 'Network',
    description: 'Wi-Fi, Ethernet, VPN, proxy and firewall.',
    category: SettingCategory.network,
    route: '/network',
    icon: Icons.wifi,
    keywords: ['wifi', 'wireless', 'ethernet', 'vpn', 'proxy', 'internet'],
    synonyms: ['wireless network', 'network connections', 'internet'],
    searchWeight: 2,
  ),
  SettingDefinition(
    id: 'bluetooth-devices',
    title: 'Bluetooth & devices',
    description: 'Bluetooth, printers, USB and input devices.',
    category: SettingCategory.bluetoothDevices,
    route: '/bluetooth-devices',
    icon: Icons.bluetooth,
    keywords: ['bluetooth', 'printer', 'usb', 'mouse', 'keyboard'],
    synonyms: ['bt', 'devices', 'pair device'],
    searchWeight: 1.5,
  ),
  SettingDefinition(
    id: 'personalization',
    title: 'Personalization',
    description: 'Appearance, colors, wallpaper, themes and fonts.',
    category: SettingCategory.personalization,
    route: '/personalization',
    icon: Icons.palette,
    keywords: ['appearance', 'theme', 'wallpaper', 'colors', 'accent'],
    synonyms: ['background', 'look and feel', 'dark mode'],
    searchWeight: 1.5,
  ),
  SettingDefinition(
    id: 'accessibility',
    title: 'Accessibility',
    description: 'Vision, hearing, mobility and input assistance.',
    category: SettingCategory.accessibility,
    route: '/accessibility',
    icon: Icons.accessibility,
    keywords: ['vision', 'hearing', 'contrast', 'magnifier', 'screen reader'],
    synonyms: ['a11y', 'assistive', 'high contrast'],
  ),
  SettingDefinition(
    id: 'accounts',
    title: 'Accounts',
    description: 'Users, groups, login and session settings.',
    category: SettingCategory.accounts,
    route: '/accounts',
    icon: Icons.people,
    keywords: ['users', 'login', 'password', 'account'],
    synonyms: ['user accounts', 'sign in'],
  ),
  SettingDefinition(
    id: 'privacy-security',
    title: 'Privacy & security',
    description: 'Permissions, location, firewall and security status.',
    category: SettingCategory.privacySecurity,
    route: '/privacy-security',
    icon: Icons.shield,
    keywords: ['privacy', 'security', 'permissions', 'location', 'firewall'],
    synonyms: ['app permissions', 'tracking'],
  ),
  SettingDefinition(
    id: 'applications',
    title: 'Applications',
    description: 'Default apps, startup apps and installed packages.',
    category: SettingCategory.applications,
    route: '/applications',
    icon: Icons.apps,
    keywords: ['apps', 'default', 'startup', 'flatpak', 'snap'],
    synonyms: ['programs', 'software', 'packages'],
  ),
  SettingDefinition(
    id: 'updates',
    title: 'Updates',
    description: 'System updates, sources and firmware.',
    category: SettingCategory.updates,
    route: '/updates',
    icon: Icons.system_update,
    keywords: ['update', 'upgrade', 'firmware', 'packages'],
    synonyms: ['software update', 'system upgrade'],
  ),
  SettingDefinition(
    id: 'developer',
    title: 'Developer',
    description: 'Developer options, Wayland/X11, logs and diagnostics.',
    category: SettingCategory.developer,
    route: '/developer',
    icon: Icons.terminal,
    keywords: ['developer', 'debug', 'logs', 'wayland', 'terminal'],
    synonyms: ['dev options', 'debugging'],
    searchWeight: 0.8,
  ),
];
