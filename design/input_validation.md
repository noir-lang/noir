# Input Validation

Values that cross into constrained code from an untrusted source — a circuit's entry-point
parameters, and oracle results — carry representation invariants the type system does not itself
enforce. A `BoundedVec` decoded from calldata can claim a `len` greater than its capacity; a struct
can be reconstructed with a field combination its safe constructors would never produce. Code that
relies on those invariants without checking them is unsound. This is the bug behind
[noir-claude#814](https://github.com/noir-lang/noir-claude/issues/814), and the broader concern
tracked in [#4218](https://github.com/noir-lang/noir/issues/4218); the design here is from
[#13188](https://github.com/noir-lang/noir/issues/13188).

The pieces are the `std::validate::Validate` trait, a `#[derive(Validate)]` that builds it
structurally, an opt-in `#[validate(...)]` for invariants the structure can't express, and
automatic injection of a `validate()` call for every entry-point input.

## The `Validate` trait

`Validate` (in `noir_stdlib/src/validate.nr`) has one method, `fn validate(&self)`, which `assert`s
the type's invariants. It is in the prelude, so `value.validate()` and `#[derive(Validate)]` need no
import.

`#[derive(Validate)]` implements it by validating each field in turn. Primitive types
(`Field`, the integers, `bool`, `()`, `str<N>`) have hand-written **no-op** leaf impls: their
representation is their value, so there is nothing to assert. The no-ops matter — they make
`where T: Validate` satisfiable for any primitive, so the derive's field recursion bottoms out
without the user implementing anything. Aggregates (`[T; N]`, `[T]`, tuples, `Option<T>`) recurse
into their elements.

A type can also assert invariants the field recursion cannot see — ones spanning several fields, or
held in a hand-written impl — by naming methods with `#[validate(Type::method)]`. The derived
`validate` calls each before recursing into the fields. Each method must take `&self` and return
nothing; the `validate` attribute (lower-case) is **not** in the prelude and must be imported,
acting as a sign-post that something non-standard is happening. The signature is checked at the
attribute site, which is why a function path coerces to `FunctionDefinition`
([#13186](https://github.com/noir-lang/noir/issues/13186)) and the attribute can read its sibling
arguments ([#13187](https://github.com/noir-lang/noir/issues/13187)).

`validate` is an explicit, point-in-time check: nothing runs it on construction or mutation, so a
value can hold invariant-violating data (e.g. via `BoundedVec::push` of a bad element) until the
next `validate` surfaces it.

### Why a nominal trait, and no-op leaves

Validation is nominal rather than structural: a type is validated because it (or a type it contains)
*implements* `Validate`, not because of its shape. This mirrors Rust's `#[derive(Default)]` and lets
a type opt in deliberately. The no-op primitive leaves are what keep that ergonomic — without them,
`#[derive(Validate)]` on a struct of `Field`s would fail to find `Field: Validate`.

### `BoundedVec`

`BoundedVec`'s impl is **hand-written**, not derived (`noir_stdlib/src/collections/bounded_vec.nr`):
it asserts `len <= MaxLen` and validates only the *live* elements (the first `len`, via `for_each`),
skipping the unused backing storage, whose slots hold arbitrary leftover values. A derive could not
express either part. It is hand-written rather than `#[derive(Validate)] #[validate(...)]` because
`#[derive]` runs the registered handler, and `collections` is elaborated before `validate` registers
it; a plain `impl` has no such ordering constraint (the same reason `Option`'s impl is hand-written).

## Automatic validation of entry-point inputs

For every entry-point function (`main`, and a contract's entry-point functions — `is_entry_point`),
the elaborator injects a `param.validate()` call at the start of the body for each parameter whose
type implements `Validate`. See `Elaborator::entry_point_validation_statements` and
`prepend_statements` in `compiler/noirc_frontend/src/elaborator/function.rs`.

Decisions:

- **Inputs only, not oracle results or return values.** Outputs the circuit produces are already
  constrained by the code that produced them; validating them would mostly add no-ops. More
  importantly, an over-eager "validate everything" pass trains users to ignore it. The ACIR boundary
  already range-checks scalars on entry, so the gap that matters is composite inputs (a `BoundedVec`
  with `len > MaxLen` is accepted by the raw witness map). Oracle returns are validated by the
  Brillig VM's `MemoryValue::new_checked`, so the remaining hole is entry calldata.

- **In the elaborator, not the monomorphizer.** The monomorphizer has more information, but by then
  a `BoundedVec` has become an anonymous tuple — the nominal `Validate` impl is gone and the `len`
  field is indistinguishable from any other. The elaborator still sees nominal types and can resolve
  trait impls, so the check is emitted there.

- **Skip provably-trivial types.** Injecting `x.validate()` for `x: Field` (or `[Field; N]`, or
  `(Field, bool)`) is a no-op that only adds boilerplate to every program. `ValidationWorth` (same
  file) walks the type and returns `false` only when just our own machinery is involved — a
  primitive, or an array/slice/tuple/`Option` built recursively from such. Any other named type
  (a user struct/enum, `BoundedVec`) is injected, *including* a type the user derived `Validate` for
  whose fields happen to be all-primitive: silently skipping a type the user opted into would be
  surprising. The traversal is bounded by `TypeRecursionContext` like every other type recursion.

- **Resolve `Validate` explicitly in the stdlib crate.** The injected reference resolves
  `std::validate::Validate` via a `PathKind::Resolved` path rooted at the stdlib crate
  (`TypedPath::resolved_in_crate`), not the unqualified name, so a user type named `Validate` that
  shadows the prelude cannot divert it. Absent in minimal programs compiled without the stdlib
  (`CrateGraph::try_stdlib_crate_id` returns `None`), in which case nothing is injected or warned —
  the feature is inert without the stdlib, which is why frontend unit tests (no stdlib crate) never
  see it.

- **The call lives in the body.** Because injection prepends to `main`'s body rather than wrapping
  the call site, anything that calls `main` runs the validation — including a `#[test]`. `main` stays
  an entry point under `nargo test` (`is_entry_point` is static: root module + the name `main`), so
  the validation is testable by calling `main` with an invalid value.

## The `unvalidated_input` warning

A parameter whose type does **not** implement `Validate` cannot be checked, so the elaborator emits
the `ResolverError::UnvalidatedInput` warning (in `compiler/noirc_frontend/src/hir/resolution/errors.rs`).
It is on by default; `--silence-warnings` drops it and `--deny-warnings` promotes it to an error,
making validation mandatory for the build — the model an integrity-critical project (e.g. Aztec
contracts) would adopt.

There is deliberately **no `#[allow]` attribute**. Noir has no per-parameter attributes, so a
function-level allow would be all-or-nothing, and `derive` reads from Rust whereas a bespoke
`#[allow(unvalidated_input)]` would not. Instead the opt-out is the `_` prefix (`fn main(_x: T)`),
the same convention the unused-variable lint uses. Crucially the `_` prefix opts out of the
**warning only**, not of validation: a `_`-prefixed parameter whose type implements `Validate` is
still validated, because a `_`-prefixed binding *can* still be used and an unchecked-but-used input
would be a silent gap.

## Not masking the unused-variable lint

The injected `param.validate()` references the parameter, which would normally mark it used and
suppress a genuine "unused variable" warning — the auto-validation would hide a real mistake. To
avoid that, the elaborator carries a `mark_variables_used` flag (default true) that
`Elaborator::use_variable` honors; the injected statements are elaborated with it off. The injection
therefore happens *separately* from the user body (the body records real usage), and the unused lint,
which runs afterwards on both, still flags an input the user never reads.

## Limitations

- **Tuples can't carry custom invariants.** `Validate` for tuples is a blanket impl that only
  recurses; a user cannot `impl Validate for (A, B)` (orphan rule) nor attach `#[validate(...)]` (no
  definition to annotate). To validate a cross-element invariant on a tuple input, wrap it in a named
  struct and derive/implement there — as in Rust.
- **`BoundedVec` detection blind spot.** Validating only live elements means a malicious `len` that
  is *within* capacity but exposes uninitialized storage (via `set_len`) is not caught by element
  validation; the `len <= MaxLen` assert is the invariant that holds. This is the accepted trade-off
  from #13188 — validating dead slots would reject well-formed vectors whose unused storage is
  arbitrary.
