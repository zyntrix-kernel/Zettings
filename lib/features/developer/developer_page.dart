import 'package:flutter/material.dart';
import 'package:zettings/core/ipc/zettings_ipc_client.dart';
import 'package:zettings/core/services/service_locator.dart';
import 'package:zettings/shared/widgets/hero_status_card.dart';
import 'package:zettings/shared/widgets/section_header.dart';
import 'package:zettings/shared/widgets/setting_group_card.dart';
import 'package:zettings/shared/widgets/setting_row.dart';
import 'package:zettings/shared/widgets/settings_page.dart';

/// Developer settings page: Rust IPC backend status, Wayland diagnostics,
/// and debugging tools.
class DeveloperPage extends StatefulWidget {
  const DeveloperPage({super.key});

  @override
  State<DeveloperPage> createState() => _DeveloperPageState();
}

class _DeveloperPageState extends State<DeveloperPage> {
  bool _debugLogging = true;
  bool _showPerformanceOverlay = false;
  bool _inspectSemantics = false;
  bool _pinging = false;

  Future<void> _pingDaemon() async {
    if (_pinging) return;
    setState(() => _pinging = true);
    final sw = Stopwatch()..start();
    try {
      if (!ServiceLocator.isRegistered<ZettingsIpcClient>()) {
        throw 'IPC Client is not registered';
      }
      final client = ServiceLocator.get<ZettingsIpcClient>();
      final version = await client.ping();
      sw.stop();
      final ms = (sw.elapsedMicroseconds / 1000.0).toStringAsFixed(2);
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text('Daemon response: "$version" · Latency: ${ms}ms'),
          backgroundColor: Colors.green.shade800,
        ),
      );
    } catch (e) {
      sw.stop();
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text('IPC communication failed: $e'),
          backgroundColor: Theme.of(context).colorScheme.error,
        ),
      );
    } finally {
      if (mounted) setState(() => _pinging = false);
    }
  }

  void _showJournalDialog() {
    final entries = [
      {
        'time': '16:29:10',
        'unit': 'systemd[1]',
        'msg': 'Started Zettings D-Bus System Integration Service.',
      },
      {
        'time': '16:29:11',
        'unit': 'zsettings-daemon',
        'msg': 'Registered bus name "org.zyntrix.Zettings" on session bus.',
      },
      {
        'time': '16:29:12',
        'unit': 'NetworkManager',
        'msg': 'wlan0: state change 70 -> 100 (IP configuration complete).',
      },
      {
        'time': '16:29:12',
        'unit': 'pipewire',
        'msg': 'Core audio graph initialized (48000Hz, 1024 buffer).',
      },
      {
        'time': '16:29:14',
        'unit': 'polkitd[942]',
        'msg': 'Session c2 authorized for org.freedesktop.login1.power-off.',
      },
      {
        'time': '16:29:15',
        'unit': 'zsettings-daemon',
        'msg': 'Discovered 2 GPUs, 1 active display (Wayland 3840x2160@144Hz).',
      },
    ];

    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Systemd Journal (journalctl -u zsettings)'),
        content: SizedBox(
          width: 580,
          child: Container(
            decoration: BoxDecoration(
              color: const Color(0xFF101216),
              borderRadius: BorderRadius.circular(10),
              border: Border.all(color: const Color(0xFF2B2E36)),
            ),
            padding: const EdgeInsets.all(12),
            child: SingleChildScrollView(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                mainAxisSize: MainAxisSize.min,
                children: [
                  for (final e in entries) ...[
                    Padding(
                      padding: const EdgeInsets.symmetric(vertical: 4),
                      child: RichText(
                        text: TextSpan(
                          style: const TextStyle(
                            fontFamily: 'monospace',
                            fontSize: 12,
                            color: Color(0xFFD1D5DB),
                          ),
                          children: [
                            TextSpan(
                              text: '${e['time']} ',
                              style: const TextStyle(color: Color(0xFF9CA3AF)),
                            ),
                            TextSpan(
                              text: '${e['unit']}: ',
                              style: const TextStyle(
                                color: Color(0xFF60A5FA),
                                fontWeight: FontWeight.bold,
                              ),
                            ),
                            TextSpan(text: e['msg']!),
                          ],
                        ),
                      ),
                    ),
                  ],
                ],
              ),
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
              ScaffoldMessenger.of(context).showSnackBar(
                const SnackBar(content: Text('Journal log output copied')),
              );
            },
            child: const Text('Copy Output'),
          ),
        ],
      ),
    );
  }

  void _showDbusMonitorDialog() {
    final messages = [
      {
        'type': 'method_call',
        'path': '/org/zyntrix/Zettings',
        'interface': 'org.zyntrix.Zettings',
        'member': 'GetSystemInfo',
      },
      {
        'type': 'method_return',
        'path': '/org/zyntrix/Zettings',
        'interface': 'org.zyntrix.Zettings',
        'member': 'GetSystemInfo -> (SystemInfo struct)',
      },
      {
        'type': 'signal',
        'path': '/org/freedesktop/NetworkManager',
        'interface': 'org.freedesktop.NetworkManager',
        'member': 'StateChanged(uint32 70)',
      },
      {
        'type': 'signal',
        'path': '/org/bluez/hci0',
        'interface': 'org.freedesktop.DBus.Properties',
        'member': 'PropertiesChanged("org.bluez.Adapter1", ...)',
      },
    ];

    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('D-Bus Session Bus Monitor'),
        content: SizedBox(
          width: 540,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              for (final m in messages) ...[
                ListTile(
                  contentPadding: EdgeInsets.zero,
                  leading: Icon(
                    m['type'] == 'signal'
                        ? Icons.sensors
                        : (m['type'] == 'method_call'
                              ? Icons.arrow_forward
                              : Icons.check),
                    size: 18,
                    color: m['type'] == 'signal' ? Colors.amber : Colors.blue,
                  ),
                  title: Text(
                    m['member']!,
                    style: const TextStyle(
                      fontFamily: 'monospace',
                      fontWeight: FontWeight.w600,
                      fontSize: 12,
                    ),
                  ),
                  subtitle: Text('${m['interface']}\n${m['path']}'),
                  isThreeLine: true,
                ),
                const Divider(height: 8),
              ],
            ],
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
      title: 'Developer',
      subtitle: 'Developer options, Wayland/X11, logs and diagnostics.',
      children: [
        // Hero Card
        const HeroStatusCard(
          icon: Icons.terminal,
          title: 'Developer & Diagnostics',
          subtitle: 'Rust IPC backend, D-Bus session bus, and platform subsystem telemetry',
          statusBadges: [
            Chip(
              avatar: Icon(Icons.bolt, size: 16, color: Colors.amber),
              label: Text('IPC: org.zyntrix.Zettings Active'),
            ),
            Chip(
              avatar: Icon(Icons.desktop_windows, size: 16),
              label: Text('Display: Wayland (KWin 6.0)'),
            ),
          ],
        ),

        // Backend Architecture
        const SectionHeader(title: 'Rust backend & D-Bus IPC'),
        SettingGroupCard(
          description: 'Typed IPC communication between Flutter presentation and Linux system daemon.',
          children: [
            const ZettingsRow(
              icon: Icons.memory,
              title: 'Zettings Rust Daemon',
              subtitle: 'tokio runtime + zbus D-Bus framework · Session bus registered',
              trailing: Chip(
                label: Text(
                  'Connected',
                  style: TextStyle(
                    color: Colors.green,
                    fontWeight: FontWeight.w600,
                  ),
                ),
              ),
            ),
            const ZettingsRow(
              icon: Icons.route,
              title: 'D-Bus service path',
              subtitle: 'Destination: org.zyntrix.Zettings · Path: /org/zyntrix/Zettings',
            ),
            ZettingsRow(
              icon: Icons.refresh,
              title: 'Test IPC roundtrip',
              subtitle:
                  'Executes ping / probe diagnostics against backend daemon',
              trailing: OutlinedButton(
                onPressed: _pinging ? null : _pingDaemon,
                child: _pinging
                    ? const SizedBox(
                        width: 14,
                        height: 14,
                        child: CircularProgressIndicator(strokeWidth: 2),
                      )
                    : const Text('Ping daemon'),
              ),
            ),
          ],
        ),

        // Debugging & Telemetry
        const SectionHeader(title: 'UI debugging & performance overlays'),
        SettingGroupCard(
          children: [
            ZettingsSwitchRow(
              icon: Icons.bug_report,
              title: 'Verbose debug logging',
              subtitle: 'Emit internal route, service, and lifecycle events to stdout / systemd journal',
              value: _debugLogging,
              onChanged: (val) => setState(() => _debugLogging = val),
            ),
            ZettingsSwitchRow(
              icon: Icons.speed,
              title: 'Flutter performance overlay',
              subtitle: 'Display GPU and UI thread frame rasterization graphs',
              value: _showPerformanceOverlay,
              onChanged: (val) => setState(() => _showPerformanceOverlay = val),
            ),
            ZettingsSwitchRow(
              icon: Icons.accessibility_new,
              title: 'Inspect accessibility semantics',
              subtitle: 'Visualize screen-reader bounding boxes and accessibility tree',
              value: _inspectSemantics,
              onChanged: (val) => setState(() => _inspectSemantics = val),
            ),
          ],
        ),

        // System Logs
        const SectionHeader(title: 'System logs & utilities'),
        SettingGroupCard(
          children: [
            ZettingsRow(
              icon: Icons.receipt_long,
              title: 'Systemd journal (journalctl)',
              subtitle: 'Inspect system and application daemon log entries',
              trailing: const Icon(Icons.chevron_right),
              onTap: _showJournalDialog,
            ),
            ZettingsRow(
              icon: Icons.data_object,
              title: 'D-Bus monitor tool',
              subtitle: 'Inspect live signals and method calls passing through the session bus',
              trailing: const Icon(Icons.chevron_right),
              onTap: _showDbusMonitorDialog,
            ),
          ],
        ),
      ],
    );
  }
}
