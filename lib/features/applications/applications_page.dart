import 'package:flutter/material.dart';
import 'package:zettings/shared/widgets/hero_status_card.dart';
import 'package:zettings/shared/widgets/section_header.dart';
import 'package:zettings/shared/widgets/setting_group_card.dart';
import 'package:zettings/shared/widgets/setting_row.dart';
import 'package:zettings/shared/widgets/settings_page.dart';

/// Applications settings page: Default apps, startup programs, and package management.
class ApplicationsPage extends StatefulWidget {
  const ApplicationsPage({super.key});

  @override
  State<ApplicationsPage> createState() => _ApplicationsPageState();
}

class _ApplicationsPageState extends State<ApplicationsPage> {
  String _browser = 'Firefox';
  String _email = 'Thunderbird';
  String _terminal = 'Konsole';
  String _fileManager = 'Dolphin';

  final List<Map<String, dynamic>> _autostartApps = [
    {
      'name': 'Klipper Clipboard History',
      'cmd': 'klipper --silent',
      'desc': 'Persists clipboard history across desktop sessions',
      'enabled': true,
      'icon': Icons.content_paste,
    },
    {
      'name': 'Nextcloud Desktop Client',
      'cmd': 'nextcloud --background',
      'desc': 'Cloud file synchronization and instant notifications',
      'enabled': true,
      'icon': Icons.cloud_sync,
    },
    {
      'name': 'Redshift Blue Light Filter',
      'cmd': 'redshift -l 37.77:-122.41',
      'desc': 'Adjusts display color temperature according to time of day',
      'enabled': true,
      'icon': Icons.nightlight_round,
    },
    {
      'name': 'Discord',
      'cmd': 'flatpak run com.discordapp.Discord --start-minimized',
      'desc': 'Voice, video, and text communication platform',
      'enabled': false,
      'icon': Icons.chat,
    },
  ];

  void _showAutostartDialog() {
    showDialog(
      context: context,
      builder: (ctx) => StatefulBuilder(
        builder: (context, setDialogState) {
          final enabledCount = _autostartApps
              .where((a) => a['enabled'] == true)
              .length;
          return AlertDialog(
            title: const Text('Autostart Programs'),
            content: SizedBox(
              width: 480,
              child: SingleChildScrollView(
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      '$enabledCount applications configured to launch on login:',
                      style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                        color: Theme.of(context).colorScheme.onSurfaceVariant,
                      ),
                    ),
                    const SizedBox(height: 12),
                    for (final app in _autostartApps) ...[
                      SwitchListTile(
                        contentPadding: EdgeInsets.zero,
                        secondary: Container(
                          width: 36,
                          height: 36,
                          decoration: BoxDecoration(
                            color: Theme.of(context)
                                .colorScheme
                                .primaryContainer,
                            borderRadius: BorderRadius.circular(8),
                          ),
                          alignment: Alignment.center,
                          child: Icon(
                            app['icon'] as IconData,
                            size: 18,
                            color: Theme.of(context)
                                .colorScheme
                                .onPrimaryContainer,
                          ),
                        ),
                        title: Text(
                          app['name'] as String,
                          style: const TextStyle(fontWeight: FontWeight.w600),
                        ),
                        subtitle: Text(app['desc'] as String),
                        value: app['enabled'] as bool,
                        onChanged: (val) {
                          setDialogState(() => app['enabled'] = val);
                          setState(() {});
                        },
                      ),
                      const Divider(height: 12),
                    ],
                  ],
                ),
              ),
            ),
            actions: [
              TextButton(
                onPressed: () => Navigator.of(context).pop(),
                child: const Text('Done'),
              ),
              FilledButton.tonal(
                onPressed: () {
                  Navigator.of(context).pop();
                  ScaffoldMessenger.of(context).showSnackBar(
                    const SnackBar(
                      content: Text(
                        'Autostart configurations saved to ~/.config/autostart/',
                      ),
                    ),
                  );
                },
                child: const Text('Save Changes'),
              ),
            ],
          );
        },
      ),
    );
  }

  void _showRuntimeInfo(String title, String details) {
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        title: Text(title),
        content: Text(details),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('OK'),
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final enabledAutostartCount = _autostartApps
        .where((a) => a['enabled'] == true)
        .length;

    return ZettingsPage(
      title: 'Applications',
      subtitle: 'Default apps, startup apps and installed packages.',
      children: [
        // Hero Card
        const HeroStatusCard(
          icon: Icons.apps,
          title: 'App Management',
          subtitle: 'Configure default handlers, startup services, and package runtimes',
          statusBadges: [
            Chip(
              avatar: Icon(Icons.check, size: 16),
              label: Text('Flatpak runtime: Ready'),
            ),
            Chip(
              avatar: Icon(Icons.check, size: 16),
              label: Text('Snap daemon: Active'),
            ),
            Chip(
              avatar: Icon(Icons.check, size: 16),
              label: Text('AppImage integration: Enabled'),
            ),
          ],
        ),

        // Default Applications
        const SectionHeader(title: 'Default applications'),
        SettingGroupCard(
          description: 'Choose default applications for web browsing, email, files, and terminal.',
          children: [
            ZettingsRow(
              icon: Icons.language,
              title: 'Web browser',
              subtitle: 'Opens web links and HTML documents',
              trailing: DropdownButton<String>(
                value: _browser,
                underline: const SizedBox.shrink(),
                items: const [
                  DropdownMenuItem(value: 'Firefox', child: Text('Firefox')),
                  DropdownMenuItem(value: 'Chromium', child: Text('Chromium')),
                  DropdownMenuItem(value: 'Brave', child: Text('Brave')),
                ],
                onChanged: (v) => setState(() => _browser = v ?? _browser),
              ),
            ),
            ZettingsRow(
              icon: Icons.email,
              title: 'Email client',
              subtitle: 'Handles mailto: links and inbox messages',
              trailing: DropdownButton<String>(
                value: _email,
                underline: const SizedBox.shrink(),
                items: const [
                  DropdownMenuItem(
                    value: 'Thunderbird',
                    child: Text('Thunderbird'),
                  ),
                  DropdownMenuItem(value: 'KMail', child: Text('KMail')),
                ],
                onChanged: (v) => setState(() => _email = v ?? _email),
              ),
            ),
            ZettingsRow(
              icon: Icons.folder,
              title: 'File manager',
              subtitle: 'Browses local directories and archives',
              trailing: DropdownButton<String>(
                value: _fileManager,
                underline: const SizedBox.shrink(),
                items: const [
                  DropdownMenuItem(
                    value: 'Dolphin',
                    child: Text('Dolphin (KDE)'),
                  ),
                  DropdownMenuItem(value: 'Nautilus', child: Text('Nautilus')),
                ],
                onChanged: (v) =>
                    setState(() => _fileManager = v ?? _fileManager),
              ),
            ),
            ZettingsRow(
              icon: Icons.terminal,
              title: 'Terminal emulator',
              subtitle: 'Command line terminal',
              trailing: DropdownButton<String>(
                value: _terminal,
                underline: const SizedBox.shrink(),
                items: const [
                  DropdownMenuItem(
                    value: 'Konsole',
                    child: Text('Konsole (KDE)'),
                  ),
                  DropdownMenuItem(
                    value: 'Alacritty',
                    child: Text('Alacritty'),
                  ),
                ],
                onChanged: (v) => setState(() => _terminal = v ?? _terminal),
              ),
            ),
          ],
        ),

        // Startup Applications
        const SectionHeader(title: 'Startup applications'),
        SettingGroupCard(
          children: [
            ZettingsRow(
              icon: Icons.power_settings_new,
              title: 'Autostart programs',
              subtitle:
                  '$enabledAutostartCount applications scheduled to launch automatically on login',
              trailing: const Icon(Icons.chevron_right),
              onTap: _showAutostartDialog,
            ),
          ],
        ),

        // Package Ecosystem
        const SectionHeader(title: 'Package management & runtimes'),
        SettingGroupCard(
          children: [
            ZettingsRow(
              icon: Icons.layers,
              title: 'Flatpak (Flathub)',
              subtitle: 'Universal sandboxed applications · Version 1.14.6',
              trailing: const Icon(Icons.info_outline, size: 20),
              onTap: () => _showRuntimeInfo(
                'Flatpak Runtime',
                'Flathub remote is configured as system-wide repository.\n'
                    'Sandboxed app runtime: org.freedesktop.Platform 23.08\n'
                    'Permission portal: xdg-desktop-portal-kde active.',
              ),
            ),
            ZettingsRow(
              icon: Icons.all_inclusive,
              title: 'Snap Store (Canonical)',
              subtitle: 'snapd daemon active · Version 2.62',
              trailing: const Icon(Icons.info_outline, size: 20),
              onTap: () => _showRuntimeInfo(
                'Snap Daemon',
                'snapd 2.62 is running via systemd socket.\n'
                    'Active core runtime: core22\n'
                    'App confinement: AppArmor strict enforcing.',
              ),
            ),
            ZettingsRow(
              icon: Icons.extension,
              title: 'AppImage Launcher',
              subtitle:
                  'Seamless desktop menu integration for portable binaries',
              trailing: const Icon(Icons.info_outline, size: 20),
              onTap: () => _showRuntimeInfo(
                'AppImage Integration',
                'AppImageLauncher daemon monitors ~/Applications.\n'
                    'Desktop entries automatically registered into KDE Plasma menu and mime database.',
              ),
            ),
          ],
        ),
      ],
    );
  }
}
