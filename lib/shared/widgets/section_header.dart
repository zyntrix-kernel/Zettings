import 'package:flutter/material.dart';
import 'package:zettings/core/theme/spacing.dart';

/// Section header used above every group of settings on a page.
class SectionHeader extends StatelessWidget {
  const SectionHeader({super.key, required this.title, this.action});

  final String title;
  final Widget? action;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final action = this.action;
    return Padding(
      padding: const EdgeInsets.fromLTRB(
        ZdlSpacing.lg,
        ZdlSpacing.xl,
        ZdlSpacing.lg,
        ZdlSpacing.sm,
      ),
      child: Row(
        children: [
          Expanded(
            child: Text(
              title,
              style: theme.textTheme.titleSmall?.copyWith(
                color: theme.colorScheme.primary,
              ),
              semanticsLabel: title,
            ),
          ),
          action ?? const SizedBox.shrink(),
        ],
      ),
    );
  }
}
