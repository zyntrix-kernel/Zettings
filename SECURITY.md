# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | Yes                |

## Reporting a Vulnerability

The Zyntrix OS team takes the security of our desktop environment and system utilities seriously. If you discover a security vulnerability in Zettings or the underlying D-Bus daemon, please report it privately.

- **Email:** contact@zyntrix.org
- **Subject:** [SECURITY] Zettings Vulnerability Report

Please include:
1. Description of the vulnerability and its potential impact.
2. Steps to reproduce or proof-of-concept payload.
3. Affected components (Flutter UI, Rust daemon, D-Bus interface, or Polkit actions).
4. Proposed mitigation if available.

We will acknowledge receipt within 48 hours and provide status updates as we investigate and remediate the issue. Please refrain from public disclosure until a security advisory and patch have been released.

## Security Architecture & Guarantees

Zettings adheres to strict least-privilege engineering principles:

1. **Privilege Separation:**
   - The Flutter user interface runs as an unprivileged user process.
   - The UI never invokes raw shell commands or elevates privileges directly.
   - All system-facing operations are mediated through a native Rust daemon communicating over D-Bus (`org.zyntrix.Zettings`).

2. **Strict Input Validation:**
   - All D-Bus methods validate parameter bounds, string lengths, and encodings prior to execution (`rust/src/ipc/validation.rs` and `lib/core/utils/validators.dart`).
   - Out-of-bounds or malformed requests are rejected with typed `InvalidInput` errors without propagating to the system.

3. **PolicyKit (Polkit) Authorization:**
   - Privileged operations (modifying system repositories, managing root user accounts, modifying network interface routing) enforce explicit PolicyKit authorization rules (`linux/packaging/org.zyntrix.zettings.policy`).
   - UI state is never trusted as authorization.

4. **Memory Safety:**
   - The backend is written in safe Rust with compile-time memory guarantees, avoiding buffer overflows, use-after-free, and data races.
