import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:zettings/app/app.dart';
import 'package:zettings/app/router.dart';

import 'test_helpers.dart';

void main() {
  setUp(registerTestServices);

  testWidgets('home grid navigates to category', (WidgetTester tester) async {
    await tester.pumpWidget(const ZettingsApp());
    await tester.pumpAndSettle();

    expect(find.text('Settings'), findsOneWidget);
    expect(find.text('Network'), findsOneWidget);

    await tester.tap(find.text('Network'));
    await tester.pumpAndSettle();

    expect(
      find.text('Wi-Fi, Ethernet, VPN, proxy and firewall.'),
      findsOneWidget,
    );
  });

  testWidgets('unknown route shows typed error page', (
    WidgetTester tester,
  ) async {
    await tester.pumpWidget(const ZettingsApp());
    await tester.pumpAndSettle();

    Navigator.of(tester.element(find.text('Settings'))).pushNamed('/nope');
    await tester.pumpAndSettle();

    expect(find.textContaining('Page not found'), findsOneWidget);
    expect(find.text('Back to Home'), findsOneWidget);

    await tester.tap(find.text('Back to Home'));
    await tester.pumpAndSettle();
    expect(find.text('Settings'), findsOneWidget);
  });

  testWidgets('deep links navigate or decline cleanly', (
    WidgetTester tester,
  ) async {
    await tester.pumpWidget(const ZettingsApp());
    await tester.pumpAndSettle();

    final el = tester.element(find.text('Settings'));
    expect(SettingsRoute.openDeepLink(el, 'zettings://network'), isTrue);
    await tester.pumpAndSettle();
    expect(
      find.text('Wi-Fi, Ethernet, VPN, proxy and firewall.'),
      findsOneWidget,
    );

    expect(SettingsRoute.openDeepLink(el, 'zettings://nope'), isFalse);
    expect(SettingsRoute.openDeepLink(el, 'https://example.com'), isFalse);
  });
}
