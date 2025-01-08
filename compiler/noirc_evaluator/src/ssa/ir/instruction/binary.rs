use acvm::{acir::AcirField, FieldElement};
use serde::{Deserialize, Serialize};

use super::{
    DataFlowGraph, Instruction, InstructionResultType, NumericType, SimplifyResult, Type, ValueId,
};

/// Binary Operations allowed in the IR.
/// Aside from the comparison operators (Eq and Lt), all operators
/// will return the same type as their operands.
/// The operand types must match for all binary operators.
/// All binary operators are also only for numeric types. To implement
/// e.g. equality for a compound type like a struct, one must add a
/// separate Eq operation for each field and combine them later with And.
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize)]
pub(crate) enum BinaryOp {
    /// Addition of lhs + rhs.
    Add,
    /// Subtraction of lhs - rhs.
    Sub,
    /// Multiplication of lhs * rhs.
    Mul,
    /// Division of lhs / rhs.
    Div,
    /// Modulus of lhs % rhs.
    Mod,
    /// Checks whether two types are equal.
    /// Returns true if the types were equal and
    /// false otherwise.
    Eq,
    /// Checks whether the lhs is less than the rhs.
    /// All other comparison operators should be translated
    /// to less than. For example (a > b) = (b < a) = !(a >= b) = !(b <= a).
    /// The result will always be a u1.
    Lt,
    /// Bitwise and (&)
    And,
    /// Bitwise or (|)
    Or,
    /// Bitwise xor (^)
    Xor,
    /// Bitshift left (<<)
    Shl,
    /// Bitshift right (>>)
    Shr,
}

impl std::fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOp::Add => write!(f, "add"),
            BinaryOp::Sub => write!(f, "sub"),
            BinaryOp::Mul => write!(f, "mul"),
            BinaryOp::Div => write!(f, "div"),
            BinaryOp::Eq => write!(f, "eq"),
            BinaryOp::Mod => write!(f, "mod"),
            BinaryOp::Lt => write!(f, "lt"),
            BinaryOp::And => write!(f, "and"),
            BinaryOp::Or => write!(f, "or"),
            BinaryOp::Xor => write!(f, "xor"),
            BinaryOp::Shl => write!(f, "shl"),
            BinaryOp::Shr => write!(f, "shr"),
        }
    }
}

/// A binary instruction in the IR.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub(crate) struct Binary {
    /// Left hand side of the binary operation
    pub(crate) lhs: ValueId,
    /// Right hand side of the binary operation
    pub(crate) rhs: ValueId,
    /// The binary operation to apply
    pub(crate) operator: BinaryOp,
}

impl Binary {
    /// The type of this Binary instruction's result
    pub(crate) fn result_type(&self) -> InstructionResultType {
        match self.operator {
            BinaryOp::Eq | BinaryOp::Lt => InstructionResultType::Known(Type::bool()),
            _ => InstructionResultType::Operand(self.lhs),
        }
    }

    /// Try to simplify this binary instruction, returning the new value if possible.
    pub(super) fn simplify(&self, dfg: &mut DataFlowGraph) -> SimplifyResult {
        let lhs = dfg.get_numeric_constant(self.lhs);
        let rhs = dfg.get_numeric_constant(self.rhs);
        let operand_type = dfg.type_of_value(self.lhs).unwrap_numeric();

        if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
            return match eval_constant_binary_op(lhs, rhs, self.operator, operand_type) {
                Some((result, result_type)) => {
                    let value = dfg.make_constant(result, result_type);
                    SimplifyResult::SimplifiedTo(value)
                }
                None => SimplifyResult::None,
            };
        }

        let lhs_is_zero = lhs.map_or(false, |lhs| lhs.is_zero());
        let rhs_is_zero = rhs.map_or(false, |rhs| rhs.is_zero());

        let lhs_is_one = lhs.map_or(false, |lhs| lhs.is_one());
        let rhs_is_one = rhs.map_or(false, |rhs| rhs.is_one());

        match self.operator {
            BinaryOp::Add => {
                if lhs_is_zero {
                    return SimplifyResult::SimplifiedTo(self.rhs);
                }
                if rhs_is_zero {
                    return SimplifyResult::SimplifiedTo(self.lhs);
                }
            }
            BinaryOp::Sub => {
                if rhs_is_zero {
                    return SimplifyResult::SimplifiedTo(self.lhs);
                }
            }
            BinaryOp::Mul => {
                if lhs_is_one {
                    return SimplifyResult::SimplifiedTo(self.rhs);
                }
                if rhs_is_one {
                    return SimplifyResult::SimplifiedTo(self.lhs);
                }
                if lhs_is_zero || rhs_is_zero {
                    let zero = dfg.make_constant(FieldElement::zero(), operand_type);
                    return SimplifyResult::SimplifiedTo(zero);
                }
                if dfg.get_value_max_num_bits(self.lhs) == 1 {
                    // Squaring a boolean value is a noop.
                    if dfg.resolve(self.lhs) == dfg.resolve(self.rhs) {
                        return SimplifyResult::SimplifiedTo(self.lhs);
                    }
                    // b*(b*x) = b*x if b is boolean
                    if let super::Value::Instruction { instruction, .. } = &dfg[self.rhs] {
                        if let Instruction::Binary(Binary { lhs, rhs, operator }) =
                            dfg[*instruction]
                        {
                            if operator == BinaryOp::Mul
                                && (dfg.resolve(self.lhs) == dfg.resolve(lhs)
                                    || dfg.resolve(self.lhs) == dfg.resolve(rhs))
                            {
                                return SimplifyResult::SimplifiedTo(self.rhs);
                            }
                        }
                    }
                }
                // (b*x)*b = b*x if b is boolean
                if dfg.get_value_max_num_bits(self.rhs) == 1 {
                    if let super::Value::Instruction { instruction, .. } = &dfg[self.lhs] {
                        if let Instruction::Binary(Binary { lhs, rhs, operator }) =
                            dfg[*instruction]
                        {
                            if operator == BinaryOp::Mul
                                && (dfg.resolve(self.rhs) == dfg.resolve(lhs)
                                    || dfg.resolve(self.rhs) == dfg.resolve(rhs))
                            {
                                return SimplifyResult::SimplifiedTo(self.lhs);
                            }
                        }
                    }
                }
            }
            BinaryOp::Div => {
                if rhs_is_one {
                    return SimplifyResult::SimplifiedTo(self.lhs);
                }
            }
            BinaryOp::Mod => {
                if rhs_is_one {
                    let zero = dfg.make_constant(FieldElement::zero(), operand_type);
                    return SimplifyResult::SimplifiedTo(zero);
                }
                if operand_type.is_unsigned() {
                    // lhs % 2**bit_size is equivalent to truncating `lhs` to `bit_size` bits.
                    // We then convert to a truncation for consistency, allowing more optimizations.
                    if let Some(modulus) = rhs {
                        let modulus = modulus.to_u128();
                        if modulus.is_power_of_two() {
                            let bit_size = modulus.ilog2();
                            return SimplifyResult::SimplifiedToInstruction(
                                Instruction::Truncate {
                                    value: self.lhs,
                                    bit_size,
                                    max_bit_size: operand_type.bit_size(),
                                },
                            );
                        }
                    }
                }
            }
            BinaryOp::Eq => {
                if dfg.resolve(self.lhs) == dfg.resolve(self.rhs) {
                    let one = dfg.make_constant(FieldElement::one(), NumericType::bool());
                    return SimplifyResult::SimplifiedTo(one);
                }

                if operand_type == NumericType::bool() {
                    // Simplify forms of `(boolean == true)` into `boolean`
                    if lhs_is_one {
                        return SimplifyResult::SimplifiedTo(self.rhs);
                    }
                    if rhs_is_one {
                        return SimplifyResult::SimplifiedTo(self.lhs);
                    }
                    // Simplify forms of `(boolean == false)` into `!boolean`
                    if lhs_is_zero {
                        return SimplifyResult::SimplifiedToInstruction(Instruction::Not(self.rhs));
                    }
                    if rhs_is_zero {
                        return SimplifyResult::SimplifiedToInstruction(Instruction::Not(self.lhs));
                    }
                }
            }
            BinaryOp::Lt => {
                if dfg.resolve(self.lhs) == dfg.resolve(self.rhs) {
                    let zero = dfg.make_constant(FieldElement::zero(), NumericType::bool());
                    return SimplifyResult::SimplifiedTo(zero);
                }
                if operand_type.is_unsigned() {
                    if rhs_is_zero {
                        // Unsigned values cannot be less than zero.
                        let zero = dfg.make_constant(FieldElement::zero(), NumericType::bool());
                        return SimplifyResult::SimplifiedTo(zero);
                    } else if rhs_is_one {
                        let zero = dfg.make_constant(FieldElement::zero(), operand_type);
                        return SimplifyResult::SimplifiedToInstruction(Instruction::binary(
                            BinaryOp::Eq,
                            self.lhs,
                            zero,
                        ));
                    }
                }
            }
            BinaryOp::And => {
                if lhs_is_zero || rhs_is_zero {
                    let zero = dfg.make_constant(FieldElement::zero(), operand_type);
                    return SimplifyResult::SimplifiedTo(zero);
                }
                if dfg.resolve(self.lhs) == dfg.resolve(self.rhs) {
                    return SimplifyResult::SimplifiedTo(self.lhs);
                }
                if operand_type == NumericType::bool() {
                    // Boolean AND is equivalent to multiplication, which is a cheaper operation.
                    let instruction = Instruction::binary(BinaryOp::Mul, self.lhs, self.rhs);
                    return SimplifyResult::SimplifiedToInstruction(instruction);
                }
                if operand_type.is_unsigned() {
                    // It's common in other programming languages to truncate values to a certain bit size using
                    // a bitwise AND with a bit mask. However this operation is quite inefficient inside a snark.
                    //
                    // We then replace this bitwise operation with an equivalent truncation instruction.
                    match (lhs, rhs) {
                        (Some(bitmask), None) | (None, Some(bitmask)) => {
                            // This substitution requires the bitmask to retain all of the lower bits.
                            // The bitmask must then be one less than a power of 2.
                            let bitmask_plus_one = bitmask.to_u128() + 1;
                            if bitmask_plus_one.is_power_of_two() {
                                let value = if lhs.is_some() { self.rhs } else { self.lhs };
                                let num_bits = bitmask_plus_one.ilog2();
                                return SimplifyResult::SimplifiedToInstruction(
                                    Instruction::Truncate {
                                        value,
                                        bit_size: num_bits,
                                        max_bit_size: operand_type.bit_size(),
                                    },
                                );
                            }
                        }

                        _ => (),
                    }
                }
            }
            BinaryOp::Or => {
                if lhs_is_zero {
                    return SimplifyResult::SimplifiedTo(self.rhs);
                }
                if rhs_is_zero {
                    return SimplifyResult::SimplifiedTo(self.lhs);
                }
                if operand_type == NumericType::bool() && (lhs_is_one || rhs_is_one) {
                    let one = dfg.make_constant(FieldElement::one(), operand_type);
                    return SimplifyResult::SimplifiedTo(one);
                }
                if dfg.resolve(self.lhs) == dfg.resolve(self.rhs) {
                    return SimplifyResult::SimplifiedTo(self.lhs);
                }
            }
            BinaryOp::Xor => {
                if lhs_is_zero {
                    return SimplifyResult::SimplifiedTo(self.rhs);
                }
                if rhs_is_zero {
                    return SimplifyResult::SimplifiedTo(self.lhs);
                }
                if dfg.resolve(self.lhs) == dfg.resolve(self.rhs) {
                    let zero = dfg.make_constant(FieldElement::zero(), operand_type);
                    return SimplifyResult::SimplifiedTo(zero);
                }
            }
            BinaryOp::Shl => return SimplifyResult::None,
            BinaryOp::Shr => {
                // Bit shifts by constants can be treated as divisions.
                if let Some(rhs_const) = rhs {
                    if rhs_const >= FieldElement::from(operand_type.bit_size() as u128) {
                        // Shifting by the full width of the operand type, any `lhs` goes to zero.
                        let zero = dfg.make_constant(FieldElement::zero(), operand_type);
                        return SimplifyResult::SimplifiedTo(zero);
                    }
                    return SimplifyResult::None;
                }
            }
        };
        SimplifyResult::None
    }

    /// Check if unsigned overflow is possible, and if so return some message to be used if it fails.
    pub(crate) fn check_unsigned_overflow_msg(
        &self,
        dfg: &DataFlowGraph,
        bit_size: u32,
    ) -> Option<&'static str> {
        // We try to optimize away operations that are guaranteed not to overflow
        let max_lhs_bits = dfg.get_value_max_num_bits(self.lhs);
        let max_rhs_bits = dfg.get_value_max_num_bits(self.rhs);

        let msg = match self.operator {
            BinaryOp::Add => {
                if std::cmp::max(max_lhs_bits, max_rhs_bits) < bit_size {
                    // `lhs` and `rhs` have both been casted up from smaller types and so cannot overflow.
                    return None;
                }
                "attempt to add with overflow"
            }
            BinaryOp::Sub => {
                if dfg.is_constant(self.lhs) && max_lhs_bits > max_rhs_bits {
                    // `lhs` is a fixed constant and `rhs` is restricted such that `lhs - rhs > 0`
                    // Note strict inequality as `rhs > lhs` while `max_lhs_bits == max_rhs_bits` is possible.
                    return None;
                }
                "attempt to subtract with overflow"
            }
            BinaryOp::Mul => {
                if bit_size == 1
                    || max_lhs_bits + max_rhs_bits <= bit_size
                    || max_lhs_bits == 1
                    || max_rhs_bits == 1
                {
                    // Either performing boolean multiplication (which cannot overflow),
                    // or `lhs` and `rhs` have both been casted up from smaller types and so cannot overflow.
                    return None;
                }
                "attempt to multiply with overflow"
            }
            _ => return None,
        };
        Some(msg)
    }
}

/// Evaluate a binary operation with constant arguments.
pub(crate) fn eval_constant_binary_op(
    lhs: FieldElement,
    rhs: FieldElement,
    operator: BinaryOp,
    mut operand_type: NumericType,
) -> Option<(FieldElement, NumericType)> {
    let value = match operand_type {
        NumericType::NativeField => {
            // If the rhs of a division is zero, attempting to evaluate the division will cause a compiler panic.
            // Thus, we do not evaluate the division in this method, as we want to avoid triggering a panic,
            // and the operation should be handled by ACIR generation.
            if matches!(operator, BinaryOp::Div | BinaryOp::Mod) && rhs == FieldElement::zero() {
                return None;
            }
            operator.get_field_function()?(lhs, rhs)
        }
        NumericType::Unsigned { bit_size } => {
            let function = operator.get_u128_function();

            let lhs = truncate(lhs.try_into_u128()?, bit_size);
            let rhs = truncate(rhs.try_into_u128()?, bit_size);

            // The divisor is being truncated into the type of the operand, which can potentially
            // lead to the rhs being zero.
            // If the rhs of a division is zero, attempting to evaluate the division will cause a compiler panic.
            // Thus, we do not evaluate the division in this method, as we want to avoid triggering a panic,
            // and the operation should be handled by ACIR generation.
            if matches!(operator, BinaryOp::Div | BinaryOp::Mod) && rhs == 0 {
                return None;
            }
            let result = function(lhs, rhs)?;
            // Check for overflow
            if result >= 1 << bit_size {
                return None;
            }
            result.into()
        }
        NumericType::Signed { bit_size } => {
            let function = operator.get_i128_function();

            let lhs = try_convert_field_element_to_signed_integer(lhs, bit_size)?;
            let rhs = try_convert_field_element_to_signed_integer(rhs, bit_size)?;
            // The divisor is being truncated into the type of the operand, which can potentially
            // lead to the rhs being zero.
            // If the rhs of a division is zero, attempting to evaluate the division will cause a compiler panic.
            // Thus, we do not evaluate the division in this method, as we want to avoid triggering a panic,
            // and the operation should be handled by ACIR generation.
            if matches!(operator, BinaryOp::Div | BinaryOp::Mod) && rhs == 0 {
                return None;
            }

            let result = function(lhs, rhs)?;
            // Check for overflow
            let two_pow_bit_size_minus_one = 1i128 << (bit_size - 1);
            if result >= two_pow_bit_size_minus_one || result < -two_pow_bit_size_minus_one {
                return None;
            }
            convert_signed_integer_to_field_element(result, bit_size)
        }
    };

    if matches!(operator, BinaryOp::Eq | BinaryOp::Lt) {
        operand_type = NumericType::bool();
    }

    Some((value, operand_type))
}

/// Values in the range `[0, 2^(bit_size-1))` are interpreted as positive integers
///
/// Values in the range `[2^(bit_size-1), 2^bit_size)` are interpreted as negative integers.
fn try_convert_field_element_to_signed_integer(field: FieldElement, bit_size: u32) -> Option<i128> {
    let unsigned_int = truncate(field.try_into_u128()?, bit_size);

    let max_positive_value = 1 << (bit_size - 1);
    let is_positive = unsigned_int < max_positive_value;

    let signed_int = if is_positive {
        unsigned_int as i128
    } else {
        let x = (1u128 << bit_size) - unsigned_int;
        -(x as i128)
    };

    Some(signed_int)
}

fn convert_signed_integer_to_field_element(int: i128, bit_size: u32) -> FieldElement {
    if int >= 0 {
        FieldElement::from(int)
    } else {
        // We add an offset of `bit_size` bits to shift the negative values into the range [2^(bitsize-1), 2^bitsize)
        let offset_int = (1i128 << bit_size) + int;
        FieldElement::from(offset_int)
    }
}

fn truncate(int: u128, bit_size: u32) -> u128 {
    let max = 1 << bit_size;
    int % max
}

impl BinaryOp {
    fn get_field_function(self) -> Option<fn(FieldElement, FieldElement) -> FieldElement> {
        match self {
            BinaryOp::Add => Some(std::ops::Add::add),
            BinaryOp::Sub => Some(std::ops::Sub::sub),
            BinaryOp::Mul => Some(std::ops::Mul::mul),
            BinaryOp::Div => Some(std::ops::Div::div),
            BinaryOp::Eq => Some(|x, y| (x == y).into()),
            BinaryOp::Lt => Some(|x, y| (x < y).into()),
            // Bitwise operators are unsupported for Fields
            BinaryOp::Mod => None,
            BinaryOp::And => None,
            BinaryOp::Or => None,
            BinaryOp::Xor => None,
            BinaryOp::Shl => None,
            BinaryOp::Shr => None,
        }
    }

    fn get_u128_function(self) -> fn(u128, u128) -> Option<u128> {
        match self {
            BinaryOp::Add => u128::checked_add,
            BinaryOp::Sub => u128::checked_sub,
            BinaryOp::Mul => u128::checked_mul,
            BinaryOp::Div => u128::checked_div,
            BinaryOp::Mod => u128::checked_rem,
            BinaryOp::And => |x, y| Some(x & y),
            BinaryOp::Or => |x, y| Some(x | y),
            BinaryOp::Xor => |x, y| Some(x ^ y),
            BinaryOp::Eq => |x, y| Some((x == y) as u128),
            BinaryOp::Lt => |x, y| Some((x < y) as u128),
            BinaryOp::Shl => |x, y| Some(x << y),
            BinaryOp::Shr => |x, y| Some(x >> y),
        }
    }

    fn get_i128_function(self) -> fn(i128, i128) -> Option<i128> {
        match self {
            BinaryOp::Add => i128::checked_add,
            BinaryOp::Sub => i128::checked_sub,
            BinaryOp::Mul => i128::checked_mul,
            BinaryOp::Div => i128::checked_div,
            BinaryOp::Mod => i128::checked_rem,
            BinaryOp::And => |x, y| Some(x & y),
            BinaryOp::Or => |x, y| Some(x | y),
            BinaryOp::Xor => |x, y| Some(x ^ y),
            BinaryOp::Eq => |x, y| Some((x == y) as i128),
            BinaryOp::Lt => |x, y| Some((x < y) as i128),
            BinaryOp::Shl => |x, y| Some(x << y),
            BinaryOp::Shr => |x, y| Some(x >> y),
        }
    }
}

#[cfg(test)]
mod test {
    use proptest::prelude::*;

    use super::{
        convert_signed_integer_to_field_element, try_convert_field_element_to_signed_integer,
    };

    proptest! {
        #[test]
        fn signed_int_roundtrip(int: i128, bit_size in 1u32..=64) {
            let int = int % (1i128 << (bit_size - 1));

            let int_as_field = convert_signed_integer_to_field_element(int, bit_size);
            let recovered_int = try_convert_field_element_to_signed_integer(int_as_field, bit_size).unwrap();

            prop_assert_eq!(int, recovered_int);
        }
    }
}
