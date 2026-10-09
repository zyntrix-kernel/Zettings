import 'dart:async';

import 'package:flutter/material.dart';
import 'package:zettings/shared/widgets/hero_status_card.dart';
import 'package:zettings/shared/widgets/section_header.dart';
import 'package:zettings/shared/widgets/setting_group_card.dart';
import 'package:zettings/shared/widgets/setting_row.dart';
import 'package:zettings/shared/widgets/settings_page.dart';

/// Updates settings page: PackageKit / APT updates, firmware updates (fwupd),
/// and automatic update preferences.
class UpdatesPage extends StatefulWidget {
  const UpdatesPage({super.key});

  @override
  State<UpdatesPage> createState() => _UpdatesPageState();
}

class _UpdatesPageState extends State<UpdatesPage> {
  bool _checking = false;
  bool _autoUpdates = true;
  bool _meteredDownloads = false;

  void _checkUpdates() {
    setState(() => _checking = true);
    Timer(const Duration(milliseconds: 1200), () {
      if (mounted) {
        setState(() => _checking = false);
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(
            content: Text(
              'All software packages and security definitions are up to date',
            ),
            duration: Duration(seconds: 2),
          ),
        );
      }
    });
  }

  String _channel = 'Current';

  void _showPackageSourcesDialog() {
    final repos = [
      {
        'title': 'Zyntrix OS Core',
        'url': 'deb http://archive.zyntrix.org/noble noble main restricted',
        'enabled': true,
      },
      {
        'title': 'Kubuntu Backports',
        'url': 'deb http://ppa.launchpad.net/kubuntu-ppa/backports/ubuntu noble main',
        'enabled': true,
      },
      {
        'title': 'Canonical Partner Repositories',
        'url': 'deb http://archive.canonical.com/ubuntu noble partner',
        'enabled': false,
      },
      {
        'title': 'LVFS Hardware Firmware Mirror',
        'url': 'https://cdn.fwupd.org/downloads/firmware.xml.gz',
        'enabled': true,
      },
    ];

    showDialog(
      context: context,
      builder: (ctx) => StatefulBuilder(
        builder: (context, setDialogState) {
          return AlertDialog(
            title: const Text('Package Sources (APT / LVFS)'),
            content: SizedBox(
              width: 520,
              child: SingleChildScrollView(
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    for (final r in repos) ...[
                      SwitchListTile(
                        contentPadding: EdgeInsets.zero,
                        value: r['enabled'] as bool,
                        title: Text(
                          r['title'] as String,
                          style: const TextStyle(fontWeight: FontWeight.w600),
                        ),
                        subtitle: Text(
                          r['url'] as String,
                          style: const TextStyle(fontSize: 12),
                        ),
                        onChanged: (val) {
                          setDialogState(() => r['enabled'] = val);
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
                child: const Text('Close'),
              ),
              FilledButton.tonal(
                onPressed: () {
                  Navigator.of(context).pop();
                  ScaffoldMessenger.of(context).showSnackBar(
                    const SnackBar(
                      content: Text(
                        'APT repository cache refreshed successfully',
                      ),
                    ),
                  );
                },
                child: const Text('Refresh Cache'),
              ),
            ],
          );
        },
      ),
    );
  }

  void _showFirmwareDialog() {
    final devices = [
      {
        'name': 'System Motherboard (UEFI BIOS)',
        'vendor': 'InsydeH2O / AMI',
        'version': '1.14.0',
        'status': 'Up to date',
      },
      {
        'name': 'Samsung NVMe SSD 980 PRO',
        'vendor': 'Samsung Semiconductor',
        'version': '5B2QGXA7',
        'status': 'Up to date',
      },
      {
        'name': 'Intel AX210 Wi-Fi 6E Controller',
        'vendor': 'Intel Corp.',
        'version': '22.130.0',
        'status': 'Up to date',
      },
    ];

    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Device Firmware (fwupd / LVFS)'),
        content: SizedBox(
          width: 480,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              for (final d in devices) ...[
                ListTile(
                  contentPadding: EdgeInsets.zero,
                  leading: const Icon(Icons.developer_board),
                  title: Text(d['name']!),
                  subtitle: Text('${d['vendor']} · Version ${d['version']}'),
                  trailing: Chip(
                    label: Text(
                      d['status']!,
                      style: const TextStyle(color: Colors.green, fontSize: 11),
                    ),
                  ),
                ),
              ],
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('Close'),
          ),
          FilledButton(
            onPressed: () {
              Navigator.of(context).pop();
              ScaffoldMessenger.of(context).showSnackBar(
                const SnackBar(
                  content: Text(
                    'LVFS check complete: All firmware is up to date',
                  ),
                ),
              );
            },
            child: const Text('Check LVFS'),
          ),
        ],
      ),
    );
  }

  void _showUpdateHistoryDialog() {
    final history = [
      {
        'title': 'linux-image-6.8.0-31-generic',
        'desc': 'Kernel security update and stability improvements',
        'date': 'Today, 10:14 AM',
      },
      {
        'title': 'pipewire 1.0.5-1ubuntu1',
        'desc': 'PipeWire low-latency multimedia routing framework',
        'date': 'Yesterday',
      },
      {
        'title': 'plasma-workspace 6.0.4-0zyntrix1',
        'desc': 'KDE Plasma desktop workspace and Wayland compositor',
        'date': 'May 12, 2026',
      },
      {
        'title': 'openssl 3.0.13-0ubuntu3.1',
        'desc': 'Cryptographic library and TLS protocol update',
        'date': 'May 8, 2026',
      },
    ];

    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Recent Update History'),
        content: SizedBox(
          width: 500,
          child: SingleChildScrollView(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                for (final item in history) ...[
                  ListTile(
                    contentPadding: EdgeInsets.zero,
                    leading: const Icon(
                      Icons.check_circle_outline,
                      color: Colors.green,
                    ),
                    title: Text(
                      item['title']!,
                      style: const TextStyle(fontWeight: FontWeight.w600),
                    ),
                    subtitle: Text('${item['desc']}\n${item['date']}'),
                    isThreeLine: true,
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
            child: const Text('Done'),
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return ZettingsPage(
      title: 'Updates',
      subtitle: 'System updates, sources and firmware.',
      actions: [
        FilledButton.icon(
          onPressed: _checking ? null : _checkUpdates,
          icon: _checking
              ? const SizedBox(
                  width: 14,
                  height: 14,
                  child: CircularProgressIndicator(strokeWidth: 2),
                )
              : const Icon(Icons.refresh, size: 18),
          label: Text(_checking ? 'Checking…' : 'Check for updates'),
        ),
      ],
      children: [
        // Hero Status Banner
        const HeroStatusCard(
          icon: Icons.check_circle,
          title: 'System is Up to Date',
          subtitle:
              'Last checked: Today · Zyntrix OS 24.04 LTS (Kubuntu 24.04 base)',
          statusBadges: [
            Chip(
              avatar: Icon(Icons.security, size: 16, color: Colors.green),
              label: Text('Security updates applied'),
            ),
            Chip(
              avatar: Icon(Icons.memory, size: 16),
              label: Text('Kernel 6.8.0 latest'),
            ),
          ],
        ),

        // Update Configuration
        const SectionHeader(title: 'Update preferences'),
        SettingGroupCard(
          children: [
            ZettingsSegmentedRow<String>(
              icon: Icons.alt_route,
              title: 'Update release channel',
              subtitle: 'Choose between production stability and early access features',
              segments: const [
                ButtonSegment(
                  value: 'Current',
                  label: Text('Current'),
                  icon: Icon(Icons.recommend),
                ),
                ButtonSegment(
                  value: 'LTS',
                  label: Text('LTS only'),
                  icon: Icon(Icons.shield),
                ),
                ButtonSegment(
                  value: 'Preview',
                  label: Text('Preview'),
                  icon: Icon(Icons.science),
                ),
              ],
              selected: _channel,
              onSelectionChanged: (val) => setState(() => _channel = val),
            ),
            ZettingsSwitchRow(
              icon: Icons.auto_mode,
              title: 'Automatic background updates',
              subtitle: 'Download and apply critical security patches automatically in the background',
              value: _autoUpdates,
              onChanged: (val) => setState(() => _autoUpdates = val),
            ),
            ZettingsSwitchRow(
              icon: Icons.data_saver_on,
              title: 'Download over metered connections',
              subtitle: 'Allow package downloads when connected to mobile hotspots or metered Wi-Fi',
              value: _meteredDownloads,
              onChanged: (val) => setState(() => _meteredDownloads = val),
            ),
          ],
        ),

        // Sources & Firmware
        const SectionHeader(title: 'Package sources & hardware firmware'),
        SettingGroupCard(
          children: [
            ZettingsRow(
              icon: Icons.source,
              title: 'Package sources (APT repositories)',
              subtitle: 'Manage official Zyntrix repositories, Ubuntu mirrors, and PPAs',
              trailing: const Icon(Icons.chevron_right),
              onTap: _showPackageSourcesDialog,
            ),
            ZettingsRow(
              icon: Icons.developer_board,
              title: 'Device firmware updates (fwupd / LVFS)',
              subtitle: 'UEFI BIOS and peripheral firmware up to date',
              trailing: const Icon(Icons.chevron_right),
              onTap: _showFirmwareDialog,
            ),
            ZettingsRow(
              icon: Icons.history,
              title: 'Update history',
              subtitle:
                  'Review recently installed packages and system upgrades',
              trailing: const Icon(Icons.chevron_right),
              onTap: _showUpdateHistoryDialog,
            ),
          ],
        ),
      ],
    );
  }
}
