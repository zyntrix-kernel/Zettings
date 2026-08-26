# Capability Model (Phase 1 contract)

Every setting, page, and section declares its requirements declaratively; the
capability layer evaluates them against live system probes and produces honest
states. Nothing is ever faked as working (`prompt.txt` hard rule).

## CapabilityRequirement

```rust
pub enum CapabilityRequirement {
    None,                          // pure UI state
    SessionOnly,                   // user-scope mutation, no auth
    Polkit { action: PolkitActionId },   // privileged: org.zyntrix.<domain>.<verb>
    Hardware(HardwareProbe),       // device presence required
    All(Vec<CapabilityRequirement>),     // conjunction
    Any(Vec<CapabilityRequirement>),     // disjunction
}
```

## HardwareProbe

```rust
pub enum HardwareProbe {
    BluetoothAdapterPresent,
    NetworkWirelessPresent,
    AudioOutputPresent,
    AudioInputPresent,
    BatteryPresent,
    DisplayOutputPresent,
    PrinterPresent,
}
```

Probes run once at startup and re-run on relevant D-Bus signals (e.g.
BlueZ `InterfacesAdded`); results feed the availability predicates in the
registry.

## Evaluated states

| State | Meaning | UI rendering |
|---|---|---|
| `Available` | Requirement satisfied | Normal control |
| `Unavailable { reason }` | Hardware/platform missing | Disabled row + honest reason text |
| `RequiresAuth { action }` | Mutation needs polkit decision | Control enabled; auth dialog on commit |
| `ReadOnly { reason }` | Readable but not mutable here (e.g. managed by MDM, lockdown) | Value shown, control disabled with explanation |

Reason strings are structured enums serialized to QML — the UI maps them to
localized copy; adapters never return raw English from the backend layer.

## Authorization flow (write path)

```text
SettingDefinition.write
  → capability evaluation (cheap, cached)
  → if RequiresAuth: polkit check via zettings-polkit gateway
      → Allowed            → proceed
      → AuthenticationRequired → KDE agent dialog → retry once on grant
      → Denied             → audit-log denial, surface honest error
  → validation constraints
  → adapter operation
  → result + audit entry
```

The polkit gateway is the ONLY code path that touches
`org.freedesktop.PolicyKit1`; adapters receive pre-authorized operations and
cannot bypass it. Policy action ids mirror the packaging templates:
`org.zyntrix.zettings.<domain>.<verb>` (see
`packaging/org.zyntrix.zettings.policy`).

## Failure semantics

- Adapter connection failure ⇒ all settings owned by that backend evaluate to
  `Unavailable("backend-unreachable")`; the app starts degraded, never fails
  startup because one daemon is down.
- Probe failure ⇒ treated as probe-not-satisfied (`Unavailable("probe-failed")`),
  never as success.
