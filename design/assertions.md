# Constraints known to be false at compile time

When ACIR generation reaches a constraint whose two sides are both compile-time constants and are
not equal (e.g. a user `assert(0 == 1)`, or a compiler-inserted check such as an integer overflow,
division by zero, or out-of-bounds index whose operands fold to constants), the constraint can
never be satisfied: the circuit is guaranteed to fail for every input.

Such a constraint aborts compilation with a hard error (`RuntimeError::ConstraintIsAlwaysFalse`)
that carries the user's assertion message (when it is a static string) and the source-level call
stack. This matches the behavior of `nargo execute` and `std::static_assert`, and prevents `nargo
compile` from emitting an artifact containing an unsatisfiable `AssertZero` opcode — which would
otherwise surface to the user only as a backend opcode-level error with no source location.

Previously this was reported as a non-fatal `bug` warning and compilation succeeded anyway. The
warning path is still used in one case (see below).

## Exception: test and fuzzing harnesses

When compiling a test (`nargo test`) or a fuzzing harness, an always-false constraint is *not* a
hard error: it falls back to the non-fatal `bug` warning so the program still compiles and the
assertion fails at runtime instead. A test may deliberately trigger an always-failing assertion —
for example a `#[test(should_fail_with = "...")]` test that asserts a comptime-known-false
condition — and needs to observe that runtime failure rather than fail to compile.

This is controlled by `CompileOptions::allow_constant_false_assertions`, which the test and fuzzing
entry points (`run_test` / `fuzz_test`) set, and which maps to
`SsaEvaluatorOptions::fail_on_false_constraint` via `CompileOptions::as_ssa_options`.

## Scope

Only constant-false *constraints* abort compilation. Other non-fatal `bug` reports — the
underconstrained-values check (`InternalBug::IndependentSubgraph`) and the missing-Brillig-call
constraints check (`InternalBug::UncheckedBrilligCall`) — remain warnings and do not block
compilation.
