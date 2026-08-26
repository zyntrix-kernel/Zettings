# Settings Registry Schema (Phase 1 contract)

The registry is the single source of truth for everything navigable, searchable,
and mutable in ZETTINGS. It is built once at startup by the application core from
static category/page definitions plus runtime capability probes, then frozen.

## Data model

```text
Registry
 ├── Category            L1 hub
 │    └── Page           L2 settings page
 │         └── Section   L3 group card
 │              └── SettingDefinition   L4 row/control
 └── RouteIndex          route -> Page lookup (unique)
```

## Category

| Field | Type | Rules |
|---|---|---|
| `id` | `CategoryId` (string, kebab-case) | Stable forever; used in routes and tests |
| `title` | i18n key | Resolved via `qsTr` catalogs |
| `icon` | icon token | ZDL icon set name |
| `order` | integer | Hub sort order |
| `keywords` | list\<string\> | Extra search terms |
| `availability` | Availability predicate | Evaluated against device capabilities |

## Page

| Field | Type | Rules |
|---|---|---|
| `id` | `PageId` (kebab-case) | Unique across registry; target of exactly one route |
| `category_id` | `CategoryId` | Parent hub |
| `title`, `description` | i18n keys | Description appears in search hits |
| `template` | enum | One of `CategoryHub`, `SettingsPage`, `DeviceList`, `Detail` |
| `sections` | ordered list\<Section\> | Render order |
| `breadcrumbs` | derived | Computed from category chain — never hand-authored |
| `required_capabilities` | list\<CapabilityRequirement\> | AND-combined gate for entering |

## Section

| Field | Type | Rules |
|---|---|---|
| `id` | string | Unique within page |
| `title` | i18n key or none | Headerless groups allowed |
| `settings` | ordered list\<SettingDefinition\> | Render order |
| `visibility` | Availability predicate | Sections hide when predicate fails |

## SettingDefinition

Every setting carries all of the following (`prompt.txt` §3 mandate):

| Field | Type | Purpose |
|---|---|---|
| `id` | `SettingId` (dot-path, e.g. `sound.output.volume`) | Stable identity; search primary key |
| `title` | i18n key | Row label |
| `description` | i18n key | Row subtitle |
| `category_id`, `page_id`, `section_id` | back-references | Placement |
| `route` | `Route` | Deep link to the owning page |
| `aliases` | list\<i18n string\> | Alternate names ("Wi-Fi", "Wireless") |
| `keywords` | list\<string\> | Search boost terms |
| `control` | `ControlKind` | Renderer selection |
| `read` | `ReadBinding` | Current-value provider handle |
| `write` | Option\<WriteAction\> | Set-value action; `None` = read-only/informational |
| `validation` | Option\<ValueConstraint\> | Range/enum/string constraints applied pre-write |
| `permission` | `CapabilityRequirement` | See capability-model.md |
| `hardware_dependency` | Option\<HardwareProbe\> | Device presence predicate |
| `reboot_hint` | enum | `None` / `Recommended` / `Required` after change |
| `search_weight` | integer | Baseline rank modifier (Windows baseline table, PLAN §5) |
| `backend` | `BackendId` | Owning adapter (`timedate`, `network`, `audio`, …) |

### ControlKind enum

```text
Toggle, Slider, ComboBox, TextField, Button, ActionRow, Expander,
ColorPicker, RadioGroup, FilePicker, DateTimePicker, InfoBadge
```

Renderers for these are the only controls Pages may instantiate (PLAN §4 template rule).

### ReadBinding / WriteAction

```rust
// Contract shape (zettings-core); concrete adapters bind these at registration.
pub struct ReadBinding {
    pub backend: BackendId,
    pub key: ValueKey,               // adapter-namespaced value key
    pub poll: PollPolicy,            // OnDemand | Interval(Duration) | Signal
}

pub struct WriteAction {
    pub backend: BackendId,
    pub operation: OperationKey,
    pub confirmation: ConfirmationPolicy,  // None | Preview | ExplicitConfirm
}
```

AI-assisted changes are constrained at the schema level: any setting whose
`write.confirmation != None` cannot be applied without an explicit user step
(PLAN §5 "never silently change arbitrary configuration").

## Invariants (enforced by registry builder, hard-fail at startup)

1. Every `SettingDefinition.route` resolves to exactly one existing page.
2. Every page has ≥ 1 section; every section has ≥ 1 setting.
3. `SettingId`s are globally unique; `Route`s are unique per page.
4. Every `backend` id referenced must have a registered adapter or the setting
   reports `Unavailable("backend-missing")` — it may not disappear silently.
5. All i18n keys resolve in the shipped catalog; unresolved keys fail the build.

## Serialization

The frozen registry snapshot serializes to `registry-snapshot.json`
(serde) used by: integration tests (golden diff), search index construction,
and documentation generation. Schema version field: `schema_version: u32`.
