import 'dart:async';

import 'package:flutter/material.dart';
import 'package:zettings/core/services/prefs_service.dart';
import 'package:zettings/core/services/service_locator.dart';
import 'package:zettings/core/state/app_state.dart';
import 'package:zettings/core/theme/appearance.dart';
import 'package:zettings/core/theme/zettings_theme.dart';
import 'package:zettings/shared/widgets/hero_status_card.dart';
import 'package:zettings/shared/widgets/section_header.dart';
import 'package:zettings/shared/widgets/setting_group_card.dart';
import 'package:zettings/shared/widgets/setting_row.dart';
import 'package:zettings/shared/widgets/settings_page.dart';

/// Personalization settings page: interactive theme mode switching,
/// real-time dynamic accent seeding, typography scale, and desktop visual effects.
class PersonalizationPage extends StatefulWidget {
  const PersonalizationPage({super.key});

  @override
  State<PersonalizationPage> createState() => _PersonalizationPageState();
}

class _PersonalizationPageState extends State<PersonalizationPage> {
  bool _transparencyEnabled = true;
  double _fontScale = 1.0;
  String _animationLevel = 'Full';

  String _selectedWallpaper = 'Zyntrix Nebula';
  String _desktopStyle = 'Zyntrix Breeze Material (Breeze Dark)';
  String _cursorTheme = 'Breeze Snow (24px)';

  static const List<Map<String, dynamic>> _wallpapers = [
    {
      'name': 'Zyntrix Nebula',
      'colors': [Color(0xFF1E1035), Color(0xFF3B1E6D), Color(0xFF6C3BFF)],
      'tag': '4K HDR',
    },
    {
      'name': 'Aurora Borealis',
      'colors': [Color(0xFF062226), Color(0xFF0F5A60), Color(0xFF00B4D8)],
      'tag': '4K HDR',
    },
    {
      'name': 'Obsidian Flow',
      'colors': [Color(0xFF0F1115), Color(0xFF1E2129), Color(0xFF323642)],
      'tag': 'OLED',
    },
    {
      'name': 'Solar Flare',
      'colors': [Color(0xFF3E1208), Color(0xFF8B2500), Color(0xFFFF6B35)],
      'tag': '4K',
    },
    {
      'name': 'Cyber Dusk',
      'colors': [Color(0xFF120B2E), Color(0xFF3A1C71), Color(0xFFD76D77)],
      'tag': '4K',
    },
  ];

  void _showDesktopStyleDialog() {
    final styles = [
      'Zyntrix Breeze Material (Breeze Dark)',
      'Breeze Dark (Standard KDE)',
      'Breeze Light (Clean)',
      'Oxygen Modern (Classic)',
    ];

    showDialog(
      context: context,
      builder: (ctx) {
        final theme = Theme.of(context);
        return AlertDialog(
          title: const Text('Plasma Desktop Style'),
          content: SizedBox(
            width: 380,
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                for (final s in styles) ...[
                  ListTile(
                    contentPadding: EdgeInsets.zero,
                    leading: Icon(
                      _desktopStyle == s
                          ? Icons.radio_button_checked
                          : Icons.radio_button_off,
                      color: _desktopStyle == s
                          ? theme.colorScheme.primary
                          : null,
                    ),
                    title: Text(s),
                    onTap: () {
                      setState(() => _desktopStyle = s);
                      Navigator.of(context).pop();
                      ScaffoldMessenger.of(context).showSnackBar(
                        SnackBar(content: Text('Applied desktop style: $s')),
                      );
                    },
                  ),
                ],
              ],
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: const Text('Cancel'),
            ),
          ],
        );
      },
    );
  }

  void _showCursorThemeDialog() {
    final cursors = [
      'Breeze Snow (24px)',
      'Breeze Classic (24px)',
      'Zyntrix Glow (32px)',
      'DMZ White (24px)',
    ];

    showDialog(
      context: context,
      builder: (ctx) {
        final theme = Theme.of(context);
        return AlertDialog(
          title: const Text('Cursor Theme'),
          content: SizedBox(
            width: 380,
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                for (final c in cursors) ...[
                  ListTile(
                    contentPadding: EdgeInsets.zero,
                    leading: Icon(
                      _cursorTheme == c
                          ? Icons.radio_button_checked
                          : Icons.radio_button_off,
                      color: _cursorTheme == c
                          ? theme.colorScheme.primary
                          : null,
                    ),
                    title: Text(c),
                    onTap: () {
                      setState(() => _cursorTheme = c);
                      Navigator.of(context).pop();
                      ScaffoldMessenger.of(context).showSnackBar(
                        SnackBar(content: Text('Applied cursor theme: $c')),
                      );
                    },
                  ),
                ],
              ],
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: const Text('Cancel'),
            ),
          ],
        );
      },
    );
  }

  void _showWallpaperChooser() {
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Select Desktop Wallpaper'),
        content: SizedBox(
          width: 520,
          child: SingleChildScrollView(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                const Text(
                  'Choose from official curated Zyntrix OS backgrounds:',
                ),
                const SizedBox(height: 16),
                Wrap(
                  spacing: 12,
                  runSpacing: 12,
                  children: [
                    for (final wp in _wallpapers)
                      InkWell(
                        onTap: () {
                          setState(
                            () => _selectedWallpaper = wp['name'] as String,
                          );
                          Navigator.of(context).pop();
                          ScaffoldMessenger.of(context).showSnackBar(
                            SnackBar(
                              content: Text(
                                'Wallpaper changed to "${wp['name']}"',
                              ),
                            ),
                          );
                        },
                        borderRadius: BorderRadius.circular(12),
                        child: Container(
                          width: 150,
                          height: 100,
                          decoration: BoxDecoration(
                            borderRadius: BorderRadius.circular(12),
                            gradient: LinearGradient(
                              begin: Alignment.topLeft,
                              end: Alignment.bottomRight,
                              colors: wp['colors'] as List<Color>,
                            ),
                            border: Border.all(
                              color: _selectedWallpaper == wp['name']
                                  ? Theme.of(context).colorScheme.primary
                                  : Colors.transparent,
                              width: 3,
                            ),
                          ),
                          padding: const EdgeInsets.all(8),
                          alignment: Alignment.bottomLeft,
                          child: Text(
                            wp['name'] as String,
                            style: const TextStyle(
                              color: Colors.white,
                              fontWeight: FontWeight.w600,
                              fontSize: 12,
                              shadows: [
                                Shadow(blurRadius: 4, color: Colors.black),
                              ],
                            ),
                          ),
                        ),
                      ),
                  ],
                ),
              ],
            ),
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

  @override
  Widget build(BuildContext context) {
    final state = AppScope.of(context);
    final prefs = ServiceLocator.get<PrefsService>();

    return ZettingsPage(
      title: 'Personalization',
      subtitle: 'Appearance, colors, wallpaper, themes and fonts.',
      children: [
        // Hero Card
        HeroStatusCard(
          icon: Icons.palette,
          title: 'Look & Feel',
          subtitle:
              'Customize Zyntrix desktop theme, colors, and visual effects',
          statusBadges: [
            Chip(
              avatar: const Icon(Icons.style, size: 16),
              label: Text('Theme: ${state.appearance.title}'),
            ),
            Chip(
              avatar: const Icon(Icons.auto_awesome, size: 16),
              label: Text(
                state.accent == null
                    ? 'Accent: Default'
                    : 'Custom accent active',
              ),
            ),
          ],
        ),

        // Appearance Mode
        const SectionHeader(title: 'Theme mode'),
        SettingGroupCard(
          description: 'Select system appearance mode. OLED mode offers true-black power saving.',
          children: [
            Padding(
              padding: const EdgeInsets.all(16),
              child: SegmentedButton<AppearanceMode>(
                segments: const [
                  ButtonSegment(
                    value: AppearanceMode.light,
                    icon: Icon(Icons.light_mode),
                    label: Text('Light'),
                  ),
                  ButtonSegment(
                    value: AppearanceMode.dark,
                    icon: Icon(Icons.dark_mode),
                    label: Text('Dark'),
                  ),
                  ButtonSegment(
                    value: AppearanceMode.oled,
                    icon: Icon(Icons.contrast),
                    label: Text('OLED black'),
                  ),
                ],
                selected: {state.appearance},
                onSelectionChanged: (selected) {
                  final mode = selected.first;
                  state.setAppearance(mode);
                  unawaited(prefs.setAppearance(mode));
                },
              ),
            ),
          ],
        ),

        // Accent Colors
        const SectionHeader(title: 'Accent color'),
        SettingGroupCard(
          description: 'Dynamically re-seeds the Material 3 color system across Zyntrix OS.',
          children: [
            Padding(
              padding: const EdgeInsets.all(16),
              child: Wrap(
                spacing: 12,
                runSpacing: 12,
                crossAxisAlignment: WrapCrossAlignment.center,
                children: [
                  _AccentColorChip(
                    color: ZettingsTheme.defaultSeed,
                    label: 'Default',
                    isSelected: state.accent == null,
                    onTap: () {
                      state.setAccent(null);
                      unawaited(prefs.setAccent(null));
                    },
                  ),
                  for (final opt in ZettingsTheme.accentPalette)
                    _AccentColorChip(
                      color: opt.color,
                      label: opt.label,
                      isSelected:
                          state.accent?.toARGB32() == opt.color.toARGB32(),
                      onTap: () {
                        state.setAccent(opt.color);
                        unawaited(prefs.setAccent(opt.color));
                      },
                    ),
                ],
              ),
            ),
          ],
        ),

        // Desktop Visual Effects
        const SectionHeader(title: 'Visual effects & window behavior'),
        SettingGroupCard(
          children: [
            ZettingsSwitchRow(
              icon: Icons.blur_on,
              title: 'Window transparency & blur',
              subtitle:
                  'Enable subtle background blur and translucent surfaces',
              value: _transparencyEnabled,
              onChanged: (val) => setState(() => _transparencyEnabled = val),
            ),
            ZettingsSegmentedRow<String>(
              icon: Icons.animation,
              title: 'Window animation intensity',
              subtitle: 'Controls transition speeds and physics effects',
              segments: const [
                ButtonSegment(
                  value: 'Full',
                  label: Text('Full'),
                  icon: Icon(Icons.speed),
                ),
                ButtonSegment(
                  value: 'Reduced',
                  label: Text('Reduced'),
                  icon: Icon(Icons.slow_motion_video),
                ),
                ButtonSegment(
                  value: 'Off',
                  label: Text('Off'),
                  icon: Icon(Icons.motion_photos_off),
                ),
              ],
              selected: _animationLevel,
              onSelectionChanged: (val) =>
                  setState(() => _animationLevel = val),
            ),
            ZettingsSliderRow(
              icon: Icons.format_size,
              title: 'Text scale factor',
              subtitle: 'Adjust desktop font size dynamically',
              value: _fontScale,
              min: 0.8,
              max: 1.4,
              divisions: 6,
              valueLabel: '${(_fontScale * 100).toInt()}%',
              onChanged: (val) => setState(() => _fontScale = val),
            ),
          ],
        ),

        // Plasma Themes & Wallpapers
        const SectionHeader(title: 'KDE Plasma themes & wallpapers'),
        SettingGroupCard(
          children: [
            ZettingsRow(
              icon: Icons.wallpaper,
              title: 'Wallpaper',
              subtitle: '$_selectedWallpaper (4K Ultra HD)',
              trailing: OutlinedButton(
                onPressed: _showWallpaperChooser,
                child: const Text('Change wallpaper'),
              ),
            ),
            // Live wallpaper preview strip
            Padding(
              padding: const EdgeInsets.fromLTRB(16, 4, 16, 16),
              child: SizedBox(
                height: 72,
                child: ListView.separated(
                  scrollDirection: Axis.horizontal,
                  itemCount: _wallpapers.length,
                  separatorBuilder: (_, _) => const SizedBox(width: 10),
                  itemBuilder: (context, index) {
                    final wp = _wallpapers[index];
                    final isSelected = _selectedWallpaper == wp['name'];
                    return InkWell(
                      onTap: () => setState(
                        () => _selectedWallpaper = wp['name'] as String,
                      ),
                      borderRadius: BorderRadius.circular(10),
                      child: Container(
                        width: 110,
                        decoration: BoxDecoration(
                          borderRadius: BorderRadius.circular(10),
                          gradient: LinearGradient(
                            begin: Alignment.topLeft,
                            end: Alignment.bottomRight,
                            colors: wp['colors'] as List<Color>,
                          ),
                          border: Border.all(
                            color: isSelected
                                ? Theme.of(context).colorScheme.primary
                                : Theme.of(context).colorScheme.outlineVariant
                                      .withValues(alpha: 0.4),
                            width: isSelected ? 2.5 : 1.0,
                          ),
                        ),
                        padding: const EdgeInsets.all(6),
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          mainAxisAlignment: MainAxisAlignment.spaceBetween,
                          children: [
                            Align(
                              alignment: Alignment.topRight,
                              child: isSelected
                                  ? Container(
                                      decoration: const BoxDecoration(
                                        color: Colors.white,
                                        shape: BoxShape.circle,
                                      ),
                                      padding: const EdgeInsets.all(2),
                                      child: const Icon(
                                        Icons.check,
                                        size: 10,
                                        color: Colors.black,
                                      ),
                                    )
                                  : const SizedBox(height: 14),
                            ),
                            Text(
                              wp['name'] as String,
                              style: const TextStyle(
                                color: Colors.white,
                                fontWeight: FontWeight.w600,
                                fontSize: 10,
                                shadows: [
                                  Shadow(blurRadius: 3, color: Colors.black),
                                ],
                              ),
                              maxLines: 1,
                              overflow: TextOverflow.ellipsis,
                            ),
                          ],
                        ),
                      ),
                    );
                  },
                ),
              ),
            ),
            ZettingsRow(
              icon: Icons.format_paint,
              title: 'Plasma desktop style',
              subtitle: _desktopStyle,
              trailing: const Icon(Icons.chevron_right),
              onTap: _showDesktopStyleDialog,
            ),
            ZettingsRow(
              icon: Icons.mouse,
              title: 'Cursor theme',
              subtitle: _cursorTheme,
              trailing: const Icon(Icons.chevron_right),
              onTap: _showCursorThemeDialog,
            ),
          ],
        ),
      ],
    );
  }
}

class _AccentColorChip extends StatelessWidget {
  const _AccentColorChip({
    required this.color,
    required this.label,
    required this.isSelected,
    required this.onTap,
  });

  final Color color;
  final String label;
  final bool isSelected;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return Tooltip(
      message: label,
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(20),
        child: Container(
          width: 36,
          height: 36,
          decoration: BoxDecoration(
            color: color,
            shape: BoxShape.circle,
            border: Border.all(
              color: isSelected ? Colors.white : Colors.transparent,
              width: 3,
            ),
            boxShadow: [
              if (isSelected)
                BoxShadow(
                  color: color.withValues(alpha: 0.5),
                  blurRadius: 8,
                  spreadRadius: 1,
                ),
            ],
          ),
          alignment: Alignment.center,
          child: isSelected
              ? const Icon(Icons.check, color: Colors.white, size: 18)
              : null,
        ),
      ),
    );
  }
}
