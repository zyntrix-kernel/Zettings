# Route Schema (Phase 1 contract)

ZETTINGS owns its route scheme (PLAN §6). Routes are stable identifiers,
serializable strings, and the only navigation addressing mechanism — search
hits, breadcrumbs, deep links, and back/forward history all resolve through it.

## Grammar

```text
zyntrix-settings://<category>[/<page>][?<params>]

category : kebab-case CategoryId        (required)
page     : kebab-case PageId            (optional; defaults to the category hub)
params   : application/x-www-form-urlencoded key=value pairs
```

Scheme name is `zyntrix-settings` (distinct from `ms-settings:` by design).
In-app canonical form used by the router state machine drops the scheme:
`/<category>/<page>`.

## Examples

```text
zyntrix-settings://system
zyntrix-settings://system/display
zyntrix-settings://system/sound?tab=output
zyntrix-settings://devices/bluetooth
zyntrix-settings://network/wifi?network=SSID-escaped
zyntrix-settings://personalization/themes
zyntrix-settings://updates
```

## Rules

1. Every navigable page has exactly one route; routes are unique.
2. Route → `PageId` resolution is a pure function over the frozen registry
   (`RouteIndex`). Unknown category/page resolves to the hub with an
   "address not found" state — never a crash, never a blank page.
3. Params are validated against the page's declared param schema; unknown or
   malformed params are dropped (page renders default tab/state).
4. Permission/hardware failure on the target page does not affect routing —
   the page loads and renders its own `Unavailable`/permission state (PLAN §6).
5. Back/forward: every successful navigation pushes a history entry; deep-link
   activation replaces the stack root. History entries serialize as plain
   route strings for session restore.

## Deep-link intake

| Entry point | Mechanism |
|---|---|
| Desktop file handler | `zettings.desktop` declares `X-Zyntrix-Scheme=zyntrix-settings`; `MimeType=x-scheme-handler/zyntrix-settings` |
| Second launch | Single-instance activation forwards argv URL to running instance over its D-Bus `org.zyntrix.Zettings` `/activate` method, then exits |
| CLI | `zettings 'zyntrix-settings://system/display'` behaves identically |

## Router contract (QML side)

```qml
// RouterBridge exposes to QML (see api-contracts.md):
//   property url currentRoute          (canonical in-app form)
//   function navigate(route: string)   // pushes history
//   function replace(route: string)    // deep-link root swap
//   signal navigated(route: string)    // QML animates transitions
```

The router never inspects page content; pages self-report availability.
Route changes are committed before animation starts so back/forward stays
consistent even if a transition is interrupted (motion engine requirement).
