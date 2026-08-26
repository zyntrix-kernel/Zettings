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

## Blocker (diagnosis as of 2026-08-26, empirically established — FINAL)

`cxx-qt 0.10.0`'s bridge parser rejects `include!` inside foreign blocks
**unconditionally, on every compiler** (stable 1.97 AND nightly 1.100 fail
identically, before macro expansion can even be dumped via
`-Zunpretty=expanded`). Even a primitives-only bridge with zero foreign C++
declarations fails, proving the parser trips on its own internally generated
fragments.

### Ruled out (all empirically)

| Hypothesis | Disproved by |
|---|---|
| User include syntax / edition / rustc version | identical failure on 1.97 + nightly 1.100, editions 2021/2024 |
| syn version drift | pinning syn 2.0.98: no effect |
| Emission sites in writer/qobject/threading/signals | vendored patch to plain extern: no effect |
| Toolchain | nightly fails pre-expansion |

Conclusion: cxx-qt 0.10.0 cannot compile ANY QObject bridge under the syn/
parser combination it ships with in our resolution — most likely an upstream
release regression (published 2026-08-24) where `parse_quote!`-built
extern-with-macro fragments are re-parsed by syn ≥ some version that no
longer accepts macros in foreign-item position.

## Unblock procedure

1. File/check KDAB issue with the minimal repro (primitives-only bridge,
   this toolchain set). Likely fixed within days given release cadence.
2. Try cxx-qt 0.9.1 (July 2026) — may predate the regression; adjust for
   API differences if any.
3. On fix: re-add member, drop `[patch.crates-io]` + `vendor/`, run gates,
   then Corrosion wiring per scaffold notes below.

build.rs already carries the cc_builder force-include trick needed for
QString-returning invokables once parsing works.

