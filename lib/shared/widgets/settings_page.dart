import 'package:flutter/material.dart';
import 'package:zettings/core/theme/spacing.dart';

/// Standard desktop settings page layout: max-width constrained,
/// scrollable content with desktop header hierarchy.
class ZettingsPage extends StatelessWidget {
  const ZettingsPage({
    super.key,
    required this.title,
    required this.children,
    this.subtitle,
    this.actions,
    this.leading,
  });

  final String title;
  final String? subtitle;
  final List<Widget> children;
  final List<Widget>? actions;
  final Widget? leading;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Scaffold(
      backgroundColor: Colors.transparent,
      appBar: AppBar(
        backgroundColor: Colors.transparent,
        elevation: 0,
        scrolledUnderElevation: 0,
        leading: leading,
        title: Text(
          title,
          style: theme.textTheme.headlineSmall?.copyWith(
            fontWeight: FontWeight.w600,
          ),
        ),
        actions: actions,
      ),
      body: Align(
        alignment: Alignment.topCenter,
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 860),
          child: ListView(
            padding: const EdgeInsets.fromLTRB(
              ZdlSpacing.pageHorizontal,
              ZdlSpacing.xs,
              ZdlSpacing.pageHorizontal,
              ZdlSpacing.xxl,
            ),
            children: [
              if (subtitle != null) ...[
                Padding(
                  padding: const EdgeInsets.only(bottom: 16),
                  child: Text(
                    subtitle!,
                    style: theme.textTheme.bodyMedium?.copyWith(
                      color: theme.colorScheme.onSurfaceVariant,
                    ),
                  ),
                ),
              ],
              ...children,
            ],
          ),
        ),
      ),
    );
  }
}
