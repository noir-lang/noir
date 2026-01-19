//! Implementations for [binary field operations][acir::brillig::Opcode::BinaryFieldOp] and
//! [binary integer operations][acir::brillig::Opcode::BinaryIntOp].
use std::ops::{BitAnd, BitOr, BitXor, Shl, Shr};

use acir::AcirField;
use acir::brillig::{BinaryFieldOp, BinaryIntOp, BitSize, IntegerBitSize};
use num_bigint::BigUint;
use num_traits::{CheckedDiv, ToPrimitive, WrappingAdd, WrappingMul, WrappingSub, Zero};

use crate::memory::{MemoryTypeError, MemoryValue};

#[derive(Debug, PartialEq, thiserror::Error)]
pub(crate) enum BrilligArithmeticError {
    #[error("Bit size for lhs {lhs_bit_size} does not match op bit size {op_bit_size}")]
    MismatchedLhsBitSize { lhs_bit_size: u32, op_bit_size: u32 },
    #[error("Bit size for rhs {rhs_bit_size} does not match op bit size {op_bit_size}")]
    MismatchedRhsBitSize { rhs_bit_size: u32, op_bit_size: u32 },
    #[error("Attempted to shift by {shift_size} bits on a type of bit size {bit_size}")]
    BitshiftOverflow { bit_size: u32, shift_size: u128 },
    #[error("Attempted to divide by zero")]
    DivisionByZero,
}

/// Evaluate a binary operation on two FieldElement memory values.
pub(crate) fn evaluate_binary_field_op<F: AcirField>(
    op: &BinaryFieldOp,
    lhs: MemoryValue<F>,
    rhs: MemoryValue<F>,
) -> Result<MemoryValue<F>, BrilligArithmeticError> {
    let expect_field = |value: MemoryValue<F>, make_err: fn(u32, u32) -> BrilligArithmeticError| {
        value.expect_field().map_err(|err| {
            if let MemoryTypeError::MismatchedBitSize { value_bit_size, expected_bit_size } = err {
                make_err(value_bit_size, expected_bit_size)
            } else {
                unreachable!("MemoryTypeError NotInteger is only produced by to_u128")
            }
        })
    };
    let a = expect_field(lhs, |vbs, ebs| BrilligArithmeticError::MismatchedLhsBitSize {
        lhs_bit_size: vbs,
        op_bit_size: ebs,
    })?;
    let b = expect_field(rhs, |vbs, ebs| BrilligArithmeticError::MismatchedRhsBitSize {
        rhs_bit_size: vbs,
        op_bit_size: ebs,
    })?;

    Ok(match op {
        // Perform addition, subtraction, multiplication, and division based on the BinaryOp variant.
        BinaryFieldOp::Add => MemoryValue::new_field(a + b),
        BinaryFieldOp::Sub => MemoryValue::new_field(a - b),
        BinaryFieldOp::Mul => MemoryValue::new_field(a * b),
        BinaryFieldOp::Div => {
            if b.is_zero() {
                return Err(BrilligArithmeticError::DivisionByZero);
            } else if b.is_one() {
                MemoryValue::new_field(a)
            } else if b == -F::one() {
                MemoryValue::new_field(-a)
            } else {
                MemoryValue::new_field(a / b)
            }
        }
        BinaryFieldOp::IntegerDiv => {
            // IntegerDiv is only meant to represent unsigned integer division.
            // The operands must be valid non-negative integers within the field's range.
            // Because AcirField is modulo the prime field, it does not natively track
            // "negative" numbers as any value is already reduced modulo the prime.
            //
            // Therefore, we do not check for negative inputs here. It is the responsibility
            // of the code generator to ensure that operands for IntegerDiv are valid unsigned integers.
            // The only runtime error we check for is division by zero.
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
    match op {
        BinaryIntOp::Add
        | BinaryIntOp::Sub
        | BinaryIntOp::Mul
        | BinaryIntOp::Div
        | BinaryIntOp::And
        | BinaryIntOp::Or
        | BinaryIntOp::Xor => match (lhs, rhs, bit_size) {
            (MemoryValue::U1(lhs), MemoryValue::U1(rhs), IntegerBitSize::U1) => {
                evaluate_binary_int_op_u1(op, lhs, rhs).map(MemoryValue::U1)
            }
            (MemoryValue::U8(lhs), MemoryValue::U8(rhs), IntegerBitSize::U8) => {
                evaluate_binary_int_op_arith(op, lhs, rhs).map(MemoryValue::U8)
            }
            (MemoryValue::U16(lhs), MemoryValue::U16(rhs), IntegerBitSize::U16) => {
                evaluate_binary_int_op_arith(op, lhs, rhs).map(MemoryValue::U16)
            }
            (MemoryValue::U32(lhs), MemoryValue::U32(rhs), IntegerBitSize::U32) => {
                evaluate_binary_int_op_arith(op, lhs, rhs).map(MemoryValue::U32)
            }
            (MemoryValue::U64(lhs), MemoryValue::U64(rhs), IntegerBitSize::U64) => {
                evaluate_binary_int_op_arith(op, lhs, rhs).map(MemoryValue::U64)
            }
            (MemoryValue::U128(lhs), MemoryValue::U128(rhs), IntegerBitSize::U128) => {
                evaluate_binary_int_op_arith(op, lhs, rhs).map(MemoryValue::U128)
            }
            (lhs, _, _) if lhs.bit_size() != BitSize::Integer(bit_size) => {
                Err(BrilligArithmeticError::MismatchedLhsBitSize {
                    lhs_bit_size: lhs.bit_size().to_u32::<F>(),
                    op_bit_size: bit_size.into(),
                })
            }
            (_, rhs, _) if rhs.bit_size() != BitSize::Integer(bit_size) => {
                Err(BrilligArithmeticError::MismatchedRhsBitSize {
                    rhs_bit_size: rhs.bit_size().to_u32::<F>(),
                    op_bit_size: bit_size.into(),
                })
            }
            _ => unreachable!("Invalid arguments are covered by the two arms above."),
        },

        BinaryIntOp::Equals | BinaryIntOp::LessThan | BinaryIntOp::LessThanEquals => {
            match (lhs, rhs, bit_size) {
                (MemoryValue::U1(lhs), MemoryValue::U1(rhs), IntegerBitSize::U1) => {
                    Ok(MemoryValue::U1(evaluate_binary_int_op_cmp(op, lhs, rhs)))
                }
                (MemoryValue::U8(lhs), MemoryValue::U8(rhs), IntegerBitSize::U8) => {
                    Ok(MemoryValue::U1(evaluate_binary_int_op_cmp(op, lhs, rhs)))
                }
                (MemoryValue::U16(lhs), MemoryValue::U16(rhs), IntegerBitSize::U16) => {
                    Ok(MemoryValue::U1(evaluate_binary_int_op_cmp(op, lhs, rhs)))
                }
                (MemoryValue::U32(lhs), MemoryValue::U32(rhs), IntegerBitSize::U32) => {
                    Ok(MemoryValue::U1(evaluate_binary_int_op_cmp(op, lhs, rhs)))
                }
                (MemoryValue::U64(lhs), MemoryValue::U64(rhs), IntegerBitSize::U64) => {
                    Ok(MemoryValue::U1(evaluate_binary_int_op_cmp(op, lhs, rhs)))
                }
                (MemoryValue::U128(lhs), MemoryValue::U128(rhs), IntegerBitSize::U128) => {
                    Ok(MemoryValue::U1(evaluate_binary_int_op_cmp(op, lhs, rhs)))
                }
                (lhs, _, _) if lhs.bit_size() != BitSize::Integer(bit_size) => {
                    Err(BrilligArithmeticError::MismatchedLhsBitSize {
                        lhs_bit_size: lhs.bit_size().to_u32::<F>(),
                        op_bit_size: bit_size.into(),
                    })
                }
                (_, rhs, _) if rhs.bit_size() != BitSize::Integer(bit_size) => {
                    Err(BrilligArithmeticError::MismatchedRhsBitSize {
                        rhs_bit_size: rhs.bit_size().to_u32::<F>(),
                        op_bit_size: bit_size.into(),
                    })
                }
                _ => unreachable!("Invalid arguments are covered by the two arms above."),
            }
        }

        BinaryIntOp::Shl | BinaryIntOp::Shr => match (lhs, rhs, bit_size) {
            (MemoryValue::U1(lhs), MemoryValue::U1(rhs), IntegerBitSize::U1) => {
                if rhs {
                    Err(BrilligArithmeticError::BitshiftOverflow { bit_size: 1, shift_size: 1 })
                } else {
                    Ok(MemoryValue::U1(lhs))
                }
            }
            (MemoryValue::U8(lhs), MemoryValue::U8(rhs), IntegerBitSize::U8) => {
                Ok(MemoryValue::U8(evaluate_binary_int_op_shifts(op, lhs, rhs)?))
            }
            (MemoryValue::U16(lhs), MemoryValue::U16(rhs), IntegerBitSize::U16) => {
                Ok(MemoryValue::U16(evaluate_binary_int_op_shifts(op, lhs, rhs)?))
            }
            (MemoryValue::U32(lhs), MemoryValue::U32(rhs), IntegerBitSize::U32) => {
                Ok(MemoryValue::U32(evaluate_binary_int_op_shifts(op, lhs, rhs)?))
            }
            (MemoryValue::U64(lhs), MemoryValue::U64(rhs), IntegerBitSize::U64) => {
                Ok(MemoryValue::U64(evaluate_binary_int_op_shifts(op, lhs, rhs)?))
            }
            (MemoryValue::U128(lhs), MemoryValue::U128(rhs), IntegerBitSize::U128) => {
                Ok(MemoryValue::U128(evaluate_binary_int_op_shifts(op, lhs, rhs)?))
            }
            _ => Err(BrilligArithmeticError::MismatchedLhsBitSize {
                lhs_bit_size: lhs.bit_size().to_u32::<F>(),
                op_bit_size: bit_size.into(),
            }),
        },
    }
}

/// Evaluates binary operations on 1-bit unsigned integers (booleans).
///
/// # Returns
/// - Ok(result) if successful.
/// - Err([BrilligArithmeticError::DivisionByZero]) if division by zero occurs.
///
/// # Panics
/// If an operation other than `Add`, `Sub`, `Mul`, `Div`, `And`, `Or`, `Xor`, `Equals`, `LessThan`,
/// or `LessThanEquals` is supplied as an argument.
fn evaluate_binary_int_op_u1(
    op: &BinaryIntOp,
    lhs: bool,
    rhs: bool,
) -> Result<bool, BrilligArithmeticError> {
    let result = match op {
        BinaryIntOp::Equals => lhs == rhs,
        BinaryIntOp::LessThan => !lhs & rhs,
        BinaryIntOp::LessThanEquals => lhs <= rhs,
        BinaryIntOp::And | BinaryIntOp::Mul => lhs & rhs,
        BinaryIntOp::Or => lhs | rhs,
        BinaryIntOp::Xor | BinaryIntOp::Add | BinaryIntOp::Sub => lhs ^ rhs,
        BinaryIntOp::Div => {
            if !rhs {
                return Err(BrilligArithmeticError::DivisionByZero);
            } else {
                lhs
            }
        }
        _ => unreachable!("Operator not handled by this function: {op:?}"),
    };
    Ok(result)
}

/// Evaluates comparison operations (`Equals`, `LessThan`, `LessThanEquals`)
/// between two values of an ordered type (e.g., fields are unordered).
///
/// # Panics
/// If an unsupported operator is provided (i.e., not `Equals`, `LessThan`, or `LessThanEquals`).
fn evaluate_binary_int_op_cmp<T: Ord + PartialEq>(op: &BinaryIntOp, lhs: T, rhs: T) -> bool {
    match op {
        BinaryIntOp::Equals => lhs == rhs,
        BinaryIntOp::LessThan => lhs < rhs,
        BinaryIntOp::LessThanEquals => lhs <= rhs,
        _ => unreachable!("Operator not handled by this function: {op:?}"),
    }
}

/// Evaluates shift operations (`Shl`, `Shr`) for unsigned integers.
/// Ensures that shifting beyond the type width returns zero.
///
/// # Returns
/// - Ok(result) if successful.
/// - Err([BrilligArithmeticError::DivisionByZero]) if the RHS is not less than the bit size.
///
/// # Panics
/// If an unsupported operator is provided (i.e., not `Shl` or `Shr`).
fn evaluate_binary_int_op_shifts<
    T: ToPrimitive + Zero + Shl<Output = T> + Shr<Output = T> + Into<u128>,
>(
    op: &BinaryIntOp,
    lhs: T,
    rhs: T,
) -> Result<T, BrilligArithmeticError> {
    let bit_size = 8 * size_of::<T>();
    let rhs_usize: usize = rhs.to_usize().expect("Could not convert rhs to usize");
    if rhs_usize >= bit_size {
        return Err(BrilligArithmeticError::BitshiftOverflow {
            bit_size: bit_size as u32,
            shift_size: rhs.into(),
        });
    }
    match op {
        BinaryIntOp::Shl => Ok(lhs << rhs),
        BinaryIntOp::Shr => Ok(lhs >> rhs),
        _ => unreachable!("Operator not handled by this function: {op:?}"),
    }
}

/// Evaluates arithmetic or bitwise operations on unsigned integer types,
/// using wrapping arithmetic for [add][BinaryIntOp::Add], [sub][BinaryIntOp::Sub], and [mul][BinaryIntOp::Mul].
///
/// # Returns
/// - Ok(result) if successful.
/// - Err([BrilligArithmeticError::DivisionByZero]) if division by zero occurs.
///
/// # Panics
/// If there an operation other than Add, Sub, Mul, Div, And, Or, Xor is supplied as an argument.
fn evaluate_binary_int_op_arith<
    T: WrappingAdd
        + WrappingSub
        + WrappingMul
        + CheckedDiv
        + BitAnd<Output = T>
        + BitOr<Output = T>
        + BitXor<Output = T>,
>(
    op: &BinaryIntOp,
    lhs: T,
    rhs: T,
) -> Result<T, BrilligArithmeticError> {
    let result = match op {
        BinaryIntOp::Add => lhs.wrapping_add(&rhs),
        BinaryIntOp::Sub => lhs.wrapping_sub(&rhs),
        BinaryIntOp::Mul => lhs.wrapping_mul(&rhs),
        BinaryIntOp::Div => lhs.checked_div(&rhs).ok_or(BrilligArithmeticError::DivisionByZero)?,
        BinaryIntOp::And => lhs & rhs,
        BinaryIntOp::Or => lhs | rhs,
        BinaryIntOp::Xor => lhs ^ rhs,
        _ => unreachable!("Operator not handled by this function: {op:?}"),
    };
    Ok(result)
}

#[cfg(test)]
mod int_ops {
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

        // Mismatched bit sizes should error
        assert_eq!(
            evaluate_binary_int_op(
                &BinaryIntOp::Add,
                MemoryValue::<FieldElement>::U8(1),
                MemoryValue::<FieldElement>::U16(2),
                IntegerBitSize::U8
            ),
            Err(BrilligArithmeticError::MismatchedRhsBitSize { rhs_bit_size: 16, op_bit_size: 8 })
        );
        assert_eq!(
            evaluate_binary_int_op(
                &BinaryIntOp::Add,
                MemoryValue::<FieldElement>::U16(2),
                MemoryValue::<FieldElement>::U8(1),
                IntegerBitSize::U8
            ),
            Err(BrilligArithmeticError::MismatchedLhsBitSize { lhs_bit_size: 16, op_bit_size: 8 })
        );
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

        // Mismatched bit sizes should error
        assert_eq!(
            evaluate_binary_int_op(
                &BinaryIntOp::Sub,
                MemoryValue::<FieldElement>::U8(1),
                MemoryValue::<FieldElement>::U16(1),
                IntegerBitSize::U8
            ),
            Err(BrilligArithmeticError::MismatchedRhsBitSize { rhs_bit_size: 16, op_bit_size: 8 })
        );
        assert_eq!(
            evaluate_binary_int_op(
                &BinaryIntOp::Sub,
                MemoryValue::<FieldElement>::U16(1),
                MemoryValue::<FieldElement>::U8(1),
                IntegerBitSize::U8
            ),
            Err(BrilligArithmeticError::MismatchedLhsBitSize { lhs_bit_size: 16, op_bit_size: 8 })
        );
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

        // Mismatched bit sizes should error
        assert_eq!(
            evaluate_binary_int_op(
                &BinaryIntOp::Mul,
                MemoryValue::<FieldElement>::U8(1),
                MemoryValue::<FieldElement>::U16(1),
                IntegerBitSize::U8
            ),
            Err(BrilligArithmeticError::MismatchedRhsBitSize { rhs_bit_size: 16, op_bit_size: 8 })
        );
        assert_eq!(
            evaluate_binary_int_op(
                &BinaryIntOp::Mul,
                MemoryValue::<FieldElement>::U16(1),
                MemoryValue::<FieldElement>::U8(1),
                IntegerBitSize::U8
            ),
            Err(BrilligArithmeticError::MismatchedLhsBitSize { lhs_bit_size: 16, op_bit_size: 8 })
        );
    }

    #[test]
    fn div_test() {
        let bit_size = IntegerBitSize::U8;

        let test_ops =
            vec![TestParams { a: 5, b: 3, result: 1 }, TestParams { a: 5, b: 10, result: 0 }];

        evaluate_int_ops(test_ops, BinaryIntOp::Div, bit_size);

        // Mismatched bit sizes should error
        assert_eq!(
            evaluate_binary_int_op(
                &BinaryIntOp::Div,
                MemoryValue::<FieldElement>::U8(1),
                MemoryValue::<FieldElement>::U16(1),
                IntegerBitSize::U8
            ),
            Err(BrilligArithmeticError::MismatchedRhsBitSize { rhs_bit_size: 16, op_bit_size: 8 })
        );
        assert_eq!(
            evaluate_binary_int_op(
                &BinaryIntOp::Div,
                MemoryValue::<FieldElement>::U16(1),
                MemoryValue::<FieldElement>::U8(1),
                IntegerBitSize::U8
            ),
            Err(BrilligArithmeticError::MismatchedLhsBitSize { lhs_bit_size: 16, op_bit_size: 8 })
        );

        // Division by zero should error
        assert_eq!(
            evaluate_binary_int_op(
                &BinaryIntOp::Div,
                MemoryValue::<FieldElement>::U8(1),
                MemoryValue::<FieldElement>::U8(0),
                IntegerBitSize::U8
            ),
            Err(BrilligArithmeticError::DivisionByZero)
        );
    }

    #[test]
    fn shl_test() {
        let bit_size = IntegerBitSize::U8;

        let test_ops =
            vec![TestParams { a: 1, b: 7, result: 128 }, TestParams { a: 5, b: 7, result: 128 }];

        evaluate_int_ops(test_ops, BinaryIntOp::Shl, bit_size);

        assert_eq!(
            evaluate_binary_int_op(
                &BinaryIntOp::Shl,
                MemoryValue::<FieldElement>::U8(1u8),
                MemoryValue::<FieldElement>::U8(8u8),
                IntegerBitSize::U8
            ),
            Err(BrilligArithmeticError::BitshiftOverflow { bit_size: 8, shift_size: 8 })
        );
    }

    #[test]
    fn shr_test() {
        let bit_size = IntegerBitSize::U8;

        let test_ops =
            vec![TestParams { a: 1, b: 0, result: 1 }, TestParams { a: 5, b: 1, result: 2 }];

        evaluate_int_ops(test_ops, BinaryIntOp::Shr, bit_size);

        assert_eq!(
            evaluate_binary_int_op(
                &BinaryIntOp::Shr,
                MemoryValue::<FieldElement>::U8(1u8),
                MemoryValue::<FieldElement>::U8(8u8),
                IntegerBitSize::U8
            ),
            Err(BrilligArithmeticError::BitshiftOverflow { bit_size: 8, shift_size: 8 })
        );
    }

    #[test]
    fn comparison_ops_test() {
        let bit_size = IntegerBitSize::U8;

        // Equals
        let test_ops = vec![
            TestParams { a: 5, b: 5, result: 1 },
            TestParams { a: 10, b: 5, result: 0 },
            TestParams { a: 0, b: 0, result: 1 },
        ];
        evaluate_int_ops(test_ops, BinaryIntOp::Equals, bit_size);

        // LessThan
        let test_ops = vec![
            TestParams { a: 4, b: 5, result: 1 },
            TestParams { a: 5, b: 4, result: 0 },
            TestParams { a: 5, b: 5, result: 0 },
        ];
        evaluate_int_ops(test_ops, BinaryIntOp::LessThan, bit_size);

        // LessThanEquals
        let test_ops = vec![
            TestParams { a: 4, b: 5, result: 1 },
            TestParams { a: 5, b: 4, result: 0 },
            TestParams { a: 5, b: 5, result: 1 },
        ];
        evaluate_int_ops(test_ops, BinaryIntOp::LessThanEquals, bit_size);

        // Mismatched bit sizes should error
        assert_eq!(
            evaluate_binary_int_op(
                &BinaryIntOp::Equals,
                MemoryValue::<FieldElement>::U8(1),
                MemoryValue::<FieldElement>::U16(1),
                IntegerBitSize::U8
            ),
            Err(BrilligArithmeticError::MismatchedRhsBitSize { rhs_bit_size: 16, op_bit_size: 8 })
        );
        assert_eq!(
            evaluate_binary_int_op(
                &BinaryIntOp::LessThan,
                MemoryValue::<FieldElement>::U16(1),
                MemoryValue::<FieldElement>::U8(1),
                IntegerBitSize::U8
            ),
            Err(BrilligArithmeticError::MismatchedLhsBitSize { lhs_bit_size: 16, op_bit_size: 8 })
        );
        assert_eq!(
            evaluate_binary_int_op(
                &BinaryIntOp::LessThanEquals,
                MemoryValue::<FieldElement>::U16(1),
                MemoryValue::<FieldElement>::U8(1),
                IntegerBitSize::U8
            ),
            Err(BrilligArithmeticError::MismatchedLhsBitSize { lhs_bit_size: 16, op_bit_size: 8 })
        );
    }
}

#[cfg(test)]
mod field_ops {
    use super::*;
    use acir::{AcirField, FieldElement};

    struct TestParams {
        a: FieldElement,
        b: FieldElement,
        result: FieldElement,
    }

    fn evaluate_field_u128(op: &BinaryFieldOp, a: FieldElement, b: FieldElement) -> FieldElement {
        let result_value: MemoryValue<FieldElement> =
            evaluate_binary_field_op(op, MemoryValue::new_field(a), MemoryValue::new_field(b))
                .unwrap();
        // Convert back to FieldElement
        result_value.to_field()
    }

    fn evaluate_field_ops(test_params: Vec<TestParams>, op: BinaryFieldOp) {
        for test in test_params {
            assert_eq!(evaluate_field_u128(&op, test.a, test.b), test.result);
        }
    }

    #[test]
    fn add_test() {
        let test_ops = vec![
            TestParams { a: 1u32.into(), b: 2u32.into(), result: 3u32.into() },
            TestParams { a: 5u32.into(), b: 10u32.into(), result: 15u32.into() },
            TestParams { a: 250u32.into(), b: 10u32.into(), result: 260u32.into() },
        ];
        evaluate_field_ops(test_ops, BinaryFieldOp::Add);

        // Mismatched bit sizes
        assert_eq!(
            evaluate_binary_field_op(
                &BinaryFieldOp::Add,
                MemoryValue::new_field(FieldElement::from(1u32)),
                MemoryValue::<FieldElement>::U16(2),
            ),
            Err(BrilligArithmeticError::MismatchedRhsBitSize {
                rhs_bit_size: 16,
                op_bit_size: 254
            })
        );

        assert_eq!(
            evaluate_binary_field_op(
                &BinaryFieldOp::Add,
                MemoryValue::<FieldElement>::U16(1),
                MemoryValue::new_field(FieldElement::from(2u32)),
            ),
            Err(BrilligArithmeticError::MismatchedLhsBitSize {
                lhs_bit_size: 16,
                op_bit_size: 254
            })
        );
    }

    #[test]
    fn sub_test() {
        let test_ops = vec![
            TestParams { a: 5u32.into(), b: 3u32.into(), result: 2u32.into() },
            TestParams { a: 2u32.into(), b: 10u32.into(), result: FieldElement::from(-8_i128) },
        ];
        evaluate_field_ops(test_ops, BinaryFieldOp::Sub);

        // Mismatched bit sizes
        assert_eq!(
            evaluate_binary_field_op(
                &BinaryFieldOp::Sub,
                MemoryValue::new_field(FieldElement::from(1u32)),
                MemoryValue::<FieldElement>::U16(1),
            ),
            Err(BrilligArithmeticError::MismatchedRhsBitSize {
                rhs_bit_size: 16,
                op_bit_size: 254
            })
        );

        assert_eq!(
            evaluate_binary_field_op(
                &BinaryFieldOp::Sub,
                MemoryValue::<FieldElement>::U16(1),
                MemoryValue::new_field(FieldElement::from(1u32)),
            ),
            Err(BrilligArithmeticError::MismatchedLhsBitSize {
                lhs_bit_size: 16,
                op_bit_size: 254
            })
        );
    }

    #[test]
    fn mul_test() {
        let test_ops = vec![
            TestParams { a: 2u32.into(), b: 3u32.into(), result: 6u32.into() },
            TestParams { a: 10u32.into(), b: 25u32.into(), result: 250u32.into() },
        ];
        evaluate_field_ops(test_ops, BinaryFieldOp::Mul);

        // Mismatched bit sizes
        assert_eq!(
            evaluate_binary_field_op(
                &BinaryFieldOp::Mul,
                MemoryValue::new_field(FieldElement::from(1u32)),
                MemoryValue::<FieldElement>::U16(1),
            ),
            Err(BrilligArithmeticError::MismatchedRhsBitSize {
                rhs_bit_size: 16,
                op_bit_size: 254
            })
        );

        assert_eq!(
            evaluate_binary_field_op(
                &BinaryFieldOp::Mul,
                MemoryValue::<FieldElement>::U16(1),
                MemoryValue::new_field(FieldElement::from(1u32)),
            ),
            Err(BrilligArithmeticError::MismatchedLhsBitSize {
                lhs_bit_size: 16,
                op_bit_size: 254
            })
        );
    }

    #[test]
    fn div_test() {
        let test_ops = vec![
            TestParams { a: 10u32.into(), b: 2u32.into(), result: 5u32.into() },
            TestParams { a: 9u32.into(), b: 3u32.into(), result: 3u32.into() },
            TestParams {
                a: 10u32.into(),
                b: FieldElement::from(-1_i128),
                result: FieldElement::from(-10_i128),
            },
            TestParams { a: 10u32.into(), b: 1u32.into(), result: 10u32.into() },
            // Field division is a * 1/b. The inverse of 20 is 7660885005143746327786242010840046280991927540145612020294371465301532973466
            TestParams {
                a: 10u32.into(),
                b: 20u32.into(),
                result: FieldElement::try_from_str(
                    "10944121435919637611123202872628637544274182200208017171849102093287904247809",
                )
                .unwrap(),
            },
            // The inverse of 7 is 3126891838834182174606629392179610726935480628630862049099743455225115499374.
            TestParams {
                a: 100u32.into(),
                b: 7u32.into(),
                result: FieldElement::try_from_str(
                    "6253783677668364349213258784359221453870961257261724098199486910450230998762",
                )
                .unwrap(),
            },
        ];
        evaluate_field_ops(test_ops, BinaryFieldOp::Div);

        // Division by zero
        assert_eq!(
            evaluate_binary_field_op(
                &BinaryFieldOp::Div,
                MemoryValue::new_field(FieldElement::from(1u128)),
                MemoryValue::new_field(FieldElement::zero()),
            ),
            Err(BrilligArithmeticError::DivisionByZero)
        );

        // Mismatched bit sizes
        assert_eq!(
            evaluate_binary_field_op(
                &BinaryFieldOp::Div,
                MemoryValue::new_field(FieldElement::from(1u32)),
                MemoryValue::<FieldElement>::U16(1),
            ),
            Err(BrilligArithmeticError::MismatchedRhsBitSize {
                rhs_bit_size: 16,
                op_bit_size: 254
            })
        );

        assert_eq!(
            evaluate_binary_field_op(
                &BinaryFieldOp::Div,
                MemoryValue::<FieldElement>::U16(1),
                MemoryValue::new_field(FieldElement::from(1u32)),
            ),
            Err(BrilligArithmeticError::MismatchedLhsBitSize {
                lhs_bit_size: 16,
                op_bit_size: 254
            })
        );
    }

    #[test]
    fn integer_div_test() {
        let test_ops = vec![
            TestParams { a: 10u32.into(), b: 2u32.into(), result: 5u32.into() },
            TestParams { a: 9u32.into(), b: 3u32.into(), result: 3u32.into() },
            // Negative numbers are treated as large unsigned numbers, thus we expect a result of 0 here
            TestParams { a: 10u32.into(), b: FieldElement::from(-1_i128), result: 0u32.into() },
            TestParams { a: 10u32.into(), b: 1u32.into(), result: 10u32.into() },
            TestParams { a: 10u32.into(), b: 20u32.into(), result: 0u32.into() },
            // 100 / 7 == 14 with a remainder of 2. The remainder is discarded.
            TestParams { a: 100u32.into(), b: 7u32.into(), result: 14u32.into() },
        ];
        evaluate_field_ops(test_ops, BinaryFieldOp::IntegerDiv);

        // Division by zero should error
        assert_eq!(
            evaluate_binary_field_op(
                &BinaryFieldOp::IntegerDiv,
                MemoryValue::new_field(FieldElement::from(1u128)),
                MemoryValue::new_field(FieldElement::zero()),
            ),
            Err(BrilligArithmeticError::DivisionByZero)
        );

        // Mismatched bit sizes should error
        assert_eq!(
            evaluate_binary_field_op(
                &BinaryFieldOp::IntegerDiv,
                MemoryValue::new_field(FieldElement::from(1u32)),
                MemoryValue::<FieldElement>::U16(1),
            ),
            Err(BrilligArithmeticError::MismatchedRhsBitSize {
                rhs_bit_size: 16,
                op_bit_size: 254
            })
        );

        assert_eq!(
            evaluate_binary_field_op(
                &BinaryFieldOp::IntegerDiv,
                MemoryValue::<FieldElement>::U16(1),
                MemoryValue::new_field(FieldElement::from(1u32)),
            ),
            Err(BrilligArithmeticError::MismatchedLhsBitSize {
                lhs_bit_size: 16,
                op_bit_size: 254
            })
        );
    }

    #[test]
    fn comparison_ops_test() {
        // Equals
        let test_ops = vec![
            TestParams { a: 5u32.into(), b: 5u32.into(), result: 1u32.into() },
            TestParams { a: 10u32.into(), b: 5u32.into(), result: 0u32.into() },
            TestParams { a: 0u32.into(), b: 0u32.into(), result: 1u32.into() },
        ];
        evaluate_field_ops(test_ops, BinaryFieldOp::Equals);

        // LessThan
        let test_ops = vec![
            TestParams { a: 4u32.into(), b: 5u32.into(), result: 1u32.into() },
            TestParams { a: 5u32.into(), b: 4u32.into(), result: 0u32.into() },
            TestParams { a: 5u32.into(), b: 5u32.into(), result: 0u32.into() },
        ];
        evaluate_field_ops(test_ops, BinaryFieldOp::LessThan);

        // LessThanEquals
        let test_ops = vec![
            TestParams { a: 4u32.into(), b: 5u32.into(), result: 1u32.into() },
            TestParams { a: 5u32.into(), b: 4u32.into(), result: 0u32.into() },
            TestParams { a: 5u32.into(), b: 5u32.into(), result: 1u32.into() },
        ];
        evaluate_field_ops(test_ops, BinaryFieldOp::LessThanEquals);

        // Mismatched bit sizes should error
        assert_eq!(
            evaluate_binary_field_op(
                &BinaryFieldOp::Equals,
                MemoryValue::new_field(1u32.into()),
                MemoryValue::<FieldElement>::U16(1),
            ),
            Err(BrilligArithmeticError::MismatchedRhsBitSize {
                rhs_bit_size: 16,
                op_bit_size: 254
            })
        );

        assert_eq!(
            evaluate_binary_field_op(
                &BinaryFieldOp::LessThan,
                MemoryValue::<FieldElement>::U16(1),
                MemoryValue::new_field(1u32.into()),
            ),
            Err(BrilligArithmeticError::MismatchedLhsBitSize {
                lhs_bit_size: 16,
                op_bit_size: 254
            })
        );

        assert_eq!(
            evaluate_binary_field_op(
                &BinaryFieldOp::LessThanEquals,
                MemoryValue::<FieldElement>::U16(1),
                MemoryValue::new_field(1u32.into()),
            ),
            Err(BrilligArithmeticError::MismatchedLhsBitSize {
                lhs_bit_size: 16,
                op_bit_size: 254
            })
        );
    }
}
