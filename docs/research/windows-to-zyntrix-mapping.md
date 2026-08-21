# Windows 11 Settings → ZETTINGS Mapping

> Source of truth: `ZETTINGS_Windows_11_Settings_Deep_UI_Spec.md` (read in full, 2026-08-21).
> Status labels follow spec §24: `public` / `observed` / `inferred`.
>
> Rule: ZETTINGS reproduces the **information architecture and interaction grammar**,
> never Microsoft's proprietary implementation. Visual language is ZDL (see `DESIGN.md`).

## 1. Top-level information architecture (preserved as-is)

| # | Category | ZETTINGS route | Primary Linux backend |
|---|----------|----------------|----------------------|
| 1 | Home | `zettings://home` | registry + usage signals |
| 2 | System | `zettings://system` | KScreen/KWin, PipeWire, UPower, PPD, udisks2 |
| 3 | Bluetooth & devices | `zettings://devices` | BlueZ, CUPS, libinput, udev |
| 4 | Network & internet | `zettings://network` | NetworkManager, iwd/wpa_supplicant via NM |
| 5 | Personalization | `zettings://personalization` | KDE look-and-feel, plasma API, fontconfig |
| 6 | Apps | `zettings://apps` | PackageKit, Flatpak, Snap, XDG MIME |
| 7 | Accounts | `zettings://accounts` | AccountsService, PolicyKit |
| 8 | Time & language | `zettings://time-language` | timedatectl/locale1, KDE KCMLocale |
| 9 | Gaming | `zettings://gaming` | GameMode, MangoHud (capability-gated) |
| 10 | Accessibility | `zettings://accessibility` | AT-SPI2, KDE a11y, Orca integration points |
| 11 | Privacy & security | `zettings://privacy` | PolicyKit, firewalld/ufw, KDE portal policy |
| 12 | Updates | `zettings://updates` | PackageKit / unattended-upgrades |

Home is a launch surface with interactive cards for frequent/recommended settings
(spec §1) — implemented from the same registry, not hand-built.

## 2. Shell anatomy → ZETTINGS components

| Spec evidence (§2–3) | ZETTINGS component | Notes |
|---|---|---|
| `NavigationView#PermanentNavigationView` (observed) | `<AppShell>` + `<NavPane>` | left rail; expanded/compact/minimal modes per §16 breakpoints |
| Search box atop nav pane | `<SearchSurface>` | opens Spotlight-style overlay; full search subsystem (PLAN §5) |
| Back affordance | `<BackButton>` + history stack | Alt+Backspace binding; back/forward serializable |
| `CategoryPage` / `L2Page` split (observed) | `CategoryPageTemplate` / `L2PageTemplate` | two distinct page layers — not every screen is bespoke |
| `SettingsListView` / `SettingsListViewItem` / `EntityItem` (observed) | `<SettingsList>` / `<SettingsCard>` | single-column, constrained width (~1000–1100 px), scrollable |
| `SettingsExpander` (observed) | `<SettingsExpander>` | one expansion level only (spec §5.3) |

## 3. Page hierarchy model

```text
L0 shell            → AppShell (window, nav, search)
L1 category/hub     → CategoryPageTemplate
L2 settings page    → L2PageTemplate
L3 section/group    → SettingsGroupTemplate
L4 setting entity   → SettingsCard / control renderer
L5 inline expander  → SettingsExpander
L6 dialog/flyout    → Dialog/Flyout/Picker primitives
```

## 4. SettingsCard contract

```text
┌────────────────────────────────────────────────────┐
│ [icon]  Title                          [control]   │
│         Description / status text                  │
└────────────────────────────────────────────────────┘
```

Right-side controls: ToggleSwitch · Button · ComboBox · Slider · Hyperlink ·
status text · navigation chevron. A card is EITHER a toggle OR a navigation
affordance — never ambiguous (spec §18).

## 5. Control inventory to implement (spec §7)

ToggleSwitch, CheckBox, RadioButton, ComboBox, Slider, TextBox, PasswordBox,
Button, Hyperlink, ListView, Expander, SettingsCard, SettingsExpander, InfoBar,
ProgressBar, ProgressRing, DatePicker, TimePicker, ColorPicker, Flyout, Menu,
ContextMenu, SearchBox/AutoSuggest, DevicePicker, FilePicker, PermissionList,
NavigationRow.

Each control: accessible name/role/state, keyboard operation, visible focus,
ZDL states (hover/press/disabled), reduced-motion behavior.

## 6. Page templates (spec §14)

- **A — Category hub**: title, description, cards with status values.
- **B — Standard settings list**: breadcrumb, title, sections of cards.
- **C — Device manager**: status overview card + device rows + actions.
- **D — Detail page**: back, icon/title/description, main+secondary settings, advanced expander.
- **E — Search results**: query, ranked settings with category > page trail, open action.

## 7. State behavior (spec §15)

Loading → skeleton; Disabled → keep visible + explain why; Permission →
explanation + action; No device → empty-state card; Offline → stateful UI;
Error → human-readable + retry; simple settings apply immediately (no Apply dialog);
explicit Apply/Save only where technically required.

## 8. Responsiveness (spec §16)

| Content width | Navigation |
|---|---|
| >1100px | full pane (icon+text), spacious content |
| 800–1100px | compact icon rail |
| 560–800px | minimal/overlay navigation |
| <560px | single-column mobile-style |

Content max-width ~1000–1100px regardless of window width.

## 9. Accessibility floor (spec §17)

Ctrl+F → search; Alt+Backspace → back; arrows → list nav; Enter/Space → activate;
Esc → dismiss overlay; Tab/Shift+Tab traversal. Every control: name, role, state,
focus indicator, high-contrast compatibility, reduced-motion behavior.
ZETTINGS targets WCAG 2.2 AA minimum (AAA where applicable) per prompt.txt.

## 10. Deep links

Full ms-settings catalog (spec §10) is the *reference inventory* for coverage.
ZETTINGS uses its own scheme (`zettings://…`, spec §11). Deprecated/conditional
Windows URIs are NOT blindly ported (PLAN §20 warning); each route must map to a
real ZETTINGS capability or be omitted.

## 11. Registry-driven architecture (spec §19 + §25 + Rules)

Every setting carries: stable ID, title, description, category/page/section/route,
aliases, keywords, icon, control type, value provider, set action, validation,
requires-admin/hardware/reboot flags, search weight, backend capability reference.
Pages are data: template + sections + setting IDs → rendered by generic templates.
Rules 1–10 from spec §26 are adopted verbatim as implementation law.

## 12. Backend mapping (spec §13)

Display→KScreen/KWin · Sound→PipeWire/WirePlumber · Network→NetworkManager ·
Bluetooth→BlueZ · Power→power-profiles-daemon/UPower · Storage→udisks2 ·
Users→AccountsService · Firewall→ufw/firewalld · A11y→AT-SPI2 ·
Default apps→XDG MIME · Printers→CUPS · Fonts→fontconfig · Time→timedatectl.
Reproduce the user model on native APIs; never fake a Windows backend.

## 13. Conflicts resolved (PLAN §0.1 hierarchy)

| Conflict | Resolution |
|---|---|
| Spec §20 proposes 12 parallel agents | AGENTS.md §1 mandates a **single agent**. Spec roles become sequential workstreams executed by the one active agent. |
| Spec §22 Option B (Qt/QML) | Rejected — prompt.txt + AGENTS.md fix the stack at **Tauri v2 + React 19**. Option A selected. |
| Spec §6.2 Fluent corner radii (4–8 px) | Superseded for major surfaces by ZDL G2/G3 superellipse geometry (AGENTS.md forbids standard border-radius on cards/panels). Small-control radii remain ZDL tokens informed by the reference scale. |
| Spec §6.3 Mica/Acrylic | Translated to KWin-compatible translucent surfaces + ZDL glass stack with opaque fallback when compositing is off. |

