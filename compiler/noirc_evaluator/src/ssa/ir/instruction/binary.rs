use acvm::{FieldElement, acir::AcirField};
use num_traits::ToPrimitive as _;
use serde::{Deserialize, Serialize};

use super::{InstructionResultType, NumericType, Type, ValueId};

/// Binary Operations allowed in the IR.
/// Aside from the comparison operators (Eq and Lt), all operators
/// will return the same type as their operands.
/// The operand types must match for all binary operators.
/// All binary operators are also only for numeric types. To implement
/// e.g. equality for a compound type like a struct, one must add a
/// separate Eq operation for each field and combine them later with And.
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum BinaryOp {
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
pub struct Binary {
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
}

#[derive(Debug)]
pub(crate) enum BinaryEvaluationResult {
    /// The binary operation could not be evaluated
    CouldNotEvaluate,
    /// The binary operation could be evaluated and it was successful
    Success(FieldElement, NumericType),
    /// The binary operation could be evaluated but it is guaranteed to fail
    /// (for example: overflow or division by zero).
    Failure(String),
}

/// Evaluate a binary operation with constant arguments.
pub(crate) fn eval_constant_binary_op(
    lhs: FieldElement,
    rhs: FieldElement,
    operator: BinaryOp,
    mut operand_type: NumericType,
) -> BinaryEvaluationResult {
    use BinaryEvaluationResult::{CouldNotEvaluate, Failure, Success};

    let value = match operand_type {
        NumericType::NativeField => {
            // If the rhs of a division is zero, attempting to evaluate the division will cause a compiler panic.
            // Thus, we do not evaluate the division in this method, as we want to avoid triggering a panic,
            // and the operation should be handled by ACIR generation.
            if matches!(operator, BinaryOp::Div | BinaryOp::Mod) && rhs == FieldElement::zero() {
                return Failure("attempt to divide by zero".to_string());
            }
            let Some(function) = operator.get_field_function() else {
                return CouldNotEvaluate;
            };
            function(lhs, rhs)
        }
        NumericType::Unsigned { bit_size } => {
            let function = operator.get_u128_function();

            let Some(lhs) = lhs.try_into_u128() else {
                return CouldNotEvaluate;
            };
            let Some(rhs) = rhs.try_into_u128() else {
                return CouldNotEvaluate;
            };

            let lhs = truncate(lhs, bit_size);
            let rhs = truncate(rhs, bit_size);

            // The divisor is being truncated into the type of the operand, which can potentially
            // lead to the rhs being zero.
            // If the rhs of a division is zero, attempting to evaluate the division will cause a compiler panic.
            // Thus, we do not evaluate the division in this method, as we want to avoid triggering a panic,
            // and the operation should be handled by ACIR generation.
            if matches!(operator, BinaryOp::Div | BinaryOp::Mod) && rhs == 0 {
                return Failure("attempt to divide by zero".to_string());
            }

            // Check for overflow
            if operator == BinaryOp::Shl || operator == BinaryOp::Shr {
                let op = "bit-shift";

                if rhs >= u128::from(bit_size) {
                    return Failure(format!("attempt to {op} with overflow"));
                }
            }

            let Some(result) = function(lhs, rhs) else {
                if let BinaryOp::Shl = operator {
                    return CouldNotEvaluate;
                }

                if let BinaryOp::Shr = operator {
                    return Success(FieldElement::zero(), operand_type);
                }

                let op = binary_op_function_name(operator);
                return Failure(format!("attempt to {op} with overflow"));
            };

            // Check for overflow
            if result != 0 && result.ilog2() >= bit_size {
                if let BinaryOp::Shl = operator {
                    // Right now `shl` might return zero or overflow depending on its values
                    // so don't assume the final value here.
                    // See https://github.com/noir-lang/noir/issues/9022
                    return CouldNotEvaluate;
                }

                if let BinaryOp::Shr = operator {
                    return Success(FieldElement::zero(), operand_type);
                }

                let op = binary_op_function_name(operator);
                return Failure(format!("attempt to {op} with overflow"));
            }

            result.into()
        }
        NumericType::Signed { bit_size } => {
            let function = operator.get_i128_function();

            let Some(lhs) = try_convert_field_element_to_signed_integer(lhs, bit_size) else {
                return CouldNotEvaluate;
            };
            let Some(rhs) = try_convert_field_element_to_signed_integer(rhs, bit_size) else {
                return CouldNotEvaluate;
            };

            let two_pow_bit_size_minus_one = 1i128 << (bit_size - 1);

            // Because we always perform signed operations using i128, an operation like `-128_i8 / -1`
            // will not overflow as it'll actually be done via `-128_i128 / -1`. Thus we need to
            // manually check this specific case.
            if matches!(operator, BinaryOp::Div | BinaryOp::Mod) && rhs == -1 {
                assert!(bit_size < 128);
                let min_value = -two_pow_bit_size_minus_one;
                if lhs == min_value {
                    return Failure(if operator == BinaryOp::Div {
                        "attempt to divide with overflow".to_string()
                    } else {
                        "attempt to calculate the remainder with overflow".to_string()
                    });
                }
            }

            let result = function(lhs, rhs);

            let result = {
                match operator {
                    BinaryOp::Div | BinaryOp::Mod if rhs == 0 => {
                        // The divisor is being truncated into the type of the operand, which can potentially
                        // lead to the rhs being zero.
                        // If the rhs of a division is zero, attempting to evaluate the division will cause a compiler panic.
                        // Thus, we do not evaluate the division in this method, as we want to avoid triggering a panic,
                        // and the operation should be handled by ACIR generation.
                        return Failure("attempt to divide by zero".to_string());
                    }
                    BinaryOp::Shr | BinaryOp::Shl if rhs >= i128::from(bit_size) => {
                        let op = binary_op_function_name(operator);
                        return Failure(format!("attempt to {op} with overflow"));
                    }
                    _ => (),
                }

                // Check for overflow
                let Some(result) = result else {
                    if let BinaryOp::Shl = operator {
                        return CouldNotEvaluate;
                    }

                    let op = binary_op_function_name(operator);
                    return Failure(format!("attempt to {op} with overflow"));
                };

                if result >= two_pow_bit_size_minus_one || result < -two_pow_bit_size_minus_one {
                    if let BinaryOp::Shl = operator {
                        return CouldNotEvaluate;
                    }

                    let op = binary_op_function_name(operator);
                    return Failure(format!("attempt to {op} with overflow"));
                }

                result
            };
            convert_signed_integer_to_field_element(result, bit_size)
        }
    };

    if matches!(operator, BinaryOp::Eq | BinaryOp::Lt) {
        operand_type = NumericType::bool();
    }

    Success(value, operand_type)
}

fn binary_op_function_name(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add { .. } => "add",
        BinaryOp::Sub { .. } => "subtract",
        BinaryOp::Mul { .. } => "multiply",
        BinaryOp::Div => "divide",
        BinaryOp::Mod => "modulo",
        BinaryOp::Shl => "shift left",
        BinaryOp::Shr => "shift right",
        BinaryOp::Eq | BinaryOp::Lt | BinaryOp::And | BinaryOp::Or | BinaryOp::Xor => {
            panic!("Shouldn't need binary op function name of {op}")
        }
    }
}

/// Values in the range `[0, 2^(bit_size-1))` are interpreted as positive integers
///
/// Values in the range `[2^(bit_size-1), 2^bit_size)` are interpreted as negative integers.
pub(crate) fn try_convert_field_element_to_signed_integer(
    field: FieldElement,
    bit_size: u32,
) -> Option<i128> {
    let unsigned_int = truncate(field.try_into_u128()?, bit_size);

    let max_positive_value = 1 << (bit_size - 1);
    let is_positive = unsigned_int < max_positive_value;

    let signed_int = if is_positive {
        unsigned_int as i128
    } else {
        assert!(bit_size < 128);
        let x = (1u128 << bit_size) - unsigned_int;
        -(x as i128)
    };

    Some(signed_int)
}

pub(crate) fn convert_signed_integer_to_field_element(int: i128, bit_size: u32) -> FieldElement {
    if int >= 0 {
        FieldElement::from(int)
    } else if bit_size == 128 {
        // signed to u128 conversion
        FieldElement::from(int as u128)
    } else {
        let two_complement = match bit_size {
            8 => u128::from((int as i8) as u8),
            16 => u128::from((int as i16) as u16),
            32 => u128::from((int as i32) as u32),
            64 => u128::from((int as i64) as u64),
            _ => unreachable!("ICE - invalid bit size {bit_size} for signed integer"),
        };
        FieldElement::from(two_complement)
    }
}

/// Truncates `int` to fit within `bit_size` bits.
pub(crate) fn truncate(int: u128, bit_size: u32) -> u128 {
    if bit_size == 128 {
        int
    } else {
        let max = 1 << bit_size;
        int % max
    }
}

pub(crate) fn truncate_field<F: AcirField>(int: F, bit_size: u32) -> F {
    if bit_size == 0 {
        return F::zero();
    }
    let num_bytes = bit_size.div_ceil(8);
    let mut be_bytes: Vec<u8> =
        int.to_be_bytes().into_iter().rev().take(num_bytes as usize).rev().collect();

    // We need to apply a mask to the largest byte to handle non-divisible bit sizes.
    let mask = match bit_size % 8 {
        0 => 0xff,
        1 => 0x01,
        2 => 0x03,
        3 => 0x07,
        4 => 0x0f,
        5 => 0x1f,
        6 => 0x3f,
        7 => 0x7f,
        _ => unreachable!("We cover the full range of x % 8"),
    };
    be_bytes[0] &= mask;

    F::from_be_bytes_reduce(&be_bytes)
}

impl BinaryOp {
    fn get_field_function(self) -> Option<fn(FieldElement, FieldElement) -> FieldElement> {
        match self {
            BinaryOp::Add { .. } => Some(std::ops::Add::add),
            BinaryOp::Sub { .. } => Some(std::ops::Sub::sub),
            BinaryOp::Mul { .. } => Some(std::ops::Mul::mul),
            BinaryOp::Div => Some(std::ops::Div::div),
            BinaryOp::Eq => Some(|x, y| (x == y).into()),
            // "less then" comparison is not supported for Fields
            BinaryOp::Lt => None,
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
            BinaryOp::Eq => |x, y| Some(u128::from(x == y)),
            BinaryOp::Lt => |x, y| Some(u128::from(x < y)),
            BinaryOp::Shl => |x, y| y.to_u32().and_then(|y| x.checked_shl(y)),
            BinaryOp::Shr => |x, y| y.to_u32().and_then(|y| x.checked_shr(y)),
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
            BinaryOp::Eq => |x, y| Some(i128::from(x == y)),
            BinaryOp::Lt => |x, y| Some(i128::from(x < y)),
            BinaryOp::Shl => |x, y| y.to_u32().and_then(|y| x.checked_shl(y)),
            BinaryOp::Shr => |x, y| y.to_u32().and_then(|y| x.checked_shr(y)),
        }
    }

    pub(crate) fn into_unchecked(self) -> Self {
        match self {
            BinaryOp::Add { .. } => BinaryOp::Add { unchecked: true },
            BinaryOp::Sub { .. } => BinaryOp::Sub { unchecked: true },
            BinaryOp::Mul { .. } => BinaryOp::Mul { unchecked: true },
            _ => self,
        }
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::{
        BinaryOp, convert_signed_integer_to_field_element, truncate_field,
        try_convert_field_element_to_signed_integer,
    };
    use acvm::{AcirField, FieldElement};
    use num_bigint::BigUint;
    use num_traits::One;

    proptest! {
        #[test]
        fn signed_int_roundtrip(int: i128, bit_size in 0usize..=3) {
            let bit_sizes = [8,16,32,64];
            let bit_size = bit_sizes[bit_size];
            let int = int % (1i128 << (bit_size - 1));

            let int_as_field = convert_signed_integer_to_field_element(int, bit_size);
            let recovered_int = try_convert_field_element_to_signed_integer(int_as_field, bit_size).unwrap();

            prop_assert_eq!(int, recovered_int);
        }

        #[test]
        fn truncate_field_agrees_with_bigint_modulo(input: u128, bit_size in (0..=253u32)) {
            let field = FieldElement::from(input);
            let truncated_as_field = truncate_field(field, bit_size);

            let integer_modulus = BigUint::from(2_u128).pow(bit_size);
            let truncated_as_bigint = BigUint::from(input)
                        .modpow(&BigUint::one(), &integer_modulus); // cSpell:disable-line
            let truncated_as_bigint = FieldElement::from_be_bytes_reduce(&truncated_as_bigint.to_bytes_be());
            prop_assert_eq!(truncated_as_field, truncated_as_bigint);
        }
    }

    #[test]
    fn get_u128_function_shift_works_with_values_larger_than_127() {
        assert!(BinaryOp::Shr.get_u128_function()(1, 128).is_none());
        assert!(BinaryOp::Shl.get_u128_function()(1, 128).is_none());
    }

    #[test]
    fn get_i128_function_shift_works_with_values_larger_than_127() {
        assert!(BinaryOp::Shr.get_i128_function()(1, 128).is_none());
        assert!(BinaryOp::Shl.get_i128_function()(1, 128).is_none());
    }

    #[test]
    fn test_plus_minus_one_as_field() {
        for (i, u) in [(-1i64, u64::MAX), (-2i64, u64::MAX - 1), (1i64, 1u64)] {
            let i: i128 = i.into();
            let f = convert_signed_integer_to_field_element(i, 64);
            assert_eq!(f.to_u128(), u128::from(u));
            assert_eq!(i, try_convert_field_element_to_signed_integer(f, 64).unwrap());
        }
    }
}
