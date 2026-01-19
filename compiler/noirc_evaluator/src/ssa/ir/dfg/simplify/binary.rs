use acvm::{AcirField as _, FieldElement};

use crate::ssa::ir::{
    basic_block::BasicBlockId,
    dfg::DataFlowGraph,
    instruction::{
        Binary, BinaryOp, Instruction,
        binary::{BinaryEvaluationResult, eval_constant_binary_op},
    },
    types::NumericType,
};
use noirc_errors::call_stack::CallStackId;

use super::SimplifyResult;

/// Try to simplify this binary instruction, returning the new value if possible.
pub(super) fn simplify_binary(
    binary: &Binary,
    dfg: &mut DataFlowGraph,
    block: BasicBlockId,
    call_stack: CallStackId,
) -> SimplifyResult {
    let lhs = binary.lhs;
    let rhs = binary.rhs;

    let lhs_value = dfg.get_numeric_constant(lhs);
    let rhs_value = dfg.get_numeric_constant(rhs);

    let lhs_type = dfg.type_of_value(lhs).unwrap_numeric();
    let rhs_type = dfg.type_of_value(rhs).unwrap_numeric();

    let operator = binary.operator;
    assert_eq!(lhs_type, rhs_type, "ICE - Binary instruction operands must have the same type");

    let operator = if lhs_type == NumericType::NativeField {
        // Unchecked operations between fields don't make sense, so we convert those to non-unchecked
        // to reduce noise and confusion in the generated SSA.
        match operator {
            BinaryOp::Add { unchecked: true } => BinaryOp::Add { unchecked: false },
            BinaryOp::Sub { unchecked: true } => BinaryOp::Sub { unchecked: false },
            BinaryOp::Mul { unchecked: true } => BinaryOp::Mul { unchecked: false },
            _ => operator,
        }
    } else if lhs_type == NumericType::bool() {
        // When multiplying bools there can never be an overflow so using checked or unchecked
        // should be the same. However, acir/brillig will check overflow for unsigned operations
        // so here we turn checked bool multiplications to unchecked.
        if let BinaryOp::Mul { unchecked: false } = operator {
            BinaryOp::Mul { unchecked: true }
        } else {
            operator
        }
    } else {
        operator
    };

    // We never return `SimplifyResult::None` here because `operator` might have changed.
    let simplified = Instruction::Binary(Binary { lhs, rhs, operator });

    if let (Some(lhs), Some(rhs)) = (lhs_value, rhs_value) {
        return match eval_constant_binary_op(lhs, rhs, operator, lhs_type) {
            BinaryEvaluationResult::Success(result, result_type) => {
                let value = dfg.make_constant(result, result_type);
                SimplifyResult::SimplifiedTo(value)
            }
            BinaryEvaluationResult::CouldNotEvaluate | BinaryEvaluationResult::Failure(..) => {
                SimplifyResult::SimplifiedToInstruction(simplified)
            }
        };
    }

    let lhs_is_zero = lhs_value.is_some_and(|lhs| lhs.is_zero());
    let rhs_is_zero = rhs_value.is_some_and(|rhs| rhs.is_zero());

    let lhs_is_one = lhs_value.is_some_and(|lhs| lhs.is_one());
    let rhs_is_one = rhs_value.is_some_and(|rhs| rhs.is_one());
    let lhs_is_max =
        lhs_value.is_some_and(|lhs| lhs_type.max_value().is_ok_and(|max_value| lhs == max_value));
    let rhs_is_max =
        rhs_value.is_some_and(|rhs| rhs_type.max_value().is_ok_and(|max_value| rhs == max_value));

    match binary.operator {
        BinaryOp::Add { .. } => {
            if lhs_is_zero {
                return SimplifyResult::SimplifiedTo(rhs);
            }
            if rhs_is_zero {
                return SimplifyResult::SimplifiedTo(lhs);
            }
        }
        BinaryOp::Sub { .. } => {
            if lhs == rhs {
                let zero = dfg.make_constant(FieldElement::zero(), lhs_type);
                return SimplifyResult::SimplifiedTo(zero);
            }

            if rhs_is_zero {
                return SimplifyResult::SimplifiedTo(lhs);
            }
        }
        BinaryOp::Mul { .. } => {
            if lhs_is_one {
                return SimplifyResult::SimplifiedTo(rhs);
            }
            if rhs_is_one {
                return SimplifyResult::SimplifiedTo(lhs);
            }
            if lhs_is_zero || rhs_is_zero {
                let zero = dfg.make_constant(FieldElement::zero(), lhs_type);
                return SimplifyResult::SimplifiedTo(zero);
            }
            if dfg.get_value_max_num_bits(lhs) == 1 {
                // Squaring a boolean value is a noop.
                if lhs == rhs {
                    return SimplifyResult::SimplifiedTo(lhs);
                }
                // b*(b*x) = b*x if b is boolean
                if let super::Value::Instruction { instruction, .. } = &dfg[rhs] {
                    if let Instruction::Binary(Binary { lhs: b_lhs, rhs: b_rhs, operator }) =
                        dfg[*instruction]
                    {
                        if matches!(operator, BinaryOp::Mul { .. })
                            && (lhs == b_lhs || lhs == b_rhs)
                        {
                            return SimplifyResult::SimplifiedTo(rhs);
                        }
                    }
                }
            }
            // (b*x)*b = b*x if b is boolean
            if dfg.get_value_max_num_bits(rhs) == 1 {
                if let super::Value::Instruction { instruction, .. } = &dfg[lhs] {
                    if let Instruction::Binary(Binary { lhs: b_lhs, rhs: b_rhs, operator }) =
                        dfg[*instruction]
                    {
                        if matches!(operator, BinaryOp::Mul { .. })
                            && (rhs == b_lhs || rhs == b_rhs)
                        {
                            return SimplifyResult::SimplifiedTo(lhs);
                        }
                    }
                }
            }
        }
        BinaryOp::Div => {
            if rhs_is_one {
                return SimplifyResult::SimplifiedTo(lhs);
            }
            if let Some(rhs_value) = rhs_value {
                if lhs_type == NumericType::NativeField && !rhs_value.is_zero() {
                    let rhs = dfg.make_constant(rhs_value.inverse(), NumericType::NativeField);
                    return SimplifyResult::SimplifiedToInstruction(Instruction::Binary(Binary {
                        lhs,
                        rhs,
                        operator: BinaryOp::Mul { unchecked: false },
                    }));
                }
            }
        }
        BinaryOp::Mod => {
            if rhs_is_one {
                let zero = dfg.make_constant(FieldElement::zero(), lhs_type);
                return SimplifyResult::SimplifiedTo(zero);
            }
            if lhs_type.is_unsigned() {
                // lhs % 2**bit_size is equivalent to truncating `lhs` to `bit_size` bits.
                // We then convert to a truncation for consistency, allowing more optimizations.
                if let Some(modulus) = rhs_value {
                    let modulus = modulus.to_u128();
                    if modulus.is_power_of_two() {
                        let bit_size = modulus.ilog2();
                        return SimplifyResult::SimplifiedToInstruction(Instruction::Truncate {
                            value: lhs,
                            bit_size,
                            max_bit_size: lhs_type.bit_size::<FieldElement>(),
                        });
                    }
                }
            }
        }
        BinaryOp::Eq => {
            if lhs == rhs {
                let one = dfg.make_constant(FieldElement::one(), NumericType::bool());
                return SimplifyResult::SimplifiedTo(one);
            }

            if lhs_type == NumericType::bool() {
                // Simplify forms of `(boolean == true)` into `boolean`
                if lhs_is_one {
                    return SimplifyResult::SimplifiedTo(rhs);
                }
                if rhs_is_one {
                    return SimplifyResult::SimplifiedTo(lhs);
                }
                // Simplify forms of `(boolean == false)` into `!boolean`
                if lhs_is_zero {
                    return SimplifyResult::SimplifiedToInstruction(Instruction::Not(rhs));
                }
                if rhs_is_zero {
                    return SimplifyResult::SimplifiedToInstruction(Instruction::Not(lhs));
                }
            }
        }
        BinaryOp::Lt => {
            if lhs == rhs {
                let zero = dfg.make_constant(FieldElement::zero(), NumericType::bool());
                return SimplifyResult::SimplifiedTo(zero);
            }
            if lhs_type.is_unsigned() {
                if rhs_is_zero {
                    // Unsigned values cannot be less than zero.
                    let zero = dfg.make_constant(FieldElement::zero(), NumericType::bool());
                    return SimplifyResult::SimplifiedTo(zero);
                } else if rhs_is_one {
                    let zero = dfg.make_constant(FieldElement::zero(), lhs_type);
                    return SimplifyResult::SimplifiedToInstruction(Instruction::binary(
                        BinaryOp::Eq,
                        lhs,
                        zero,
                    ));
                } else if lhs_is_zero && dfg.runtime.is_acir() {
                    // `0 < rhs` for unsigned values is the same as `rhs != 0`,
                    // which is slightly more performant in ACIR
                    let zero = dfg.make_constant(FieldElement::zero(), lhs_type);
                    let instruction =
                        Instruction::Binary(Binary { lhs: rhs, rhs: zero, operator: BinaryOp::Eq });
                    let eq = dfg
                        .insert_instruction_and_results(instruction, block, None, call_stack)
                        .first();
                    let neq = Instruction::Not(eq);
                    return SimplifyResult::SimplifiedToInstruction(neq);
                }
            }
        }
        BinaryOp::And => {
            if lhs_is_zero || rhs_is_zero {
                let zero = dfg.make_constant(FieldElement::zero(), lhs_type);
                return SimplifyResult::SimplifiedTo(zero);
            }
            if lhs == rhs {
                return SimplifyResult::SimplifiedTo(lhs);
            }
            if lhs_type == NumericType::bool() {
                // Boolean AND is equivalent to multiplication, which is a cheaper operation.
                // (mul unchecked because these are bools so it doesn't matter really)
                let instruction = Instruction::binary(BinaryOp::Mul { unchecked: true }, lhs, rhs);
                return SimplifyResult::SimplifiedToInstruction(instruction);
            }
            if lhs_type.is_unsigned() {
                // It's common in other programming languages to truncate values to a certain bit size using
                // a bitwise AND with a bit mask. However this operation is quite inefficient inside a snark.
                //
                // We then replace this bitwise operation with an equivalent truncation instruction.
                match (lhs_value, rhs_value) {
                    (Some(bitmask), None) | (None, Some(bitmask)) => {
                        // This substitution requires the bitmask to retain all of the lower bits.
                        // The bitmask must then be one less than a power of 2.
                        let bitmask = bitmask.to_u128();
                        if bitmask == u128::MAX || (bitmask + 1).is_power_of_two() {
                            let value = if lhs_value.is_some() { rhs } else { lhs };
                            let bit_size =
                                if bitmask == u128::MAX { 128 } else { (bitmask + 1).ilog2() };
                            let max_bit_size = lhs_type.bit_size::<FieldElement>();

                            if bit_size == max_bit_size {
                                // If we're truncating a value into the full size of its type then
                                // the truncation is a noop.
                                return SimplifyResult::SimplifiedTo(value);
                            } else {
                                return SimplifyResult::SimplifiedToInstruction(
                                    Instruction::Truncate { value, bit_size, max_bit_size },
                                );
                            }
                        }
                    }

                    _ => (),
                }
            }
        }
        BinaryOp::Or => {
            if lhs_is_zero {
                return SimplifyResult::SimplifiedTo(rhs);
            }
            if rhs_is_zero {
                return SimplifyResult::SimplifiedTo(lhs);
            }
            if lhs_type == NumericType::bool() && (lhs_is_one || rhs_is_one) {
                let one = dfg.make_constant(FieldElement::one(), lhs_type);
                return SimplifyResult::SimplifiedTo(one);
            }
            if lhs == rhs {
                return SimplifyResult::SimplifiedTo(lhs);
            }

            if lhs_is_max || rhs_is_max {
                let max = dfg.make_constant(lhs_type.max_value().unwrap(), lhs_type);
                return SimplifyResult::SimplifiedTo(max);
            }
        }
        BinaryOp::Xor => {
            if lhs_is_zero {
                return SimplifyResult::SimplifiedTo(rhs);
            }
            if rhs_is_zero {
                return SimplifyResult::SimplifiedTo(lhs);
            }
            if lhs == rhs {
                let zero = dfg.make_constant(FieldElement::zero(), lhs_type);
                return SimplifyResult::SimplifiedTo(zero);
            }
        }
        BinaryOp::Shl | BinaryOp::Shr => {
            if rhs_is_zero {
                return SimplifyResult::SimplifiedTo(lhs);
            }
            return SimplifyResult::SimplifiedToInstruction(simplified);
        }
    };
    SimplifyResult::SimplifiedToInstruction(simplified)
}

#[cfg(test)]
mod tests {
    use crate::{assert_ssa_snapshot, ssa::ssa_gen::Ssa};

    #[test]
    fn replaces_shl_identity_with_lhs() {
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u8):
            v1 = shl v0, u8 0
            return v1
        }
        ";
        let ssa = Ssa::from_str_simplifying(src).unwrap();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u8):
            return v0
        }
        ");
    }

    #[test]
    fn replaces_shr_identity_with_lhs() {
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u8):
            v1 = shr v0, u8 0
            return v1
        }
        ";
        let ssa = Ssa::from_str_simplifying(src).unwrap();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u8):
            return v0
        }
        ");
    }

    #[test]
    fn simplifies_zero_less_than_unsigned_value_to_not_equals_in_acir() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u8):
            v1 = lt u8 0, v0
            return v1
        }
        ";

        let ssa = Ssa::from_str_simplifying(src).unwrap();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u8):
            v2 = eq v0, u8 0
            v3 = not v2
            return v3
        }
        ");
    }
}
