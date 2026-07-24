# Lints and the lint registry

A **lint** is an opinionated warning: a diagnostic that flags legitimate but
usually-undesirable code (an unused item, an unnecessary `mut`, a `pub` that isn't
needed, …). Unlike a hard error, a lint can be silenced by the author when the flagged
code is intentional, by naming its slug in an `#[allow(...)]` attribute.

## Behaviour

`#[allow(<lint>)]` silences the named lint on the item or statement it is attached to. The
recognised lints, and where each takes effect:

- `dead_code` — on a function, struct, enum, trait, or impl method: its "never used" /
  "never constructed" warning is suppressed.
- `unused_variables` — on a `let` statement: no warning when the bound variable is never
  used.
- `unused_mut` — on a `let` statement: no warning when the binding is `mut` but never
  mutated.

A name that is not one of these — for example a typo, `#[allow(dead_cod)]` — produces an
`unknown lint` warning, and because the name matched nothing the lint it was meant to
silence still fires. The unknown-lint warning does not stop compilation.

`#[allow(...)]` attaches to items and `let` statements today; blocks, expressions, and
match arms are not yet supported (see [Intended direction](#intended-direction)).

This mirrors Rust's lint-control attributes; see
[Rust's lint levels](https://doc.rust-lang.org/rustc/lints/levels.html). Noir implements the
`allow` level only so far.

## Slugs are a closed set

The set of valid lint names is a closed set (the `Lint` enum) rather than free-form strings.
This is a deliberate decision: because the set is closed, an unrecognised name can be
reported to the author instead of being accepted as an inert no-op that is mistaken for a
working suppression.

`Lint` lists only the lints that `#[allow(...)]` actually silences. A name is treated as
"known" only when naming it has a real effect; registering a name that suppresses nothing
would recreate the silent-no-op hazard the closed set exists to remove. As each opinionated
warning is wired to consult the registry, its lint is added to `Lint`, so every registered
name always corresponds to a lint that can actually be controlled.

The registry and the mechanics of how names are validated and consumed live in
[`compiler/noirc_frontend/src/lint.rs`](../compiler/noirc_frontend/src/lint.rs).

## Intended direction

The registry is meant to cover every opinionated warning the compiler emits (unnecessary
`pub`, unreachable code, unbounded recursion, the backend `return_constant` warning, …) and
to back a fuller lint-control surface matching Rust's lint levels: `#[warn]` / `#[deny]` /
`#[expect]`, lint lists, and central location-based level resolution so that backend
diagnostics can be controlled by source-level attributes too. The full plan is tracked in
[noir-lang/noir#7461](https://github.com/noir-lang/noir/issues/7461).
