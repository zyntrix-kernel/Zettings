import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:zettings/app/router.dart';
import 'package:zettings/core/models/setting.dart';
import 'package:zettings/core/search/search.dart';
import 'package:zettings/core/services/prefs_service.dart';
import 'package:zettings/core/services/service_locator.dart';
import 'package:zettings/core/state/app_state.dart';
import 'package:zettings/core/theme/appearance.dart';
import 'package:zettings/core/theme/breakpoints.dart';
import 'package:zettings/core/theme/spacing.dart';
import 'package:zettings/core/theme/zettings_theme.dart';
import 'package:zettings/core/utils/log.dart';

/// Focus-the-search-field intent (Ctrl/Cmd+F).
class _FocusSearchIntent extends Intent {
  const _FocusSearchIntent();
}

/// Navigate back to home intent (Alt+Left or Escape).
class _NavigateHomeIntent extends Intent {
  const _NavigateHomeIntent();
}

/// Application shell: responsive desktop navigation + content area.
///
/// Wide (≥1000): expanded rail with branded header.
/// Medium (600–1000): collapsed rail.
/// Narrow (<600): modal drawer with hamburger button.
class ZettingsShell extends StatefulWidget {
  const ZettingsShell({super.key, required this.current, required this.child});

  final SettingsRoute current;
  final Widget child;

  @override
  State<ZettingsShell> createState() => _ZettingsShellState();
}

class _ZettingsShellState extends State<ZettingsShell> {
  final FocusNode _searchFocus = FocusNode();

  @override
  void dispose() {
    _searchFocus.dispose();
    super.dispose();
  }

  void _goTo(String route) {
    if (route == widget.current.path) return;
    Navigator.of(context).pushReplacementNamed(route);
  }

  int? get _selectedIndex {
    if (widget.current.path == '/') return null;
    return SettingCategory.values.indexOf(widget.current.category);
  }

  List<NavigationRailDestination> _railDestinations(bool extended) {
    return SettingCategory.values
        .map(
          (c) => NavigationRailDestination(
            icon: Icon(c.icon),
            selectedIcon: Icon(c.icon),
            label: extended
                ? Text(
                    c.title,
                    style: const TextStyle(fontWeight: FontWeight.w500),
                  )
                : const SizedBox.shrink(),
          ),
        )
        .toList();
  }

  void _onRailSelected(int index) {
    _goTo(SettingCategory.values[index].routePath);
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final isOled = theme.scaffoldBackgroundColor == Colors.black;

    return Shortcuts(
      shortcuts: const {
        SingleActivator(LogicalKeyboardKey.keyF, control: true):
            _FocusSearchIntent(),
        SingleActivator(LogicalKeyboardKey.arrowLeft, alt: true):
            _NavigateHomeIntent(),
      },
      child: Actions(
        actions: {
          _FocusSearchIntent: CallbackAction<_FocusSearchIntent>(
            onInvoke: (_) {
              _searchFocus.requestFocus();
              return null;
            },
          ),
          _NavigateHomeIntent: CallbackAction<_NavigateHomeIntent>(
            onInvoke: (_) {
              if (widget.current.path != '/') {
                _goTo('/');
              }
              return null;
            },
          ),
        },
        child: LayoutBuilder(
          builder: (context, constraints) {
            final width = constraints.maxWidth;
            if (width < ZdlBreakpoints.compactMax) {
              return _narrowScaffold(context);
            }
            final expanded = width >= ZdlBreakpoints.largeMin;
            return Scaffold(
              body: Row(
                children: [
                  Container(
                    color: isOled
                        ? Colors.black
                        : theme.colorScheme.surfaceContainerLowest,
                    child: SingleChildScrollView(
                      child: ConstrainedBox(
                        constraints: BoxConstraints(
                          minHeight: constraints.maxHeight,
                          minWidth: expanded ? 240 : 72,
                        ),
                        child: IntrinsicHeight(
                          child: NavigationRail(
                            extended: expanded,
                            selectedIndex: _selectedIndex,
                            labelType: expanded
                                ? NavigationRailLabelType.none
                                : NavigationRailLabelType.selected,
                            destinations: _railDestinations(expanded),
                            onDestinationSelected: _onRailSelected,
                            leading: Padding(
                              padding: EdgeInsets.symmetric(
                                vertical: 16,
                                horizontal: expanded ? 16 : 8,
                              ),
                              child: InkWell(
                                borderRadius: BorderRadius.circular(12),
                                onTap: () => _goTo('/'),
                                child: Padding(
                                  padding: const EdgeInsets.symmetric(
                                    vertical: 6,
                                    horizontal: 8,
                                  ),
                                  child: Row(
                                    mainAxisSize: MainAxisSize.min,
                                    children: [
                                      Container(
                                        width: 32,
                                        height: 32,
                                        decoration: BoxDecoration(
                                          color: theme
                                              .colorScheme
                                              .primaryContainer,
                                          borderRadius: BorderRadius.circular(
                                            8,
                                          ),
                                        ),
                                        alignment: Alignment.center,
                                        child: Icon(
                                          Icons.settings,
                                          size: 20,
                                          color: theme
                                              .colorScheme
                                              .onPrimaryContainer,
                                        ),
                                      ),
                                      if (expanded) ...[
                                        const SizedBox(width: 12),
                                        Column(
                                          crossAxisAlignment:
                                              CrossAxisAlignment.start,
                                          children: [
                                            Text(
                                              'Zettings',
                                              style: theme.textTheme.titleMedium
                                                  ?.copyWith(
                                                    fontWeight: FontWeight.w700,
                                                  ),
                                            ),
                                            Text(
                                              'Zyntrix OS',
                                              style: theme.textTheme.labelSmall
                                                  ?.copyWith(
                                                    color: theme
                                                        .colorScheme
                                                        .primary,
                                                    fontWeight: FontWeight.w600,
                                                  ),
                                            ),
                                          ],
                                        ),
                                      ],
                                    ],
                                  ),
                                ),
                              ),
                            ),
                            trailing: expanded
                                ? Padding(
                                    padding: const EdgeInsets.fromLTRB(
                                      16,
                                      24,
                                      16,
                                      16,
                                    ),
                                    child: InkWell(
                                      borderRadius: BorderRadius.circular(14),
                                      onTap: () => _goTo('/accounts'),
                                      child: Container(
                                        padding: const EdgeInsets.symmetric(
                                          horizontal: 12,
                                          vertical: 10,
                                        ),
                                        decoration: BoxDecoration(
                                          color: isOled
                                              ? const Color(0xFF141518)
                                              : theme
                                                    .colorScheme
                                                    .surfaceContainerLow,
                                          borderRadius: BorderRadius.circular(
                                            14,
                                          ),
                                          border: Border.all(
                                            color: isOled
                                                ? const Color(0xFF282A30)
                                                : theme
                                                      .colorScheme
                                                      .outlineVariant
                                                      .withValues(alpha: 0.35),
                                          ),
                                        ),
                                        child: Row(
                                          mainAxisSize: MainAxisSize.min,
                                          children: [
                                            Container(
                                              width: 32,
                                              height: 32,
                                              decoration: BoxDecoration(
                                                color: theme
                                                    .colorScheme
                                                    .primaryContainer,
                                                shape: BoxShape.circle,
                                              ),
                                              alignment: Alignment.center,
                                              child: Icon(
                                                Icons.person,
                                                size: 18,
                                                color: theme
                                                    .colorScheme
                                                    .onPrimaryContainer,
                                              ),
                                            ),
                                            const SizedBox(width: 10),
                                            Column(
                                              crossAxisAlignment:
                                                  CrossAxisAlignment.start,
                                              mainAxisSize: MainAxisSize.min,
                                              children: [
                                                Text(
                                                  'Zyntrix User',
                                                  style: theme
                                                      .textTheme
                                                      .labelMedium
                                                      ?.copyWith(
                                                        fontWeight:
                                                            FontWeight.w600,
                                                      ),
                                                ),
                                                Text(
                                                  'Administrator',
                                                  style: theme
                                                      .textTheme
                                                      .labelSmall
                                                      ?.copyWith(
                                                        color: theme
                                                            .colorScheme
                                                            .onSurfaceVariant,
                                                      ),
                                                ),
                                              ],
                                            ),
                                          ],
                                        ),
                                      ),
                                    ),
                                  )
                                : null,
                          ),
                        ),
                      ),
                    ),
                  ),
                  VerticalDivider(
                    width: 1,
                    thickness: 1,
                    color: theme.colorScheme.outlineVariant.withValues(
                      alpha: 0.35,
                    ),
                  ),
                  Expanded(child: _content(context, showMenuButton: false)),
                ],
              ),
            );
          },
        ),
      ),
    );
  }

  Widget _narrowScaffold(BuildContext context) {
    final theme = Theme.of(context);
    return Scaffold(
      drawer: NavigationDrawer(
        selectedIndex: _selectedIndex,
        onDestinationSelected: (index) {
          Navigator.of(context).pop();
          _onRailSelected(index);
        },
        children: [
          DrawerHeader(
            decoration: BoxDecoration(
              color: theme.colorScheme.surfaceContainerHigh,
            ),
            child: Row(
              children: [
                Container(
                  width: 44,
                  height: 44,
                  decoration: BoxDecoration(
                    color: theme.colorScheme.primaryContainer,
                    borderRadius: BorderRadius.circular(10),
                  ),
                  alignment: Alignment.center,
                  child: Icon(
                    Icons.settings,
                    size: 26,
                    color: theme.colorScheme.onPrimaryContainer,
                  ),
                ),
                const SizedBox(width: 14),
                Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    Text(
                      'Zettings',
                      style: theme.textTheme.titleLarge?.copyWith(
                        fontWeight: FontWeight.w700,
                      ),
                    ),
                    Text(
                      'Zyntrix OS',
                      style: theme.textTheme.labelMedium?.copyWith(
                        color: theme.colorScheme.primary,
                      ),
                    ),
                  ],
                ),
              ],
            ),
          ),
          for (final c in SettingCategory.values)
            NavigationDrawerDestination(
              icon: Icon(c.icon),
              label: Flexible(
                child: Text(
                  c.title,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                ),
              ),
            ),
        ],
      ),
      body: Builder(
        builder: (context) => _content(context, showMenuButton: true),
      ),
    );
  }

  Widget _content(BuildContext context, {required bool showMenuButton}) {
    final theme = Theme.of(context);
    return SafeArea(
      child: Column(
        children: [
          Container(
            padding: const EdgeInsets.fromLTRB(
              ZdlSpacing.lg,
              ZdlSpacing.md,
              ZdlSpacing.lg,
              ZdlSpacing.sm,
            ),
            child: Row(
              children: [
                if (showMenuButton)
                  Padding(
                    padding: const EdgeInsets.only(right: 8),
                    child: Builder(
                      builder: (context) => IconButton(
                        tooltip: 'Open navigation',
                        icon: const Icon(Icons.menu),
                        onPressed: () => Scaffold.of(context).openDrawer(),
                      ),
                    ),
                  ),
                if (widget.current.path != '/')
                  Padding(
                    padding: const EdgeInsets.only(right: 8),
                    child: IconButton(
                      tooltip: 'Back to Home',
                      icon: const Icon(Icons.arrow_back),
                      onPressed: () => _goTo('/'),
                    ),
                  ),
                Expanded(child: _SearchField(focusNode: _searchFocus)),
                const SizedBox(width: ZdlSpacing.sm),
                const AppearanceMenu(),
              ],
            ),
          ),
          Divider(
            height: 1,
            thickness: 1,
            color: theme.colorScheme.outlineVariant.withValues(alpha: 0.25),
          ),
          Expanded(child: widget.child),
        ],
      ),
    );
  }
}

/// Settings search field wired to the real [SettingsSearchEngine].
/// Empty query shows recent searches; selecting a result navigates.
class _SearchField extends StatelessWidget {
  const _SearchField({required this.focusNode});

  final FocusNode focusNode;

  void _goTo(BuildContext context, SettingDefinition entry, String query) {
    final history = ServiceLocator.get<SearchHistory>();
    final prefs = ServiceLocator.get<PrefsService>();
    history.record(query);
    unawaited(prefs.setRecents(history.recent));
    Navigator.of(context).pushReplacementNamed(entry.route);
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final engine = ServiceLocator.get<SettingsSearchEngine>();
    final history = ServiceLocator.get<SearchHistory>();

    return SearchAnchor(
      builder: (context, controller) => SearchBar(
        focusNode: focusNode,
        controller: controller,
        leading: const Icon(Icons.search),
        hintText: 'Search settings',
        trailing: [
          if (controller.text.isNotEmpty)
            IconButton(
              icon: const Icon(Icons.clear, size: 18),
              tooltip: 'Clear search',
              onPressed: () => controller.clear(),
            ),
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
            margin: const EdgeInsets.only(right: 4),
            decoration: BoxDecoration(
              color: theme.colorScheme.surfaceContainerHighest,
              borderRadius: BorderRadius.circular(6),
            ),
            child: Text(
              'Ctrl+F',
              style: theme.textTheme.labelSmall?.copyWith(
                color: theme.colorScheme.onSurfaceVariant,
                fontWeight: FontWeight.w600,
              ),
            ),
          ),
        ],
        onTap: () => controller.openView(),
        onChanged: (_) {
          if (!controller.isOpen) controller.openView();
        },
      ),
      suggestionsBuilder: (context, controller) {
        final query = controller.text;
        if (query.trim().isEmpty) {
          return [
            Padding(
              padding: const EdgeInsets.fromLTRB(16, 12, 16, 4),
              child: Text(
                'SUGGESTED TOPICS',
                style: theme.textTheme.labelSmall?.copyWith(
                  color: theme.colorScheme.primary,
                  fontWeight: FontWeight.w700,
                  letterSpacing: 0.5,
                ),
              ),
            ),
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 6),
              child: Wrap(
                spacing: 8,
                runSpacing: 8,
                children: [
                  ActionChip(
                    avatar: const Icon(Icons.wifi, size: 16),
                    label: const Text('Wi-Fi'),
                    onPressed: () {
                      controller.text = 'Wi-Fi';
                      controller.openView();
                    },
                  ),
                  ActionChip(
                    avatar: const Icon(Icons.palette, size: 16),
                    label: const Text('Theme'),
                    onPressed: () {
                      controller.text = 'Theme';
                      controller.openView();
                    },
                  ),
                  ActionChip(
                    avatar: const Icon(Icons.computer, size: 16),
                    label: const Text('System specs'),
                    onPressed: () {
                      controller.text = 'System';
                      controller.openView();
                    },
                  ),
                  ActionChip(
                    avatar: const Icon(Icons.bluetooth, size: 16),
                    label: const Text('Bluetooth'),
                    onPressed: () {
                      controller.text = 'Bluetooth';
                      controller.openView();
                    },
                  ),
                  ActionChip(
                    avatar: const Icon(Icons.security, size: 16),
                    label: const Text('Privacy'),
                    onPressed: () {
                      controller.text = 'Privacy';
                      controller.openView();
                    },
                  ),
                ],
              ),
            ),
            if (history.recent.isNotEmpty) ...[
              const Divider(height: 16),
              Padding(
                padding: const EdgeInsets.fromLTRB(16, 8, 16, 4),
                child: Text(
                  'RECENT SEARCHES',
                  style: theme.textTheme.labelSmall?.copyWith(
                    color: theme.colorScheme.primary,
                    fontWeight: FontWeight.w700,
                    letterSpacing: 0.5,
                  ),
                ),
              ),
              ...history.recent.map(
                (q) => ListTile(
                  leading: const Icon(Icons.history),
                  title: Text(q),
                  onTap: () {
                    controller.closeView(q);
                    final hits = engine.search(q);
                    if (hits.isNotEmpty && context.mounted) {
                      _goTo(context, hits.first.entry, q);
                    }
                  },
                ),
              ),
            ],
          ];
        }
        final hits = engine.search(query);
        if (hits.isEmpty) {
          return [
            const ListTile(
              leading: Icon(Icons.search_off),
              title: Text('No matching settings found'),
              subtitle: Text('Check your spelling or try broader terms'),
            ),
          ];
        }
        return hits.map(
          (hit) => ListTile(
            leading: Container(
              width: 36,
              height: 36,
              decoration: BoxDecoration(
                color: theme.colorScheme.primaryContainer,
                borderRadius: BorderRadius.circular(8),
              ),
              alignment: Alignment.center,
              child: Icon(
                hit.entry.icon,
                size: 20,
                color: theme.colorScheme.onPrimaryContainer,
              ),
            ),
            title: Text(
              hit.entry.title,
              style: const TextStyle(fontWeight: FontWeight.w600),
            ),
            subtitle: Text(
              '${hit.entry.category.title} · ${hit.entry.description}',
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
            ),
            trailing: Chip(
              visualDensity: VisualDensity.compact,
              label: Text(
                hit.entry.category.title,
                style: const TextStyle(fontSize: 11),
              ),
            ),
            onTap: () {
              controller.closeView(query);
              _goTo(context, hit.entry, query);
            },
          ),
        );
      },
    );
  }
}

/// Appearance menu: theme-mode + dynamic accent switching backed by
/// [ZettingsState] and persisted through [PrefsService].
class AppearanceMenu extends StatelessWidget {
  const AppearanceMenu({super.key});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final state = AppScope.of(context);
    final prefs = ServiceLocator.get<PrefsService>();

    return MenuAnchor(
      builder: (context, controller, child) => IconButton.filledTonal(
        tooltip: 'Appearance',
        icon: const Icon(Icons.contrast),
        onPressed: () =>
            controller.isOpen ? controller.close() : controller.open(),
      ),
      menuChildren: [
        Padding(
          padding: const EdgeInsets.fromLTRB(16, 8, 16, 4),
          child: Text(
            'THEME MODE',
            style: theme.textTheme.labelSmall?.copyWith(
              color: theme.colorScheme.primary,
              fontWeight: FontWeight.w700,
            ),
          ),
        ),
        for (final mode in AppearanceMode.values)
          MenuItemButton(
            leadingIcon: state.appearance == mode
                ? Icon(Icons.check, color: theme.colorScheme.primary, size: 18)
                : const SizedBox(width: 18),
            onPressed: () {
              state.setAppearance(mode);
              unawaited(prefs.setAppearance(mode));
              ZettingsLog.info('Appearance → ${mode.title}');
            },
            child: Text(mode.title),
          ),
        const Divider(),
        Padding(
          padding: const EdgeInsets.fromLTRB(16, 8, 16, 4),
          child: Text(
            'ACCENT COLOR',
            style: theme.textTheme.labelSmall?.copyWith(
              color: theme.colorScheme.primary,
              fontWeight: FontWeight.w700,
            ),
          ),
        ),
        MenuItemButton(
          leadingIcon: state.accent == null
              ? Icon(Icons.check, color: theme.colorScheme.primary, size: 18)
              : const SizedBox(width: 18),
          onPressed: () {
            state.setAccent(null);
            unawaited(prefs.setAccent(null));
          },
          child: const Text('Default (Zyntrix Blue)'),
        ),
        for (final opt in ZettingsTheme.accentPalette)
          MenuItemButton(
            leadingIcon: Icon(Icons.circle, color: opt.color, size: 18),
            onPressed: () {
              state.setAccent(opt.color);
              unawaited(prefs.setAccent(opt.color));
            },
            child: Text(opt.label),
          ),
      ],
    );
  }
}
