use acvm::{AcirField as _, FieldElement};

use crate::ssa::ir::{
    dfg::DataFlowGraph,
    instruction::{Binary, BinaryOp, Instruction, binary::eval_constant_binary_op},
    types::NumericType,
};

use super::SimplifyResult;

/// Try to simplify this binary instruction, returning the new value if possible.
pub(super) fn simplify_binary(binary: &Binary, dfg: &mut DataFlowGraph) -> SimplifyResult {
    let lhs = dfg.resolve(binary.lhs);
    let rhs = dfg.resolve(binary.rhs);

    let lhs_value = dfg.get_numeric_constant(lhs);
    let rhs_value = dfg.get_numeric_constant(rhs);

    let lhs_type = dfg.type_of_value(lhs).unwrap_numeric();
    let rhs_type = dfg.type_of_value(rhs).unwrap_numeric();

    let operator = binary.operator;
    if operator != BinaryOp::Shl && operator != BinaryOp::Shr {
        assert_eq!(lhs_type, rhs_type, "ICE - Binary instruction operands must have the same type");
    }

    let operator = if lhs_type == NumericType::NativeField {
        // Unchecked operations between fields or bools don't make sense, so we convert those to non-unchecked
        // to reduce noise and confusion in the generated SSA.
        match operator {
            BinaryOp::Add { unchecked: true } => BinaryOp::Add { unchecked: false },
            BinaryOp::Sub { unchecked: true } => BinaryOp::Sub { unchecked: false },
            BinaryOp::Mul { unchecked: true } => BinaryOp::Mul { unchecked: false },
            _ => operator,
        }
    } else if lhs_type == NumericType::bool() {
        // Unchecked mul between bools doesn't make sense, so we convert that to non-unchecked
        if let BinaryOp::Mul { unchecked: true } = operator {
            BinaryOp::Mul { unchecked: false }
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
            Some((result, result_type)) => {
                let value = dfg.make_constant(result, result_type);
                SimplifyResult::SimplifiedTo(value)
            }
            None => SimplifyResult::SimplifiedToInstruction(simplified),
        };
    }

    let lhs_is_zero = lhs_value.is_some_and(|lhs| lhs.is_zero());
    let rhs_is_zero = rhs_value.is_some_and(|rhs| rhs.is_zero());

    let lhs_is_one = lhs_value.is_some_and(|lhs| lhs.is_one());
    let rhs_is_one = rhs_value.is_some_and(|rhs| rhs.is_one());

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
                            && (lhs == dfg.resolve(b_lhs) || lhs == dfg.resolve(b_rhs))
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
                            && (rhs == dfg.resolve(b_lhs) || rhs == dfg.resolve(b_rhs))
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
                            max_bit_size: lhs_type.bit_size(),
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
                        let bitmask_plus_one = bitmask.to_u128() + 1;
                        if bitmask_plus_one.is_power_of_two() {
                            let value = if lhs_value.is_some() { rhs } else { lhs };
                            let bit_size = bitmask_plus_one.ilog2();
                            let max_bit_size = lhs_type.bit_size();

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
        BinaryOp::Shl => return SimplifyResult::SimplifiedToInstruction(simplified),
        BinaryOp::Shr => {
            // Bit shifts by constants can be treated as divisions.
            if let Some(rhs_const) = rhs_value {
                if rhs_const >= FieldElement::from(lhs_type.bit_size() as u128) {
                    // Shifting by the full width of the operand type, any `lhs` goes to zero.
                    let zero = dfg.make_constant(FieldElement::zero(), lhs_type);
                    return SimplifyResult::SimplifiedTo(zero);
                }
                return SimplifyResult::SimplifiedToInstruction(simplified);
            }
        }
    };
    SimplifyResult::SimplifiedToInstruction(simplified)
}
