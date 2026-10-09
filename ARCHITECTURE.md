# ZETTINGS — Architecture

**Status:** Phases 0–9 Complete (Architecture, Material 3 Design System, ZDL, Shell, Routing, Registry, Search, Rust IPC, System Info & Full M3 Categories).
**Authority:** Implements `PLAN.md` §2 and `prompt.txt` §§14–15. Research basis: `RESEARCH.md`.

---

## 1. Layer diagram

```text
Flutter UI (Material 3 + ZDL)          lib/features/*, lib/shared/*
    ↓ intent (typed calls, never shell strings)
Application / Domain Layer             lib/app/*, lib/core/*
    SettingsRegistry · Router · Search · AppState · Service interfaces
    ↓ D-Bus method calls via generated typed clients
ZettingsIpcClient                      lib/core/ipc/*  (package:dbus, Phase 8)
    ↓ session bus, well-known name
Rust Backend daemon                    rust/*  (zbus + tokio, Phase 8)
    org.zyntrix.Zettings @ /org/zyntrix/Zettings
    ↓ system bus / direct syscalls, least privilege + polkit
Linux / KDE                            NM · UPower · PipeWire · BlueZ ·
                                       AccountsService · UDisks2 · PackageKit ·
                                       KScreen · logind · sysfs/procfs
```

Rules: widgets never touch D-Bus; Rust never holds presentation logic; no generic
`execute(command)` channel anywhere — every operation is a typed method with a typed
error (see §5).

## 2. Flutter module map

```text
lib/
  main.dart               bootstrap: binding, global error hooks, AppScope
  app/
    app.dart              ZettingsApp (MaterialApp, centralized M3 theme + OLED + dynamic accent)
    router.dart           SettingsRoute table, onGenerateRoute, deep-link parser, dedicated category pages
    shell.dart            ZettingsShell (Desktop NavigationRail/Drawer, M3 SearchBar/SearchAnchor, AppearanceMenu)
  core/
    errors/app_error.dart AppError + AppErrorType (user message + recovery + details)
    platform/capability.dart  ServiceAvailability, PermissionState, SystemCapability
    models/setting.dart   SettingCategory, SettingDefinition (registry schema)
    models/system_info.dart   Typed SystemInfo, CpuInfo, DiskInfo, GpuInfo D-Bus serialization
    state/app_state.dart  ZettingsState (ChangeNotifier) + AppScope
    services/service_locator.dart  manual DI
    services/system_service.dart   SystemService.probe() contract
    services/prefs_service.dart    SharedPreferences persistence for theme mode, accent, recents
    theme/zettings_theme.dart      Centralized Material 3 ThemeData with dynamic accent palettes & OLED
    utils/log.dart        ZettingsLog
  features/
    home/                 HomePage (M3 category grid, desktop hub)
    system/               SystemPage (Hardware, OS snapshot, RAM/Disk progress, Copy specs)
    personalization/      PersonalizationPage (Live theme switcher, 7 dynamic accents, font scaling, visuals)
    network/              NetworkPage (Wi-Fi master toggle, network list, Ethernet, VPN, Firewall)
    bluetooth/            BluetoothDevicesPage (BlueZ adapter toggle, paired devices, input peripherals)
    accessibility/        AccessibilityPage (Vision, motion, hearing, mobility options)
    accounts/             AccountsPage (User profile, password management, session settings)
    privacy/              PrivacySecurityPage (XDG portal permissions, firewall, diagnostic telemetry)
    applications/         ApplicationsPage (Default apps, autostart programs, Flatpak/Snap/AppImage runtimes)
    updates/              UpdatesPage (PackageKit/APT status, check for updates, firmware preferences)
    developer/            DeveloperPage (Rust daemon IPC telemetry, Wayland/X11 inspector, debug tools)
  shared/widgets/
    setting_group_card.dart Grouped rounded M3 surface containers with automatic dividers
    setting_row.dart      M3 ZettingsRow with tonal icon containers, ZettingsSwitchRow, ZettingsSliderRow
    hero_status_card.dart Top overview banner card with status badges and actions
    settings_page.dart    Responsive desktop page shell with typography hierarchy
    section_header.dart   Categorized section headers
    state_views.dart      LoadingView, EmptyView, ErrorView with recovery actions
```

## 3. Routing & deep links

- One `SettingsRoute(path, title, category)` per page; `SettingsRoute.values` is the
  single table. `onGenerateRoute` resolves; unknown paths → typed error page
  (`AppErrorType.notFound`), never a crash, never a silent redirect.
- Deep links: `zettings://<host>/<path…>` maps to `/<host>/<path…>`
  (e.g. `zettings://network` → `/network`). Parser is pure and unit-tested.
- Categories (10): system, network, bluetooth & devices, personalization,
  accessibility, accounts, privacy & security, applications, updates, developer.

## 4. State management

`ZettingsState extends ChangeNotifier`: selected route, appearance mode (system/light/dark/oled),
dynamic accent seed color, capability map.
`AppScope extends InheritedNotifier` exposes it. Async system ops live in services,
never in widgets; widgets observe state only.

## 5. Error model

```text
Unavailable · Unsupported · PermissionDenied · InvalidInput · Timeout ·
ServiceUnavailable · HardwareUnavailable · SystemError · NotFound
```

`AppError{type, message, recovery?, details?}`. UI shows message + recovery action
(+ retry where sensible); details go to an expandable diagnostic section for logs.
Rust `BackendError` maps 1:1 onto these over D-Bus (Phase 8).

## 6. Capability model & external state

Every service implements `probe() → SystemCapability{availability, permission, reason}`.
UI renders from capability: hide/disable with explanation when anything is not
`available`. System state can change externally → services expose change streams;
UI subscribes, never polls in a loop.

## 7. Dependency policy log

| Package | Justification |
|---|---|
| `flutter_lints` | SDK-recommended static analysis. |
| `shared_preferences` | Standard Flutter prefs persistence (appearance, accent, recents). |
| `dbus` (Canonical) | Dart↔D-Bus IPC path for Linux integration with the Rust backend. |

## 8. Rust backend layout

```text
rust/src/{main.rs, lib.rs, error.rs, models.rs, ipc.rs, services.rs, services/*.rs}
```

One module per system service, each with unit tests; D-Bus failures are values, not
panics. Privileged ops follow validate → authorize (polkit) → perform → verify →
return actual result.

## 9. Verification mapping (AGENTS.md §§33–34)

| Claim level | How | Result |
|---|---|---|
| Format/analyze | `dart format`, `flutter analyze` | Clean formatting, 0 issues |
| Dart Unit/Widget tests | `flutter test` | 99 / 99 tests passing |
| Rust Unit/Integration tests | `cargo test --workspace` | 37 / 37 tests passing (36 unit + 1 integration) |
| Rust Clippy | `cargo clippy --workspace --all-targets -- -D warnings` | 0 warnings |
| Build (Web & Desktop) | `flutter build web` | Clean compilation |
| Runtime integration | Kubuntu/Zyntrix hardware | D-Bus daemon on `org.zyntrix.Zettings` |
