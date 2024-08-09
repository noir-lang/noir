use acir::brillig::{BinaryFieldOp, BinaryIntOp, IntegerBitSize};
use acir::AcirField;
use num_bigint::BigUint;

use crate::memory::{MemoryTypeError, MemoryValue};

#[derive(Debug, thiserror::Error)]
pub(crate) enum BrilligArithmeticError {
    #[error("Bit size for lhs {lhs_bit_size} does not match op bit size {op_bit_size}")]
    MismatchedLhsBitSize { lhs_bit_size: u32, op_bit_size: u32 },
    #[error("Bit size for rhs {rhs_bit_size} does not match op bit size {op_bit_size}")]
    MismatchedRhsBitSize { rhs_bit_size: u32, op_bit_size: u32 },
    #[error("Attempted to divide by zero")]
    DivisionByZero,
}

/// Evaluate a binary operation on two FieldElement memory values.
pub(crate) fn evaluate_binary_field_op<F: AcirField>(
    op: &BinaryFieldOp,
    lhs: MemoryValue<F>,
    rhs: MemoryValue<F>,
) -> Result<MemoryValue<F>, BrilligArithmeticError> {
    let a = match lhs {
        MemoryValue::Field(a) => a,
        MemoryValue::Integer(_, bit_size) => {
            return Err(BrilligArithmeticError::MismatchedLhsBitSize {
                lhs_bit_size: bit_size.into(),
                op_bit_size: F::max_num_bits(),
            });
        }
    };
    let b = match rhs {
        MemoryValue::Field(b) => b,
        MemoryValue::Integer(_, bit_size) => {
            return Err(BrilligArithmeticError::MismatchedRhsBitSize {
                rhs_bit_size: bit_size.into(),
                op_bit_size: F::max_num_bits(),
            });
        }
    };

    Ok(match op {
        // Perform addition, subtraction, multiplication, and division based on the BinaryOp variant.
        BinaryFieldOp::Add => MemoryValue::new_field(a + b),
        BinaryFieldOp::Sub => MemoryValue::new_field(a - b),
        BinaryFieldOp::Mul => MemoryValue::new_field(a * b),
        BinaryFieldOp::Div => MemoryValue::new_field(a / b),
        BinaryFieldOp::IntegerDiv => {
            if b.is_zero() {
                return Err(BrilligArithmeticError::DivisionByZero);
            } else {
                let a_big = BigUint::from_bytes_be(&a.to_be_bytes());
                let b_big = BigUint::from_bytes_be(&b.to_be_bytes());

                let result = a_big / b_big;
                MemoryValue::new_field(F::from_be_bytes_reduce(&result.to_bytes_be()))
            }
        }
        BinaryFieldOp::Equals => (a == b).into(),
        BinaryFieldOp::LessThan => (a < b).into(),
        BinaryFieldOp::LessThanEquals => (a <= b).into(),
    })
}

/// Evaluate a binary operation on two unsigned big integers with a given bit size.
pub(crate) fn evaluate_binary_int_op<F: AcirField>(
    op: &BinaryIntOp,
    lhs: MemoryValue<F>,
    rhs: MemoryValue<F>,
    bit_size: IntegerBitSize,
) -> Result<MemoryValue<F>, BrilligArithmeticError> {
    let lhs = lhs.expect_integer_with_bit_size(bit_size).map_err(|err| match err {
        MemoryTypeError::MismatchedBitSize { value_bit_size, expected_bit_size } => {
            BrilligArithmeticError::MismatchedLhsBitSize {
                lhs_bit_size: value_bit_size,
                op_bit_size: expected_bit_size,
            }
        }
    })?;

    let rhs_bit_size = if op == &BinaryIntOp::Shl || op == &BinaryIntOp::Shr {
        IntegerBitSize::U8
    } else {
        bit_size
    };

    let rhs = rhs.expect_integer_with_bit_size(rhs_bit_size).map_err(|err| match err {
        MemoryTypeError::MismatchedBitSize { value_bit_size, expected_bit_size } => {
            BrilligArithmeticError::MismatchedRhsBitSize {
                rhs_bit_size: value_bit_size,
                op_bit_size: expected_bit_size,
            }
        }
    })?;

    let result = if bit_size == IntegerBitSize::U128 {
        evaluate_binary_int_op_128(op, lhs, rhs)?
    } else {
        evaluate_binary_int_op_generic(op, lhs, rhs, bit_size)?
    };

    Ok(match op {
        BinaryIntOp::Equals | BinaryIntOp::LessThan | BinaryIntOp::LessThanEquals => {
            MemoryValue::new_integer(result, IntegerBitSize::U1)
        }
        _ => MemoryValue::new_integer(result, bit_size),
    })
}

fn evaluate_binary_int_op_128(
    op: &BinaryIntOp,
    lhs: u128,
    rhs: u128,
) -> Result<u128, BrilligArithmeticError> {
    let result = match op {
        BinaryIntOp::Add => lhs.wrapping_add(rhs),
        BinaryIntOp::Sub => lhs.wrapping_sub(rhs),
        BinaryIntOp::Mul => lhs.wrapping_mul(rhs),
        BinaryIntOp::Div => {
            if rhs == 0 {
                return Err(BrilligArithmeticError::DivisionByZero);
            } else {
                lhs / rhs
            }
        }
        BinaryIntOp::Equals => (lhs == rhs) as u128,
        BinaryIntOp::LessThan => (lhs < rhs) as u128,
        BinaryIntOp::LessThanEquals => (lhs <= rhs) as u128,
        BinaryIntOp::And => lhs & rhs,
        BinaryIntOp::Or => lhs | rhs,
        BinaryIntOp::Xor => lhs ^ rhs,
        BinaryIntOp::Shl => {
            if rhs >= 128 {
                0
            } else {
                lhs.wrapping_shl(rhs as u32)
            }
        }
        BinaryIntOp::Shr => {
            if rhs >= 128 {
                0
            } else {
                lhs.wrapping_shr(rhs as u32)
            }
        }
    };
    Ok(result)
}

fn evaluate_binary_int_op_generic(
    op: &BinaryIntOp,
    lhs: u128,
    rhs: u128,
    bit_size: IntegerBitSize,
) -> Result<u128, BrilligArithmeticError> {
    let bit_size: u32 = bit_size.into();
    let bit_modulo = 1 << bit_size;
    let result = match op {
        // Perform addition, subtraction, and multiplication, applying a modulo operation to keep the result within the bit size.
        BinaryIntOp::Add => (lhs + rhs) % bit_modulo,
        BinaryIntOp::Sub => (bit_modulo + lhs - rhs) % bit_modulo,
        BinaryIntOp::Mul => (lhs * rhs) % bit_modulo,
        // Perform unsigned division using the modulo operation on a and b.
        BinaryIntOp::Div => {
            if rhs == 0 {
                return Err(BrilligArithmeticError::DivisionByZero);
            } else {
                lhs / rhs
            }
        }
        // Perform a == operation, returning 0 or 1
        BinaryIntOp::Equals => (lhs == rhs) as u128,
        // Perform a < operation, returning 0 or 1
        BinaryIntOp::LessThan => (lhs < rhs) as u128,
        // Perform a <= operation, returning 0 or 1
        BinaryIntOp::LessThanEquals => (lhs <= rhs) as u128,
        // Perform bitwise AND, OR, XOR, left shift, and right shift operations, applying a modulo operation to keep the result within the bit size.
        BinaryIntOp::And => lhs & rhs,
        BinaryIntOp::Or => lhs | rhs,
        BinaryIntOp::Xor => lhs ^ rhs,
        BinaryIntOp::Shl => {
            if rhs >= (bit_size as u128) {
                0
            } else {
                (lhs << rhs) % bit_modulo
            }
        }
        BinaryIntOp::Shr => {
            if rhs >= (bit_size as u128) {
                0
            } else {
                lhs >> rhs
            }
        }
    };
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use acir::{AcirField, FieldElement};

    struct TestParams {
        a: u128,
        b: u128,
        result: u128,
    }

    fn evaluate_u128(op: &BinaryIntOp, a: u128, b: u128, bit_size: IntegerBitSize) -> u128 {
        let result_value: MemoryValue<FieldElement> = evaluate_binary_int_op(
            op,
            MemoryValue::new_integer(a, bit_size),
            MemoryValue::new_integer(b, bit_size),
            bit_size,
        )
        .unwrap();
        // Convert back to u128
        result_value.to_field().to_u128()
    }

    fn to_negative(a: u128, bit_size: IntegerBitSize) -> u128 {
        assert!(a > 0);
        if bit_size == IntegerBitSize::U128 {
            0_u128.wrapping_sub(a)
        } else {
            let two_pow = 2_u128.pow(bit_size.into());
            two_pow - a
        }
    }

    fn evaluate_int_ops(test_params: Vec<TestParams>, op: BinaryIntOp, bit_size: IntegerBitSize) {
        for test in test_params {
            assert_eq!(evaluate_u128(&op, test.a, test.b, bit_size), test.result);
        }
    }

    #[test]
    fn add_test() {
        let bit_size = IntegerBitSize::U8;

        let test_ops = vec![
            TestParams { a: 50, b: 100, result: 150 },
            TestParams { a: 250, b: 10, result: 4 },
            TestParams { a: 5, b: to_negative(3, bit_size), result: 2 },
            TestParams { a: to_negative(3, bit_size), b: 1, result: to_negative(2, bit_size) },
            TestParams { a: 5, b: to_negative(6, bit_size), result: to_negative(1, bit_size) },
        ];
        evaluate_int_ops(test_ops, BinaryIntOp::Add, bit_size);

        let bit_size = IntegerBitSize::U128;
        let test_ops = vec![
            TestParams { a: 5, b: to_negative(3, bit_size), result: 2 },
            TestParams { a: to_negative(3, bit_size), b: 1, result: to_negative(2, bit_size) },
        ];

        evaluate_int_ops(test_ops, BinaryIntOp::Add, bit_size);
    }

    #[test]
    fn sub_test() {
        let bit_size = IntegerBitSize::U8;

        let test_ops = vec![
            TestParams { a: 50, b: 30, result: 20 },
            TestParams { a: 5, b: 10, result: to_negative(5, bit_size) },
            TestParams { a: 5, b: to_negative(3, bit_size), result: 8 },
            TestParams { a: to_negative(3, bit_size), b: 2, result: to_negative(5, bit_size) },
            TestParams { a: 254, b: to_negative(3, bit_size), result: 1 },
        ];
        evaluate_int_ops(test_ops, BinaryIntOp::Sub, bit_size);

        let bit_size = IntegerBitSize::U128;

        let test_ops = vec![
            TestParams { a: 5, b: 10, result: to_negative(5, bit_size) },
            TestParams { a: to_negative(3, bit_size), b: 2, result: to_negative(5, bit_size) },
        ];
        evaluate_int_ops(test_ops, BinaryIntOp::Sub, bit_size);
    }

    #[test]
    fn mul_test() {
        let bit_size = IntegerBitSize::U8;

        let test_ops = vec![
            TestParams { a: 5, b: 3, result: 15 },
            TestParams { a: 5, b: 100, result: 244 },
            TestParams { a: to_negative(1, bit_size), b: to_negative(5, bit_size), result: 5 },
            TestParams { a: to_negative(1, bit_size), b: 5, result: to_negative(5, bit_size) },
            TestParams { a: to_negative(2, bit_size), b: 7, result: to_negative(14, bit_size) },
        ];

        evaluate_int_ops(test_ops, BinaryIntOp::Mul, bit_size);

        let bit_size = IntegerBitSize::U64;
        let a = 2_u128.pow(bit_size.into()) - 1;
        let b = 3;

        // ( 2**(n-1) - 1 ) * 3 = 2*2**(n-1) - 2 + (2**(n-1) - 1) => wraps to (2**(n-1) - 1) - 2
        assert_eq!(evaluate_u128(&BinaryIntOp::Mul, a, b, bit_size), a - 2);

        let bit_size = IntegerBitSize::U128;

        let test_ops = vec![
            TestParams { a: to_negative(1, bit_size), b: to_negative(5, bit_size), result: 5 },
            TestParams { a: to_negative(1, bit_size), b: 5, result: to_negative(5, bit_size) },
            TestParams { a: to_negative(2, bit_size), b: 7, result: to_negative(14, bit_size) },
        ];

        evaluate_int_ops(test_ops, BinaryIntOp::Mul, bit_size);
    }

    #[test]
    fn div_test() {
        let bit_size = IntegerBitSize::U8;

        let test_ops =
            vec![TestParams { a: 5, b: 3, result: 1 }, TestParams { a: 5, b: 10, result: 0 }];

        evaluate_int_ops(test_ops, BinaryIntOp::Div, bit_size);
    }
}
