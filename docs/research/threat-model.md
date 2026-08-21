# Threat Model — ZETTINGS

> Framework: STRIDE-lite, scoped to a desktop settings application that is the
> single configuration authority of Zyntrix OS. Requirements sources: prompt.txt
> (Zero Trust, least privilege, PolicyKit, audit logging) and spec §26 Rules 6–9.

## Assets

1. **System configuration state** — NM profiles, user accounts, systemd units,
   firewall rules, polkit grants.
2. **Privilege boundary** — root/polkit actions (`zettings-polkit` gateway).
3. **IPC surface** — Tauri commands exposed to the webview.
4. **User secrets** — WiFi PSKs, VPN credentials, account data (transit only;
   never stored by ZETTINGS).
5. **Audit log integrity** — record of who changed what, when.

## Trust boundaries

```text
Untrusted: rendered web content, search queries, device names from network,
           file paths from pickers, package metadata
    │  ← Tauri IPC validation layer
Semi-trusted: React frontend (sandboxed webview, no direct system access)
    │  ← typed commands + capability checks
Trusted-core: Rust app layer (input validation, policy enforcement)
    │  ← polkit authorization + adapter allowlists
System: D-Bus services, daemons, kernel
```

## Threat register

| # | Threat | Vector | Mitigation |
|---|--------|--------|------------|
| T1 | Arbitrary command execution via UI | crafted strings reaching shells | **No shell interpolation ever.** CLI transports = fixed argv templates with typed placeholders; no `sh -c`; deny-by-default allowlist |
| T2 | Privilege escalation via IPC | compromised/buggy webview invokes privileged command | every privileged command re-validates args in Rust; polkit action per operation; webview capability file lists minimal allowed commands |
| T3 | Path traversal / arbitrary file write | setting values containing paths | canonicalize+scope paths to declared config files; no generic "write file" command exists |
| T4 | Config injection (INI/conf metacharacters) | names/values with newlines, `[sections]` | strict validators per field type; reject control characters; escape per target format |
| T5 | Secret leakage into logs/audit | passwords/PSKs logged | redaction layer: secret-carrying structs implement custom Debug/Serialize that elides values |
| T6 | Audit log tampering | local attacker erases history | journal-forwarded entries (`tracing-journald`); journald sealing/rotation owned by systemd, not ZETTINGS |
| T7 | CSRF-style cross-app deep-link abuse | any app launches `zettings://…` URIs | deep links navigate/read-only by default; mutations always require explicit UI interaction; no URI-triggered apply |
| T8 | Malicious plugin/module | third-party settings module | plugin SDK: signed manifests (ed25519), sandboxed capability declarations, no ambient authority |
| T9 | Denial of service via IPC flood | webview spamming commands | per-command rate limits on expensive operations; async cancellation tokens |
| T10 | Downgrade/insecure defaults | shipping permissive capabilities | capabilities JSON reviewed as security-relevant code; CI check that generated ACL matches source-declared set |

## Security invariants (checked at review + tests)

1. The UI layer cannot mutate system state except through registered typed commands.
2. Every privileged mutation maps to exactly one polkit action ID.
3. No command accepts raw shell strings, SQL, or file contents.
4. Secrets are never serialized to the webview beyond masked display forms.
5. Adapter failures degrade to honest unavailable states — never silent fallbacks
   that change behavior without disclosure.

## Out of scope (documented)

- Hardening of third-party daemons (NM, BlueZ, CUPS).
- Multi-user session isolation beyond polkit semantics.
- Network-level attacks against D-Bus (system bus is socket-authenticated locally).
