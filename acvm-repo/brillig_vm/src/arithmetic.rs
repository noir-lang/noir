use acir::brillig::{BinaryFieldOp, BinaryIntOp};
use acir::FieldElement;
use num_bigint::BigUint;
use num_traits::{One, ToPrimitive, Zero};

/// Evaluate a binary operation on two FieldElements and return the result as a FieldElement.
pub(crate) fn evaluate_binary_field_op(
    op: &BinaryFieldOp,
    a: FieldElement,
    b: FieldElement,
) -> FieldElement {
    match op {
        // Perform addition, subtraction, multiplication, and division based on the BinaryOp variant.
        BinaryFieldOp::Add => a + b,
        BinaryFieldOp::Sub => a - b,
        BinaryFieldOp::Mul => a * b,
        BinaryFieldOp::Div => a / b,
        BinaryFieldOp::IntegerDiv => {
            let a_big = BigUint::from_bytes_be(&a.to_be_bytes());
            let b_big = BigUint::from_bytes_be(&b.to_be_bytes());

            let result = a_big / b_big;
            FieldElement::from_be_bytes_reduce(&result.to_bytes_be())
        }
        BinaryFieldOp::Equals => (a == b).into(),
        BinaryFieldOp::LessThan => (a < b).into(),
        BinaryFieldOp::LessThanEquals => (a <= b).into(),
    }
}

/// Evaluate a binary operation on two unsigned big integers with a given bit size and return the result as a big integer.
pub(crate) fn evaluate_binary_bigint_op(
    op: &BinaryIntOp,
    a: BigUint,
    b: BigUint,
    bit_size: u32,
) -> Result<BigUint, String> {
    let bit_modulo = &(BigUint::one() << bit_size);
    let result = match op {
        // Perform addition, subtraction, and multiplication, applying a modulo operation to keep the result within the bit size.
        BinaryIntOp::Add => (a + b) % bit_modulo,
        BinaryIntOp::Sub => (bit_modulo + a - b) % bit_modulo,
        BinaryIntOp::Mul => (a * b) % bit_modulo,
        // Perform unsigned division using the modulo operation on a and b.
        BinaryIntOp::Div => {
            let b_mod = b % bit_modulo;
            if b_mod.is_zero() {
                BigUint::zero()
            } else {
                (a % bit_modulo) / b_mod
            }
        }
        // Perform a == operation, returning 0 or 1
        BinaryIntOp::Equals => {
            if (a % bit_modulo) == (b % bit_modulo) {
                BigUint::one()
            } else {
                BigUint::zero()
            }
        }
        // Perform a < operation, returning 0 or 1
        BinaryIntOp::LessThan => {
            if (a % bit_modulo) < (b % bit_modulo) {
                BigUint::one()
            } else {
                BigUint::zero()
            }
        }
        // Perform a <= operation, returning 0 or 1
        BinaryIntOp::LessThanEquals => {
            if (a % bit_modulo) <= (b % bit_modulo) {
                BigUint::one()
            } else {
                BigUint::zero()
            }
        }
        // Perform bitwise AND, OR, XOR, left shift, and right shift operations, applying a modulo operation to keep the result within the bit size.
        BinaryIntOp::And => (a & b) % bit_modulo,
        BinaryIntOp::Or => (a | b) % bit_modulo,
        BinaryIntOp::Xor => (a ^ b) % bit_modulo,
        BinaryIntOp::Shl => {
            assert!(bit_size <= 128, "unsupported bit size for right shift");
            let b = b.to_u128().unwrap();
            (a << b) % bit_modulo
        }
        BinaryIntOp::Shr => {
            assert!(bit_size <= 128, "unsupported bit size for right shift");
            let b = b.to_u128().unwrap();
            (a >> b) % bit_modulo
        }
    };

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestParams {
        a: u128,
        b: u128,
        result: u128,
    }

    fn evaluate_u128(op: &BinaryIntOp, a: u128, b: u128, bit_size: u32) -> u128 {
        // Convert to big integers
        let lhs_big = BigUint::from(a);
        let rhs_big = BigUint::from(b);
        let result_value = evaluate_binary_bigint_op(op, lhs_big, rhs_big, bit_size).unwrap();
        // Convert back to u128
        result_value.to_u128().unwrap()
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
