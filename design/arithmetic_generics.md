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
  `from` may contain type variables that simplification cancelled out of `to` (e.g.
  `from = (M + N) - M`, `to = N` with `M` unbound), and because canonicalization itself
  evaluates subexpressions speculatively while variables are still unbound.

The monomorphizer's `check_checked_cast` performs a similar (stricter) check for
`CheckedCast`s it encounters structurally, but array/string lengths never reach it: length
types are resolved directly via `evaluate_to_u32`, so the evaluation rules above are the
mechanism that catches intermediate over/underflow in lengths. For the same reason,
`convert_type`/`check_type` do not recurse structurally into `from`: evaluation already
traverses it (including nested `CheckedCast`s introduced by generic substitution), and a
structural `check_type` on `from` could falsely reject unbound variables that were legitimately
simplified away.
