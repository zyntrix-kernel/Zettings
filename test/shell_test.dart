import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:zettings/app/app.dart';
import 'package:zettings/core/services/prefs_service.dart';
import 'package:zettings/core/services/service_locator.dart';
import 'package:zettings/core/theme/appearance.dart';

import 'test_helpers.dart';

void _setSize(WidgetTester tester, Size size) {
  tester.view.devicePixelRatio = 1.0;
  tester.view.physicalSize = size;
  addTearDown(tester.view.resetPhysicalSize);
}

void main() {
  setUp(registerTestServices);

  testWidgets('wide layout shows expanded rail and navigates', (
    WidgetTester tester,
  ) async {
    _setSize(tester, const Size(1200, 800));
    await tester.pumpWidget(const ZettingsApp());
    await tester.pumpAndSettle();

    final rail = tester.widget<NavigationRail>(find.byType(NavigationRail));
    expect(rail.extended, isTrue);

    await tester.tap(
      find.descendant(
        of: find.byType(NavigationRail),
        matching: find.byIcon(Icons.wifi),
      ),
    );
    await tester.pumpAndSettle();
    expect(
      find.text('Wi-Fi, Ethernet, VPN, proxy and firewall.'),
      findsOneWidget,
    );
  });

  testWidgets('narrow layout uses drawer navigation', (
    WidgetTester tester,
  ) async {
    _setSize(tester, const Size(500, 800));
    await tester.pumpWidget(const ZettingsApp());
    await tester.pumpAndSettle();

    expect(find.byType(NavigationRail), findsNothing);

    await tester.tap(find.byTooltip('Open navigation'));
    await tester.pumpAndSettle();
    await tester.tap(
      find.descendant(
        of: find.byType(NavigationDrawer),
        matching: find.text('Network'),
      ),
    );
    await tester.pumpAndSettle();
    expect(
      find.text('Wi-Fi, Ethernet, VPN, proxy and firewall.'),
      findsOneWidget,
    );
  });

  testWidgets('search finds a setting and navigates', (
    WidgetTester tester,
  ) async {
    _setSize(tester, const Size(1200, 800));
    await tester.pumpWidget(const ZettingsApp());
    await tester.pumpAndSettle();

    await tester.tap(find.byType(SearchBar).first);
    await tester.pumpAndSettle();
    await tester.enterText(find.byType(EditableText).last, 'wifi');
    await tester.pumpAndSettle();

    // Note: the suggestion tile holds 'Network' twice (title + category),
    // so the ancestor finder matches it twice. Tapping .first is exact.
    final suggestion = find.widgetWithText(ListTile, 'Network');
    expect(suggestion, findsWidgets);
    await tester.tap(suggestion.first);
    await tester.pumpAndSettle();

    expect(
      find.text('Wi-Fi, Ethernet, VPN, proxy and firewall.'),
      findsOneWidget,
    );
  });

  testWidgets('appearance menu switches mode and persists', (
    WidgetTester tester,
  ) async {
    _setSize(tester, const Size(1200, 800));
    await tester.pumpWidget(const ZettingsApp());
    await tester.pumpAndSettle();

    await tester.tap(find.byTooltip('Appearance'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('OLED black'));
    await tester.pumpAndSettle();

    final prefs = ServiceLocator.get<PrefsService>();
    expect(prefs.appearance(), AppearanceMode.oled);
  });
}
