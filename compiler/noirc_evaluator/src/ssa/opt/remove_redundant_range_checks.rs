//! This SSA pass removes `range_check` instructions and unsigned `lt` comparisons against
//! a constant that are provably implied by an earlier, dominating fact about the same value,
//! as requested in [issue #9463](https://github.com/noir-lang/noir/issues/9463).
//!
//! The pass keeps a map from each value to the smallest *exclusive* upper bound proven so far
//! (`value < bound`, as an integer). Bounds are learned from three sources:
//!
//! - `range_check value to N bits` proves `value < 2^N`.
//! - `constrain (lt value, c) == u1 1` (for an unsigned `value` and constant `c`)
//!   proves `value < c`.
//! - `result = mod value, c` (for an unsigned result and constant `c != 0`)
//!   proves `result < c`.
//!
//! and consumed in two ways:
//!
//! - a `range_check value to N bits` that is implied by a dominating `range_check` of the
//!   same value to at most `N` bits is removed. Only `range_check`-derived facts may elide
//!   a `range_check`: the narrowing-cast validation rule (see `validate_narrowing_cast_invariant`)
//!   justifies a narrowing cast by the range checks that remain in the SSA, so the removed
//!   check's justification must itself stay visible as a `range_check` on the same value;
//! - an unsigned `v = lt value, c` with `known <= c` (from any of the three fact sources)
//!   is replaced by `u1 1`. If `v` only feeds a `constrain v == u1 1`, that constrain
//!   becomes trivial and is removed when it is re-simplified during the same traversal.
//!
//! ## Soundness
//!
//! - **Dominance**: a fact may only justify eliding an instruction it dominates. This pass
//!   uses the same discipline as [`remove_truncate_after_range_check`][super::remove_truncate_after_range_check]:
//!   blocks are traversed in reverse post-order and the bound map is cleared whenever the
//!   traversal moves to a block that is not dominated by the previously visited one, so every
//!   fact in the map dominates the instruction currently being visited.
//! - **Direction**: a bound only implies *weaker* (larger-or-equal) bounds: `x < 16` implies
//!   `x < 17`, never the reverse. Strictly tighter checks are kept (and tighten the map).
//! - **Predication**: `constrain` and `range_check` are enforced unconditionally regardless of
//!   the current `enable_side_effects` condition (flattening bakes the predicate into their
//!   operands, and ACIR generation emits them with a constant-true predicate), so bounds
//!   learned from them are unconditional facts at their program point. `mod` however is
//!   predicated during ACIR generation — under a false predicate the remainder witness is not
//!   constrained to be below the divisor — so a `mod`-derived bound is only recorded while the
//!   side-effects condition is the constant `true`.
//! - **No type-derived facts**: the pass never derives a bound from a value's type. A
//!   `range_check` can itself be the instruction that *establishes* a value's type invariant,
//!   so assuming the invariant could remove the very check that enforces it. (Checks that are
//!   implied by a value's statically-known bit width are already removed when instructions are
//!   simplified on insertion, via `get_value_max_num_bits`.) For the same reason there is no
//!   circularity here: a fact is learned only when its *enforcing* instruction is visited, so
//!   it can only elide strictly later instructions, never the instruction that enforces it.
//! - **Failure ordering**: eliding an implied check never changes which assertion fails first.
//!   The implying instruction is enforced earlier on every path (dominance): if a witness
//!   violates the elided (weaker) check it also violates the dominating (stronger) fact, which
//!   fails first — exactly as it did before the elision. If the dominating fact holds, the
//!   elided check could never have failed. Executions that don't fail are unchanged.
//! - **Signedness**: `lt` on signed operands orders by signed value, not by the underlying
//!   representation the bound map tracks, so signed comparisons are neither learned from nor
//!   elided.

use acvm::AcirField;
use num_bigint::BigUint;
use rustc_hash::FxHashMap as HashMap;

use crate::ssa::{
    ir::{
        cfg::ControlFlowGraph,
        dfg::DataFlowGraph,
        dom::DominatorTree,
        function::Function,
        instruction::{Binary, BinaryOp, Instruction},
        post_order::PostOrder,
        types::NumericType,
        value::{Value, ValueId},
    },
    ssa_gen::Ssa,
};

impl Ssa {
    /// Removes `range_check` instructions and unsigned `lt`-against-constant comparisons
    /// that are implied by an earlier, dominating bound on the same value.
    ///
    /// See the [`remove_redundant_range_checks`][self] module documentation for more details.
    pub(crate) fn remove_redundant_range_checks(mut self) -> Self {
        for function in self.functions.values_mut() {
            function.remove_redundant_range_checks();
        }
        self
    }
}

impl Function {
    fn remove_redundant_range_checks(&mut self) {
        let cfg = ControlFlowGraph::with_function(self);
        let post_order = PostOrder::with_cfg(&cfg);
        let dom_tree = DominatorTree::with_cfg_and_post_order(&cfg, &post_order);

        // Keeps the smallest exclusive upper bound proven for each value by a
        // dominating instruction: `value < bound`.
        let mut bounds: HashMap<ValueId, BigUint> = HashMap::default();
        // Keeps the smallest bit size each value was `range_check`ed against by a
        // dominating `range_check` specifically. Only these facts may elide another
        // `range_check`: the narrowing-cast validation rule justifies casts by the
        // range checks that remain in the SSA, so a removed check's justification
        // must itself remain visible as a (tighter) `range_check` on the same value.
        let mut range_checked_bits: HashMap<ValueId, u32> = HashMap::default();
        let mut previous_block = None;
        self.simple_optimization(|context| {
            if previous_block != Some(context.block_id) {
                // Clear the bounds when the new block is not dominated by the previous one
                if let Some(prev) = previous_block
                    && !dom_tree.dominates(prev, context.block_id)
                {
                    bounds.clear();
                    range_checked_bits.clear();
                }
                previous_block = Some(context.block_id);
            }
            match context.instruction() {
                Instruction::RangeCheck { value, max_bit_size, .. } => {
                    let (value, max_bit_size) = (*value, *max_bit_size);
                    if range_checked_bits.get(&value).is_some_and(|known| *known <= max_bit_size) {
                        // A dominating, still-present range check already guarantees this
                        // check passes (and keeps justifying any later narrowing cast).
                        context.remove_current_instruction();
                    } else {
                        range_checked_bits
                            .entry(value)
                            .and_modify(|current| *current = max_bit_size.min(*current))
                            .or_insert(max_bit_size);
                        insert_bound(&mut bounds, value, BigUint::from(1u8) << max_bit_size);
                    }
                }
                Instruction::Constrain(lhs, rhs, _) => {
                    if let Some((value, bound)) = constrained_lt_bound(context.dfg, *lhs, *rhs) {
                        insert_bound(&mut bounds, value, bound);
                    }
                }
                Instruction::Binary(Binary { lhs, rhs, operator: BinaryOp::Lt }) => {
                    let (lhs, rhs) = (*lhs, *rhs);
                    if is_unsigned(context.dfg, lhs)
                        && let Some(check_bound) = constant_as_biguint(context.dfg, rhs)
                        && bounds.get(&lhs).is_some_and(|known| *known <= check_bound)
                    {
                        // A dominating fact already guarantees this comparison is true.
                        let [result] = context.dfg.instruction_result(context.instruction_id);
                        let one = context
                            .dfg
                            .make_constant(acvm::FieldElement::one(), NumericType::bool());
                        context.replace_value(result, one);
                        context.remove_current_instruction();
                    }
                }
                Instruction::Binary(Binary { lhs: _, rhs, operator: BinaryOp::Mod }) => {
                    // `result = value mod c` guarantees `result < c`, but only while side
                    // effects are unconditionally enabled: ACIR generation predicates the
                    // division, leaving the remainder unconstrained under a false predicate.
                    let rhs = *rhs;
                    let [result] = context.dfg.instruction_result(context.instruction_id);
                    if is_unsigned(context.dfg, result)
                        && context
                            .dfg
                            .get_numeric_constant(context.enable_side_effects)
                            .is_some_and(|condition| condition.is_one())
                        && let Some(divisor) = context.dfg.get_numeric_constant(rhs)
                        && !divisor.is_zero()
                    {
                        let bound = BigUint::from_bytes_be(&divisor.to_be_bytes());
                        insert_bound(&mut bounds, result, bound);
                    }
                }
                _ => (),
            }
        });
    }
}

/// Records `value < bound`, keeping the smallest known bound for the value.
fn insert_bound(bounds: &mut HashMap<ValueId, BigUint>, value: ValueId, bound: BigUint) {
    bounds
        .entry(value)
        .and_modify(|current| {
            if bound < *current {
                current.clone_from(&bound);
            }
        })
        .or_insert(bound);
}

/// If `constrain lhs == rhs` asserts that an unsigned less-than against a constant is true
/// (`constrain (lt value, c) == u1 1`, in either operand order), returns `(value, c)`:
/// the constraint proves `value < c`.
fn constrained_lt_bound(
    dfg: &DataFlowGraph,
    lhs: ValueId,
    rhs: ValueId,
) -> Option<(ValueId, BigUint)> {
    lt_true_bound(dfg, lhs, rhs).or_else(|| lt_true_bound(dfg, rhs, lhs))
}

/// Helper for [`constrained_lt_bound`]: matches one operand order.
fn lt_true_bound(
    dfg: &DataFlowGraph,
    comparison: ValueId,
    constant: ValueId,
) -> Option<(ValueId, BigUint)> {
    if !dfg.get_numeric_constant(constant).is_some_and(|constant| constant.is_one()) {
        return None;
    }
    let Value::Instruction { instruction, .. } = &dfg[comparison] else {
        return None;
    };
    let Instruction::Binary(Binary { lhs: value, rhs, operator: BinaryOp::Lt }) =
        dfg[*instruction]
    else {
        return None;
    };
    if !is_unsigned(dfg, value) {
        return None;
    }
    let bound = constant_as_biguint(dfg, rhs)?;
    Some((value, bound))
}

/// Returns whether the value has an unsigned numeric type.
fn is_unsigned(dfg: &DataFlowGraph, value: ValueId) -> bool {
    dfg.type_of_value(value).is_unsigned()
}

/// Returns the value's numeric constant as an integer, if it is one.
fn constant_as_biguint(dfg: &DataFlowGraph, value: ValueId) -> Option<BigUint> {
    let constant = dfg.get_numeric_constant(value)?;
    Some(BigUint::from_bytes_be(&constant.to_be_bytes()))
}

#[cfg(test)]
mod tests {
    use crate::{
        assert_ssa_snapshot,
        ssa::{opt::assert_ssa_does_not_change, ssa_gen::Ssa},
    };

    #[test]
    fn removes_lt_implied_by_dominating_lt_constraint() {
        // The example from issue #9463: `assert(a < 16); assert(a < 17)`.
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u64):
            v2 = lt v0, u64 16
            constrain v2 == u1 1
            v5 = lt v0, u64 17
            constrain v5 == u1 1
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_redundant_range_checks();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u64):
            v2 = lt v0, u64 16
            constrain v2 == u1 1
            return
        }
        ");
    }

    #[test]
    fn removes_duplicate_lt_constraint_but_keeps_the_enforcing_one() {
        // The bound learned from the first `constrain` must never remove that same
        // constrain (it is learned strictly after the `lt` it constrains), only later
        // duplicates.
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u64):
            v2 = lt v0, u64 16
            constrain v2 == u1 1
            v4 = lt v0, u64 16
            constrain v4 == u1 1
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_redundant_range_checks();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u64):
            v2 = lt v0, u64 16
            constrain v2 == u1 1
            return
        }
        ");
    }

    #[test]
    fn removes_implied_lt_in_dominated_block_and_handles_replaced_values() {
        // The elision applies across dominated blocks, and later instructions using the
        // replaced comparison results are re-simplified (both eliminated pairs here, one
        // of which chains after the other's replacement).
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u64):
            v2 = lt v0, u64 16
            constrain v2 == u1 1
            v5 = lt v0, u64 17
            constrain v5 == u1 1
            jmp b1()
          b1():
            v7 = lt v0, u64 18
            constrain v7 == u1 1
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_redundant_range_checks();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u64):
            v2 = lt v0, u64 16
            constrain v2 == u1 1
            jmp b1()
          b1():
            return
        }
        ");
    }

    #[test]
    fn does_not_remove_lt_with_tighter_bound() {
        // Wrong direction: `a < 16` does not imply `a < 15`.
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u64):
            v2 = lt v0, u64 16
            constrain v2 == u1 1
            v4 = lt v0, u64 15
            constrain v4 == u1 1
            return
        }
        ";
        assert_ssa_does_not_change(src, Ssa::remove_redundant_range_checks);
    }

    #[test]
    fn does_not_remove_lt_implied_by_non_dominating_constraint() {
        // The constraint in b1 does not dominate the check in b3: on the b2 path
        // nothing bounds v0.
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u64, v1: u1):
            jmpif v1 then: b1(), else: b2()
          b1():
            v3 = lt v0, u64 16
            constrain v3 == u1 1
            jmp b3()
          b2():
            jmp b3()
          b3():
            v5 = lt v0, u64 17
            constrain v5 == u1 1
            return
        }
        ";
        assert_ssa_does_not_change(src, Ssa::remove_redundant_range_checks);
    }

    #[test]
    fn does_not_remove_signed_lt() {
        // Signed comparisons order by signed value, not by the representation the
        // bound map tracks, so they are neither learned from nor elided.
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: i64):
            v2 = lt v0, i64 16
            constrain v2 == u1 1
            v4 = lt v0, i64 17
            constrain v4 == u1 1
            return
        }
        ";
        assert_ssa_does_not_change(src, Ssa::remove_redundant_range_checks);
    }

    #[test]
    fn removes_range_check_implied_by_dominating_range_check() {
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: Field):
            range_check v0 to 8 bits
            jmp b1()
          b1():
            range_check v0 to 16 bits
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_redundant_range_checks();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: Field):
            range_check v0 to 8 bits
            jmp b1()
          b1():
            return
        }
        ");
    }

    #[test]
    fn does_not_remove_range_check_implied_by_lt_constraint() {
        // `v0 < 256` implies the 8-bit range check, but a `range_check` may only be
        // elided by another `range_check`: the narrowing-cast validation rule justifies
        // casts by the range checks remaining in the SSA (here, the cast to u8).
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u32):
            v2 = lt v0, u32 256
            constrain v2 == u1 1
            range_check v0 to 8 bits
            v3 = cast v0 as u8
            return v3
        }
        ";
        assert_ssa_does_not_change(src, Ssa::remove_redundant_range_checks);
    }

    #[test]
    fn does_not_remove_range_check_with_tighter_bound() {
        // Wrong direction: `v0 < 2^16` does not imply `v0 < 2^8`.
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: Field):
            range_check v0 to 16 bits
            range_check v0 to 8 bits
            return
        }
        ";
        assert_ssa_does_not_change(src, Ssa::remove_redundant_range_checks);
    }

    #[test]
    fn removes_lt_implied_by_dominating_range_check() {
        // `v0 < 2^8` implies `v0 < 256`.
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u32):
            range_check v0 to 8 bits
            v2 = lt v0, u32 256
            constrain v2 == u1 1
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_redundant_range_checks();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u32):
            range_check v0 to 8 bits
            return
        }
        ");
    }

    #[test]
    fn removes_lt_implied_by_dominating_mod() {
        // The array-indexing example from issue #9463: the out-of-bounds check on
        // `t[a % 3]` is implied by the `mod`.
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u32):
            v2 = mod v0, u32 3
            v4 = lt v2, u32 3
            constrain v4 == u1 1, "Index out of bounds"
            return v2
        }
        "#;

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_redundant_range_checks();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u32):
            v2 = mod v0, u32 3
            return v2
        }
        ");
    }

    #[test]
    fn does_not_learn_mod_bound_under_non_constant_predicate() {
        // Under a non-constant side-effects condition the division is predicated during
        // ACIR generation, so the remainder is not guaranteed to be below the divisor.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32, v1: u1):
            enable_side_effects v1
            v3 = mod v0, u32 3
            v5 = lt v3, u32 3
            constrain v5 == u1 1
            return v3
        }
        ";
        assert_ssa_does_not_change(src, Ssa::remove_redundant_range_checks);
    }

    #[test]
    fn removes_lt_implied_by_constraint_under_non_constant_predicate() {
        // Unlike `mod`, `constrain` is enforced regardless of the side-effects condition
        // (its predication is baked into its operands during flattening), so its bound
        // remains a sound fact under a non-constant condition.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u64, v1: u1):
            enable_side_effects v1
            v3 = lt v0, u64 16
            constrain v3 == u1 1
            v6 = lt v0, u64 17
            constrain v6 == u1 1
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_redundant_range_checks();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u64, v1: u1):
            enable_side_effects v1
            v3 = lt v0, u64 16
            constrain v3 == u1 1
            return
        }
        ");
    }
}
