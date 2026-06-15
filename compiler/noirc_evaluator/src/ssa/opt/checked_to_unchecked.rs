//! This SSA pass will turn checked unsigned binary additions, subtractions and multiplications
//! into unchecked ones if it's guaranteed that the operations cannot overflow.
//!
//! Signed checked binary operations should have already been converted to unchecked ones with
//! an explicit overflow check during [`super::expand_signed_checks`].
use acvm::AcirField as _;
use rustc_hash::FxHashMap as HashMap;

use crate::ssa::{
    ir::{
        dfg::DataFlowGraph,
        function::Function,
        instruction::{Binary, BinaryOp, Instruction, TerminatorInstruction},
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
                    max_lhs_bits < bit_size && max_rhs_bits < bit_size
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

/// Pre-check condition for [Function::checked_to_unchecked].
///
/// Panics if:
///   - The function contains any checked signed binary operations (add, sub, mul).
///   - These should have already been converted by the expand_signed_checks pass.
#[cfg(debug_assertions)]
fn checked_to_unchecked_pre_check(func: &Function) {
    // expand_signed_checks must have run
    super::checks::for_each_instruction(func, |instruction, dfg| {
        super::checks::assert_not_checked_signed_add_sub_mul(instruction, dfg);
    });

    assert_no_predicated_checked_arithmetic_is_returned(func);
}

/// A checked unsigned `add`/`sub`/`mul` is side-effectful: during ACIR generation its operands are
/// zeroed by the active side-effects predicate, so in a disabled branch it yields `0`. This pass
/// rewrites such an op to its unchecked form, which is not predicated and yields the raw value.
/// That difference only becomes observable as a circuit output if the raw value reaches a return
/// value of the function while the governing predicate has been lifted, as in
///
/// ```text
/// enable_side_effects v0
/// v3 = add v2, u32 1
/// enable_side_effects u1 1
/// return v3            // v3 returned with v0 no longer governing it
/// ```
///
/// `flatten_cfg` never emits that shape — it exposes a branch value to the rest of the program
/// through the predicating merge `c * then + !c * else`, not by returning a raw checked-arithmetic
/// result — so asserting its absence here keeps the rewrite sound without rejecting any SSA the
/// real pipeline produces. A predicated value that is only reused *internally* after its window
/// (e.g. relied upon as a zeroed operand of a later instruction) is a legitimate flattened shape
/// and is intentionally not flagged.
#[cfg(debug_assertions)]
fn assert_no_predicated_checked_arithmetic_is_returned(func: &Function) {
    if func.runtime().is_brillig() {
        // Brillig functions never contain `enable_side_effects`.
        return;
    }

    // `enable_side_effects` is only present once the function has been flattened to a single
    // block; with control flow still present there is no active predicate to track.
    if func.reachable_blocks().len() > 1 {
        return;
    }

    let dfg = &func.dfg;
    let block = func.entry_block();

    // The condition of the most recent `enable_side_effects`. `None` means the active predicate is
    // the constant `u1 1` (no side effects disabled), which is also the initial state.
    let mut active_condition: Option<ValueId> = None;
    // Maps the result of a checked unsigned add/sub/mul to the non-trivial predicate that was
    // active when it was produced.
    let mut predicated_results = HashMap::<ValueId, ValueId>::default();

    for instruction_id in dfg[block].instructions() {
        let instruction = &dfg[*instruction_id];

        if let Instruction::EnableSideEffectsIf { condition } = instruction {
            let is_one =
                dfg.get_numeric_constant(*condition).is_some_and(|condition| condition.is_one());
            active_condition = if is_one { None } else { Some(*condition) };
            continue;
        }

        if let Some(condition) = active_condition
            && is_checked_unsigned_arithmetic(instruction, dfg)
        {
            for result in dfg.instruction_results(*instruction_id) {
                predicated_results.insert(*result, condition);
            }
        }
    }

    let Some(TerminatorInstruction::Return { return_values, .. }) = dfg[block].terminator() else {
        return;
    };

    for value in return_values {
        if let Some(definition_condition) = predicated_results.get(value)
            && active_condition != Some(*definition_condition)
        {
            panic!(
                "checked_to_unchecked pre-check failed in function {}: returned value {value} is \
                 produced by a checked unsigned arithmetic instruction under predicate \
                 {definition_condition}, but is returned where that predicate no longer governs \
                 it; rewriting it to unchecked would change its value in a disabled side-effect \
                 window",
                func.id(),
            );
        }
    }
}

/// Whether `instruction` is a checked unsigned `add`/`sub`/`mul` — exactly the set of instructions
/// [`Function::checked_to_unchecked`] can rewrite to their unchecked form.
#[cfg(debug_assertions)]
fn is_checked_unsigned_arithmetic(instruction: &Instruction, dfg: &DataFlowGraph) -> bool {
    let Instruction::Binary(binary) = instruction else {
        return false;
    };
    if !matches!(
        binary.operator,
        BinaryOp::Add { unchecked: false }
            | BinaryOp::Sub { unchecked: false }
            | BinaryOp::Mul { unchecked: false }
    ) {
        return false;
    }
    matches!(dfg.type_of_value(binary.lhs).unwrap_numeric(), NumericType::Unsigned { .. })
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

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "in a disabled side-effect window")]
    fn rejects_predicated_checked_arithmetic_returned_out_of_its_window() {
        // The audit shape (#1391): a checked add governed by `v0` is returned directly after the
        // predicate is restored to `1`, so its disabled-branch value would escape unpredicated.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u1, v1: u16):
            enable_side_effects v0
            v2 = cast v1 as u32
            v3 = add v2, u32 1
            enable_side_effects u1 1
            return v3
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let _ = ssa.checked_to_unchecked();
    }

    #[test]
    fn allows_predicated_checked_arithmetic_reused_internally() {
        // A disabled checked sub whose zero is legitimately consumed by a later instruction (not
        // returned raw); `checked_to_unchecked` would not even rewrite this underflowing sub.
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1):
            enable_side_effects v0
            v1 = sub u32 0, u32 1
            enable_side_effects u1 1
            v2 = add v1, u32 1
            return v2
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let _ = ssa.checked_to_unchecked();
    }

    #[test]
    fn allows_predicated_checked_add_consumed_by_predicate_merge() {
        // The shape `flatten_cfg` actually emits: the predicated value is re-multiplied by the
        // predicate (`c * then`) before being observed, so the disabled branch still yields `0`.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u1, v1: u32):
            enable_side_effects v0
            v3 = add v1, u32 1
            enable_side_effects u1 1
            v4 = cast v0 as u32
            v5 = mul v4, v3
            return v5
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let _ = ssa.checked_to_unchecked();
    }
}
