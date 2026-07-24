# Lints and the lint registry

A **lint** is an opinionated warning: a diagnostic that flags legitimate but
usually-undesirable code (an unused item, an unnecessary `mut`, a `pub` that isn't
needed, …). Unlike a hard error, a lint can be silenced by the author when the flagged
code is intentional, by naming it in an `#[allow(...)]` attribute.

## Behaviour

`#[allow(<lint>)]` silences the named lint on the item or statement it is attached to. It
attaches to items and `let` statements today; blocks, expressions, and match arms are not
yet supported (see [Intended direction](#intended-direction)). The recognised lints:

- `dead_code` — suppresses the "never used" / "never constructed" warning on a function,
  struct, enum, trait, or impl method.
- `unused_variables` — suppresses the unused-variable warning on a `let` binding.
- `unused_mut` — suppresses the never-mutated-`mut` warning on a `let` binding.
- `constant_return` — suppresses the warning for an entry point that always returns a
  constant value.

An unrecognised name — a typo such as `#[allow(dead_cod)]` — is reported as an `unknown
lint` warning and silences nothing, so the lint it was meant to suppress still fires; that
warning does not stop compilation.

Each of these behaviours is pinned by a test, which is the authoritative specification:
`allow_dead_code_on_unused_function`,
`does_not_error_on_unused_impl_method_if_marked_as_allow_dead_code`,
`silences_unused_variable_warning`, `warns_on_unknown_lint_in_allow_attribute`, and
`typo_in_allow_does_not_suppress_the_lint` in
[`unused_items.rs`](../compiler/noirc_frontend/src/tests/unused_items.rs), and
`does_not_trigger_unnecessary_mut_on_variable_if_annotated_with_allow_unused_mut` in
[`expressions.rs`](../compiler/noirc_frontend/src/tests/expressions.rs). The
`constant_return` warning is covered end-to-end by
[`allow_warnings.rs`](../compiler/noirc_driver/tests/allow_warnings.rs).

This mirrors Rust's lint-control attributes; see
[Rust's lint levels](https://doc.rust-lang.org/rustc/lints/levels.html). Noir implements the
`allow` level only so far.

## Lint names are a closed set

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
`pub`, unreachable code, unbounded recursion, …) and to back a fuller lint-control surface
matching Rust's lint levels: `#[warn]` / `#[deny]` / `#[expect]`, lint lists, and central
location-based level resolution so that backend diagnostics can be controlled by
source-level attributes too. The full plan is tracked in
[noir-lang/noir#7461](https://github.com/noir-lang/noir/issues/7461).
