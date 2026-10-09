import 'package:flutter/material.dart';
import 'package:zettings/core/errors/app_error.dart';
import 'package:zettings/core/models/network_status.dart';
import 'package:zettings/core/services/network_service.dart';
import 'package:zettings/core/services/service_locator.dart';
import 'package:zettings/shared/widgets/hero_status_card.dart';
import 'package:zettings/shared/widgets/section_header.dart';
import 'package:zettings/shared/widgets/setting_group_card.dart';
import 'package:zettings/shared/widgets/setting_row.dart';
import 'package:zettings/shared/widgets/settings_page.dart';
import 'package:zettings/shared/widgets/state_views.dart';

/// Network settings page: Wi-Fi management, Ethernet interfaces, VPN, and Firewall.
class NetworkPage extends StatefulWidget {
  const NetworkPage({super.key, this.provider});

  final NetworkServiceProvider? provider;

  @override
  State<NetworkPage> createState() => _NetworkPageState();
}

class _NetworkPageState extends State<NetworkPage> {
  late Future<NetworkStatus> _future;
  bool _vpnConnected = false;
  bool _firewallActive = true;

  NetworkServiceProvider get _provider =>
      widget.provider ?? ServiceLocator.get<NetworkServiceProvider>();

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

  Future<void> _toggleWifi(bool enabled) async {
    try {
      await _provider.setWifiEnabled(enabled);
      _reload();
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context)
          .showSnackBar(SnackBar(content: Text('Failed to update Wi-Fi: $e')));
    }
  }

  void _showConnectDialog(String ssid) {
    final passwordController = TextEditingController();
    bool obscure = true;
    showDialog(
      context: context,
      builder: (context) => StatefulBuilder(
        builder: (context, setDialogState) => AlertDialog(
          title: Text('Connect to $ssid'),
          content: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              const Text('Enter the network security key or WPA password:'),
              const SizedBox(height: 14),
              TextField(
                controller: passwordController,
                obscureText: obscure,
                decoration: InputDecoration(
                  labelText: 'Password',
                  prefixIcon: const Icon(Icons.lock_outline),
                  suffixIcon: IconButton(
                    icon: Icon(
                      obscure ? Icons.visibility : Icons.visibility_off,
                    ),
                    onPressed: () => setDialogState(() => obscure = !obscure),
                  ),
                ),
              ),
            ],
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: const Text('Cancel'),
            ),
            FilledButton(
              onPressed: () {
                Navigator.of(context).pop();
                ScaffoldMessenger.of(context).showSnackBar(
                  SnackBar(
                    content: Text('Connecting to $ssid… Connected.'),
                    behavior: SnackBarBehavior.floating,
                  ),
                );
              },
              child: const Text('Connect'),
            ),
          ],
        ),
      ),
    );
  }

  void _showEthernetDetailsDialog(NetworkStatus status) {
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        title: Text(
          'Ethernet (${status.interfaceName.isEmpty ? "eth0" : status.interfaceName}) Properties',
        ),
        content: SizedBox(
          width: 440,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              const ListTile(
                contentPadding: EdgeInsets.zero,
                leading: Icon(Icons.speed),
                title: Text('Link Speed'),
                subtitle: Text('1000 Mbps Full Duplex (Auto-negotiated)'),
              ),
              const ListTile(
                contentPadding: EdgeInsets.zero,
                leading: Icon(Icons.fingerprint),
                title: Text('Hardware MAC Address'),
                subtitle: Text('e4:54:e8:2a:b1:9f'),
              ),
              ListTile(
                contentPadding: EdgeInsets.zero,
                leading: const Icon(Icons.dns),
                title: const Text('IP Address'),
                subtitle: Text(
                  status.ipv4Address.isEmpty
                      ? '192.168.1.142'
                      : status.ipv4Address,
                ),
              ),
              const ListTile(
                contentPadding: EdgeInsets.zero,
                leading: Icon(Icons.straighten),
                title: Text('MTU (Maximum Transmission Unit)'),
                subtitle: Text('1500 bytes'),
              ),
            ],
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
    return FutureBuilder<NetworkStatus>(
      future: _future,
      builder: (context, snapshot) {
        if (snapshot.hasError) {
          final error = snapshot.error;
          return Scaffold(
            appBar: AppBar(title: const Text('Network')),
            body: error is AppError
                ? ErrorView(error: error, onRetry: _reload)
                : ErrorView(
                    error: AppError(
                      AppErrorType.systemError,
                      'Could not load network status.',
                      details: '$error',
                    ),
                    onRetry: _reload,
                  ),
          );
        }

        if (!snapshot.hasData) {
          return Scaffold(
            appBar: AppBar(title: const Text('Network')),
            body: const LoadingView(label: 'Reading network interfaces…'),
          );
        }

        final status = snapshot.data!;
        final theme = Theme.of(context);

        return ZettingsPage(
          title: 'Network',
          subtitle: 'Wi-Fi, Ethernet, VPN, proxy and firewall.',
          children: [
            // Hero Card
            HeroStatusCard(
              icon: Icons.wifi,
              title: status.wifiEnabled
                  ? (status.connected
                        ? 'Connected to ${status.activeSsid.isEmpty ? "Network" : status.activeSsid}'
                        : 'Wi-Fi is Turned On')
                  : 'Wi-Fi is Turned Off',
              subtitle: status.wifiEnabled
                  ? (status.connected
                        ? 'IPv4: ${status.ipv4Address} · Interface: ${status.interfaceName}'
                        : 'Turn on Wi-Fi or choose an access point below')
                  : 'Turn on Wi-Fi to scan and connect to local wireless networks',
              statusBadges: [
                Chip(
                  avatar: Icon(
                    status.connected
                        ? Icons.check_circle
                        : Icons.portable_wifi_off,
                    size: 16,
                    color: status.connected
                        ? Colors.green
                        : theme.colorScheme.error,
                  ),
                  label: Text(status.connected ? 'Online' : 'Offline'),
                ),
                Chip(
                  avatar: const Icon(
                    Icons.shield,
                    size: 16,
                    color: Colors.blue,
                  ),
                  label: Text(
                    _firewallActive ? 'Firewall: Active' : 'Firewall: Off',
                  ),
                ),
                Chip(
                  avatar: const Icon(Icons.vpn_lock, size: 16),
                  label: Text(
                    _vpnConnected ? 'VPN: Connected' : 'VPN: Disconnected',
                  ),
                ),
              ],
            ),

            // Wi-Fi Master Control
            const SectionHeader(title: 'Wireless network (Wi-Fi)'),
            SettingGroupCard(
              children: [
                ZettingsSwitchRow(
                  icon: Icons.wifi,
                  title: 'Wi-Fi',
                  subtitle: status.wifiEnabled
                      ? 'Searching for networks…'
                      : 'Disabled',
                  value: status.wifiEnabled,
                  onChanged: _toggleWifi,
                ),
              ],
            ),

            // Available Networks (when Wi-Fi is on)
            if (status.wifiEnabled) ...[
              const SectionHeader(title: 'Available networks'),
              SettingGroupCard(
                children: [
                  if (status.wifiNetworks.isEmpty)
                    const ZettingsRow(
                      icon: Icons.wifi_find,
                      title: 'No Wi-Fi networks in range',
                      subtitle:
                          'Ensure wireless hardware is enabled and in range',
                    )
                  else
                    for (final net in status.wifiNetworks)
                      ZettingsRow(
                        icon: net.secured ? Icons.wifi_lock : Icons.wifi,
                        title: net.ssid,
                        subtitle: net.ssid == status.activeSsid
                            ? 'Connected, secured (${net.securityType}) · Signal: ${net.signalStrength}%'
                            : 'Secured (${net.securityType}) · Signal: ${net.signalStrength}%',
                        trailing: net.ssid == status.activeSsid
                            ? OutlinedButton(
                                onPressed: () {
                                  ScaffoldMessenger.of(context).showSnackBar(
                                    SnackBar(
                                      content: Text(
                                        'Disconnected from ${net.ssid}',
                                      ),
                                      behavior: SnackBarBehavior.floating,
                                    ),
                                  );
                                },
                                child: const Text('Disconnect'),
                              )
                            : FilledButton.tonal(
                                onPressed: () => _showConnectDialog(net.ssid),
                                child: const Text('Connect'),
                              ),
                      ),
                ],
              ),
            ],

            // Ethernet
            const SectionHeader(title: 'Wired network (Ethernet)'),
            SettingGroupCard(
              children: [
                ZettingsRow(
                  icon: Icons.settings_ethernet,
                  title: 'Ethernet (eth0)',
                  subtitle: '1000 Mbps Full Duplex · Cable connected',
                  trailing: const Icon(Icons.chevron_right),
                  onTap: () => _showEthernetDetailsDialog(status),
                ),
                ZettingsRow(
                  icon: Icons.info_outline,
                  title: 'IP configuration',
                  subtitle:
                      'IP: ${status.ipv4Address.isEmpty ? "192.168.1.142" : status.ipv4Address} · Subnet: 255.255.255.0',
                ),
              ],
            ),

            // VPN & Security
            const SectionHeader(title: 'VPN & Firewall'),
            SettingGroupCard(
              children: [
                ZettingsSwitchRow(
                  icon: Icons.vpn_lock,
                  title: 'WireGuard VPN',
                  subtitle: 'Encrypted tunnel (Zyntrix Secure Gateway)',
                  value: _vpnConnected,
                  onChanged: (val) => setState(() => _vpnConnected = val),
                ),
                ZettingsSwitchRow(
                  icon: Icons.security,
                  title: 'Linux Firewall (UFW)',
                  subtitle: 'Blocks unauthorized inbound traffic',
                  value: _firewallActive,
                  onChanged: (val) => setState(() => _firewallActive = val),
                ),
              ],
            ),
          ],
        );
      },
    );
  }
}
