use acvm::{acir::AcirField, FieldElement};
use serde::{Deserialize, Serialize};

use crate::ssa::ir::value::Value;

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
    Add { unchecked: bool },
    /// Subtraction of lhs - rhs.
    Sub { unchecked: bool },
    /// Multiplication of lhs * rhs.
    Mul { unchecked: bool },
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
            BinaryOp::Add { unchecked: false } => write!(f, "add"),
            BinaryOp::Add { unchecked: true } => write!(f, "unchecked_add"),
            BinaryOp::Sub { unchecked: false } => write!(f, "sub"),
            BinaryOp::Sub { unchecked: true } => write!(f, "unchecked_sub"),
            BinaryOp::Mul { unchecked: false } => write!(f, "mul"),
            BinaryOp::Mul { unchecked: true } => write!(f, "unchecked_mul"),
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
        let lhs = dfg.resolve(self.lhs);
        let rhs = dfg.resolve(self.rhs);

        let lhs_const = dfg.get_numeric_constant(lhs);
        let rhs_const = dfg.get_numeric_constant(rhs);

        let lhs_type = dfg.type_of_value(lhs).unwrap_numeric();
        let rhs_type = dfg.type_of_value(rhs).unwrap_numeric();

        let operator = self.operator;
        if operator != BinaryOp::Shl && operator != BinaryOp::Shr {
            assert_eq!(
                lhs_type, rhs_type,
                "ICE - Binary instruction operands must have the same type"
            );
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

        if let (Some(lhs), Some(rhs)) = (lhs_const, rhs_const) {
            return match eval_constant_binary_op(lhs, rhs, operator, lhs_type) {
                Some((result, result_type)) => {
                    let value = dfg.make_constant(result, result_type);
                    SimplifyResult::SimplifiedTo(value)
                }
                None => SimplifyResult::SimplifiedToInstruction(simplified),
            };
        }

        let lhs_is_zero = lhs_const.map_or(false, |lhs| lhs.is_zero());
        let rhs_is_zero = rhs_const.map_or(false, |rhs| rhs.is_zero());

        let lhs_is_one = lhs_const.map_or(false, |lhs| lhs.is_one());
        let rhs_is_one = rhs_const.map_or(false, |rhs| rhs.is_one());

        match self.operator {
            BinaryOp::Add { .. } => {
                if lhs_is_zero {
                    return SimplifyResult::SimplifiedTo(rhs);
                }
                if rhs_is_zero {
                    return SimplifyResult::SimplifiedTo(lhs);
                }
                if let Some(instruction) =
                    self.simplify_consecutive(lhs, lhs_const, rhs_const, lhs_type, dfg)
                {
                    return SimplifyResult::SimplifiedToInstruction(instruction);
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
                if let Some(instruction) =
                    self.simplify_consecutive(lhs, lhs_const, rhs_const, lhs_type, dfg)
                {
                    return SimplifyResult::SimplifiedToInstruction(instruction);
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
                    if let Value::Instruction { instruction, .. } = &dfg[rhs] {
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
                    if let Value::Instruction { instruction, .. } = &dfg[lhs] {
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
                if let Some(instruction) =
                    self.simplify_consecutive(lhs, lhs_const, rhs_const, lhs_type, dfg)
                {
                    return SimplifyResult::SimplifiedToInstruction(instruction);
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
                    if let Some(modulus) = rhs_const {
                        let modulus = modulus.to_u128();
                        if modulus.is_power_of_two() {
                            let bit_size = modulus.ilog2();
                            return SimplifyResult::SimplifiedToInstruction(
                                Instruction::Truncate {
                                    value: lhs,
                                    bit_size,
                                    max_bit_size: lhs_type.bit_size(),
                                },
                            );
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
                    let instruction =
                        Instruction::binary(BinaryOp::Mul { unchecked: true }, lhs, rhs);
                    return SimplifyResult::SimplifiedToInstruction(instruction);
                }
                if lhs_type.is_unsigned() {
                    // It's common in other programming languages to truncate values to a certain bit size using
                    // a bitwise AND with a bit mask. However this operation is quite inefficient inside a snark.
                    //
                    // We then replace this bitwise operation with an equivalent truncation instruction.
                    match (lhs_const, rhs_const) {
                        (Some(bitmask), None) | (None, Some(bitmask)) => {
                            // This substitution requires the bitmask to retain all of the lower bits.
                            // The bitmask must then be one less than a power of 2.
                            let bitmask_plus_one = bitmask.to_u128() + 1;
                            if bitmask_plus_one.is_power_of_two() {
                                let value = if lhs_const.is_some() { rhs } else { lhs };
                                let num_bits = bitmask_plus_one.ilog2();
                                return SimplifyResult::SimplifiedToInstruction(
                                    Instruction::Truncate {
                                        value,
                                        bit_size: num_bits,
                                        max_bit_size: lhs_type.bit_size(),
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
                if let Some(rhs_const) = rhs_const {
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

    fn simplify_consecutive(
        &self,
        lhs: ValueId,
        lhs_const: Option<FieldElement>,
        rhs_const: Option<FieldElement>,
        typ: NumericType,
        dfg: &mut DataFlowGraph,
    ) -> Option<Instruction> {
        let (None, Some(rhs_const)) = (lhs_const, rhs_const) else {
            return None;
        };

        let Value::Instruction { instruction, .. } = &dfg[lhs] else {
            return None;
        };

        let instruction = &dfg[*instruction];
        let Instruction::Binary(Binary { lhs: lhs2, rhs: rhs2, operator }) = instruction else {
            return None;
        };

        if operator != &self.operator {
            return None;
        }

        let lhs2 = dfg.resolve(*lhs2);
        let rhs2 = dfg.resolve(*rhs2);

        let lhs2_const = dfg.get_numeric_constant(lhs2);
        let rhs2_const = dfg.get_numeric_constant(rhs2);

        let (None, Some(rhs2_const)) = (lhs2_const, rhs2_const) else {
            return None;
        };

        let new_const = match self.operator {
            BinaryOp::Add { .. } => rhs_const + rhs2_const,
            BinaryOp::Sub { .. } => rhs_const + rhs2_const,
            BinaryOp::Mul { .. } => rhs_const * rhs2_const,
            BinaryOp::Div
            | BinaryOp::Mod
            | BinaryOp::Eq
            | BinaryOp::Lt
            | BinaryOp::And
            | BinaryOp::Or
            | BinaryOp::Xor
            | BinaryOp::Shl
            | BinaryOp::Shr => {
                unreachable!("simplify_consecutive shouldn't be called for {}", self.operator)
            }
        };
        let new_const = dfg.make_constant(new_const, typ);
        Some(Instruction::Binary(Binary { lhs: lhs2, rhs: new_const, operator: self.operator }))
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
            BinaryOp::Add { unchecked: false } => {
                if std::cmp::max(max_lhs_bits, max_rhs_bits) < bit_size {
                    // `lhs` and `rhs` have both been casted up from smaller types and so cannot overflow.
                    return None;
                }
                "attempt to add with overflow"
            }
            BinaryOp::Sub { unchecked: false } => {
                if dfg.is_constant(self.lhs) && max_lhs_bits > max_rhs_bits {
                    // `lhs` is a fixed constant and `rhs` is restricted such that `lhs - rhs > 0`
                    // Note strict inequality as `rhs > lhs` while `max_lhs_bits == max_rhs_bits` is possible.
                    return None;
                }
                "attempt to subtract with overflow"
            }
            BinaryOp::Mul { unchecked: false } => {
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
            BinaryOp::Add { .. } => Some(std::ops::Add::add),
            BinaryOp::Sub { .. } => Some(std::ops::Sub::sub),
            BinaryOp::Mul { .. } => Some(std::ops::Mul::mul),
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
            BinaryOp::Add { .. } => u128::checked_add,
            BinaryOp::Sub { .. } => u128::checked_sub,
            BinaryOp::Mul { .. } => u128::checked_mul,
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
            BinaryOp::Add { .. } => i128::checked_add,
            BinaryOp::Sub { .. } => i128::checked_sub,
            BinaryOp::Mul { .. } => i128::checked_mul,
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

    use crate::ssa::{opt::assert_normalized_ssa_equals, ssa_gen::Ssa};

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

    #[test]
    fn simplifies_consecutive_additions() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field):
            v2 = add v0, Field 1
            v4 = add v2, Field 2
            return v4
        }
        ";
        let ssa = Ssa::from_str_simplifying(src).unwrap();

        let expected = "
        acir(inline) fn main f0 {
          b0(v0: Field):
            v2 = add v0, Field 1
            v4 = add v0, Field 3
            return v4
        }";
        assert_normalized_ssa_equals(ssa, expected);
    }

    #[test]
    fn simplifies_consecutive_subtractions() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field):
            v2 = sub v0, Field 1
            v4 = sub v2, Field 2
            return v4
        }
        ";
        let ssa = Ssa::from_str_simplifying(src).unwrap();

        let expected = "
        acir(inline) fn main f0 {
          b0(v0: Field):
            v2 = sub v0, Field 1
            v4 = sub v0, Field 3
            return v4
        }";
        assert_normalized_ssa_equals(ssa, expected);
    }

    #[test]
    fn simplifies_consecutive_multiplications() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field):
            v2 = mul v0, Field 2
            v4 = mul v2, Field 3
            return v4
        }
        ";
        let ssa = Ssa::from_str_simplifying(src).unwrap();

        let expected = "
        acir(inline) fn main f0 {
          b0(v0: Field):
            v2 = mul v0, Field 2
            v4 = mul v0, Field 6
            return v4
        }";
        assert_normalized_ssa_equals(ssa, expected);
    }
}
