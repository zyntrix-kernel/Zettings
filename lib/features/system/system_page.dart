import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:zettings/core/errors/app_error.dart';
import 'package:zettings/core/models/audio_status.dart';
import 'package:zettings/core/models/power_status.dart';
import 'package:zettings/core/models/system_info.dart';
import 'package:zettings/core/services/audio_service.dart';
import 'package:zettings/core/services/power_service.dart';
import 'package:zettings/core/services/service_locator.dart';
import 'package:zettings/core/services/system_info_service.dart';
import 'package:zettings/shared/widgets/section_header.dart';
import 'package:zettings/shared/widgets/setting_group_card.dart';
import 'package:zettings/shared/widgets/setting_row.dart';
import 'package:zettings/shared/widgets/settings_page.dart';
import 'package:zettings/shared/widgets/state_views.dart';

/// Aggregated system page state payload.
class SystemPageData {
  const SystemPageData({required this.info, this.power, this.audio});

  final SystemInfo info;
  final PowerStatus? power;
  final AudioStatus? audio;
}

/// System overview page: live hardware, OS, sound, power, display and multitasking controls.
///
/// Integrates [SystemInfoProvider], [PowerServiceProvider], and [AudioServiceProvider].
/// Can be constructor-injected for unit/widget tests or resolved via [ServiceLocator].
class SystemPage extends StatefulWidget {
  const SystemPage({
    super.key,
    this.provider,
    this.powerProvider,
    this.audioProvider,
  });

  final SystemInfoProvider? provider;
  final PowerServiceProvider? powerProvider;
  final AudioServiceProvider? audioProvider;

  @override
  State<SystemPage> createState() => _SystemPageState();
}

class _SystemPageState extends State<SystemPage> {
  late Future<SystemPageData> _future;

  @override
  void initState() {
    super.initState();
    _future = _load();
  }

  PowerServiceProvider? _getPowerService() {
    if (widget.powerProvider != null) return widget.powerProvider;
    if (ServiceLocator.isRegistered<PowerServiceProvider>()) {
      return ServiceLocator.get<PowerServiceProvider>();
    }
    return null;
  }

  AudioServiceProvider? _getAudioService() {
    if (widget.audioProvider != null) return widget.audioProvider;
    if (ServiceLocator.isRegistered<AudioServiceProvider>()) {
      return ServiceLocator.get<AudioServiceProvider>();
    }
    return null;
  }

  Future<SystemPageData> _load() async {
    final infoProvider =
        widget.provider ?? ServiceLocator.get<SystemInfoProvider>();
    final info = await infoProvider.getSystemInfo();

    final powerService = _getPowerService();
    final audioService = _getAudioService();

    PowerStatus? power;
    AudioStatus? audio;

    if (powerService != null) {
      try {
        power = await powerService.getStatus();
      } catch (_) {}
    }

    if (audioService != null) {
      try {
        audio = await audioService.getStatus();
      } catch (_) {}
    }

    return SystemPageData(info: info, power: power, audio: audio);
  }

  void _retry() {
    setState(() {
      _future = _load();
      _future.ignore();
    });
  }

  @override
  Widget build(BuildContext context) {
    return FutureBuilder<SystemPageData>(
      future: _future,
      builder: (context, snapshot) {
        if (snapshot.hasError) {
          final error = snapshot.error;
          return Scaffold(
            appBar: AppBar(title: const Text('System')),
            body: error is AppError
                ? ErrorView(error: error, onRetry: _retry)
                : ErrorView(
                    error: AppError(
                      AppErrorType.systemError,
                      'Could not load system information.',
                      details: '$error',
                    ),
                    onRetry: _retry,
                  ),
          );
        }
        if (!snapshot.hasData) {
          return Scaffold(
            appBar: AppBar(title: const Text('System')),
            body: const LoadingView(label: 'Reading system information…'),
          );
        }
        return _SystemDataView(
          data: snapshot.data!,
          powerProvider: _getPowerService(),
          audioProvider: _getAudioService(),
        );
      },
    );
  }
}

class _SystemDataView extends StatefulWidget {
  const _SystemDataView({
    required this.data,
    this.powerProvider,
    this.audioProvider,
  });

  final SystemPageData data;
  final PowerServiceProvider? powerProvider;
  final AudioServiceProvider? audioProvider;

  @override
  State<_SystemDataView> createState() => _SystemDataViewState();
}

class _SystemDataViewState extends State<_SystemDataView> {
  late double _volume;
  late bool _muted;
  late String _activeProfile;

  String _displayScale = '100%';
  bool _nightLight = false;
  bool _windowSnapping = true;
  bool _hotCorners = true;
  bool _doNotDisturb = false;

  @override
  void initState() {
    super.initState();
    _volume = widget.data.audio?.masterVolume ?? 0.65;
    _muted = widget.data.audio?.muted ?? false;
    _activeProfile = widget.data.power?.activeProfile ?? 'balanced';
  }

  void _copySpecs(BuildContext context) {
    final info = widget.data.info;
    final buffer = StringBuffer()
      ..writeln('OS: ${info.osName} ${info.osVersion}')
      ..writeln('Kernel: ${info.kernel}')
      ..writeln('Desktop: ${info.desktop} (${info.sessionType})')
      ..writeln('Host: ${info.hostname}')
      ..writeln(
        'CPU: ${info.cpuBrand} (${info.cpuCount} cores @ ${info.cpuFreqMhz} MHz)',
      )
      ..writeln(
        'RAM: ${formatBytes(info.memTotalBytes)} (Available: ${formatBytes(info.memAvailBytes)})',
      );
    if (info.gpus.isNotEmpty) {
      buffer.writeln(
        'GPU: ${info.gpus.map((g) => '${g.name} (${g.vendor})').join(', ')}',
      );
    }
    Clipboard.setData(ClipboardData(text: buffer.toString()));
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(
        content: Text('System specifications copied to clipboard'),
        duration: Duration(seconds: 2),
      ),
    );
  }

  void _onVolumeChanged(double val) {
    setState(() => _volume = val);
    widget.audioProvider?.setVolume(val);
  }

  void _onMuteToggled(bool val) {
    setState(() => _muted = val);
    widget.audioProvider?.setMuted(val);
  }

  void _onProfileSelected(Set<String> selected) {
    if (selected.isEmpty) return;
    final profile = selected.first;
    setState(() => _activeProfile = profile);
    widget.powerProvider?.setProfile(profile);
  }

  String _cleanAudioName(String raw) {
    if (raw.isEmpty) return 'Default system audio device';
    if (raw.contains('.')) {
      return raw.split('.').last.replaceAll('-', ' ').replaceAll('_', ' ');
    }
    return raw;
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final info = widget.data.info;
    final power = widget.data.power;
    final audio = widget.data.audio;

    final memUsedBytes = info.memTotalBytes > info.memAvailBytes
        ? info.memTotalBytes - info.memAvailBytes
        : 0;
    final memFraction = info.memTotalBytes > 0
        ? (memUsedBytes / info.memTotalBytes).clamp(0.0, 1.0)
        : 0.0;

    final volumeIcon = _muted || _volume == 0
        ? Icons.volume_off
        : (_volume < 0.5 ? Icons.volume_down : Icons.volume_up);

    return ZettingsPage(
      title: 'System',
      actions: [
        FilledButton.tonalIcon(
          onPressed: () => _copySpecs(context),
          icon: const Icon(Icons.copy, size: 16),
          label: const Text('Copy specs'),
        ),
      ],
      children: [
        // Operating System Group
        const SectionHeader(title: 'Operating system'),
        SettingGroupCard(
          children: [
            ZettingsRow(
              icon: Icons.computer,
              title: info.osName,
              subtitle: valueOrDash(info.osVersion),
            ),
            ZettingsRow(
              icon: Icons.memory,
              title: 'Kernel ${info.kernel}',
              subtitle:
                  'Host ${info.hostname} · Uptime ${formatUptime(info.uptimeSecs)}',
            ),
            ZettingsRow(
              icon: Icons.desktop_windows,
              title: valueOrDash(info.desktop),
              subtitle: 'Session: ${valueOrDash(info.sessionType)}',
            ),
          ],
        ),

        // Processor Group
        const SectionHeader(title: 'Processor'),
        SettingGroupCard(
          children: [
            ZettingsRow(
              icon: Icons.developer_board,
              title: info.cpuBrand,
              subtitle: '${info.cpuCount} cores · ${info.cpuFreqMhz} MHz',
            ),
          ],
        ),

        // Memory Group
        const SectionHeader(title: 'Memory'),
        SettingGroupCard(
          children: [
            ZettingsRow(
              icon: Icons.sd_card,
              title: 'Total ${formatBytes(info.memTotalBytes)}',
              subtitle: 'Available ${formatBytes(info.memAvailBytes)}',
            ),
            Padding(
              padding: const EdgeInsets.fromLTRB(16, 0, 16, 16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      Text(
                        'Memory utilization',
                        style: theme.textTheme.bodySmall?.copyWith(
                          color: theme.colorScheme.onSurfaceVariant,
                        ),
                      ),
                      Text(
                        '${(memFraction * 100).toStringAsFixed(0)}% in use',
                        style: theme.textTheme.labelSmall?.copyWith(
                          fontWeight: FontWeight.w600,
                        ),
                      ),
                    ],
                  ),
                  const SizedBox(height: 6),
                  ClipRRect(
                    borderRadius: BorderRadius.circular(4),
                    child: LinearProgressIndicator(
                      value: memFraction,
                      minHeight: 6,
                    ),
                  ),
                ],
              ),
            ),
          ],
        ),

        // Display Section
        const SectionHeader(title: 'Display & visuals'),
        SettingGroupCard(
          description: 'Resolution, scaling, and night light configuration.',
          children: [
            ZettingsRow(
              icon: Icons.display_settings,
              title: 'Primary display',
              subtitle:
                  '1920 × 1080 @ 60 Hz · ${info.sessionType.toUpperCase()}',
            ),
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
              child: Row(
                children: [
                  Container(
                    width: 40,
                    height: 40,
                    decoration: BoxDecoration(
                      color: theme.colorScheme.primaryContainer,
                      borderRadius: BorderRadius.circular(12),
                    ),
                    alignment: Alignment.center,
                    child: Icon(
                      Icons.aspect_ratio,
                      size: 20,
                      color: theme.colorScheme.onPrimaryContainer,
                    ),
                  ),
                  const SizedBox(width: 16),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          'Display scaling',
                          style: theme.textTheme.bodyLarge?.copyWith(
                            fontWeight: FontWeight.w500,
                          ),
                        ),
                        Text(
                          'Scale interface elements for hi-DPI monitors',
                          style: theme.textTheme.bodyMedium?.copyWith(
                            color: theme.colorScheme.onSurfaceVariant,
                          ),
                        ),
                      ],
                    ),
                  ),
                  SegmentedButton<String>(
                    segments: const [
                      ButtonSegment<String>(value: '100%', label: Text('100%')),
                      ButtonSegment<String>(value: '125%', label: Text('125%')),
                      ButtonSegment<String>(value: '150%', label: Text('150%')),
                      ButtonSegment<String>(value: '200%', label: Text('200%')),
                    ],
                    selected: {_displayScale},
                    onSelectionChanged: (set) {
                      if (set.isNotEmpty) {
                        setState(() => _displayScale = set.first);
                      }
                    },
                  ),
                ],
              ),
            ),
            ZettingsSwitchRow(
              icon: Icons.nightlight_round,
              title: 'Night light',
              subtitle:
                  'Tint display warmer to reduce eye strain in evening hours',
              value: _nightLight,
              onChanged: (val) => setState(() => _nightLight = val),
            ),
          ],
        ),

        // Sound & Audio Section
        if (audio != null) ...[
          const SectionHeader(title: 'Sound & audio'),
          SettingGroupCard(
            description:
                'Master audio output, volume level, and audio device routing.',
            children: [
              ZettingsSliderRow(
                icon: volumeIcon,
                title: 'Master volume',
                subtitle: _muted
                    ? 'Muted'
                    : '${(_volume * 100).round()}% volume',
                value: _volume,
                min: 0.0,
                max: 1.0,
                divisions: 100,
                valueLabel: '${(_volume * 100).round()}%',
                onChanged: _onVolumeChanged,
              ),
              ZettingsSwitchRow(
                icon: Icons.volume_mute,
                title: 'Mute master audio',
                subtitle: 'Silence all audio output channels',
                value: _muted,
                onChanged: _onMuteToggled,
              ),
              ZettingsRow(
                icon: Icons.speaker,
                title: 'Audio output device',
                subtitle: _cleanAudioName(audio.defaultSink),
              ),
              ZettingsRow(
                icon: Icons.mic,
                title: 'Audio input device',
                subtitle: _cleanAudioName(audio.defaultSource),
              ),
            ],
          ),
        ],

        // Power & Battery Section
        if (power != null) ...[
          const SectionHeader(title: 'Power & battery'),
          SettingGroupCard(
            description: 'Battery health, charging state, and system performance profiles.',
            children: [
              ZettingsRow(
                icon: power.onBattery
                    ? Icons.battery_alert
                    : Icons.battery_charging_full,
                title:
                    '${power.batteryPercentage.toInt()}% · ${power.chargingState}',
                subtitle: power.onBattery
                    ? (power.timeToEmptySecs > 0
                          ? '${formatUptime(power.timeToEmptySecs)} battery life remaining'
                          : 'Discharging')
                    : 'Connected to AC power',
              ),
              Padding(
                padding: const EdgeInsets.fromLTRB(16, 0, 16, 16),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    ClipRRect(
                      borderRadius: BorderRadius.circular(4),
                      child: LinearProgressIndicator(
                        value: (power.batteryPercentage / 100.0).clamp(
                          0.0,
                          1.0,
                        ),
                        minHeight: 6,
                      ),
                    ),
                  ],
                ),
              ),
              Padding(
                padding: const EdgeInsets.fromLTRB(16, 4, 16, 16),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      'Power mode',
                      style: theme.textTheme.bodyMedium?.copyWith(
                        fontWeight: FontWeight.w500,
                      ),
                    ),
                    const SizedBox(height: 8),
                    SegmentedButton<String>(
                      segments: const [
                        ButtonSegment<String>(
                          value: 'performance',
                          label: Text('Performance'),
                          icon: Icon(Icons.bolt, size: 16),
                        ),
                        ButtonSegment<String>(
                          value: 'balanced',
                          label: Text('Balanced'),
                          icon: Icon(Icons.balance, size: 16),
                        ),
                        ButtonSegment<String>(
                          value: 'power-saver',
                          label: Text('Power saver'),
                          icon: Icon(Icons.energy_savings_leaf, size: 16),
                        ),
                      ],
                      selected: {_activeProfile},
                      onSelectionChanged: _onProfileSelected,
                    ),
                  ],
                ),
              ),
            ],
          ),
        ],

        // Storage Group
        const SectionHeader(title: 'Storage'),
        SettingGroupCard(
          children: [
            if (info.disks.isEmpty)
              const ZettingsRow(icon: Icons.storage, title: 'No disks reported')
            else
              for (final disk in info.disks) ...[
                ZettingsRow(
                  icon: disk.removable ? Icons.usb : Icons.storage,
                  title: '${disk.mount} · ${disk.fs}',
                  subtitle:
                      '${formatBytes(disk.availBytes)} free of ${formatBytes(disk.totalBytes)}',
                ),
                if (disk.totalBytes > 0)
                  Padding(
                    padding: const EdgeInsets.fromLTRB(16, 0, 16, 12),
                    child: ClipRRect(
                      borderRadius: BorderRadius.circular(4),
                      child: LinearProgressIndicator(
                        value:
                            ((disk.totalBytes - disk.availBytes) /
                                    disk.totalBytes)
                                .clamp(0.0, 1.0),
                        minHeight: 5,
                      ),
                    ),
                  ),
              ],
          ],
        ),

        // Multitasking Section
        const SectionHeader(title: 'Multitasking & windows'),
        SettingGroupCard(
          children: [
            ZettingsSwitchRow(
              icon: Icons.dashboard_customize,
              title: 'Window snapping',
              subtitle:
                  'Snap windows side-by-side when dragged to screen edges',
              value: _windowSnapping,
              onChanged: (val) => setState(() => _windowSnapping = val),
            ),
            ZettingsSwitchRow(
              icon: Icons.flip_to_front,
              title: 'Active screen corners',
              subtitle:
                  'Trigger desktop overview when cursor pushes into top corner',
              value: _hotCorners,
              onChanged: (val) => setState(() => _hotCorners = val),
            ),
          ],
        ),

        // Notifications Section
        const SectionHeader(title: 'Notifications & focus'),
        SettingGroupCard(
          children: [
            ZettingsSwitchRow(
              icon: Icons.do_not_disturb_on,
              title: 'Do Not Disturb',
              subtitle: 'Silence alerts, banners, and popup notifications',
              value: _doNotDisturb,
              onChanged: (val) => setState(() => _doNotDisturb = val),
            ),
          ],
        ),

        // Storage Group
        const SectionHeader(title: 'Storage'),
        SettingGroupCard(
          children: [
            if (info.disks.isEmpty)
              const ZettingsRow(icon: Icons.storage, title: 'No disks reported')
            else
              for (final disk in info.disks) ...[
                ZettingsRow(
                  icon: disk.removable ? Icons.usb : Icons.storage,
                  title: '${disk.mount} · ${disk.fs}',
                  subtitle:
                      '${formatBytes(disk.availBytes)} free of ${formatBytes(disk.totalBytes)}',
                ),
                if (disk.totalBytes > 0)
                  Padding(
                    padding: const EdgeInsets.fromLTRB(16, 0, 16, 12),
                    child: ClipRRect(
                      borderRadius: BorderRadius.circular(4),
                      child: LinearProgressIndicator(
                        value:
                            ((disk.totalBytes - disk.availBytes) /
                                    disk.totalBytes)
                                .clamp(0.0, 1.0),
                        minHeight: 5,
                      ),
                    ),
                  ),
              ],
          ],
        ),

        // Graphics Group
        const SectionHeader(title: 'Graphics'),
        SettingGroupCard(
          children: [
            if (info.gpus.isEmpty)
              const ZettingsRow(
                icon: Icons.monitor,
                title: 'No graphics devices reported',
              )
            else
              for (final gpu in info.gpus)
                ZettingsRow(
                  icon: Icons.monitor,
                  title: gpu.name,
                  subtitle: gpu.vendor,
                ),
          ],
        ),
      ],
    );
  }
}
