import 'package:flutter/material.dart';
import 'package:zettings/core/errors/app_error.dart';
import 'package:zettings/core/models/bluetooth_status.dart';
import 'package:zettings/core/services/bluetooth_service.dart';
import 'package:zettings/core/services/service_locator.dart';
import 'package:zettings/shared/widgets/hero_status_card.dart';
import 'package:zettings/shared/widgets/section_header.dart';
import 'package:zettings/shared/widgets/setting_group_card.dart';
import 'package:zettings/shared/widgets/setting_row.dart';
import 'package:zettings/shared/widgets/settings_page.dart';
import 'package:zettings/shared/widgets/state_views.dart';

/// Bluetooth & devices settings page: BlueZ adapter management, paired devices,
/// input peripherals (mouse/touchpad/keyboard), and printers.
class BluetoothDevicesPage extends StatefulWidget {
  const BluetoothDevicesPage({super.key, this.provider});

  final BluetoothServiceProvider? provider;

  @override
  State<BluetoothDevicesPage> createState() => _BluetoothDevicesPageState();
}

class _BluetoothDevicesPageState extends State<BluetoothDevicesPage> {
  late Future<BluetoothStatus> _future;
  bool _naturalScrolling = true;
  bool _tapToClick = true;
  double _pointerSpeed = 1.0;

  BluetoothServiceProvider get _provider =>
      widget.provider ?? ServiceLocator.get<BluetoothServiceProvider>();

  @override
  void initState() {
    super.initState();
    _future = _provider.getStatus();
  }

  void _reload() {
    setState(() {
      _future = _provider.getStatus();
      _future.ignore();
    });
  }

  Future<void> _toggleBluetooth(bool enabled) async {
    try {
      await _provider.setEnabled(enabled);
      _reload();
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(SnackBar(content: Text('Failed to update Bluetooth: $e')));
    }
  }

  void _showAddDeviceDialog() {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Row(
          children: [
            Icon(Icons.bluetooth_searching, size: 24),
            SizedBox(width: 10),
            Text('Add a Device'),
          ],
        ),
        content: SizedBox(
          width: 400,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              const Text(
                'Make sure your device is turned on and discoverable.',
              ),
              const SizedBox(height: 16),
              const Center(
                child: Padding(
                  padding: EdgeInsets.symmetric(vertical: 8),
                  child: Row(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      SizedBox(
                        width: 18,
                        height: 18,
                        child: CircularProgressIndicator(strokeWidth: 2),
                      ),
                      SizedBox(width: 12),
                      Text('Scanning for nearby peripherals…'),
                    ],
                  ),
                ),
              ),
              const Divider(height: 24),
              ListTile(
                contentPadding: EdgeInsets.zero,
                leading: const Icon(Icons.headset),
                title: const Text('Bose QuietComfort 45'),
                subtitle: const Text('Audio headset · Ready to pair'),
                trailing: FilledButton.tonal(
                  onPressed: () {
                    Navigator.of(context).pop();
                    ScaffoldMessenger.of(context).showSnackBar(
                      const SnackBar(
                        content: Text('Paired with Bose QuietComfort 45'),
                        behavior: SnackBarBehavior.floating,
                      ),
                    );
                  },
                  child: const Text('Pair'),
                ),
              ),
              ListTile(
                contentPadding: EdgeInsets.zero,
                leading: const Icon(Icons.mouse),
                title: const Text('Apple Magic Trackpad'),
                subtitle: const Text('Input device · Ready to pair'),
                trailing: FilledButton.tonal(
                  onPressed: () {
                    Navigator.of(context).pop();
                    ScaffoldMessenger.of(context).showSnackBar(
                      const SnackBar(
                        content: Text('Paired with Apple Magic Trackpad'),
                        behavior: SnackBarBehavior.floating,
                      ),
                    );
                  },
                  child: const Text('Pair'),
                ),
              ),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('Cancel'),
          ),
        ],
      ),
    );
  }

  void _showPrintersDialog() {
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Printers (CUPS Server)'),
        content: const SizedBox(
          width: 460,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              ListTile(
                contentPadding: EdgeInsets.zero,
                leading: Icon(Icons.print, size: 28),
                title: Text('HP LaserJet Pro MFP M428fdw'),
                subtitle: Text('IPP Everywhere · Default system printer'),
                trailing: Chip(
                  label: Text(
                    'Ready',
                    style: TextStyle(color: Colors.green, fontSize: 11),
                  ),
                ),
              ),
              Divider(),
              ListTile(
                contentPadding: EdgeInsets.zero,
                leading: Icon(Icons.picture_as_pdf, size: 28),
                title: Text('Print to PDF'),
                subtitle: Text('Virtual print driver · Saves to ~/Documents'),
                trailing: Chip(
                  label: Text('Idle', style: TextStyle(fontSize: 11)),
                ),
              ),
            ],
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
                  content: Text('Test page sent to HP LaserJet Pro'),
                ),
              );
            },
            child: const Text('Print Test Page'),
          ),
        ],
      ),
    );
  }

  void _showUsbDevicesDialog() {
    final usbDevices = [
      {
        'name': 'Logitech USB Receiver',
        'id': '046d:c52b',
        'speed': '12 Mbps (USB 2.0)',
      },
      {
        'name': 'Keychron Q1 Pro',
        'id': '3434:0601',
        'speed': '12 Mbps (USB 2.0)',
      },
      {
        'name': 'SanDisk Extreme SSD',
        'id': '0781:55af',
        'speed': '10 Gbps (USB 3.2 Gen 2)',
      },
      {
        'name': 'Realtek Semiconductor USB Audio',
        'id': '0bda:4014',
        'speed': '480 Mbps (USB 2.0)',
      },
    ];

    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Connected USB Devices (lsusb)'),
        content: SizedBox(
          width: 480,
          child: SingleChildScrollView(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                for (final u in usbDevices) ...[
                  ListTile(
                    contentPadding: EdgeInsets.zero,
                    leading: const Icon(Icons.usb),
                    title: Text(
                      u['name']!,
                      style: const TextStyle(fontWeight: FontWeight.w600),
                    ),
                    subtitle: Text('Bus ID: ${u['id']} · Speed: ${u['speed']}'),
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

  IconData _iconForDeviceType(String type) => switch (type.toLowerCase()) {
    'headphones' || 'audio' || 'headset' => Icons.headphones,
    'mouse' => Icons.mouse,
    'keyboard' => Icons.keyboard,
    'gamepad' => Icons.sports_esports,
    _ => Icons.devices,
  };

  @override
  Widget build(BuildContext context) {
    return FutureBuilder<BluetoothStatus>(
      future: _future,
      builder: (context, snapshot) {
        if (snapshot.hasError) {
          final error = snapshot.error;
          return Scaffold(
            appBar: AppBar(title: const Text('Bluetooth & devices')),
            body: error is AppError
                ? ErrorView(error: error, onRetry: _reload)
                : ErrorView(
                    error: AppError(
                      AppErrorType.systemError,
                      'Could not load Bluetooth status.',
                      details: '$error',
                    ),
                    onRetry: _reload,
                  ),
          );
        }

        if (!snapshot.hasData) {
          return Scaffold(
            appBar: AppBar(title: const Text('Bluetooth & devices')),
            body: const LoadingView(label: 'Checking Bluetooth adapter…'),
          );
        }

        final status = snapshot.data!;

        return ZettingsPage(
          title: 'Bluetooth & devices',
          subtitle: 'Bluetooth, printers, USB and input devices.',
          actions: [
            if (status.enabled)
              FilledButton.icon(
                onPressed: _showAddDeviceDialog,
                icon: const Icon(Icons.add, size: 18),
                label: const Text('Add device'),
              ),
          ],
          children: [
            // Hero Card
            HeroStatusCard(
              icon: Icons.bluetooth,
              title: status.enabled ? 'Bluetooth is On' : 'Bluetooth is Off',
              subtitle: status.enabled
                  ? 'Adapter: ${status.adapterName} · ${status.connectedDevicesCount} devices connected'
                  : 'Turn on Bluetooth to connect wireless headphones, keyboards, and mice',
              statusBadges: [
                Chip(
                  avatar: Icon(
                    status.enabled
                        ? Icons.bluetooth_connected
                        : Icons.bluetooth_disabled,
                    size: 16,
                    color: status.enabled ? Colors.blue : null,
                  ),
                  label: Text(status.enabled ? 'Active' : 'Disabled'),
                ),
                if (status.enabled)
                  Chip(
                    avatar: const Icon(Icons.devices, size: 16),
                    label: Text(
                      '${status.connectedDevicesCount} devices connected',
                    ),
                  ),
              ],
            ),

            // Master Switch
            const SectionHeader(title: 'Bluetooth adapter'),
            SettingGroupCard(
              children: [
                ZettingsSwitchRow(
                  icon: Icons.bluetooth,
                  title: 'Bluetooth',
                  subtitle: status.enabled
                      ? 'Adapter: ${status.adapterName}'
                      : 'Disabled',
                  value: status.enabled,
                  onChanged: _toggleBluetooth,
                ),
              ],
            ),

            // Paired Devices
            if (status.enabled) ...[
              const SectionHeader(title: 'Paired devices'),
              SettingGroupCard(
                children: [
                  if (status.devices.isEmpty)
                    const ZettingsRow(
                      icon: Icons.devices_other,
                      title: 'No paired devices',
                      subtitle:
                          'Click "Add device" to pair a Bluetooth peripheral',
                    )
                  else
                    for (final dev in status.devices)
                      ZettingsRow(
                        icon: _iconForDeviceType(dev.deviceType),
                        title: dev.name,
                        subtitle:
                            '${dev.deviceType} · ${dev.connected ? "Connected" : "Paired"}',
                        trailing: Row(
                          mainAxisSize: MainAxisSize.min,
                          children: [
                            if (dev.batteryPercentage >= 0) ...[
                              Container(
                                padding: const EdgeInsets.symmetric(
                                  horizontal: 8,
                                  vertical: 4,
                                ),
                                decoration: BoxDecoration(
                                  color: Colors.green.withValues(alpha: 0.15),
                                  borderRadius: BorderRadius.circular(6),
                                ),
                                child: Row(
                                  mainAxisSize: MainAxisSize.min,
                                  children: [
                                    const Icon(
                                      Icons.battery_5_bar,
                                      size: 14,
                                      color: Colors.green,
                                    ),
                                    const SizedBox(width: 4),
                                    Text(
                                      '${dev.batteryPercentage}%',
                                      style: const TextStyle(
                                        fontSize: 12,
                                        fontWeight: FontWeight.w600,
                                        color: Colors.green,
                                      ),
                                    ),
                                  ],
                                ),
                              ),
                              const SizedBox(width: 8),
                            ],
                            if (dev.connected)
                              OutlinedButton(
                                onPressed: () {
                                  ScaffoldMessenger.of(context).showSnackBar(
                                    SnackBar(
                                      content: Text('Disconnected ${dev.name}'),
                                      behavior: SnackBarBehavior.floating,
                                    ),
                                  );
                                },
                                child: const Text('Disconnect'),
                              )
                            else
                              FilledButton.tonal(
                                onPressed: () {
                                  ScaffoldMessenger.of(context).showSnackBar(
                                    SnackBar(
                                      content: Text('Connected to ${dev.name}'),
                                      behavior: SnackBarBehavior.floating,
                                    ),
                                  );
                                },
                                child: const Text('Connect'),
                              ),
                          ],
                        ),
                      ),
                ],
              ),
            ],

            // Mouse & Touchpad
            const SectionHeader(title: 'Mouse & Touchpad'),
            SettingGroupCard(
              children: [
                ZettingsSliderRow(
                  icon: Icons.speed,
                  title: 'Pointer speed',
                  subtitle: 'Adjust cursor tracking acceleration',
                  value: _pointerSpeed,
                  min: 0.5,
                  max: 2.0,
                  divisions: 6,
                  valueLabel: '${(_pointerSpeed * 100).toInt()}%',
                  onChanged: (val) => setState(() => _pointerSpeed = val),
                ),
                ZettingsSwitchRow(
                  icon: Icons.swap_vert,
                  title: 'Natural scrolling',
                  subtitle: 'Content tracks finger movement direction',
                  value: _naturalScrolling,
                  onChanged: (val) => setState(() => _naturalScrolling = val),
                ),
                ZettingsSwitchRow(
                  icon: Icons.touch_app,
                  title: 'Tap to click',
                  subtitle: 'Single finger tap registers as primary click',
                  value: _tapToClick,
                  onChanged: (val) => setState(() => _tapToClick = val),
                ),
              ],
            ),

            // Printers & USB
            const SectionHeader(title: 'Printers & Peripherals'),
            SettingGroupCard(
              children: [
                ZettingsRow(
                  icon: Icons.print,
                  title: 'Printers (CUPS)',
                  subtitle: 'HP LaserJet Pro MFP (Ready)',
                  trailing: const Icon(Icons.chevron_right),
                  onTap: _showPrintersDialog,
                ),
                ZettingsRow(
                  icon: Icons.usb,
                  title: 'USB devices',
                  subtitle: 'Connected USB peripherals',
                  trailing: const Icon(Icons.chevron_right),
                  onTap: _showUsbDevicesDialog,
                ),
              ],
            ),
          ],
        );
      },
    );
  }
}
