import 'package:flutter/material.dart';
import 'package:zettings/shared/widgets/hero_status_card.dart';
import 'package:zettings/shared/widgets/section_header.dart';
import 'package:zettings/shared/widgets/setting_group_card.dart';
import 'package:zettings/shared/widgets/setting_row.dart';
import 'package:zettings/shared/widgets/settings_page.dart';

/// Privacy & security settings page: app permissions, Wayland portals,
/// firewall, and PolicyKit security policies.
class PrivacySecurityPage extends StatefulWidget {
  const PrivacySecurityPage({super.key});

  @override
  State<PrivacySecurityPage> createState() => _PrivacySecurityPageState();
}

class _PrivacySecurityPageState extends State<PrivacySecurityPage> {
  bool _location = false;
  bool _camera = true;
  bool _microphone = true;
  bool _screenCast = true;
  bool _crashReports = true;
  bool _firewall = true;

  void _showPolkitDialog() {
    final policies = [
      {
        'id': 'org.zyntrix.settings.system.update',
        'desc': 'Perform system-wide package updates and upgrades',
        'auth': 'Requires admin password',
        'icon': Icons.lock,
      },
      {
        'id': 'org.freedesktop.NetworkManager.settings.modify.system',
        'desc': 'Modify global system network connections',
        'auth': 'Requires admin authorization',
        'icon': Icons.wifi,
      },
      {
        'id': 'org.freedesktop.login1.power-off',
        'desc': 'Shutdown or reboot the local computer',
        'auth': 'Allowed without prompt',
        'icon': Icons.power_settings_new,
      },
      {
        'id': 'org.freedesktop.udisks2.filesystem-mount',
        'desc': 'Mount removable storage drives and filesystems',
        'auth': 'Allowed for active session',
        'icon': Icons.usb,
      },
      {
        'id': 'org.freedesktop.timedate1.set-time',
        'desc': 'Change system clock time and timezone',
        'auth': 'Requires admin password',
        'icon': Icons.access_time,
      },
    ];

    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('PolicyKit (Polkit) Rules'),
        content: SizedBox(
          width: 520,
          child: SingleChildScrollView(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'Active authorization rules enforced by Polkit daemon:',
                  style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                    color: Theme.of(context).colorScheme.onSurfaceVariant,
                  ),
                ),
                const SizedBox(height: 12),
                for (final p in policies) ...[
                  ListTile(
                    contentPadding: EdgeInsets.zero,
                    leading: Icon(p['icon'] as IconData, size: 22),
                    title: Text(
                      p['id'] as String,
                      style: const TextStyle(
                        fontWeight: FontWeight.w600,
                        fontSize: 13,
                      ),
                    ),
                    subtitle: Text(p['desc'] as String),
                    trailing: Chip(
                      visualDensity: VisualDensity.compact,
                      label: Text(
                        p['auth'] as String,
                        style: const TextStyle(fontSize: 10),
                      ),
                    ),
                  ),
                  const Divider(height: 8),
                ],
              ],
            ),
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('Close'),
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return ZettingsPage(
      title: 'Privacy & security',
      subtitle: 'Permissions, location, firewall and security status.',
      children: [
        // Hero Card
        HeroStatusCard(
          icon: Icons.shield,
          title: 'System Protected',
          subtitle:
              'PolicyKit authentication enforced · Wayland isolation active',
          statusBadges: [
            Chip(
              avatar: const Icon(
                Icons.verified_user,
                size: 16,
                color: Colors.green,
              ),
              label: Text(_firewall ? 'Firewall Active' : 'Firewall Disabled'),
            ),
            const Chip(
              avatar: Icon(Icons.lock, size: 16),
              label: Text('Flatpak Sandboxing: Enforced'),
            ),
          ],
        ),

        // Device Permissions
        const SectionHeader(title: 'Application permissions (XDG Portals)'),
        SettingGroupCard(
          description: 'Control desktop application hardware access through Wayland/PipeWire portals.',
          children: [
            ZettingsSwitchRow(
              icon: Icons.location_on,
              title: 'Location services',
              subtitle: 'Allow applications to determine approximate geographical position (GeoClue)',
              value: _location,
              onChanged: (val) => setState(() => _location = val),
            ),
            ZettingsSwitchRow(
              icon: Icons.videocam,
              title: 'Camera access',
              subtitle: 'Allow video capture through PipeWire camera portal',
              value: _camera,
              onChanged: (val) => setState(() => _camera = val),
            ),
            ZettingsSwitchRow(
              icon: Icons.mic,
              title: 'Microphone access',
              subtitle: 'Allow applications to record audio input',
              value: _microphone,
              onChanged: (val) => setState(() => _microphone = val),
            ),
            ZettingsSwitchRow(
              icon: Icons.screen_share,
              title: 'Screen casting & capture',
              subtitle: 'Prompt before sharing desktop contents with apps',
              value: _screenCast,
              onChanged: (val) => setState(() => _screenCast = val),
            ),
          ],
        ),

        // Security & Diagnostics
        const SectionHeader(title: 'Diagnostics & Security status'),
        SettingGroupCard(
          children: [
            ZettingsSwitchRow(
              icon: Icons.security,
              title: 'Firewall protection',
              subtitle: 'Uncomplicated Firewall (UFW) active on all interfaces',
              value: _firewall,
              onChanged: (val) => setState(() => _firewall = val),
            ),
            ZettingsSwitchRow(
              icon: Icons.analytics,
              title: 'Crash diagnostics',
              subtitle: 'Automatically send anonymous crash telemetry to improve Zyntrix OS',
              value: _crashReports,
              onChanged: (val) => setState(() => _crashReports = val),
            ),
            ZettingsRow(
              icon: Icons.admin_panel_settings,
              title: 'PolicyKit authentication privileges',
              subtitle: 'Manage Polkit rules for privileged actions',
              trailing: const Icon(Icons.chevron_right),
              onTap: _showPolkitDialog,
            ),
          ],
        ),
      ],
    );
  }
}
