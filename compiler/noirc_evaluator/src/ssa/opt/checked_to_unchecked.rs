//! This SSA pass will turn checked unsigned binary additions, subtractions and multiplications
//! into unchecked ones if it's guaranteed that the operations cannot overflow.
//!
//! Signed checked binary operations should have already been converted to unchecked ones with
//! an explicit overflow check during [`super::expand_signed_checks`].
//!
//! In ACIR functions this pass additionally elides the overflow check of an intermediate checked
//! `add` whose single-use result feeds a later checked `add` in the same block (issue #7161): over
//! the field an overflow of the intermediate forces the dominating add to overflow, so the later
//! range check rejects the same inputs. ACIR-only because Brillig wraps and traps per-op, so eliding
//! would move the failure point (mirrors `check_u128_mul_overflow`'s runtime split).
use acvm::AcirField as _;
use rustc_hash::FxHashMap as HashMap;
use rustc_hash::FxHashSet as HashSet;

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        dfg::DataFlowGraph,
        function::Function,
        instruction::{Binary, BinaryOp, Instruction},
        types::NumericType,
        value::{Value, ValueId},
    },
    ssa_gen::Ssa,
};

impl Ssa {
    /// See [`checked_to_unchecked`][self] module for more information.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn checked_to_unchecked(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            function.checked_to_unchecked();
        }
        self
    }
}

impl Function {
    fn checked_to_unchecked(&mut self) {
        #[cfg(debug_assertions)]
        checked_to_unchecked_pre_check(self);

        let mut value_max_num_bits = HashMap::<ValueId, u32>::default();

        // Results of intermediate checked adds whose overflow is dominated by a later checked add
        // in the same block (issue #7161). Empty for Brillig functions — see the module docs for
        // why the dominated-elision reasoning is ACIR-only.
        let dominated_add_checks = if self.runtime().is_acir() {
            self.dominated_intermediate_add_checks()
        } else {
            HashSet::default()
        };

        self.simple_optimization(|context| {
            let instruction = context.instruction();
            let Instruction::Binary(binary) = instruction else {
                return;
            };
            let lhs = binary.lhs;
            let rhs = binary.rhs;

            let lhs_type = context.dfg.type_of_value(lhs).unwrap_numeric();
            let NumericType::Unsigned { .. } = lhs_type else {
                return;
            };

            // The result of the current instruction (a single-result binary op).
            let result = context.dfg.instruction_results(context.instruction_id)[0];

            let dfg = &context.dfg;

            let unchecked = match binary.operator {
                BinaryOp::Add { unchecked: false } => {
                    let bit_size = dfg.type_of_value(lhs).bit_size();
                    let max_lhs_bits = required_bit_size(dfg, lhs, &mut value_max_num_bits);
                    let max_rhs_bits = required_bit_size(dfg, rhs, &mut value_max_num_bits);

                    // 1. If both lhs and rhs have less max bits than the result it means their
                    //    value is at most `2^(n-1) - 1`, assuming `n = bit_size`. Adding those
                    //    we get `2^(n-1) - 1 + 2^(n-1) - 1`, so `2*(2^(n-1)) - 2`,
                    //    so `2^n - 2` which fits in `0..2^n`.
                    // In that case, `lhs` and `rhs` have both been casted up from smaller types and so cannot overflow.
                    (max_lhs_bits < bit_size && max_rhs_bits < bit_size)
                        // Dominated-check elision (#7161, ACIR only): this add's overflow is
                        // caught by a later checked add in the same block that consumes its
                        // (single-use) result. See the module docs for the soundness argument.
                        || dominated_add_checks.contains(&result)
                }
                BinaryOp::Sub { unchecked: false } => {
                    // True when an unsigned subtraction `lhs - rhs` is guaranteed not to underflow.
                    //
                    // This is the case when `lhs` is a constant that is >= the maximum possible value of `rhs`
                    // (determined by its bit width). For example, `256 - (x as u32)` where `x: u8` cannot
                    // underflow because `256 >= 255`.

                    if let Some(lhs_const) = dfg.get_numeric_constant(lhs) {
                        let max_rhs_bits = required_bit_size(dfg, rhs, &mut value_max_num_bits);
                        let max_rhs =
                            if max_rhs_bits == 128 { u128::MAX } else { (1 << max_rhs_bits) - 1 };

                        // `lhs` is a fixed constant and `rhs` is restricted such that `lhs - rhs >= 0`.
                        // For example: `lhs` is 1 and `rhs` max bitsize is 1, so at most it's `1 - 1`.
                        // Another example: `lhs` is 255 and `rhs` max bitsize is 8, so at most it's `255 - 255`.
                        lhs_const >= max_rhs.into()
                    } else {
                        false
                    }
                }
                BinaryOp::Mul { unchecked: false } => {
                    let bit_size = dfg.type_of_value(lhs).bit_size();
                    let max_lhs_bits = required_bit_size(dfg, lhs, &mut value_max_num_bits);
                    let max_rhs_bits = required_bit_size(dfg, rhs, &mut value_max_num_bits);

                    // `required_bit_size` tracks the actual range of a value through casts,
                    // truncations, and boolean multiplications — it may be smaller than the
                    // type's bit_size (e.g. a u8 upcast to u64 still has max_bits == 8).
                    //
                    // The product of an `a`-bit value and a `b`-bit value needs at most
                    // `a + b` bits: `(2^a - 1) * (2^b - 1) < 2^(a+b)`. So if
                    // `max_lhs_bits + max_rhs_bits <= bit_size`, the result is guaranteed
                    // to fit and the multiplication cannot overflow.
                    //
                    // As a special case, when either operand has `max_bits == 1` its value
                    // is at most 1, so `x * 0 = 0` or `x * 1 = x` — neither can overflow.
                    // This is sound as long as `required_bit_size` never returns 1 for a
                    // value that could actually exceed 1.
                    max_lhs_bits + max_rhs_bits <= bit_size
                        || max_lhs_bits == 1
                        || max_rhs_bits == 1
                }
                _ => false,
            };
            if unchecked {
                let operator = binary.operator.into_unchecked();
                context.replace_current_instruction_with(Instruction::Binary(Binary {
                    lhs: binary.lhs,
                    rhs: binary.rhs,
                    operator,
                }));
            }
        });
    }
}

impl Function {
    /// Intermediate checked-`add` results whose overflow check is dominated by a later checked `add`
    /// and can be elided (issue #7161). ACIR-only; see the module docs.
    ///
    /// An intermediate add's result is elidable when it is used exactly once (so its un-range-checked
    /// value cannot leak), that use is an operand of a checked `add` in the same block under the same
    /// side-effects predicate (a checked range check is applied to `predicate · result`, so a
    /// differing predicate could make the dominating check vacuous), and it is unsigned. The consumer
    /// is restricted to `add` for monotonicity (`mul`'s other operand may be `0`, `sub` decreases).
    fn dominated_intermediate_add_checks(&self) -> HashSet<ValueId> {
        let dfg = &self.dfg;

        // Count uses of every value across all instructions and block terminators. A value is only
        // ever elidable if it is used exactly once, so this global count is the primary filter.
        let mut use_count = HashMap::<ValueId, u32>::default();
        for block_id in self.reachable_blocks() {
            let block = &dfg[block_id];
            for instruction_id in block.instructions() {
                dfg[*instruction_id].for_each_value(|value| {
                    *use_count.entry(value).or_default() += 1;
                });
            }
            if let Some(terminator) = block.terminator() {
                terminator.for_each_value(|value| {
                    *use_count.entry(value).or_default() += 1;
                });
            }
        }

        // For each checked-add result defined in a block, remember the (block, predicate epoch) at
        // which it was defined. The epoch is bumped by every `EnableSideEffectsIf`, so two ops share
        // an epoch iff no predicate change occurred between them within the block.
        let mut checked_add_def = HashMap::<ValueId, (BasicBlockId, u32)>::default();
        let mut elidable = HashSet::default();

        for block_id in self.reachable_blocks() {
            let block = &dfg[block_id];
            let mut epoch = 0_u32;
            for instruction_id in block.instructions() {
                let instruction = &dfg[*instruction_id];

                if matches!(instruction, Instruction::EnableSideEffectsIf { .. }) {
                    epoch += 1;
                    continue;
                }

                if let Instruction::Binary(Binary {
                    lhs,
                    rhs,
                    operator: BinaryOp::Add { unchecked: false },
                }) = instruction
                {
                    // If either operand is a checked-add result defined earlier in *this* block at
                    // the *same* predicate epoch, used exactly once (i.e. only here), and is
                    // unsigned, then its overflow is dominated by this add — elide its check.
                    for operand in [*lhs, *rhs] {
                        if use_count.get(&operand).copied() == Some(1)
                            && checked_add_def.get(&operand) == Some(&(block_id, epoch))
                            && matches!(
                                dfg.type_of_value(operand).unwrap_numeric(),
                                NumericType::Unsigned { .. }
                            )
                        {
                            elidable.insert(operand);
                        }
                    }

                    // Record this add's own result so a still-later add can dominate it in turn.
                    if let [result] = dfg.instruction_results(*instruction_id) {
                        checked_add_def.insert(*result, (block_id, epoch));
                    }
                }
            }
        }

        elidable
    }
}

/// Returns a maximum number of bits the `value` requires.
/// The logic here is almost the same as [`DataFlowGraph::get_value_max_num_bits`] except that
/// - it takes into account that the bitsize of multiplying two bools is 1
/// - it recurses by memoizing the results in `value_max_num_bits`
fn required_bit_size(
    dfg: &DataFlowGraph,
    value: ValueId,
    value_max_num_bits: &mut HashMap<ValueId, u32>,
) -> u32 {
    if let Some(bits) = value_max_num_bits.get(&value) {
        return *bits;
    }

    let value_bit_size = dfg.type_of_value(value).bit_size();

    let bits = match dfg[value] {
        Value::Instruction { instruction, .. } => {
            match dfg[instruction] {
                Instruction::Cast(original_value, _) => {
                    let original_bit_size =
                        required_bit_size(dfg, original_value, value_max_num_bits);
                    // We might have cast e.g. `u1` to `u8` to be able to do arithmetic,
                    // in which case we want to recover the original smaller bit size;
                    // OTOH if we cast down, then we don't need the higher original size.
                    value_bit_size.min(original_bit_size)
                }
                Instruction::Binary(Binary { lhs, operator: BinaryOp::Mul { .. }, rhs })
                    if required_bit_size(dfg, lhs, value_max_num_bits) == 1
                        && required_bit_size(dfg, rhs, value_max_num_bits) == 1 =>
                {
                    // When multiplying two values, if their bitsize is 1 then the result's bitsize will be 1 too
                    1
                }
                Instruction::Truncate { value, bit_size, .. } => {
                    let value_bit_size =
                        value_bit_size.min(required_bit_size(dfg, value, value_max_num_bits));
                    value_bit_size.min(bit_size)
                }
                _ => value_bit_size,
            }
        }
        Value::NumericConstant { constant, .. } => constant.num_bits(),
        _ => value_bit_size,
    };

    assert!(bits <= value_bit_size);
    value_max_num_bits.insert(value, bits);

    bits
}

/// Pre-check condition for [`Function::checked_to_unchecked`].
///
/// Panics if:
///   - The function contains any checked signed binary operations (add, sub, mul).
///   - These should have already been converted by the `expand_signed_checks` pass.
#[cfg(debug_assertions)]
fn checked_to_unchecked_pre_check(func: &Function) {
    // expand_signed_checks must have run
    super::checks::for_each_instruction(func, |instruction, dfg| {
        super::checks::assert_not_checked_signed_add_sub_mul(instruction, dfg);
    });
}

#[cfg(test)]
mod tests {
    use crate::{assert_ssa_snapshot, ssa::ssa_gen::Ssa};

    #[test]
    fn checked_to_unchecked_when_casting_two_u16_to_u32_then_adding() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u16, v1: u16):
            v2 = cast v0 as u32
            v3 = cast v1 as u32
            v4 = add v2, v3
            return v4
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.checked_to_unchecked();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u16, v1: u16):
            v2 = cast v0 as u32
            v3 = cast v1 as u32
            v4 = unchecked_add v2, v3
            return v4
        }
        ");
    }

    #[test]
    fn checked_to_unchecked_when_subtracting_u32() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u16):
            v1 = cast v0 as u32
            v2 = sub u32 65536, v1
            return v2
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.checked_to_unchecked();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u16):
            v1 = cast v0 as u32
            v3 = unchecked_sub u32 65536, v1
            return v3
        }
        ");
    }

    #[test]
    fn checked_to_unchecked_when_subtracting_from_1_a_value_that_has_1_bit() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u1):
            v1 = cast v0 as u32
            v3 = sub u32 1, v1
            return v3
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.checked_to_unchecked();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u1):
            v1 = cast v0 as u32
            v3 = unchecked_sub u32 1, v1
            return v3
        }
        ");
    }

    #[test]
    fn checked_to_unchecked_when_subtracting_from_255_a_value_that_has_8_bits() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u8):
            v1 = cast v0 as u32
            v3 = sub u32 255, v1
            return v3
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.checked_to_unchecked();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u8):
            v1 = cast v0 as u32
            v3 = unchecked_sub u32 255, v1
            return v3
        }
        ");
    }

    #[test]
    fn checked_to_unchecked_when_multiplying_bools() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u1, v1: u1):
            v2 = mul v0, v1
            return v2
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.checked_to_unchecked();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u1, v1: u1):
            v2 = unchecked_mul v0, v1
            return v2
        }
        ");
    }

    #[test]
    fn checked_to_unchecked_when_multiplying_upcasted_bool_with_u32() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u1, v1: u32):
            v2 = cast v0 as u32
            v3 = mul v2, v1
            return v2
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.checked_to_unchecked();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u1, v1: u32):
            v2 = cast v0 as u32
            v3 = unchecked_mul v2, v1
            return v2
        }
        ");
    }

    #[test]
    fn checked_to_unchecked_when_multiplying_two_upcasted_bools_to_u32_then_multiplying_again() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u1, v1: u1, v2: u32):
            v3 = cast v0 as u32
            v4 = cast v1 as u32
            v5 = mul v3, v4
            v6 = mul v2, v5
            return v6
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.checked_to_unchecked();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u1, v1: u1, v2: u32):
            v3 = cast v0 as u32
            v4 = cast v1 as u32
            v5 = unchecked_mul v3, v4
            v6 = unchecked_mul v2, v5
            return v6
        }
        ");
    }

    // ----- #7161: overflow-check elision dominated by a later checked add (ACIR only) -----

    #[test]
    fn elides_intermediate_add_check_dominated_by_later_add() {
        // The intermediate `v2 = add v0, 1`'s overflow is dominated by the later checked
        // `v3 = add v2, 1`: over the field, if `v2` overflows then `v3` overflows too, so `v3`'s
        // range check rejects exactly the inputs `v2`'s would have. `v2`'s check is elided; `v3`'s
        // check is kept.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32):
            v2 = add v0, u32 1
            v3 = add v2, u32 1
            return v3
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.checked_to_unchecked();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u32):
            v2 = unchecked_add v0, u32 1
            v3 = add v2, u32 1
            return v3
        }
        ");
    }

    #[test]
    fn elides_only_the_intermediate_checks_in_a_longer_add_chain() {
        // A 3-add chain: the two intermediate adds are elided, the final (dominating) add keeps its
        // check.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32):
            v2 = add v0, u32 1
            v3 = add v2, u32 1
            v4 = add v3, u32 1
            return v4
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.checked_to_unchecked();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u32):
            v2 = unchecked_add v0, u32 1
            v3 = unchecked_add v2, u32 1
            v4 = add v3, u32 1
            return v4
        }
        ");
    }

    #[test]
    fn does_not_elide_when_intermediate_value_is_used_more_than_once() {
        // `v2` is used both by the dominating add AND returned directly. Removing `v2`'s range check
        // would leave the returned value un-range-checked, changing the accepted inputs / output.
        // So the intermediate check must be kept.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32):
            v2 = add v0, u32 1
            v3 = add v2, u32 1
            return v2, v3
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.checked_to_unchecked();
        // v2's check must remain checked (multi-use); only the semantics-preserving cases fire.
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u32):
            v2 = add v0, u32 1
            v3 = add v2, u32 1
            return v2, v3
        }
        ");
    }

    #[test]
    fn does_not_elide_when_dominating_consumer_is_a_subtraction() {
        // `sub` is not monotone, so an overflow of `v2` is not guaranteed to be re-detected. Keep
        // the intermediate check.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32):
            v2 = add v0, u32 1
            v3 = sub v2, u32 1
            return v3
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.checked_to_unchecked();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u32):
            v2 = add v0, u32 1
            v3 = sub v2, u32 1
            return v3
        }
        ");
    }

    #[test]
    fn does_not_elide_dominated_check_in_brillig() {
        // In Brillig, arithmetic wraps at bit_size and a checked op traps at its own location.
        // Eliding the intermediate check would change the observable failure point, so the pass
        // must NOT elide dominated checks in Brillig functions.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32):
            v2 = add v0, u32 1
            v3 = add v2, u32 1
            return v3
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.checked_to_unchecked();
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: u32):
            v2 = add v0, u32 1
            v3 = add v2, u32 1
            return v3
        }
        ");
    }

    #[test]
    fn does_not_elide_across_an_enable_side_effects_boundary() {
        // An `enable_side_effects` between the intermediate add and its consumer changes the
        // side-effects predicate: the consumer's overflow check may be vacuous (predicate 0) on
        // paths where the intermediate's was active, so the intermediate check must be kept.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32, v1: u1):
            v2 = add v0, u32 1
            enable_side_effects v1
            v3 = add v2, u32 1
            return v3
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.checked_to_unchecked();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u32, v1: u1):
            v3 = add v0, u32 1
            enable_side_effects v1
            v4 = add v3, u32 1
            return v4
        }
        ");
    }

    #[test]
    fn does_not_elide_across_a_block_boundary() {
        // The dominating add is in a different block; the same-block restriction rejects it (the
        // pass conservatively does not perform inter-block dominance analysis).
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32):
            v2 = add v0, u32 1
            jmp b1(v2)
          b1(v3: u32):
            v4 = add v3, u32 1
            return v4
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.checked_to_unchecked();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u32):
            v3 = add v0, u32 1
            jmp b1(v3)
          b1(v1: u32):
            v4 = add v1, u32 1
            return v4
        }
        ");
    }

    #[test]
    fn checked_to_unchecked_when_adding_two_u32_truncated_to_16_bits() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32, v1: u32):
            v2 = truncate v0 to 16 bits, max_bit_size: 33
            v3 = truncate v1 to 16 bits, max_bit_size: 33
            v4 = add v2, v3
            return v4
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.checked_to_unchecked();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u32, v1: u32):
            v2 = truncate v0 to 16 bits, max_bit_size: 33
            v3 = truncate v1 to 16 bits, max_bit_size: 33
            v4 = unchecked_add v2, v3
            return v4
        }
        ");
    }
}
