# Lints and the lint registry

A **lint** is an opinionated warning: a diagnostic that flags legitimate but
usually-undesirable code (an unused item, an unnecessary `mut`, a `pub` that isn't
needed, …). Unlike a hard error, a lint can be silenced by the author when the flagged
code is intentional.

## Slugs are a closed set

Every lint has a stable, human-readable **slug** (`dead_code`, `unused_variables`,
`unused_mut`, …). A user names a slug in an `#[allow(...)]` attribute to silence that
lint on the annotated item.

Historically these slugs were bare strings, compared inline at each place that honoured
`#[allow(...)]`. That had a dangerous property: a typo such as `#[allow(dead_cod)]`
compiled and silently did nothing, because no consumer matched the misspelled string —
the author believed a warning was suppressed when it was not.

The registry (`compiler/noirc_frontend/src/lint.rs`) makes the set of valid slugs a
closed enum, `Lint`:

- `Lint::slug()` / `Lint::from_slug()` round-trip a lint to and from its slug.
- Consumers that honour `#[allow(...)]` compare against a `Lint` (via
  `AttributeList::has_allow` / `SecondaryAttributeKind::is_allow`) rather than a bare
  string, so the slug strings live in exactly one place.
- The parser validates the slug when it parses `#[allow(<slug>)]`
  (`parser/parser/attributes.rs`). An unrecognised slug raises the `UnknownLint`
  parser warning; parser warnings never block elaboration, so the underlying lint the
  author *meant* to silence still fires.

## Scope of the registry today

The registry currently lists only the lints that `#[allow(...)]` actually silences:
`dead_code`, `unused_variables`, and `unused_mut`. A slug is deliberately treated as
"known" only when naming it has a real effect — otherwise a valid-looking-but-inert slug
would reintroduce the same silent-no-op hazard the registry exists to remove.

## Intended direction

This is the first slice of the lint-control work tracked in
[noir-lang/noir#7461](https://github.com/noir-lang/noir/issues/7461). The registry is
meant to grow to cover every opinionated warning the compiler emits (unnecessary `pub`,
unreachable code, unbounded recursion, the backend `return_constant` warning, …), and to
back a fuller Rust-style lint-control surface (`#[warn]` / `#[deny]` / `#[expect]`, lint
lists, and central, location-based level resolution so backend diagnostics can be
controlled by source-level attributes too). Each of those lints is added to `Lint` as the
warning that produces it is wired through the registry, so that every registered slug
always corresponds to a lint that can actually be controlled.
