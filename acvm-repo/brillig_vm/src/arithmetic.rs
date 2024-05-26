use acir::brillig::{BinaryFieldOp, BinaryIntOp};
use acir::AcirField;
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use num_traits::{One, Zero};

use crate::memory::{MemoryTypeError, MemoryValue};

#[derive(Debug, thiserror::Error)]
pub(crate) enum BrilligArithmeticError {
    #[error("Bit size for lhs {lhs_bit_size} does not match op bit size {op_bit_size}")]
    MismatchedLhsBitSize { lhs_bit_size: u32, op_bit_size: u32 },
    #[error("Bit size for rhs {rhs_bit_size} does not match op bit size {op_bit_size}")]
    MismatchedRhsBitSize { rhs_bit_size: u32, op_bit_size: u32 },
    #[error("Integer operation BinaryIntOp::{op:?} is not supported on FieldElement")]
    IntegerOperationOnField { op: BinaryIntOp },
    #[error("Shift with bit size {op_bit_size} is invalid")]
    InvalidShift { op_bit_size: u32 },
}

/// Evaluate a binary operation on two FieldElement memory values.
pub(crate) fn evaluate_binary_field_op<F: AcirField>(
    op: &BinaryFieldOp,
    lhs: MemoryValue<F>,
    rhs: MemoryValue<F>,
) -> Result<MemoryValue<F>, BrilligArithmeticError> {
    let MemoryValue::Field(a) = lhs else {
        return Err(BrilligArithmeticError::MismatchedLhsBitSize {
            lhs_bit_size: lhs.bit_size(),
            op_bit_size: F::max_num_bits(),
        });
    };
    let MemoryValue::Field(b) = rhs else {
        return Err(BrilligArithmeticError::MismatchedLhsBitSize {
            lhs_bit_size: rhs.bit_size(),
            op_bit_size: F::max_num_bits(),
        });
    };

    Ok(match op {
        // Perform addition, subtraction, multiplication, and division based on the BinaryOp variant.
        BinaryFieldOp::Add => MemoryValue::new_field(a + b),
        BinaryFieldOp::Sub => MemoryValue::new_field(a - b),
        BinaryFieldOp::Mul => MemoryValue::new_field(a * b),
        BinaryFieldOp::Div => MemoryValue::new_field(a / b),
        BinaryFieldOp::IntegerDiv => {
            let a_big = BigUint::from_bytes_be(&a.to_be_bytes());
            let b_big = BigUint::from_bytes_be(&b.to_be_bytes());

            let result = a_big / b_big;
            MemoryValue::new_field(F::from_be_bytes_reduce(&result.to_bytes_be()))
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
    bit_size: u32,
) -> Result<MemoryValue<F>, BrilligArithmeticError> {
    let lhs = lhs.expect_integer_with_bit_size(bit_size).map_err(|err| match err {
        MemoryTypeError::MismatchedBitSize { value_bit_size, expected_bit_size } => {
            BrilligArithmeticError::MismatchedLhsBitSize {
                lhs_bit_size: value_bit_size,
                op_bit_size: expected_bit_size,
            }
        }
    })?;
    let rhs_bit_size =
        if op == &BinaryIntOp::Shl || op == &BinaryIntOp::Shr { 8 } else { bit_size };
    let rhs = rhs.expect_integer_with_bit_size(rhs_bit_size).map_err(|err| match err {
        MemoryTypeError::MismatchedBitSize { value_bit_size, expected_bit_size } => {
            BrilligArithmeticError::MismatchedRhsBitSize {
                rhs_bit_size: value_bit_size,
                op_bit_size: expected_bit_size,
            }
        }
    })?;

    if bit_size == F::max_num_bits() {
        return Err(BrilligArithmeticError::IntegerOperationOnField { op: *op });
    }

    let bit_modulo = &(BigUint::one() << bit_size);
    let result = match op {
        // Perform addition, subtraction, and multiplication, applying a modulo operation to keep the result within the bit size.
        BinaryIntOp::Add => (lhs + rhs) % bit_modulo,
        BinaryIntOp::Sub => (bit_modulo + lhs - rhs) % bit_modulo,
        BinaryIntOp::Mul => (lhs * rhs) % bit_modulo,
        // Perform unsigned division using the modulo operation on a and b.
        BinaryIntOp::Div => {
            if rhs.is_zero() {
                BigUint::zero()
            } else {
                lhs / rhs
            }
        }
        // Perform a == operation, returning 0 or 1
        BinaryIntOp::Equals => {
            if lhs == rhs {
                BigUint::one()
            } else {
                BigUint::zero()
            }
        }
        // Perform a < operation, returning 0 or 1
        BinaryIntOp::LessThan => {
            if lhs < rhs {
                BigUint::one()
            } else {
                BigUint::zero()
            }
        }
        // Perform a <= operation, returning 0 or 1
        BinaryIntOp::LessThanEquals => {
            if lhs <= rhs {
                BigUint::one()
            } else {
                BigUint::zero()
            }
        }
        // Perform bitwise AND, OR, XOR, left shift, and right shift operations, applying a modulo operation to keep the result within the bit size.
        BinaryIntOp::And => lhs & rhs,
        BinaryIntOp::Or => lhs | rhs,
        BinaryIntOp::Xor => lhs ^ rhs,
        BinaryIntOp::Shl => {
            if bit_size > 128 {
                return Err(BrilligArithmeticError::InvalidShift { op_bit_size: bit_size });
            }
            let rhs = rhs.to_u128().unwrap();
            (lhs << rhs) % bit_modulo
        }
        BinaryIntOp::Shr => {
            if bit_size > 128 {
                return Err(BrilligArithmeticError::InvalidShift { op_bit_size: bit_size });
            }
            let rhs = rhs.to_u128().unwrap();
            lhs >> rhs
        }
    };

    Ok(match op {
        BinaryIntOp::Equals | BinaryIntOp::LessThan | BinaryIntOp::LessThanEquals => {
            MemoryValue::new_integer(result, 1)
        }
        _ => MemoryValue::new_integer(result, bit_size),
    })
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

    fn evaluate_u128(op: &BinaryIntOp, a: u128, b: u128, bit_size: u32) -> u128 {
        let result_value: MemoryValue<FieldElement> = evaluate_binary_int_op(
            op,
            MemoryValue::new_integer(a.into(), bit_size),
            MemoryValue::new_integer(b.into(), bit_size),
            bit_size,
        )
        .unwrap();
        // Convert back to u128
        result_value.to_field().to_u128()
    }

    fn to_negative(a: u128, bit_size: u32) -> u128 {
        assert!(a > 0);
        let two_pow = 2_u128.pow(bit_size);
        two_pow - a
    }

    fn evaluate_int_ops(test_params: Vec<TestParams>, op: BinaryIntOp, bit_size: u32) {
        for test in test_params {
            assert_eq!(evaluate_u128(&op, test.a, test.b, bit_size), test.result);
        }
    }

    #[test]
    fn add_test() {
        let bit_size = 4;

        let test_ops = vec![
            TestParams { a: 5, b: 10, result: 15 },
            TestParams { a: 10, b: 10, result: 4 },
            TestParams { a: 5, b: to_negative(3, bit_size), result: 2 },
            TestParams { a: to_negative(3, bit_size), b: 1, result: to_negative(2, bit_size) },
            TestParams { a: 5, b: to_negative(6, bit_size), result: to_negative(1, bit_size) },
        ];

        evaluate_int_ops(test_ops, BinaryIntOp::Add, bit_size);
    }

    #[test]
    fn sub_test() {
        let bit_size = 4;

        let test_ops = vec![
            TestParams { a: 5, b: 3, result: 2 },
            TestParams { a: 5, b: 10, result: to_negative(5, bit_size) },
            TestParams { a: 5, b: to_negative(3, bit_size), result: 8 },
            TestParams { a: to_negative(3, bit_size), b: 2, result: to_negative(5, bit_size) },
            TestParams { a: 14, b: to_negative(3, bit_size), result: 1 },
        ];

        evaluate_int_ops(test_ops, BinaryIntOp::Sub, bit_size);
    }

    #[test]
    fn mul_test() {
        let bit_size = 4;

        let test_ops = vec![
            TestParams { a: 5, b: 3, result: 15 },
            TestParams { a: 5, b: 10, result: 2 },
            TestParams { a: to_negative(1, bit_size), b: to_negative(5, bit_size), result: 5 },
            TestParams { a: to_negative(1, bit_size), b: 5, result: to_negative(5, bit_size) },
            TestParams {
                a: to_negative(2, bit_size),
                b: 7,
                // negative 14 wraps to a 2
                result: to_negative(14, bit_size),
            },
        ];

        evaluate_int_ops(test_ops, BinaryIntOp::Mul, bit_size);

        let bit_size = 127;
        let a = 2_u128.pow(bit_size) - 1;
        let b = 3;

        // ( 2**(n-1) - 1 ) * 3 = 2*2**(n-1) - 2 + (2**(n-1) - 1) => wraps to (2**(n-1) - 1) - 2
        assert_eq!(evaluate_u128(&BinaryIntOp::Mul, a, b, bit_size), a - 2);
    }

    #[test]
    fn div_test() {
        let bit_size = 4;

        let test_ops =
            vec![TestParams { a: 5, b: 3, result: 1 }, TestParams { a: 5, b: 10, result: 0 }];

        evaluate_int_ops(test_ops, BinaryIntOp::Div, bit_size);
    }
}
