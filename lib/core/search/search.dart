import 'package:zettings/core/models/setting.dart';
import 'package:zettings/core/state/registry.dart';

/// One ranked search hit.
class SearchMatch {
  const SearchMatch({
    required this.entry,
    required this.score,
    required this.reason,
  });

  final SettingDefinition entry;
  final double score;

  /// Which signal produced the hit (title, keyword, synonym, …). Surfaced in
  /// tests and available for result-explanation UI.
  final String reason;
}

/// Local indexed settings search. Pure Dart, synchronous, deterministic:
/// same catalog + same query ⇒ same ranking. No backend calls per keystroke.
class SettingsSearchEngine {
  const SettingsSearchEngine(this.registry);

  final SettingsRegistry registry;

  List<SearchMatch> search(String query, {int limit = 8}) {
    final normalized = query.trim().toLowerCase();
    if (normalized.isEmpty) return [];
    final tokens = normalized.split(RegExp(r'\s+'));

    final matches = <SearchMatch>[];
    for (final entry in registry.entries) {
      final scored = _score(entry, normalized, tokens);
      if (scored != null) matches.add(scored);
    }
    matches.sort((a, b) {
      final byScore = b.score.compareTo(a.score);
      if (byScore != 0) return byScore;
      return a.entry.title.compareTo(b.entry.title);
    });
    return matches.take(limit).toList();
  }

  SearchMatch? _score(
    SettingDefinition entry,
    String query,
    List<String> tokens,
  ) {
    final title = entry.title.toLowerCase();
    final category = entry.category.title.toLowerCase();
    final description = entry.description.toLowerCase();

    SearchMatch hit(double base, String reason) => SearchMatch(
      entry: entry,
      score: base * entry.searchWeight,
      reason: reason,
    );

    if (title == query) return hit(100, 'title');
    if (title.startsWith(query)) return hit(80, 'title');
    if (title.contains(query)) return hit(60, 'title');
    for (final keyword in entry.keywords) {
      final k = keyword.toLowerCase();
      if (k == query) return hit(50, 'keyword');
    }
    for (final synonym in entry.synonyms) {
      if (synonym.toLowerCase().contains(query)) return hit(45, 'synonym');
    }
    for (final keyword in entry.keywords) {
      if (keyword.toLowerCase().startsWith(query)) return hit(40, 'keyword');
    }
    if (category.contains(query)) return hit(30, 'category');
    if (tokens.every(description.contains)) return hit(20, 'description');
    for (final word in title.split(' ')) {
      if (_levenshtein(word, query) <= 2) return hit(25, 'fuzzy');
    }
    return null;
  }

  int _levenshtein(String a, String b) {
    if (a == b) return 0;
    if (a.isEmpty) return b.length;
    if (b.isEmpty) return a.length;
    var prev = List.generate(b.length + 1, (i) => i);
    for (var i = 1; i <= a.length; i++) {
      var current = <int>[i];
      for (var j = 1; j <= b.length; j++) {
        current.add(
          [
            prev[j] + 1,
            current[j - 1] + 1,
            prev[j - 1] + (a[i - 1] == b[j - 1] ? 0 : 1),
          ].reduce((x, y) => x < y ? x : y),
        );
      }
      prev = current;
    }
    return prev[b.length];
  }
}

/// In-memory recent-search history (capped). Persistence arrives with the
/// preferences service; ranking stays deterministic regardless.
class SearchHistory {
  static const int maxEntries = 10;

  final List<String> _recent = [];

  List<String> get recent => List.unmodifiable(_recent);

  void record(String query) {
    final normalized = query.trim();
    if (normalized.isEmpty) return;
    _recent.remove(normalized);
    _recent.insert(0, normalized);
    if (_recent.length > maxEntries) _recent.removeLast();
  }

  void clear() => _recent.clear();
}
