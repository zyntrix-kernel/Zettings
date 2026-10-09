import 'package:flutter/material.dart';

/// Material 3 Setting Group Container.
///
/// Encapsulates related setting items into an elevated/tonal rounded card,
/// with automatic dividers between items and optional section titles.
class SettingGroupCard extends StatelessWidget {
  const SettingGroupCard({
    super.key,
    required this.children,
    this.title,
    this.description,
    this.margin = const EdgeInsets.only(bottom: 12),
  });

  final String? title;
  final String? description;
  final List<Widget> children;
  final EdgeInsetsGeometry margin;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final isOled = theme.scaffoldBackgroundColor == Colors.black;

    return Padding(
      padding: margin,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          if (title != null) ...[
            Padding(
              padding: const EdgeInsets.fromLTRB(8, 12, 8, 6),
              child: Text(
                title!,
                style: theme.textTheme.titleSmall?.copyWith(
                  color: theme.colorScheme.primary,
                  fontWeight: FontWeight.w600,
                  letterSpacing: 0.3,
                ),
              ),
            ),
          ],
          if (description != null) ...[
            Padding(
              padding: const EdgeInsets.fromLTRB(8, 0, 8, 8),
              child: Text(
                description!,
                style: theme.textTheme.bodySmall?.copyWith(
                  color: theme.colorScheme.onSurfaceVariant,
                ),
              ),
            ),
          ],
          Card(
            elevation: 0,
            clipBehavior: Clip.antiAlias,
            color: isOled
                ? const Color(0xFF141518)
                : theme.colorScheme.surfaceContainer,
            shape: RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(16),
              side: BorderSide(
                color: isOled
                    ? const Color(0xFF282A30)
                    : theme.colorScheme.outlineVariant.withValues(alpha: 0.4),
                width: 1,
              ),
            ),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                for (var i = 0; i < children.length; i++) ...[
                  children[i],
                  if (i < children.length - 1)
                    Divider(
                      height: 1,
                      indent: 68,
                      endIndent: 16,
                      color: theme.colorScheme.outlineVariant.withValues(
                        alpha: 0.3,
                      ),
                    ),
                ],
              ],
            ),
          ),
        ],
      ),
    );
  }
}
