# Lints and the lint registry

A **lint** is an opinionated warning: a diagnostic that flags legitimate but
usually-undesirable code (an unused item, an unnecessary `mut`, a `pub` that isn't
needed, …). Unlike a hard error, a lint can be silenced by the author when the flagged
code is intentional, by naming its slug in an `#[allow(...)]` attribute.

## Slugs are a closed set

The set of valid slugs is a closed enum, `Lint`, rather than free-form strings. This is a
deliberate decision: a closed set lets the compiler reject an unrecognised slug, so a typo
such as `#[allow(dead_cod)]` raises a warning and leaves the real lint firing instead of
being accepted as an inert no-op that the author mistakes for a working suppression.

`Lint` lists only the lints that `#[allow(...)]` actually silences. A slug is treated as
"known" only when naming it has a real effect; registering a slug that suppresses nothing
would recreate the silent-no-op hazard the closed set exists to remove. As each opinionated
warning is wired to consult the registry, its lint is added to `Lint`, so every registered
slug always corresponds to a lint that can actually be controlled.

The registry and the mechanics of how slugs are validated and consumed live in
[`compiler/noirc_frontend/src/lint.rs`](../compiler/noirc_frontend/src/lint.rs).

## Intended direction

The registry is meant to cover every opinionated warning the compiler emits (unnecessary
`pub`, unreachable code, unbounded recursion, the backend `return_constant` warning, …) and
to back a fuller Rust-style lint-control surface: `#[warn]` / `#[deny]` / `#[expect]`, lint
lists, and central location-based level resolution so that backend diagnostics can be
controlled by source-level attributes too. The full plan is tracked in
[noir-lang/noir#7461](https://github.com/noir-lang/noir/issues/7461).
