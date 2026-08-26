# zettings-bridge — cxx-qt QObject bridges

Status: **quarantined** (`[workspace.exclude]` — the member line is removed
from the root manifest; re-add after the unblock below).

## Blocker (diagnosis as of 2026-08-26, empirically established)

rustc 1.97 rejects `include!` inside `unsafe extern` blocks. Every
`#[cxx_qt::bridge]` expansion contains such a fragment **even when the input
bridge declares no foreign C++ types at all** — verified with a primitives-
only bridge (`u32` invokables, zero `extern "C++"` in input) still failing
with:

```
error: non-foreign item macro in foreign item position: include
error: expected one of ... found `/`
```

Both diagnostics are attributed to the macro invocation site, i.e. they are
produced from the macro's EXPANDED output.

### What was ruled out

| Hypothesis | Disproved by |
|---|---|
| User `include!` syntax | fails with no user include at all |
| cxx::bridge can't handle it on 1.97 | cxx-qt-lib compiles from source |
| Edition 2024 parsing | same failure on edition 2021 |
| syn version drift | pinning syn 2.0.98 (upstream's lock) changes nothing |
| Emissions in writer/qobject/threading/signals | all four patched to plain extern; failure persists |

The responsible emission therefore lives elsewhere in the
cxx-qt-macro/cxx-qt-gen chain (or in how the macro wraps its output) and must
be located by expanding the macro output directly.

## Unblock procedure (next session)

1. On a NIGHTLY toolchain in a scratch copy:
   `cargo rustc -p zettings-macro-probe -- -Zunpretty=expanded` to dump the
   expanded bridge and find the exact remaining `unsafe extern { include! }`.
2. Patch that site in `vendor/cxx-qt-gen` (or upstream crate) to plain
   `extern "C++"`.
3. Re-add `"crates/zettings-bridge"` to workspace members.
4. Full gates, then Corrosion wiring into apps/zettingss CMake
   (see git history: scaffold + build.rs force-include already prepared;
   QString-returning invokables additionally need the cc_builder
   `-include cxx-qt-lib/qstring.h` trick already in build.rs).
