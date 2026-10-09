import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:zettings/core/state/app_state.dart';
import 'package:zettings/core/theme/appearance.dart';
import 'package:zettings/features/accessibility/accessibility_page.dart';
import 'package:zettings/features/accounts/accounts_page.dart';
import 'package:zettings/features/applications/applications_page.dart';
import 'package:zettings/features/bluetooth/bluetooth_page.dart';
import 'package:zettings/features/developer/developer_page.dart';
import 'package:zettings/features/network/network_page.dart';
import 'package:zettings/features/personalization/personalization_page.dart';
import 'package:zettings/features/privacy/privacy_page.dart';
import 'package:zettings/features/updates/updates_page.dart';
import 'package:zettings/shared/widgets/hero_status_card.dart';
import 'package:zettings/shared/widgets/setting_group_card.dart';

import 'test_helpers.dart';

void _setSize(WidgetTester tester, [Size size = const Size(1200, 1200)]) {
  tester.view.devicePixelRatio = 1.0;
  tester.view.physicalSize = size;
  addTearDown(tester.view.resetPhysicalSize);
}

Widget _wrap(Widget child, [ZettingsState? state]) {
  final s = state ?? ZettingsState();
  return AppScope(
    state: s,
    child: MaterialApp(home: Scaffold(body: child)),
  );
}

void main() {
  setUp(registerTestServices);

  group('Material 3 Feature Pages', () {
    testWidgets('PersonalizationPage renders and changes theme mode', (
      WidgetTester tester,
    ) async {
      _setSize(tester);
      final state = ZettingsState();
      await tester.pumpWidget(_wrap(const PersonalizationPage(), state));
      await tester.pumpAndSettle();

      expect(find.text('Personalization'), findsOneWidget);
      expect(find.text('Look & Feel'), findsOneWidget);
      expect(find.text('Theme mode'), findsOneWidget);

      await tester.tap(find.text('OLED black'));
      await tester.pumpAndSettle();

      expect(state.appearance, AppearanceMode.oled);
    });

    testWidgets('NetworkPage toggles Wi-Fi and shows interfaces', (
      WidgetTester tester,
    ) async {
      _setSize(tester);
      await tester.pumpWidget(_wrap(const NetworkPage()));
      await tester.pumpAndSettle();

      expect(find.text('Network'), findsOneWidget);
      expect(find.text('Connected to Zyntrix-5G'), findsOneWidget);
      expect(find.text('Ethernet (eth0)'), findsOneWidget);
      expect(find.text('WireGuard VPN'), findsOneWidget);
    });

    testWidgets('BluetoothDevicesPage renders paired peripherals', (
      WidgetTester tester,
    ) async {
      _setSize(tester);
      await tester.pumpWidget(_wrap(const BluetoothDevicesPage()));
      await tester.pumpAndSettle();

      expect(find.text('Bluetooth & devices'), findsOneWidget);
      expect(find.text('Sony WH-1000XM5'), findsOneWidget);
      expect(find.text('Logitech MX Master 3S'), findsOneWidget);
      expect(find.text('Keychron Q1 Pro'), findsOneWidget);
    });

    testWidgets(
      'AccessibilityPage toggles high contrast and assistive options',
      (WidgetTester tester) async {
        _setSize(tester);
        final state = ZettingsState();
        await tester.pumpWidget(_wrap(const AccessibilityPage(), state));
        await tester.pumpAndSettle();

        expect(find.text('Accessibility'), findsOneWidget);
        expect(find.text('High contrast mode'), findsOneWidget);
        expect(find.text('Screen reader (Orca)'), findsOneWidget);
        expect(find.text('Reduce motion'), findsOneWidget);

        await tester.tap(find.text('High contrast mode'));
        await tester.pumpAndSettle();
        expect(state.highContrast, isTrue);

        await tester.tap(find.text('Reduce motion'));
        await tester.pumpAndSettle();
        expect(state.reducedMotion, isTrue);
      },
    );

    testWidgets('AccountsPage renders user details and password options', (
      WidgetTester tester,
    ) async {
      _setSize(tester);
      await tester.pumpWidget(_wrap(const AccountsPage()));
      await tester.pumpAndSettle();

      expect(find.text('Accounts'), findsOneWidget);
      expect(find.text('Zyntrix Admin'), findsOneWidget);
      expect(find.text('Account password'), findsOneWidget);

      await tester.tap(find.text('Change password'));
      await tester.pumpAndSettle();
      expect(find.text('Change Password'), findsOneWidget);
      await tester.tap(find.text('Cancel'));
      await tester.pumpAndSettle();
    });

    testWidgets(
      'PrivacySecurityPage renders application portals and firewall',
      (WidgetTester tester) async {
        _setSize(tester);
        await tester.pumpWidget(_wrap(const PrivacySecurityPage()));
        await tester.pumpAndSettle();

        expect(find.text('Privacy & security'), findsOneWidget);
        expect(find.text('Location services'), findsOneWidget);
        expect(find.text('Camera access'), findsOneWidget);
        expect(find.text('Firewall protection'), findsOneWidget);
      },
    );

    testWidgets(
      'ApplicationsPage renders default handlers and package runtimes',
      (WidgetTester tester) async {
        _setSize(tester);
        await tester.pumpWidget(_wrap(const ApplicationsPage()));
        await tester.pumpAndSettle();

        expect(find.text('Applications'), findsOneWidget);
        expect(find.text('Web browser'), findsOneWidget);
        expect(find.text('Email client'), findsOneWidget);
        expect(find.text('Flatpak (Flathub)'), findsOneWidget);
      },
    );

    testWidgets('UpdatesPage checks for system updates', (
      WidgetTester tester,
    ) async {
      _setSize(tester);
      await tester.pumpWidget(_wrap(const UpdatesPage()));
      await tester.pumpAndSettle();

      expect(find.text('Updates'), findsOneWidget);
      expect(find.text('System is Up to Date'), findsOneWidget);
      expect(find.text('Check for updates'), findsOneWidget);

      await tester.tap(find.text('Check for updates'));
      await tester.pump();
      expect(find.text('Checking…'), findsOneWidget);
      await tester.pump(const Duration(milliseconds: 1500));
      await tester.pumpAndSettle();
      expect(find.text('Check for updates'), findsOneWidget);
    });

    testWidgets('DeveloperPage displays IPC and telemetry diagnostics', (
      WidgetTester tester,
    ) async {
      _setSize(tester);
      await tester.pumpWidget(_wrap(const DeveloperPage()));
      await tester.pumpAndSettle();

      expect(find.text('Developer'), findsOneWidget);
      expect(find.text('Zettings Rust Daemon'), findsOneWidget);
      expect(find.text('Verbose debug logging'), findsOneWidget);
      expect(find.text('Ping daemon'), findsOneWidget);
    });

    testWidgets('SettingGroupCard and HeroStatusCard render correctly', (
      WidgetTester tester,
    ) async {
      _setSize(tester);
      await tester.pumpWidget(
        _wrap(
          Column(
            children: [
              const HeroStatusCard(
                title: 'Hero Title',
                subtitle: 'Hero Subtitle',
                icon: Icons.star,
              ),
              SettingGroupCard(
                title: 'Group Title',
                description: 'Group Description',
                children: const [Text('Item 1'), Text('Item 2')],
              ),
            ],
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.text('Hero Title'), findsOneWidget);
      expect(find.text('Hero Subtitle'), findsOneWidget);
      expect(find.text('Group Title'), findsOneWidget);
      expect(find.text('Group Description'), findsOneWidget);
      expect(find.text('Item 1'), findsOneWidget);
      expect(find.text('Item 2'), findsOneWidget);
    });
  });
}
