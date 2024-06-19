use acvm::{acir::AcirField, FieldElement};

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
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
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
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
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
        let operand_type = dfg.type_of_value(self.lhs);

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
                if dfg.resolve(self.lhs) == dfg.resolve(self.rhs)
                    && dfg.get_value_max_num_bits(self.lhs) == 1
                {
                    // Squaring a boolean value is a noop.
                    return SimplifyResult::SimplifiedTo(self.lhs);
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
                    let one = dfg.make_constant(FieldElement::one(), Type::bool());
                    return SimplifyResult::SimplifiedTo(one);
                }

                if operand_type == Type::bool() {
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
                    let zero = dfg.make_constant(FieldElement::zero(), Type::bool());
                    return SimplifyResult::SimplifiedTo(zero);
                }
                if operand_type.is_unsigned() {
                    if rhs_is_zero {
                        // Unsigned values cannot be less than zero.
                        let zero = dfg.make_constant(FieldElement::zero(), Type::bool());
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
                if operand_type == Type::bool() {
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

                    // `two_pow_rhs` is limited to be at most `2 ^ {operand_bitsize - 1}` so it fits in `operand_type`.
                    let two_pow_rhs = FieldElement::from(2u128).pow(&rhs_const);
                    let two_pow_rhs = dfg.make_constant(two_pow_rhs, operand_type);
                    return SimplifyResult::SimplifiedToInstruction(Instruction::binary(
                        BinaryOp::Div,
                        self.lhs,
                        two_pow_rhs,
                    ));
                }
            }
        };
        SimplifyResult::None
    }
}

/// Evaluate a binary operation with constant arguments.
fn eval_constant_binary_op(
    lhs: FieldElement,
    rhs: FieldElement,
    operator: BinaryOp,
    mut operand_type: Type,
) -> Option<(FieldElement, Type)> {
    let value = match &operand_type {
        Type::Numeric(NumericType::NativeField) => {
            // If the rhs of a division is zero, attempting to evaluate the division will cause a compiler panic.
            // Thus, we do not evaluate the division in this method, as we want to avoid triggering a panic,
            // and the operation should be handled by ACIR generation.
            if matches!(operator, BinaryOp::Div | BinaryOp::Mod) && rhs == FieldElement::zero() {
                return None;
            }
            operator.get_field_function()?(lhs, rhs)
        }
        Type::Numeric(NumericType::Unsigned { bit_size }) => {
            let function = operator.get_u128_function();

            let lhs = truncate(lhs.try_into_u128()?, *bit_size);
            let rhs = truncate(rhs.try_into_u128()?, *bit_size);

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
            if result >= 2u128.pow(*bit_size) {
                return None;
            }
            result.into()
        }
        Type::Numeric(NumericType::Signed { bit_size }) => {
            let function = operator.get_i128_function();

            let lhs = truncate(lhs.try_into_u128()?, *bit_size);
            let rhs = truncate(rhs.try_into_u128()?, *bit_size);
            let l_pos = lhs < 2u128.pow(bit_size - 1);
            let r_pos = rhs < 2u128.pow(bit_size - 1);
            let lhs = if l_pos { lhs as i128 } else { -((2u128.pow(*bit_size) - lhs) as i128) };
            let rhs = if r_pos { rhs as i128 } else { -((2u128.pow(*bit_size) - rhs) as i128) };
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
            if result >= 2i128.pow(*bit_size - 1) || result < -(2i128.pow(*bit_size - 1)) {
                return None;
            }
            let result =
                if result >= 0 { result as u128 } else { (2i128.pow(*bit_size) + result) as u128 };
            result.into()
        }
        _ => return None,
    };

    if matches!(operator, BinaryOp::Eq | BinaryOp::Lt) {
        operand_type = Type::bool();
    }

    Some((value, operand_type))
}

fn truncate(int: u128, bit_size: u32) -> u128 {
    let max = 2u128.pow(bit_size);
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
