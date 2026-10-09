import 'package:flutter/material.dart';

/// Top-level Hero Status Banner for desktop pages.
///
/// Features tonal background, prominent status icon/badge, title, subtitle,
/// quick status chips, and optional call-to-action buttons.
class HeroStatusCard extends StatelessWidget {
  const HeroStatusCard({
    super.key,
    required this.title,
    required this.subtitle,
    this.icon,
    this.leading,
    this.statusBadges = const [],
    this.actions = const [],
    this.margin = const EdgeInsets.only(bottom: 24),
  });

  final String title;
  final String subtitle;
  final IconData? icon;
  final Widget? leading;
  final List<Widget> statusBadges;
  final List<Widget> actions;
  final EdgeInsetsGeometry margin;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final isOled = theme.scaffoldBackgroundColor == Colors.black;

    return Padding(
      padding: margin,
      child: Container(
        decoration: BoxDecoration(
          borderRadius: BorderRadius.circular(20),
          gradient: LinearGradient(
            begin: Alignment.topLeft,
            end: Alignment.bottomRight,
            colors: isOled
                ? [const Color(0xFF16181D), const Color(0xFF0F1012)]
                : [
                    theme.colorScheme.primaryContainer.withValues(alpha: 0.4),
                    theme.colorScheme.surfaceContainerHigh,
                  ],
          ),
          border: Border.all(
            color: isOled
                ? const Color(0xFF2B2E36)
                : theme.colorScheme.outlineVariant.withValues(alpha: 0.5),
            width: 1,
          ),
        ),
        padding: const EdgeInsets.all(20),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                if (leading != null)
                  leading!
                else if (icon != null)
                  Container(
                    width: 52,
                    height: 52,
                    decoration: BoxDecoration(
                      color: isOled
                          ? const Color(0xFF22252C)
                          : theme.colorScheme.primaryContainer,
                      borderRadius: BorderRadius.circular(14),
                    ),
                    child: Icon(
                      icon,
                      size: 28,
                      color: theme.colorScheme.primary,
                    ),
                  ),
                const SizedBox(width: 16),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        title,
                        style: theme.textTheme.headlineSmall?.copyWith(
                          fontWeight: FontWeight.w700,
                        ),
                      ),
                      const SizedBox(height: 4),
                      Text(
                        subtitle,
                        style: theme.textTheme.bodyMedium?.copyWith(
                          color: theme.colorScheme.onSurfaceVariant,
                        ),
                      ),
                    ],
                  ),
                ),
              ],
            ),
            if (statusBadges.isNotEmpty) ...[
              const SizedBox(height: 16),
              Wrap(spacing: 8, runSpacing: 8, children: statusBadges),
            ],
            if (actions.isNotEmpty) ...[
              const SizedBox(height: 16),
              Row(mainAxisAlignment: MainAxisAlignment.end, children: actions),
            ],
          ],
        ),
      ),
    );
  }
}
