import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:zettings/core/errors/app_error.dart';
import 'package:zettings/shared/widgets/section_header.dart';
import 'package:zettings/shared/widgets/setting_row.dart';
import 'package:zettings/shared/widgets/state_views.dart';

Widget _wrap(Widget child) => MaterialApp(home: Scaffold(body: child));

void main() {
  group('ZettingsRow', () {
    testWidgets('renders title, subtitle and trailing', (
      WidgetTester tester,
    ) async {
      await tester.pumpWidget(
        _wrap(
          ZettingsRow(
            icon: Icons.wifi,
            title: 'Wi-Fi',
            subtitle: 'Connected',
            trailing: const Text('On'),
            onTap: () {},
          ),
        ),
      );
      expect(find.text('Wi-Fi'), findsOneWidget);
      expect(find.text('Connected'), findsOneWidget);
      expect(find.text('On'), findsOneWidget);
    });

    testWidgets('tap fires when enabled, ignored when disabled', (
      WidgetTester tester,
    ) async {
      var taps = 0;
      await tester.pumpWidget(
        _wrap(
          ZettingsRow(icon: Icons.wifi, title: 'Wi-Fi', onTap: () => taps++),
        ),
      );
      await tester.tap(find.text('Wi-Fi'));
      expect(taps, 1);

      await tester.pumpWidget(
        _wrap(
          ZettingsRow(
            icon: Icons.wifi,
            title: 'Wi-Fi',
            enabled: false,
            onTap: () => taps++,
          ),
        ),
      );
      await tester.tap(find.text('Wi-Fi'));
      expect(taps, 1);
    });
  });

  group('SectionHeader', () {
    testWidgets('renders title', (WidgetTester tester) async {
      await tester.pumpWidget(_wrap(const SectionHeader(title: 'Network')));
      expect(find.text('Network'), findsOneWidget);
    });
  });

  group('State views', () {
    testWidgets('LoadingView shows indicator and label', (
      WidgetTester tester,
    ) async {
      await tester.pumpWidget(_wrap(const LoadingView(label: 'Waiting…')));
      expect(find.byType(CircularProgressIndicator), findsOneWidget);
      expect(find.text('Waiting…'), findsOneWidget);
    });

    testWidgets('EmptyView shows title, hint and action', (
      WidgetTester tester,
    ) async {
      var acted = false;
      await tester.pumpWidget(
        _wrap(
          EmptyView(
            icon: Icons.devices,
            title: 'No devices',
            hint: 'Nothing here yet.',
            actionLabel: 'Scan',
            onAction: () => acted = true,
          ),
        ),
      );
      expect(find.text('No devices'), findsOneWidget);
      expect(find.text('Nothing here yet.'), findsOneWidget);
      await tester.tap(find.text('Scan'));
      expect(acted, isTrue);
    });

    testWidgets('ErrorView shows message, recovery and retry', (
      WidgetTester tester,
    ) async {
      var retried = false;
      await tester.pumpWidget(
        _wrap(
          ErrorView(
            error: const AppError(
              AppErrorType.unavailable,
              'Service is down.',
              recovery: 'Try again later.',
              details: 'dbus: timeout',
            ),
            onRetry: () => retried = true,
          ),
        ),
      );
      expect(find.text('Service is down.'), findsOneWidget);
      expect(find.text('Try again later.'), findsOneWidget);
      await tester.tap(find.text('Retry'));
      expect(retried, isTrue);
      // Technical details stay behind the expander.
      expect(find.text('dbus: timeout'), findsNothing);
      await tester.tap(find.text('Details'));
      await tester.pumpAndSettle();
      expect(find.text('dbus: timeout'), findsOneWidget);
    });
  });
}
