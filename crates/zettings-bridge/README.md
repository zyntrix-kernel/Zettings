# zettings-bridge — cxx-qt QObject bridges

Status: **quarantined** (`[workspace.exclude]`) pending one upstream-shaped
patch. Everything else in the workspace builds and tests green without it.

## Blocker (fully diagnosed 2026-08-26)

rustc 1.97 forbids macro invocations inside `unsafe extern` blocks.
`#[cxx_qt::bridge]` re-emits user foreign blocks (normalized) plus its own
additions into the final `cxx::bridge`, and several emitted fragments carry
`include!` inside `unsafe extern "C++"`:

| Emission site (cxx-qt-gen 0.10.0) | Trigger |
|---|---|
| `src/generator/rust/qobject.rs:47` | `qml_metadata.is_some()` |
| `src/generator/rust/qobject.rs:210` | same (second site) |
| `src/generator/rust/threading.rs:65` | `threading` enabled |
| `src/generator/rust/threading.rs:207` | same |

User-passed `include!("cxx-qt-lib/qstring.h")` also fails even when the input
block is written as plain `extern "C++"`, i.e. pass-through lands under
`unsafe` too.

## Fix shape (next session)

Vendor the crate and flip the four emission sites (and/or the pass-through
normalization) to plain `extern "C++"` — legal with macros on every edition —
then wire `[patch.crates-io] cxx-qt-gen = { path = "vendor/cxx-qt-gen" }`.
Upstream issue worth filing with KDAB referencing rustc restriction
("macros in unsafe extern blocks").

## Ready-to-go contents

* `build.rs`: CxxQtBuilder 0.10 + `.qt_module("Quick")`
* `app_info.rs`: AppInfoBridge (two-name QObject pattern, three invokables),
  mirrors official `test_inputs/invokables.rs` shape
* Workspace rejoin: remove the `exclude` entry after the patch lands.

Gates to run on rejoin: cargo fmt/clippy/check/test, then Corrosion wiring
into apps/zettings/CMakeLists.txt (FetchContent corrosion-rs/corrosion,
`corrosion_import_crate(... CRATES zettings-bridge CRATE_TYPES staticlib)`),
instantiate in main.cpp, surface in Gallery footer.
