# `simplify` rule coverage checklist

Tracks which of the compiler's actual `simplify` rewrite rules
(`compiler/noirc_evaluator/src/ssa/ir/dfg/simplify.rs` and its submodules)
have an SMT equivalence test in `smt_verify`, via
`assert_simplify_preserves_behavior` (`compiler/noirc_evaluator/src/ssa/smt_verify/mod.rs`).

Check an item off only once a `smt_verify` test exercises that specific rule
— not just the instruction it belongs to. Each item links to its source
location. This intentionally does not track whole SSA optimization passes
(mem2reg, constant folding, DIE, ...) — that's future scope, tracked
separately if/when it starts.

## Binary operators (`ir/dfg/simplify/binary.rs`)

**Operator normalization** (applied unconditionally before the per-operator
rules below, and before constant folding — discovered while testing the
`Mul` boolean rules, not originally catalogued here)
- [ ] Field: `unchecked_add`/`unchecked_sub`/`unchecked_mul` → checked (unchecked is meaningless for `Field`, kept only to reduce SSA noise)
- [x] boolean: checked `mul` → `unchecked_mul` (multiplying two `{0,1}` values can never overflow) — tested as a side effect of `mul_boolean_b_times_bx_holds`, `mul_boolean_bx_times_b_holds`

- [ ] Constant folding: both operands constant → evaluate directly (`eval_constant_binary_op`, applies to every operator below)

**Add**
- [x] `x + 0 → x` / `0 + x → x` (guarded by `can_simplify_arithmetic_identity`) — two separate branches in the source, tested separately: `add_zero_lhs_holds_for_all_field_elements`, `add_zero_rhs_holds_for_all_field_elements`

**Sub**
- [x] `x - x → 0` — tested in `sub_self_is_zero_holds_for_all_field_elements`
- [x] `x - 0 → x` (guarded by `can_simplify_arithmetic_identity`) — tested in `sub_zero_rhs_holds_for_all_field_elements`

**Mul**
- [x] `x * 1 → x` / `1 * x → x` (guarded by `can_simplify_arithmetic_identity`) — two separate branches, tested separately: `mul_one_lhs_holds_for_all_field_elements`, `mul_one_rhs_holds_for_all_field_elements`
- [x] `x * 0 → 0` / `0 * x → 0` — one shared branch, tested in `mul_zero_holds_for_all_field_elements`
- [x] `b * b → b` when `b` is boolean — tested in `mul_boolean_square_holds`
- [x] `b * (b * x) → b * x` when `b` is boolean — tested in `mul_boolean_b_times_bx_holds`
- [x] `(b * x) * b → b * x` when `b` is boolean — tested in `mul_boolean_bx_times_b_holds`

**Div**
- [ ] `x / 1 → x`
- [ ] `Field x / c → x * c⁻¹` for constant `c ≠ 0`

**Mod**
- [ ] `x % 1 → 0`
- [ ] unsigned `x % c → truncate x` when `c` is a power of two

**Eq**
- [ ] `x == x → true`
- [x] boolean `(b == true) → b`, `(true == b) → b` — tested in `eq_boolean_true_rhs_holds`, `eq_boolean_true_lhs_holds`
- [x] boolean `(b == false) → !b`, `(false == b) → !b` — tested in `eq_boolean_false_rhs_holds`, `eq_boolean_false_lhs_holds`

**Lt**
- [ ] `x < x → false`
- [ ] unsigned `x < 0 → false`
- [ ] unsigned `x < 1 → x == 0`
- [ ] unsigned ACIR `0 < x → !(x == 0)`

**And**
- [ ] `x & 0 → 0` / `0 & x → 0`
- [ ] `x & x → x`
- [ ] unsigned bitwise-AND-with-power-of-two-minus-one-mask → `Truncate`
- [x] boolean `x & y → x * y` (unchecked mul) — tested in `and_boolean_is_unchecked_mul_holds`

**Or**
- [ ] `x | 0 → x` / `0 | x → x`
- [ ] boolean, either operand `1` → `1`
- [ ] `x | x → x`
- [ ] unsigned, either operand is the type's max value → max value

**Xor**
- [ ] `x ^ 0 → x` / `0 ^ x → x`
- [ ] `x ^ x → 0`

**Shl / Shr**
- [ ] `x << 0 → x` / `x >> 0 → x`

## Other instructions (`ir/dfg/simplify.rs` top-level dispatch)

- [x] `Not`: constant boolean → constant — tested in `not_constant_holds`
- [x] `Not`: `!!x → x` — tested in `not_not_holds`
- [ ] `Constrain`: decomposed via `decompose_constrain` (see below)
- [ ] `ArrayGet`: constant out-of-bounds index → trap, collapsed to an in-bounds same-field index
- [ ] `ArrayGet`: constant index into a value built by a previous `MakeArray`/`ArraySet` → the stored value directly
- [ ] `ArrayGet`: length-1 array → assert index is zero, read the sole element
- [ ] `ArraySet`: constant out-of-bounds index → trap, forward the unmodified array
- [ ] `ArraySet`: redundant set immediately following a `get` at the same index → previous instructions reused
- [ ] `Truncate`: `bit_size >= max_bit_size → x` (no-op truncation)
- [ ] `Truncate`: constant value → truncated constant
- [ ] `Truncate`: truncating an already-truncated (smaller-or-equal) value → no-op
- [ ] `Truncate`: truncating the result of a constant unsigned division → no-op when the quotient's bound already fits
- [ ] `EnableSideEffectsIf`: consecutive redundant instance → remove the earlier one
- [ ] `RangeCheck`: value's max possible bit count already fits → remove
- [ ] `RangeCheck`: `max_bit_size == 0` → constrain the value to zero
- [ ] `RangeCheck`: constant value that doesn't fit → constrain `false` (always fails)
- [ ] `IfElse`: constant condition → the corresponding branch's value
- [ ] `IfElse`: `then_value == else_value` → that value
- [ ] `IfElse`: nested `IfElse` with the same `then_condition` on the then-branch → flattened
- [ ] `IfElse`: nested `IfElse` with the same `then_condition` on the else-branch → flattened
- [ ] `IfElse`: `else_value == then_condition` → `then_condition * then_value`
- [ ] `IfElse`: `then_value == else_condition` → `else_condition * else_value`
- [ ] `IfElse`: numeric type → merged via `ValueMerger::merge_numeric_values`
- [ ] `Noop` → removed

Always `None` (no simplification exists): `ConstrainNotEqual`, `Allocate`, `Load`, `Store`, `IncrementRc`, `DecrementRc`, `MakeArray`.

## `Cast` (`ir/dfg/simplify/cast.rs`)

- [ ] `cast x as T → x` when `x` is already of type `T`
- [ ] `cast (cast x as U) as T` → simplified/collapsed via the inner cast's own simplification
- [ ] constant, Unsigned/Signed → Field: reinterpreted as the same value
- [ ] constant, Field/Unsigned/Signed → Unsigned: truncated mod `2^bit_size`
- [ ] constant, Field/Unsigned/Signed → Signed: converted when representable in the destination type

## `Constrain` decomposition (`ir/dfg/simplify/constrain.rs`)

- [ ] `constrain x == x` → removed (trivially satisfied)
- [ ] `constrain (eq a b) == true` → `constrain a == b`
- [ ] `constrain (mul a b) == true` (`a`, `b` boolean) → `constrain a == true; constrain b == true`
- [ ] `constrain (or a b) == false` → `constrain a == false; constrain b == false`
- [ ] `constrain (not a) == true` → `constrain a == false` (and vice versa)
- [ ] `constrain (mul a a) == 0` → `constrain a == 0` (sound only when the multiplication can't wrap, see the function's own soundness note)
- [ ] `constrain (cast a) == constant` → `constrain a == constant` when the constant fits `a`'s original type
- [ ] `constrain (cast a) == (cast b)` → `constrain a == b` when both casts share a source type

## `Call` / intrinsics (`ir/dfg/simplify/call.rs`)

One line per `Intrinsic`; most only fire when their arguments are constant.

- [ ] `ToBits` — constant field → constant bit array
- [ ] `ToRadix` — constant field + constant radix → constant limb array
- [ ] `ArrayLen` — → constant length
- [ ] `ArrayAsStrUnchecked` — → its argument unchanged (same representation)
- [ ] `AsVector` — array → vector representation
- [ ] `VectorPushBack` / `VectorPushFront` / `VectorPopBack` / `VectorPopFront` / `VectorInsert` / `VectorRemove` — constant vector contents → constant result
- [ ] `StrAsBytes` — → its argument unchanged (same representation)
- [ ] `AssertConstant` — all arguments constant → removed
- [ ] `StaticAssert` — constant condition & message → removed if true
- [ ] `ApplyRangeConstraint` — → `RangeCheck` (or removed if already known to fit)
- [ ] `Hint(BlackBox)` — never simplified
- [ ] `BlackBox(_)` — see below
- [ ] `AsWitness` — never simplified
- [ ] `IsUnconstrained` — → constant (`true` in Brillig, `false` in ACIR)
- [ ] `DerivePedersenGenerators` — → constant generator array
- [ ] `FieldLessThan` — both constant → constant comparison result
- [ ] `ArrayRefCount` / `VectorRefCount` — ACIR → constant `0`

## Black-box functions (`ir/dfg/simplify/call/blackbox.rs`)

Constant-fold by invoking the real cryptographic implementation at compile
time when all inputs are constant; a `BlackBox` call over symbolic inputs is
left untouched, not simplified.

- [ ] `Blake2s` — via `simplify_hash`
- [ ] `Blake3` — via `simplify_hash`
- [ ] `Keccakf1600`
- [ ] `Poseidon2Permutation` — via `simplify_poseidon2_permutation`
- [ ] `EcdsaSecp256k1` / `EcdsaSecp256r1` — via `simplify_signature`
- [ ] `MultiScalarMul` — via `simplify_msm`
- [ ] `EmbeddedCurveAdd` — via `simplify_ec_add` (includes the point-at-infinity special case)
- [ ] `Sha256Compression`

Never simplified: `RecursiveAggregation`, `AES128Encrypt`. `AND`/`XOR`/`RANGE`
never reach this code (they're rewritten to `Binary`/`Cast` earlier in the
pipeline).
