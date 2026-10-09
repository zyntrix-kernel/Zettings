import 'package:flutter/material.dart';
import 'package:zettings/core/services/prefs_service.dart';
import 'package:zettings/core/services/service_locator.dart';
import 'package:zettings/core/state/app_state.dart';
import 'package:zettings/shared/widgets/hero_status_card.dart';
import 'package:zettings/shared/widgets/section_header.dart';
import 'package:zettings/shared/widgets/setting_group_card.dart';
import 'package:zettings/shared/widgets/setting_row.dart';
import 'package:zettings/shared/widgets/settings_page.dart';

/// Accessibility settings page: Vision, Hearing, Mobility, and Motion preferences.
class AccessibilityPage extends StatefulWidget {
  const AccessibilityPage({super.key});

  @override
  State<AccessibilityPage> createState() => _AccessibilityPageState();
}

class _AccessibilityPageState extends State<AccessibilityPage> {
  bool _screenReader = false;
  bool _monoAudio = false;
  bool _flashAlerts = false;
  bool _stickyKeys = false;
  bool _slowKeys = false;
  double _cursorSize = 24.0;

  ZettingsState? _findState(BuildContext context) {
    try {
      return AppScope.of(context);
    } catch (_) {
      return null;
    }
  }

  void _updateHighContrast(BuildContext context, bool val) {
    final state = _findState(context);
    state?.setHighContrast(val);
    if (ServiceLocator.isRegistered<PrefsService>()) {
      ServiceLocator.get<PrefsService>().setHighContrast(val);
    }
    setState(() {});
  }

  void _updateLargeText(BuildContext context, bool val) {
    final state = _findState(context);
    state?.setLargeText(val);
    if (ServiceLocator.isRegistered<PrefsService>()) {
      ServiceLocator.get<PrefsService>().setLargeText(val);
    }
    setState(() {});
  }

  void _updateReduceMotion(BuildContext context, bool val) {
    final state = _findState(context);
    state?.setReducedMotion(val);
    if (ServiceLocator.isRegistered<PrefsService>()) {
      ServiceLocator.get<PrefsService>().setReducedMotion(val);
    }
    setState(() {});
  }

  @override
  Widget build(BuildContext context) {
    final state = _findState(context);
    final highContrast = state?.highContrast ?? false;
    final largeText = state?.largeText ?? false;
    final reduceMotion = state?.reducedMotion ?? false;

    return ZettingsPage(
      title: 'Accessibility',
      subtitle: 'Vision, hearing, mobility and input assistance.',
      children: [
        // Hero Card
        const HeroStatusCard(
          icon: Icons.accessibility,
          title: 'Assistive Technologies',
          subtitle: 'Configure vision, hearing, mobility, and cognitive accessibility options',
          statusBadges: [
            Chip(
              avatar: Icon(Icons.visibility, size: 16),
              label: Text('Vision adjustments'),
            ),
            Chip(
              avatar: Icon(Icons.hearing, size: 16),
              label: Text('Hearing aids'),
            ),
            Chip(
              avatar: Icon(Icons.motion_photos_off, size: 16),
              label: Text('Reduced motion support'),
            ),
          ],
        ),

        // Vision
        const SectionHeader(title: 'Vision'),
        SettingGroupCard(
          children: [
            ZettingsSwitchRow(
              icon: Icons.contrast,
              title: 'High contrast mode',
              subtitle: 'Enhance visual contrast of borders, text, and active controls',
              value: highContrast,
              onChanged: (val) => _updateHighContrast(context, val),
            ),
            ZettingsSwitchRow(
              icon: Icons.format_size,
              title: 'Large text',
              subtitle: 'Increase system typography scale across applications',
              value: largeText,
              onChanged: (val) => _updateLargeText(context, val),
            ),
            ZettingsSwitchRow(
              icon: Icons.record_voice_over,
              title: 'Screen reader (Orca)',
              subtitle: 'Spoken feedback and braille display output',
              value: _screenReader,
              onChanged: (val) => setState(() => _screenReader = val),
            ),
            ZettingsSliderRow(
              icon: Icons.mouse,
              title: 'Mouse cursor size',
              subtitle: 'Scale cursor visibility',
              value: _cursorSize,
              min: 24,
              max: 48,
              divisions: 3,
              valueLabel: '${_cursorSize.toInt()} px',
              onChanged: (val) => setState(() => _cursorSize = val),
            ),
          ],
        ),

        // Motion
        const SectionHeader(title: 'Motion & animation'),
        SettingGroupCard(
          children: [
            ZettingsSwitchRow(
              icon: Icons.motion_photos_off,
              title: 'Reduce motion',
              subtitle: 'Minimize decorative transitions and disable parallax effects',
              value: reduceMotion,
              onChanged: (val) => _updateReduceMotion(context, val),
            ),
          ],
        ),

        // Hearing
        const SectionHeader(title: 'Hearing'),
        SettingGroupCard(
          children: [
            ZettingsSwitchRow(
              icon: Icons.hearing,
              title: 'Mono audio',
              subtitle: 'Combine left and right audio channels into one',
              value: _monoAudio,
              onChanged: (val) => setState(() => _monoAudio = val),
            ),
            ZettingsSwitchRow(
              icon: Icons.flash_on,
              title: 'Flash screen for alerts',
              subtitle: 'Provide visual screen flash when audio notifications trigger',
              value: _flashAlerts,
              onChanged: (val) => setState(() => _flashAlerts = val),
            ),
          ],
        ),

        // Mobility
        const SectionHeader(title: 'Mobility & keyboard'),
        SettingGroupCard(
          children: [
            ZettingsSwitchRow(
              icon: Icons.keyboard,
              title: 'Sticky keys',
              subtitle: 'Press modifier keys (Ctrl, Alt, Shift) sequentially instead of together',
              value: _stickyKeys,
              onChanged: (val) => setState(() => _stickyKeys = val),
            ),
            ZettingsSwitchRow(
              icon: Icons.timer,
              title: 'Slow keys',
              subtitle:
                  'Require holding a key briefly before registering keystroke',
              value: _slowKeys,
              onChanged: (val) => setState(() => _slowKeys = val),
            ),
          ],
        ),
      ],
    );
  }
}
