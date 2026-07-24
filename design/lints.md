# Lints and the lint registry

A **lint** is an opinionated warning: a diagnostic that flags legitimate but
usually-undesirable code (an unused item, an unnecessary `mut`, a `pub` that isn't
needed, …). Unlike a hard error, a lint can be silenced by the author when the flagged
code is intentional.

## Slugs are a closed set

Every lint has a stable, human-readable **slug** (`dead_code`, `unused_variables`,
`unused_mut`, …). A user names a slug in an `#[allow(...)]` attribute to silence that
lint on the annotated item.

The set of valid slugs is a closed enum, `Lint`
(`compiler/noirc_frontend/src/lint.rs`), rather than free-form strings. Making it a closed
set is what lets the compiler reject an unrecognised slug: a typo such as
`#[allow(dead_cod)]` must raise a warning and leave the real lint firing, instead of being
accepted as an inert no-op that the author mistakes for a working suppression.

- `Lint::slug()` / `Lint::from_slug()` round-trip a lint to and from its slug, and the
  variants are enumerated via `strum`'s `EnumIter` so there is no separate list to keep in
  sync.
- Consumers that honour `#[allow(...)]` compare against a `Lint` (via
  `AttributeList::has_allow` / `SecondaryAttributeKind::is_allow`) rather than a bare
  string, so the slug spellings live in exactly one place.
- The parser validates the slug when it parses `#[allow(<slug>)]`
  (`parser/parser/attributes.rs`). An unrecognised slug raises the `UnknownLint` parser
  warning. Parser warnings never block elaboration, so the underlying lint the author
  *meant* to silence still fires.

## Only effective slugs are registered

`Lint` lists only the lints that `#[allow(...)]` actually silences. A slug is treated as
"known" only when naming it has a real effect; registering a slug that suppresses nothing
would recreate the silent-no-op hazard the closed set exists to remove. As each opinionated
warning is wired to consult the registry, its lint is added to `Lint`, so every registered
slug always corresponds to a lint that can actually be controlled.

## Intended direction

The registry is meant to cover every opinionated warning the compiler emits (unnecessary
`pub`, unreachable code, unbounded recursion, the backend `return_constant` warning, …) and
to back a fuller Rust-style lint-control surface: `#[warn]` / `#[deny]` / `#[expect]`, lint
lists, and central location-based level resolution so that backend diagnostics can be
controlled by source-level attributes too. The full plan is tracked in
[noir-lang/noir#7461](https://github.com/noir-lang/noir/issues/7461).
