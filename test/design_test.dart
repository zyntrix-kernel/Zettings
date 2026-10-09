import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:zettings/core/theme/motion.dart';
import 'package:zettings/shared/widgets/settings_page.dart';

void main() {
  group('ZdlMotion', () {
    test('durations are ordered and short', () {
      expect(ZdlMotion.instant < ZdlMotion.fast, isTrue);
      expect(ZdlMotion.fast < ZdlMotion.medium, isTrue);
      expect(
        ZdlMotion.medium,
        lessThanOrEqualTo(const Duration(milliseconds: 300)),
      );
    });

    testWidgets('resolves to zero under reduced motion', (
      WidgetTester tester,
    ) async {
      Duration? normal;
      Duration? reduced;
      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) {
              normal = ZdlMotion.resolve(context, ZdlMotion.medium);
              return const SizedBox();
            },
          ),
        ),
      );
      await tester.pumpWidget(
        MediaQuery(
          data: const MediaQueryData(disableAnimations: true),
          child: Builder(
            builder: (context) {
              reduced = ZdlMotion.resolve(context, ZdlMotion.medium);
              return const SizedBox();
            },
          ),
        ),
      );
      expect(normal, ZdlMotion.medium);
      expect(reduced, Duration.zero);
    });
  });

  group('ZettingsPage', () {
    testWidgets('renders title and children', (WidgetTester tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: ZettingsPage(
            title: 'Network',
            children: [Text('Row one'), Text('Row two')],
          ),
        ),
      );
      expect(find.text('Network'), findsOneWidget);
      expect(find.text('Row one'), findsOneWidget);
      expect(find.text('Row two'), findsOneWidget);
    });
  });
}
