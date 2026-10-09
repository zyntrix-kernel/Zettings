import 'package:flutter/material.dart';
import 'package:zettings/core/models/setting.dart';
import 'package:zettings/core/services/service_locator.dart';
import 'package:zettings/core/state/registry.dart';
import 'package:zettings/core/theme/spacing.dart';

/// Home content: Category grid built from the registry with Material 3 cards.
/// Selecting a card navigates to the category route.
class HomePage extends StatelessWidget {
  const HomePage({super.key});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final registry = ServiceLocator.get<SettingsRegistry>();
    final entries = registry.entries;

    return Align(
      alignment: Alignment.topCenter,
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 1020),
        child: CustomScrollView(
          slivers: [
            SliverToBoxAdapter(
              child: Padding(
                padding: const EdgeInsets.fromLTRB(
                  ZdlSpacing.pageHorizontal,
                  ZdlSpacing.lg,
                  ZdlSpacing.pageHorizontal,
                  ZdlSpacing.xs,
                ),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      'Settings',
                      style: theme.textTheme.headlineMedium?.copyWith(
                        fontSize: 28,
                        fontWeight: FontWeight.w700,
                        letterSpacing: -0.5,
                      ),
                    ),
                    const SizedBox(height: 6),
                    Text(
                      'Configure your Zyntrix OS environment, hardware, and preferences',
                      style: theme.textTheme.bodyMedium?.copyWith(
                        color: theme.colorScheme.onSurfaceVariant,
                      ),
                    ),
                  ],
                ),
              ),
            ),
            SliverPadding(
              padding: const EdgeInsets.all(ZdlSpacing.lg),
              sliver: SliverGrid.builder(
                gridDelegate: const SliverGridDelegateWithMaxCrossAxisExtent(
                  maxCrossAxisExtent: 290,
                  mainAxisSpacing: ZdlSpacing.lg,
                  crossAxisSpacing: ZdlSpacing.lg,
                  mainAxisExtent: 184,
                ),
                itemCount: entries.length,
                itemBuilder: (context, index) {
                  final entry = entries[index];
                  return _CategoryCard(entry: entry);
                },
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _CategoryCard extends StatefulWidget {
  const _CategoryCard({required this.entry});

  final SettingDefinition entry;

  @override
  State<_CategoryCard> createState() => _CategoryCardState();
}

class _CategoryCardState extends State<_CategoryCard> {
  bool _isHovered = false;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final isOled = theme.scaffoldBackgroundColor == Colors.black;

    final baseBorderColor = isOled
        ? const Color(0xFF282A30)
        : theme.colorScheme.outlineVariant.withValues(alpha: 0.4);
    final hoveredBorderColor = isOled
        ? theme.colorScheme.primary.withValues(alpha: 0.7)
        : theme.colorScheme.primary.withValues(alpha: 0.5);

    final baseCardColor = isOled
        ? const Color(0xFF141518)
        : theme.colorScheme.surfaceContainer;
    final hoveredCardColor = isOled
        ? const Color(0xFF1B1D22)
        : theme.colorScheme.surfaceContainerHigh;

    return MouseRegion(
      onEnter: (_) => setState(() => _isHovered = true),
      onExit: (_) => setState(() => _isHovered = false),
      cursor: SystemMouseCursors.click,
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 180),
        curve: Curves.easeOutCubic,
        transform: Matrix4.translationValues(0, _isHovered ? -3 : 0, 0),
        child: Card(
          elevation: _isHovered ? 3 : 0,
          shadowColor: theme.colorScheme.shadow.withValues(alpha: 0.15),
          clipBehavior: Clip.antiAlias,
          color: _isHovered ? hoveredCardColor : baseCardColor,
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(16),
            side: BorderSide(
              color: _isHovered ? hoveredBorderColor : baseBorderColor,
              width: _isHovered ? 1.5 : 1.0,
            ),
          ),
          child: InkWell(
            onTap: () => Navigator.of(context).pushNamed(widget.entry.route),
            child: Padding(
              padding: const EdgeInsets.all(ZdlSpacing.lg),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      AnimatedContainer(
                        duration: const Duration(milliseconds: 180),
                        width: 44,
                        height: 44,
                        decoration: BoxDecoration(
                          color: _isHovered
                              ? theme.colorScheme.primary
                              : (isOled
                                    ? const Color(0xFF22252C)
                                    : theme.colorScheme.primaryContainer),
                          borderRadius: BorderRadius.circular(12),
                        ),
                        alignment: Alignment.center,
                        child: Icon(
                          widget.entry.icon,
                          size: 24,
                          color: _isHovered
                              ? theme.colorScheme.onPrimary
                              : theme.colorScheme.primary,
                        ),
                      ),
                      AnimatedOpacity(
                        duration: const Duration(milliseconds: 150),
                        opacity: _isHovered ? 1.0 : 0.0,
                        child: Icon(
                          Icons.arrow_forward,
                          size: 18,
                          color: theme.colorScheme.primary,
                        ),
                      ),
                    ],
                  ),
                  const SizedBox(height: ZdlSpacing.md),
                  Text(
                    widget.entry.title,
                    style: theme.textTheme.titleMedium?.copyWith(
                      fontWeight: FontWeight.w600,
                    ),
                  ),
                  const SizedBox(height: ZdlSpacing.xs),
                  Expanded(
                    child: Text(
                      widget.entry.description,
                      style: theme.textTheme.bodySmall?.copyWith(
                        color: theme.colorScheme.onSurfaceVariant,
                      ),
                      maxLines: 2,
                      overflow: TextOverflow.ellipsis,
                    ),
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}

/// Stable category → service-name mapping.
String serviceNameFor(SettingCategory category) => switch (category) {
  SettingCategory.system => 'system',
  SettingCategory.network => 'network',
  SettingCategory.bluetoothDevices => 'bluetooth',
  SettingCategory.personalization => 'personalization',
  SettingCategory.accessibility => 'accessibility',
  SettingCategory.accounts => 'accounts',
  SettingCategory.privacySecurity => 'privacy',
  SettingCategory.applications => 'applications',
  SettingCategory.updates => 'updates',
  SettingCategory.developer => 'developer',
};
