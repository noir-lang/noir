# SSA→ACIR codegen coverage checklist

Tracks which cases of the actual SSA→ACIR lowering
(`compiler/noirc_evaluator/src/acir/mod.rs`'s `convert_ssa_instruction_inner`,
and its helpers in `acir/arrays.rs`, `acir/acir_context/`, `acir/call/`) have
an SMT equivalence test in `smt_verify`, via `ssa_acir.rs`'s comparator.

A row here is not one checkbox per `Instruction`/`TerminatorInstruction`
variant — it's one checkbox per *case the codegen itself treats
differently*. The same instruction often produces structurally different
ACIR depending on operand type, whether operands are compile-time-constant,
whether a nonzero `EnableSideEffectsIf` predicate is active, or other
dimensions found by reading the actual match arms; each such case gets its
own row so passing coverage can't hide an untested branch.

Check an item off only once a `smt_verify` test compiles real SSA through
`Ssa::to_brillig`/`Ssa::into_acir` and checks it against the ACIR that came
out, for that specific case.

## Binary (`acir/mod.rs:501` dispatch → `convert_ssa_binary`, `acir/mod.rs:782`)

Predicate requirement is decided by `requires_acir_gen_predicate`
(`ssa/ir/instruction.rs:553`, per-operator at `:998`) before the operator
dispatch below runs.

- [x] `Add`/`Sub`/`Mul`, `Field` — tested in `add_compiles_to_equivalent_acir`
- [ ] `Add`/`Sub`/`Mul`, `Unsigned` (checked: overflow range check emitted, `check_unsigned_overflow`, `acir/mod.rs:843`)
- [ ] `Add`/`Sub`/`Mul`, `Unsigned` (`unchecked_*`: no overflow check, no predicate)
- [ ] `Add`/`Sub`/`Mul`, `Signed` (checked: overflow check)
- [ ] `Add`/`Sub`/`Mul`, `Signed` (`unchecked_*`)
- [ ] `Div`, `Field` (multiply by inverse)
- [ ] `Div`, `Unsigned` (euclidean division, always predicated)
- [ ] `Mod`, `Unsigned` (always predicated; panics for any other type)
- [ ] `Eq` (constant-difference fold vs symbolic path, `acir_context/mod.rs:358`)
- [ ] `Lt`, `Unsigned` (panics for any other type)
- [ ] `And`/`Or`/`Xor`, bit_size == 1 (boolean arithmetic identity)
- [ ] `And`/`Xor`, bit_size > 1 (`BlackBoxFunc` opcode)
- [ ] `Or`, bit_size > 1 (built from `And`+`Xor` via De Morgan's)
- [ ] `Shl`/`Shr` with a compile-time-constant, provably-safe shift amount (must already be lowered to `Mul`/`Div` before ACIR gen — confirm no `Shl`/`Shr` reaches this match arm)

## `Cast` (`acir/mod.rs:522`)

Pure reinterpretation (`convert_numeric_value` + relabel `NumericType`) —
does not itself branch on source/target type. Actual truncation must arrive
as a separate `Truncate`/`RangeCheck` instruction.

- [ ] `Cast` to `Field`
- [ ] `Cast` to `Unsigned`
- [ ] `Cast` to `Signed`

## `Not` (`acir/mod.rs:531` → `not_var`, `acir_context/mod.rs:766`)

- [ ] `Not`, `u1` (boolean complement)
- [ ] `Not`, wider `Unsigned` (`2^bit_size - 1 - x`)

## `Truncate` (`acir/mod.rs:538` → `convert_ssa_truncate`/`truncate_var`, `acir_context/mod.rs:1165`)

Always implemented via `euclidean_division_var`
(`acir_context/mod.rs:780`), which itself branches on constant vs symbolic
operands/predicate.

- [ ] `Truncate` of a binary op's result, symbolic operand
- [ ] `Truncate` of a compile-time-constant value

## `Constrain` (`acir/mod.rs:511` → `assert_eq_var`, `acir_context/mod.rs:508`)

- [ ] `constrain` with a symbolic, potentially-unequal difference (`AssertZero` emitted)
- [ ] `constrain` where the difference folds to the constant `0` (no-op, no opcode emitted)
- [ ] `constrain` where the difference folds to a nonzero constant (always-failing `AssertZero` + `InternalBug::AssertFailed` warning)

## `ConstrainNotEqual` (`acir/mod.rs:516` → `assert_neq_var`, `acir_context/mod.rs:561`)

- [ ] `constrain_not_equal`, symbolic (witness-inverse construction, reads the active predicate)
- [ ] `constrain_not_equal` where the fully-constant case emits no opcode (payload skipped, `acir_context/mod.rs:581`)

## `EnableSideEffectsIf` (`acir/mod.rs:543`)

- [ ] Sets the active predicate; a later predicated instruction (e.g. `RangeCheck`, `ArrayGet`/`ArraySet`, `Div`/`Mod`) observes it correctly

## `RangeCheck` (`acir/mod.rs:566` → `range_constrain_var`, `acir_context/mod.rs:1126`)

- [ ] Symbolic value (predicate-multiplied `RangeConstraint` opcode)
- [ ] Compile-time-constant value that fits `max_bit_size` (no-op)
- [ ] Compile-time-constant value that does *not* fit (forced-zero, constraint still emitted unless predicate is statically `0`)

## `ArrayGet`/`ArraySet` (`acir/mod.rs:548` → `handle_array_operation`, `acir/arrays.rs:222`)

The richest instruction — each bullet is a distinct code path in `arrays.rs`.

- [ ] Zero-length array/vector (`handle_zero_length_array`, `arrays.rs:340`)
- [ ] Constant index into a (non-dynamic) `AcirValue::Array`, unconditional (compile-time write/read, `handle_constant_index`, `arrays.rs:373`)
- [ ] Constant index, in-bounds, under an active (non-constant-1) predicate — blended `predicate*value + (1-predicate)*dummy` (`arrays.rs:478`)
- [ ] Constant index into a value containing a nested `DynamicArray` (falls through to the runtime path, `arrays.rs:487`)
- [ ] Statically-disabled operation: predicate is constant `0` and index isn't provably safe — dummy read / no-op write (`handle_disabled_array_operation`, `arrays.rs:293`)
- [ ] Dynamic (non-constant) index, homogeneous element layout (`get_flattened_index`, constant-step multiply, `arrays.rs:1277`)
- [ ] Dynamic index, non-homogeneous layout — nested array / array-of-structs (element-type-size table, `arrays.rs:1277`)
- [ ] Dynamic index under `IndexGating::Gated` (predicate-multiplied + fallback-offset bias for OOB safety, vs `Safe`)
- [ ] `ArraySet`, in-place mutation (`mutate_array`, `resolve_array_set_block`, `arrays.rs:911`)
- [ ] `ArraySet`, copy-into-new-block (aliased array)
- [ ] `ArraySet` with zero flattened width (aliasing shortcut, `arrays.rs:878`)
- [ ] `Type::Vector` (dynamic length, tracked separately from capacity — see `call/mod.rs:188`)

## `Allocate`/`Store`/`Load`/`IncrementRc`/`DecrementRc`

Not reachable ACIR codegen cases under normal compilation: `mem2reg` runs
after `flatten_cfg` and before ACIR gen (`ssa/mod.rs:317`, `:327`), removing
`Store`/`Load`/Rc instructions; `Store`/`Load`/`IncrementRc`/`DecrementRc`
`unreachable!` if one somehow remains (`acir/mod.rs:552`-`563`). `Allocate`
alone returns a `RuntimeError::UnknownReference` rather than panicking,
since a reference can legitimately still escape (e.g. an unresolved
global) — this is the one case in this group actually worth an
`smt_verify` test, checking the comparator handles the `Result::Err` case
rather than assuming compilation always succeeds.

- [ ] `Allocate` reaching ACIR gen (unresolved reference) → `RuntimeError`, not a panic

## `IfElse`

- Not reachable: must be eliminated by `flatten_cfg`/simplification before ACIR gen (`acir/mod.rs:579` `unreachable!`). Nothing to test here directly — coverage instead comes from testing SSA source that *would* contain `IfElse` before flattening, once whole-pipeline (not just `to_brillig`/`into_acir`) compilation is in scope.

## `MakeArray` (`acir/mod.rs:582`)

- [ ] `MakeArray` of a symbolic-element array (pure value construction; no memory opcode emitted here — deferred to the first `ArrayGet`/`ArraySet`)

## `Noop`

- [ ] Present in the instruction stream and correctly ignored (degenerate case, low priority)

## `Call`, user-defined ACIR function (`acir/call/mod.rs:44`, `:93`)

- [ ] Call to another ACIR function, non-entry-point (must already be inlined — confirm still true, i.e. this path is unreachable in-scope for now)
- [ ] Call to another ACIR function that *is* an entry point (`handle_acir_function_call` emits an ACIR `Call` opcode) — out of scope until multi-function comparison is supported (see `ssa_acir.rs`'s current single-function assumption)

## `Call`, Brillig function (`acir/call/mod.rs:133`, `acir_context/brillig_call.rs`)

- [ ] Brillig call, symbolic predicate (unconstrained outputs, no defining equation — see `acir/mod.rs`'s free-witness modeling for `BrilligCall`)
- [ ] Brillig call under a statically-`0` predicate (short-circuits to zeroed outputs, `brillig_call.rs:48`)
- [ ] Brillig call, bytecode cache hit vs miss (should be behaviorally identical — low priority to test both, but worth confirming)

## `Call`, intrinsics (`call/intrinsics/mod.rs:24`)

- [ ] `BlackBox` intrinsic (per-`BlackBoxFunc` opcode shape, `acir_context/black_box.rs:16`) — one row per black-box function as coverage grows
- [ ] `ToRadix`/`ToBits`
- [ ] Vector ops: `AsVector`/`VectorPushBack`/`VectorPushFront`/`VectorPopBack`/`VectorPopFront`/`VectorInsert`/`VectorRemove` (each a distinct predicated-memory-op path)
- [ ] `AsWitness`
- [ ] `FieldLessThan`
- [ ] Compile-time/no-op group (`IsUnconstrained`, `ArrayLen`, `ArrayRefCount`, ...)

## Terminators

ACIR-targeted SSA is fully flattened to a single block before codegen
(`flatten_cfg`, `ssa/opt/flatten_cfg.rs:29`); `convert_acir_main`
(`acir/mod.rs:264`) only ever looks at the entry block's terminator.

- [x] `Return` — exercised by every `ssa_acir.rs` test so far
- [ ] `Unreachable`, no preceding always-failing constrain (emits a trap `AssertZero`, `acir/mod.rs:667`)
- [ ] `Unreachable`, block already ends with an always-failing constrain (deduped, no extra trap opcode — `block_ends_with_always_failing_constraint`)
- `Jmp`/`JmpIf` are `unreachable!` at this point (`acir/mod.rs:664`) — nothing to test; nonlinear control flow no longer exists once flattening has run
