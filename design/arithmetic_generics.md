# Arithmetic Generics

## `CheckedCast` and intermediate over/underflow

Arithmetic expressions over numeric generics (e.g. `(N - 1) + 1`) are simplified during
elaboration so that types like `[Field; (N - 1) + 1]` and `[Field; N]` unify. Simplification
can remove intermediate steps that would over/underflow at certain instantiations: `(N - 1) + 1`
simplifies to `N`, which evaluates fine for `N = 0` even though the `(0 - 1)` step underflows
`u32`.

To keep those failures detectable, simplification does not rewrite the expression in place.
Instead the elaborator wraps every arithmetic-generic expression in
`Type::CheckedCast { from, to }`, where `to` is the fully simplified form (used for
unification and evaluation) and `from` is the original expression with only constant folding
applied (see `Type::canonicalize` in `compiler/noirc_frontend/src/hir_def/types/arithmetic.rs`).

When a `CheckedCast` is evaluated to a constant (`Type::evaluate_to_integer_helper` in
`compiler/noirc_frontend/src/hir_def/types.rs`):

- `to` is evaluated first; its errors always propagate.
- `from` is then evaluated *without* simplifications, so every intermediate step of the
  original expression is computed on the concrete constants.
- If both sides evaluate, their values must match (`TypeCanonicalizationMismatch` otherwise).
- If `from` fails with a definite arithmetic failure on constant operands — the closed set of
  errors produced by `BinaryTypeOperator::function`, see
  `TypeCheckError::is_constant_arithmetic_failure` — that error propagates even though `to`
  evaluated successfully. This is what rejects `(N - 1) + 1` at `N = 0`.
- Any other failure of `from` is tolerated and `to`'s value is used. This is required because
  `from` may contain type variables that simplification canceled out of `to` (e.g.
  `from = (M + N) - M`, `to = N` with `M` unbound), and because canonicalization itself
  evaluates subexpressions speculatively while variables are still unbound.

## Solver-synthesized expressions carry the same obligation

The elaborator is not the only source of arithmetic-generic expressions. Type unification
*derives* them too: when a numeric generic is not pinned down by any argument, it is solved by
rearranging the equation it appears in. `try_unify_by_isolating_an_unbound_type_variable_in_self`
in `compiler/noirc_frontend/src/hir_def/types/unification.rs` turns `A + rhs = other` into
`A = other - rhs` (and the three mirrored shapes), canonicalizes the right-hand side, and binds
`A` to it.

That rearrangement is subject to exactly the hazard above — canonicalizing it runs the same
simplifications, including the cancellation rules, on operands the solver chose rather than ones
a programmer wrote. So the binding is wrapped in a `CheckedCast` the same way, with `to` the
canonical form everything downstream reasons with and `from` the rearrangement as derived. Both
readings reach evaluation, and a disagreement between them is an error at the instantiation.

This matters because nothing else re-derives the equation. A numeric generic bound to a bare
`InfixExpr` is accepted by monomorphization unconditionally — `check_type_helper` tolerates any
type-level numeric value it cannot lower — so a simplification that changed the value would be
committed with nothing downstream in a position to notice, and the generic would reach codegen
holding a number the source program does not determine. Wrapping the binding is what puts it
back under the from/to comparison.

The monomorphizer's `check_checked_cast` performs a similar (stricter) check for
`CheckedCast`s it encounters structurally (e.g. in struct generic arguments), but array/string
lengths never reach it: length types are resolved directly via `evaluate_to_u32`, so the
evaluation rules above are the mechanism that catches intermediate over/underflow in lengths.
For the same reason, `convert_type`/`check_type` do not recurse structurally into `from`:
evaluation already traverses it (including nested `CheckedCast`s introduced by generic
substitution), and a structural `check_type` on `from` could falsely reject unbound variables
that were legitimately simplified away.

In `check_checked_cast`, a failure to evaluate the `to` side is itself a hard error
(`MonomorphizationError::CheckedCastEvaluationFailed`, rendered as the underlying type-check
error such as "Modulo by zero"). Without this, a destination type whose value is undefined for
the concrete generics (e.g. `W<(0 * N) / (N % N)>` at `N = 0`) would silently skip the
from/to comparison and compile, as long as the value was never forced elsewhere (e.g. used as
an array length or a runtime value). The single exception is `NonConstantEvaluated`: the `to`
side may still contain an unbound-but-defaultable generic, which the surrounding
`convert_type`/`check_type` recursion will default or reject with `NoDefaultType`.

`check_checked_cast` unifies `from` with `to` into a local set of bindings and evaluates both
sides with those bindings substituted in, without ever applying them. The type variables in a
`CheckedCast` are shared with the elaborated program, and monomorphization must leave that
program as it found it (see `compiler/noirc_frontend/src/monomorphization/purity.rs`): a binding
committed here would be visible to every later compilation against the same context. An unbound
variable on either side is therefore resolved for the purpose of the check only.
