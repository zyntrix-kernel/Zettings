import 'package:flutter/material.dart';
import 'package:zettings/app/shell.dart';
import 'package:zettings/core/errors/app_error.dart';
import 'package:zettings/core/models/setting.dart';
import 'package:zettings/core/platform/capability.dart';
import 'package:zettings/core/services/service_locator.dart';
import 'package:zettings/core/services/system_service.dart';
import 'package:zettings/core/state/registry.dart';
import 'package:zettings/features/accessibility/accessibility_page.dart';
import 'package:zettings/features/accounts/accounts_page.dart';
import 'package:zettings/features/applications/applications_page.dart';
import 'package:zettings/features/bluetooth/bluetooth_page.dart';
import 'package:zettings/features/developer/developer_page.dart';
import 'package:zettings/features/home/home_page.dart';
import 'package:zettings/features/network/network_page.dart';
import 'package:zettings/features/personalization/personalization_page.dart';
import 'package:zettings/features/privacy/privacy_page.dart';
import 'package:zettings/features/system/system_page.dart';
import 'package:zettings/features/updates/updates_page.dart';
import 'package:zettings/shared/widgets/section_header.dart';
import 'package:zettings/shared/widgets/setting_row.dart';
import 'package:zettings/shared/widgets/settings_page.dart';
import 'package:zettings/shared/widgets/state_views.dart';

/// Central route table. Paths are stable: the registry, search, and deep
/// links all resolve through these entries.
class SettingsRoute {
  const SettingsRoute._({
    required this.path,
    required this.title,
    required this.category,
  });

  final String path;
  final String title;
  final SettingCategory category;

  static const home = SettingsRoute._(
    path: '/',
    title: 'Home',
    category: SettingCategory.system,
  );

  static const List<SettingsRoute> values = [
    home,
    SettingsRoute._(
      path: '/system',
      title: 'System',
      category: SettingCategory.system,
    ),
    SettingsRoute._(
      path: '/network',
      title: 'Network',
      category: SettingCategory.network,
    ),
    SettingsRoute._(
      path: '/bluetooth-devices',
      title: 'Bluetooth & devices',
      category: SettingCategory.bluetoothDevices,
    ),
    SettingsRoute._(
      path: '/personalization',
      title: 'Personalization',
      category: SettingCategory.personalization,
    ),
    SettingsRoute._(
      path: '/accessibility',
      title: 'Accessibility',
      category: SettingCategory.accessibility,
    ),
    SettingsRoute._(
      path: '/accounts',
      title: 'Accounts',
      category: SettingCategory.accounts,
    ),
    SettingsRoute._(
      path: '/privacy-security',
      title: 'Privacy & security',
      category: SettingCategory.privacySecurity,
    ),
    SettingsRoute._(
      path: '/applications',
      title: 'Applications',
      category: SettingCategory.applications,
    ),
    SettingsRoute._(
      path: '/updates',
      title: 'Updates',
      category: SettingCategory.updates,
    ),
    SettingsRoute._(
      path: '/developer',
      title: 'Developer',
      category: SettingCategory.developer,
    ),
  ];

  /// Resolves a route path, or null when unknown.
  static SettingsRoute? resolve(String path) {
    for (final route in values) {
      if (route.path == path) return route;
    }
    return null;
  }

  /// Parses a `zettings://host/path…` deep link into a route path.
  ///
  /// Examples: `zettings://network` → `/network`,
  /// `zettings://network/wifi` → `/network/wifi`.
  /// Returns null for anything that is not a zettings link.
  static String? parseDeepLink(String url) {
    final uri = Uri.tryParse(url);
    if (uri == null || uri.scheme != 'zettings') return null;
    final segments = [if (uri.host.isNotEmpty) uri.host, ...uri.pathSegments];
    if (segments.isEmpty) return '/';
    return '/${segments.join('/')}';
  }

  /// Navigates to a `zettings://` deep link. Returns true when the link
  /// resolved to a known route, false otherwise (unknown links never throw
  /// and never navigate).
  static bool openDeepLink(BuildContext context, String url) {
    final path = parseDeepLink(url);
    if (path == null || resolve(path) == null) return false;
    Navigator.of(context).pushNamed(path);
    return true;
  }
}

/// Builds the page for [settings]. Every page is shell-wrapped so navigation,
/// search, and appearance stay consistent. Unknown names produce a typed
/// error page — never a crash, never a silent redirect.
Route<dynamic> generateSettingsRoute(RouteSettings settings) {
  final route = SettingsRoute.resolve(settings.name ?? '/');
  if (route == null) {
    return M3DesktopPageRoute(
      settings: settings,
      builder: (_) => SettingsErrorPage(
        error: AppError(
          AppErrorType.notFound,
          'Page not found: ${settings.name}',
          recovery: 'Return home and navigate from the category list.',
          details: 'Unknown route: ${settings.name}',
        ),
      ),
    );
  }
  return M3DesktopPageRoute(
    settings: settings,
    builder: (_) => ZettingsShell(
      current: route,
      child: switch (route.path) {
        '/' => const HomePage(),
        '/system' => const SystemPage(),
        '/network' => const NetworkPage(),
        '/bluetooth-devices' => const BluetoothDevicesPage(),
        '/personalization' => const PersonalizationPage(),
        '/accessibility' => const AccessibilityPage(),
        '/accounts' => const AccountsPage(),
        '/privacy-security' => const PrivacySecurityPage(),
        '/applications' => const ApplicationsPage(),
        '/updates' => const UpdatesPage(),
        '/developer' => const DeveloperPage(),
        _ => CategoryPage(route: route),
      },
    ),
  );
}

/// Smooth Material 3 desktop page transition: fade + subtle vertical translation.
/// Respects accessibility and reduced-motion preferences immediately.
class M3DesktopPageRoute<T> extends PageRouteBuilder<T> {
  M3DesktopPageRoute({required WidgetBuilder builder, super.settings})
    : super(
        pageBuilder: (context, animation, secondaryAnimation) =>
            builder(context),
        transitionDuration: const Duration(milliseconds: 200),
        reverseTransitionDuration: const Duration(milliseconds: 150),
        transitionsBuilder: (context, animation, secondaryAnimation, child) {
          if (MediaQuery.of(context).disableAnimations) {
            return child;
          }
          final curved = CurvedAnimation(
            parent: animation,
            curve: const Cubic(0.05, 0.7, 0.1, 1.0),
          );
          return FadeTransition(
            opacity: Tween<double>(begin: 0.0, end: 1.0).animate(curved),
            child: SlideTransition(
              position: Tween<Offset>(
                begin: const Offset(0.0, 0.015),
                end: Offset.zero,
              ).animate(curved),
              child: child,
            ),
          );
        },
      );
}

/// Category landing page fallback: description, live integration status
/// (only when a matching system service is registered), and the section's pages.
class CategoryPage extends StatefulWidget {
  const CategoryPage({super.key, required this.route});

  final SettingsRoute route;

  @override
  State<CategoryPage> createState() => _CategoryPageState();
}

class _CategoryPageState extends State<CategoryPage> {
  Future<SystemCapability>? _probe;

  @override
  void initState() {
    super.initState();
    _probe = _findService()?.probe();
  }

  SystemService? _findService() {
    final wanted = serviceNameFor(widget.route.category);
    for (final service in ServiceLocator.all<SystemService>()) {
      if (service.name == wanted) return service;
    }
    return null;
  }

  @override
  Widget build(BuildContext context) {
    final registry = ServiceLocator.get<SettingsRegistry>();
    final definition = registry.byRoute(widget.route.path);
    final siblings = registry
        .forCategory(widget.route.category)
        .where((e) => e.route != widget.route.path)
        .toList();

    return ZettingsPage(
      title: widget.route.title,
      children: [
        if (definition != null)
          Padding(
            padding: const EdgeInsets.only(bottom: 8),
            child: Text(definition.description),
          ),
        if (_probe != null) ...[
          const SectionHeader(title: 'Status'),
          FutureBuilder<SystemCapability>(
            future: _probe,
            builder: (context, snapshot) {
              if (snapshot.hasError) {
                return ErrorView(
                  error: AppError(
                    AppErrorType.systemError,
                    'Could not check ${widget.route.title} status.',
                    details: '${snapshot.error}',
                  ),
                  onRetry: () => setState(() {
                    _probe = _findService()?.probe();
                  }),
                );
              }
              if (!snapshot.hasData) return const LoadingView();
              final capability = snapshot.data!;
              return ZettingsRow(
                icon: widget.route.category.icon,
                title: 'Integration status',
                subtitle:
                    capability.reason ??
                    _availabilityLabel(capability.availability),
                trailing: Text(
                  _availabilityLabel(capability.availability),
                  style: Theme.of(context).textTheme.labelLarge,
                ),
              );
            },
          ),
        ],
        const SectionHeader(title: 'In this section'),
        if (siblings.isEmpty)
          const EmptyView(
            icon: Icons.folder_open,
            title: 'Nothing else here yet',
            hint:
                'Pages for this section appear as its system '
                'integration lands.',
          )
        else
          for (final entry in siblings)
            ZettingsRow(
              icon: entry.icon,
              title: entry.title,
              subtitle: entry.description,
              trailing: const Icon(Icons.chevron_right),
              onTap: () => Navigator.of(context).pushNamed(entry.route),
            ),
      ],
    );
  }

  String _availabilityLabel(ServiceAvailability availability) =>
      switch (availability) {
        ServiceAvailability.available => 'Available',
        ServiceAvailability.unavailable => 'Unavailable',
        ServiceAvailability.unsupported => 'Unsupported',
        ServiceAvailability.permissionDenied => 'Permission denied',
        ServiceAvailability.unknown => 'Unknown',
      };
}

/// Typed error page used for unknown routes (and reused by features later).
class SettingsErrorPage extends StatelessWidget {
  const SettingsErrorPage({super.key, required this.error});

  final AppError error;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Something went wrong')),
      body: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          mainAxisSize: MainAxisSize.min,
          children: [
            Text(error.message, style: Theme.of(context).textTheme.titleMedium),
            if (error.recovery != null) ...[
              const SizedBox(height: 8),
              Text(error.recovery!),
            ],
            const SizedBox(height: 16),
            FilledButton(
              onPressed: () => Navigator.of(context).popUntil((r) => r.isFirst),
              child: const Text('Back to Home'),
            ),
          ],
        ),
      ),
    );
  }
}
