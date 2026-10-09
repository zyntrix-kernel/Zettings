import 'package:flutter/material.dart';
import 'package:zettings/core/errors/app_error.dart';
import 'package:zettings/core/theme/spacing.dart';

/// Standard loading state: centered progress indicator with a label.
class LoadingView extends StatelessWidget {
  const LoadingView({super.key, this.label = 'Loading…'});

  final String label;

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Semantics(
        label: label,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const CircularProgressIndicator(),
            const SizedBox(height: ZdlSpacing.lg),
            Text(label),
          ],
        ),
      ),
    );
  }
}

/// Standard empty state: icon + title + optional hint and action.
class EmptyView extends StatelessWidget {
  const EmptyView({
    super.key,
    required this.icon,
    required this.title,
    this.hint,
    this.actionLabel,
    this.onAction,
  });

  final IconData icon;
  final String title;
  final String? hint;
  final String? actionLabel;
  final VoidCallback? onAction;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(ZdlSpacing.xl),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(icon, size: 48, color: theme.colorScheme.onSurfaceVariant),
            const SizedBox(height: ZdlSpacing.lg),
            Text(title, style: theme.textTheme.titleMedium),
            if (hint != null) ...[
              const SizedBox(height: ZdlSpacing.sm),
              Text(
                hint!,
                style: theme.textTheme.bodyMedium?.copyWith(
                  color: theme.colorScheme.onSurfaceVariant,
                ),
                textAlign: TextAlign.center,
              ),
            ],
            if (actionLabel != null) ...[
              const SizedBox(height: ZdlSpacing.lg),
              FilledButton.tonal(
                onPressed: onAction,
                child: Text(actionLabel!),
              ),
            ],
          ],
        ),
      ),
    );
  }
}

/// Standard error state driven by [AppError]: message + recovery + retry,
/// with technical [AppError.details] inside an expandable section.
class ErrorView extends StatelessWidget {
  const ErrorView({super.key, required this.error, this.onRetry});

  final AppError error;
  final VoidCallback? onRetry;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(ZdlSpacing.xl),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(Icons.error_outline, size: 48, color: theme.colorScheme.error),
            const SizedBox(height: ZdlSpacing.lg),
            Text(
              error.message,
              style: theme.textTheme.titleMedium,
              textAlign: TextAlign.center,
            ),
            if (error.recovery != null) ...[
              const SizedBox(height: ZdlSpacing.sm),
              Text(
                error.recovery!,
                style: theme.textTheme.bodyMedium?.copyWith(
                  color: theme.colorScheme.onSurfaceVariant,
                ),
                textAlign: TextAlign.center,
              ),
            ],
            if (onRetry != null) ...[
              const SizedBox(height: ZdlSpacing.lg),
              FilledButton(onPressed: onRetry, child: const Text('Retry')),
            ],
            if (error.details != null) ...[
              const SizedBox(height: ZdlSpacing.sm),
              ExpansionTile(
                title: const Text('Details'),
                children: [SelectableText(error.details!)],
              ),
            ],
          ],
        ),
      ),
    );
  }
}
