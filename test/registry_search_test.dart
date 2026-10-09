import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:zettings/core/models/setting.dart';
import 'package:zettings/core/search/search.dart';
import 'package:zettings/core/state/registry.dart';

void main() {
  late SettingsRegistry registry;
  late SettingsSearchEngine engine;

  setUp(() {
    registry = SettingsRegistry();
    engine = SettingsSearchEngine(registry);
  });

  group('SettingsRegistry', () {
    test('default catalog holds the ten category pages', () {
      expect(registry.entries, hasLength(10));
      expect(
        registry.entries.map((e) => e.category).toSet(),
        hasLength(SettingCategory.values.length),
      );
    });

    test('every entry resolves by id and by route', () {
      for (final entry in registry.entries) {
        expect(registry.byId(entry.id), same(entry));
        expect(registry.byRoute(entry.route), same(entry));
      }
      expect(registry.byId('nope'), isNull);
      expect(registry.byRoute('/nope'), isNull);
    });

    test('forCategory filters correctly', () {
      final network = registry.forCategory(SettingCategory.network);
      expect(network, hasLength(1));
      expect(network.single.id, 'network');
    });

    test('register extends the catalog', () {
      registry.register(
        const SettingDefinition(
          id: 'network-wifi',
          title: 'Wi-Fi',
          description: 'Wireless networks.',
          category: SettingCategory.network,
          route: '/network/wifi',
          icon: Icons.wifi,
        ),
      );
      expect(registry.byRoute('/network/wifi')?.title, 'Wi-Fi');
      expect(registry.forCategory(SettingCategory.network), hasLength(2));
    });
  });

  group('SettingsSearchEngine', () {
    test('empty query returns nothing', () {
      expect(engine.search(''), isEmpty);
      expect(engine.search('   '), isEmpty);
    });

    test('exact title match ranks first', () {
      final results = engine.search('network');
      expect(results, isNotEmpty);
      expect(results.first.entry.id, 'network');
      expect(results.first.reason, 'title');
    });

    test('wifi finds the network page via keyword', () {
      final results = engine.search('wifi');
      expect(results, isNotEmpty);
      expect(results.first.entry.id, 'network');
    });

    test('synonyms resolve (wireless network → network)', () {
      final results = engine.search('wireless network');
      expect(results.map((r) => r.entry.id), contains('network'));
    });

    test('screen finds display-related system page', () {
      final results = engine.search('screen');
      expect(results, isNotEmpty);
      expect(results.map((r) => r.entry.id), contains('system'));
    });

    test('typo tolerance finds bluetooth', () {
      final results = engine.search('blutooth');
      expect(results.map((r) => r.entry.id), contains('bluetooth-devices'));
    });

    test('ranking is deterministic', () {
      final first = engine.search('update').map((r) => r.entry.id).toList();
      final second = engine.search('update').map((r) => r.entry.id).toList();
      expect(first, second);
      expect(first.first, 'updates');
    });

    test('limit is honored', () {
      expect(engine.search('a', limit: 3), hasLength(3));
    });
  });

  group('SearchHistory', () {
    test('records most-recent-first without duplicates', () {
      final history = SearchHistory();
      history.record('wifi');
      history.record('display');
      history.record('wifi');
      expect(history.recent, ['wifi', 'display']);
    });

    test('caps at maxEntries and clears', () {
      final history = SearchHistory();
      for (var i = 0; i < 15; i++) {
        history.record('q$i');
      }
      expect(history.recent, hasLength(SearchHistory.maxEntries));
      expect(history.recent.first, 'q14');
      history.clear();
      expect(history.recent, isEmpty);
    });

    test('ignores blank queries', () {
      final history = SearchHistory();
      history.record('   ');
      expect(history.recent, isEmpty);
    });
  });
}
