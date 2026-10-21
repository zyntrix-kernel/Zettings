# Contributing to Zettings

Thank you for your interest in contributing to Zettings, the next-generation system settings application for Zyntrix OS.

---

## 1. Architectural Philosophy

Before contributing code, please review `AGENTS.md` and `ARCHITECTURE.md`. Zettings follows strict architectural separation:

- **Frontend:** Flutter Desktop, Dart, Material 3 (`useMaterial3: true`), Zyntrix Design Language (ZDL).
- **Backend:** Native Rust daemon, Tokio async runtime, zbus D-Bus service, PolicyKit authorization.
- **System Layer:** Direct Linux desktop integration via NetworkManager, PipeWire, BlueZ, UPower, systemd, and AccountsService.

### Prohibited Patterns
- Never execute raw shell commands from Flutter widgets.
- Do not introduce web frameworks (Node, React, Tauri, Vite, TypeScript) into the core architecture.
- Do not hardcode raw color hex codes or magic layout values; use centralized theme tokens in `lib/core/theme/`.
- No mock data or fake system state in production code paths.

---

## 2. Development Setup

### Prerequisites
- Flutter SDK (stable channel, >= 3.24)
- Rust toolchain (stable, edition 2024, >= 1.80)
- Linux desktop build dependencies (Kubuntu/Ubuntu):
  ```bash
  sudo apt install clang cmake ninja-build pkg-config libgtk-3-dev liblzma-dev libdbus-1-dev
  ```

---

## 3. Quality & Verification Gates

All pull requests must pass the full verification pipeline before merging:

### Flutter Checks
```bash
# Verify formatting
dart format --output=none --set-exit-if-changed .

# Run static analysis (must report 0 issues)
flutter analyze

# Run unit and widget test suite
flutter test
```

### Rust Backend Checks
```bash
cd rust

# Verify formatting
cargo fmt --all --check

# Run linter (must report 0 warnings)
cargo clippy --workspace --all-targets -- -D warnings

# Run all backend unit and integration tests
cargo test --workspace
```

---

## 4. Input Validation & Security

Every system-facing IPC method must validate incoming parameters using typed validators:
- Rust boundary: `rust/src/ipc/validation.rs`
- Dart presentation layer: `lib/core/utils/validators.dart`

Ensure any new settings endpoint validates bounds, string lengths, and numeric ranges.

---

## 5. Pull Request Guidelines

1. Create a descriptive branch for your feature or bug fix:
   ```bash
   git checkout -b feature/network-dns-configuration
   ```
2. Keep commits atomic and focused. Use standard conventional commit format:
   - `feat: add IPv6 address validation`
   - `fix: correct power profile enum parsing`
   - `test: add Wi-Fi passphrase boundary test`
   - `docs: update IPC architecture diagrams`
3. Verify that all tests pass locally prior to pushing.
4. Ensure no file exceeds the 10KB maintainability threshold. Decompose large models and handlers into clean submodules.
