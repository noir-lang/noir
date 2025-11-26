use acvm::AcirField as _;

use crate::ast::BinaryOpKind;
use crate::hir::Location;
use crate::hir_def::expr::HirBinaryOp;

use super::{IResult, InterpreterError, Value};

pub(super) fn evaluate_infix(
    lhs_value: Value,
    rhs_value: Value,
    operator: HirBinaryOp,
    location: Location,
) -> IResult<Value> {
    let lhs_type = lhs_value.get_type().into_owned();
    let rhs_type = rhs_value.get_type().into_owned();

    let error = |operator| {
        let lhs = lhs_type.clone();
        let rhs = rhs_type.clone();
        InterpreterError::InvalidValuesForBinary { lhs, rhs, location, operator }
    };
    let lhs_overflow = InterpreterError::BinaryOperationOverflow { operator: "<<", location };
    let rhs_overflow = InterpreterError::BinaryOperationOverflow { operator: ">>", location };
    let math_error = |operator| InterpreterError::BinaryOperationOverflow { location, operator };

    if matches!(operator.kind, BinaryOpKind::Divide | BinaryOpKind::Modulo) && rhs_value.is_zero() {
        return Err(InterpreterError::InvalidValuesForBinary {
            lhs: lhs_type,
            rhs: rhs_type,
            location,
            operator: if operator.kind == BinaryOpKind::Divide { "/" } else { "%" },
        });
    }

    /// Generate matches that can promote the type of one side to the other if they are compatible.
    macro_rules! match_values {
        (($lhs_value:ident as $lhs:ident $op:literal $rhs_value:ident as $rhs:ident) {
            $(
                ($lhs_var:ident, $rhs_var:ident) to $res_var:ident => $expr:expr
            ),*
            $(,)?
         }
        ) => {
            match ($lhs_value, $rhs_value) {
                $(
                (Value::$lhs_var($lhs), Value::$rhs_var($rhs)) => {
                    Ok(Value::$res_var(($expr).ok_or(math_error($op))?))
                },
                )*
                (_, _) => {
                    Err(error($op))
                },
            }
        };
    }

    /// Generate matches for arithmetic operations on `Field` and integers.
    macro_rules! match_arithmetic {
        (($lhs_value:ident as $lhs:ident $op:literal $rhs_value:ident as $rhs:ident) { field: $field_expr:expr, int: $int_expr:expr, u1: $u1_expr:expr, }) => {
            match_values! {
                ($lhs_value as $lhs $op $rhs_value as $rhs) {
                    (Field, Field) to Field => Some($field_expr),
                    (I8,  I8)      to I8    => $int_expr,
                    (I16, I16)     to I16   => $int_expr,
                    (I32, I32)     to I32   => $int_expr,
                    (I64, I64)     to I64   => $int_expr,
                    (U1,  U1)      to U1    => $u1_expr,
                    (U8,  U8)      to U8    => $int_expr,
                    (U16, U16)     to U16   => $int_expr,
                    (U32, U32)     to U32   => $int_expr,
                    (U64, U64)     to U64   => $int_expr,
                    (U128, U128)   to U128  => $int_expr,
                }
            }
        };
    }

    /// Generate matches for comparison operations on all types, returning `Bool`.
    macro_rules! match_cmp {
        (($lhs_value:ident as $lhs:ident $op:literal $rhs_value:ident as $rhs:ident) => $expr:expr) => {
            match_values! {
                ($lhs_value as $lhs $op $rhs_value as $rhs) {
                    (Field, Field) to Bool => Some($expr),
                    (Bool, Bool)   to Bool => Some($expr),
                    (I8,  I8)      to Bool => Some($expr),
                    (I16, I16)     to Bool => Some($expr),
                    (I32, I32)     to Bool => Some($expr),
                    (I64, I64)     to Bool => Some($expr),
                    (U1,  U1)      to Bool => Some($expr),
                    (U8,  U8)      to Bool => Some($expr),
                    (U16, U16)     to Bool => Some($expr),
                    (U32, U32)     to Bool => Some($expr),
                    (U64, U64)     to Bool => Some($expr),
                    (U128, U128)   to Bool => Some($expr),
                }
            }
        };
    }

    /// Generate matches for bitwise operations on `Bool` and integers.
    macro_rules! match_bitwise {
        (($lhs_value:ident as $lhs:ident $op:literal $rhs_value:ident as $rhs:ident) => $expr:expr) => {
            match_values! {
                ($lhs_value as $lhs $op $rhs_value as $rhs) {
                    (Bool, Bool)   to Bool => Some($expr),
                    (I8,  I8)      to I8   => Some($expr),
                    (I16, I16)     to I16  => Some($expr),
                    (I32, I32)     to I32  => Some($expr),
                    (I64, I64)     to I64  => Some($expr),
                    (U1,  U1)      to U1   => Some($expr),
                    (U8,  U8)      to U8   => Some($expr),
                    (U16, U16)     to U16  => Some($expr),
                    (U32, U32)     to U32  => Some($expr),
                    (U64, U64)     to U64  => Some($expr),
                    (U128, U128)   to U128  => Some($expr),
                }
            }
        };
    }

    /// Generate matches for operations on just integer values.
    macro_rules! match_integer {
        (($lhs_value:ident as $lhs:ident $op:literal $rhs_value:ident as $rhs:ident) { int: $int_expr:expr, u1: $u1_expr:expr, }) => {
            match_values! {
                ($lhs_value as $lhs $op $rhs_value as $rhs) {
                    (I8,  I8)      to I8   => $int_expr,
                    (I16, I16)     to I16  => $int_expr,
                    (I32, I32)     to I32  => $int_expr,
                    (I64, I64)     to I64  => $int_expr,
                    (U1,  U1)      to U1   => $u1_expr,
                    (U8,  U8)      to U8   => $int_expr,
                    (U16, U16)     to U16  => $int_expr,
                    (U32, U32)     to U32  => $int_expr,
                    (U64, U64)     to U64  => $int_expr,
                    (U128, U128)   to U128 => $int_expr,
                }
            }
        };
    }

    #[allow(clippy::bool_comparison)]
    match operator.kind {
        BinaryOpKind::Add => match_arithmetic! {
            (lhs_value as lhs "+" rhs_value as rhs) {
                field: lhs + rhs,
                int: lhs.checked_add(rhs),
                u1: if lhs && rhs { None } else { Some(lhs | rhs) },
            }
        },
        BinaryOpKind::Subtract => match_arithmetic! {
            (lhs_value as lhs "-" rhs_value as rhs) {
                field: lhs - rhs,
                int: lhs.checked_sub(rhs),
                u1: if !lhs && rhs { None } else { Some(lhs & !rhs) },
            }
        },
        BinaryOpKind::Multiply => match_arithmetic! {
            (lhs_value as lhs "*" rhs_value as rhs) {
                field: lhs * rhs,
                int: lhs.checked_mul(rhs),
                u1: Some(lhs & rhs),
            }
        },
        BinaryOpKind::Divide => match_arithmetic! {
            (lhs_value as lhs "/" rhs_value as rhs) {
                field: if rhs.absolute_value().is_zero() {
                   return Err( InterpreterError::InvalidValuesForBinary { lhs: lhs_type, rhs: rhs_type, location, operator: "/" });
                } else {
                    lhs / rhs
                },
                int: lhs.checked_div(rhs),
                u1: {
                    let _ = rhs; // Avoid unused variable warning
                    Some(lhs)
                },
            }
        },
        BinaryOpKind::Equal => match_cmp! {
            (lhs_value as lhs "==" rhs_value as rhs) => lhs == rhs
        },
        BinaryOpKind::NotEqual => match_cmp! {
            (lhs_value as lhs "!=" rhs_value as rhs) => lhs != rhs
        },
        BinaryOpKind::Less => match_cmp! {
            (lhs_value as lhs "<" rhs_value as rhs) => lhs < rhs
        },
        BinaryOpKind::LessEqual => match_cmp! {
            (lhs_value as lhs "<=" rhs_value as rhs) => lhs <= rhs
        },
        BinaryOpKind::Greater => match_cmp! {
            (lhs_value as lhs ">" rhs_value as rhs) => lhs > rhs
        },
        BinaryOpKind::GreaterEqual => match_cmp! {
            (lhs_value as lhs ">=" rhs_value as rhs) => lhs >= rhs
        },
        BinaryOpKind::And => match_bitwise! {
            (lhs_value as lhs "&" rhs_value as rhs) => lhs & rhs
        },
        BinaryOpKind::Or => match_bitwise! {
            (lhs_value as lhs "|" rhs_value as rhs) => lhs | rhs
        },
        BinaryOpKind::Xor => match_bitwise! {
            (lhs_value as lhs "^" rhs_value as rhs) => lhs ^ rhs
        },
        #[allow(trivial_numeric_casts)]
        BinaryOpKind::ShiftRight => match_integer! {
            (lhs_value as lhs ">>" rhs_value as rhs) {
                int: {
                    #[allow(unused_comparisons, clippy::absurd_extreme_comparisons)]
                    if rhs > 127 {return Err(rhs_overflow);}
                    #[allow(clippy::cast_lossless)]
                    lhs.checked_shr(rhs as u32)
                },
                u1: if rhs { return Err(rhs_overflow)} else { Some(lhs) },
            }
        },
        #[allow(trivial_numeric_casts)]
        BinaryOpKind::ShiftLeft => match_integer! {
            (lhs_value as lhs "<<" rhs_value as rhs) {
                int: {
                    #[allow(unused_comparisons, clippy::absurd_extreme_comparisons)]
                    if rhs > 127 {return Err(lhs_overflow);}
                    #[allow(clippy::cast_lossless)]
                    lhs.checked_shl(rhs as u32)
                },
                u1: if rhs { return Err(lhs_overflow)} else { Some(lhs) },
            }
        },
        BinaryOpKind::Modulo => match_integer! {
            (lhs_value as lhs "%" rhs_value as rhs) {
                int: lhs.checked_rem(rhs),
                u1: {
                    let _ = lhs; // Avoid unused variable warning
                    if rhs { Some(false) } else { None }
                },
            }
        },
    }
}

#[cfg(test)]
mod test {
    use crate::hir::comptime::InterpreterError;
    use crate::hir::comptime::tests::{interpret, interpret_expect_error};

    use super::{BinaryOpKind, HirBinaryOp, Location, Value};

    use super::evaluate_infix;

    #[test]
    /// See: https://github.com/noir-lang/noir/issues/8391
    fn regression_8391() {
        let lhs = Value::U128(340282366920938463463374607431768211455);
        let rhs = Value::U128(2);
        let operator = HirBinaryOp { kind: BinaryOpKind::Divide, location: Location::dummy() };
        let location = Location::dummy();
        let result = evaluate_infix(lhs, rhs, operator, location).unwrap();

        assert_eq!(result, Value::U128(170141183460469231731687303715884105727));
    }

    #[test]
    fn regression_9336() {
        let lhs = Value::I8(-128);
        let rhs = Value::I8(-1);
        let operator = HirBinaryOp { kind: BinaryOpKind::Modulo, location: Location::dummy() };
        let location = Location::dummy();
        let err = evaluate_infix(lhs, rhs, operator, location).unwrap_err();
        assert!(matches!(err, InterpreterError::BinaryOperationOverflow { .. }));
    }

    #[test]
    fn shl_unsigned() {
        let src = r#"
            comptime fn main() -> pub u64 {
                3 << 4
            }
        "#;
        let result = interpret(src);
        assert_eq!(result, Value::U64(48));
    }

    #[test]
    fn shl_signed() {
        let src = r#"
            comptime fn main() -> pub i64 {
                2 << 3
            }
        "#;
        let result = interpret(src);
        assert_eq!(result, Value::I64(16));
    }

    #[test]
    fn shl_unsigned_overflow() {
        let src = r#"
            comptime fn main() -> pub u64 {
                1 << 128
            }
        "#;

        let err = interpret_expect_error(src);
        let InterpreterError::BinaryOperationOverflow { operator, .. } = err else {
            panic!("Expected overflow error");
        };
        assert_eq!(operator, "<<");
    }

    #[test]
    fn shl_signed_overflow() {
        let src = r#"
            comptime fn main() -> pub i64 {
                1 << 64
            }
        "#;

        let err = interpret_expect_error(src);
        let InterpreterError::BinaryOperationOverflow { operator, .. } = err else {
            panic!("Expected overflow error");
        };
        assert_eq!(operator, "<<");
    }

    #[test]
    fn shr_unsigned() {
        let src = r#"
            comptime fn main() -> pub u64 {
                64 >> 1
            }
        "#;
        let result = interpret(src);
        assert_eq!(result, Value::U64(32));
    }

    #[test]
    fn shr_unsigned_overflow() {
        let src = r#"
            comptime fn main() -> pub u64 {
                64 >> 63
            }
        "#;
        let result = interpret(src);
        assert_eq!(result, Value::U64(0));

        let src = r#"
            comptime fn main() -> pub u64 {
                64 >> 255
            }
        "#;
        let result = interpret_expect_error(src);
        assert!(matches!(result, InterpreterError::BinaryOperationOverflow { operator: ">>", .. }));

        let src = "
            comptime fn main() -> pub u32 {
                1360887544 >> 141
            }
        ";
        let result = interpret_expect_error(src);
        assert!(matches!(result, InterpreterError::BinaryOperationOverflow { operator: ">>", .. }));
    }

    #[test]
    fn shr_signed_overflow_negative_lhs() {
        let src = "
        comptime fn main() -> pub i64 {
            -64 >> 63
        }
        ";
        let result = interpret(src);
        assert_eq!(result, Value::I64(-1));

        let src = "
        comptime fn main() -> pub i64 {
            -64 >> 255
        }
        ";
        let result = interpret_expect_error(src);
        assert!(matches!(result, InterpreterError::BinaryOperationOverflow { operator: ">>", .. }));

        let src = "
        comptime fn main() -> pub i32 {
            -1360887544 >> 141
        }
        ";
        let result = interpret_expect_error(src);
        assert!(matches!(result, InterpreterError::BinaryOperationOverflow { operator: ">>", .. }));
    }

    #[test]
    fn shr_signed_overflow_positive_lhs() {
        let src = "
        comptime fn main() -> pub i64 {
            64 >> 63
        }
        ";
        let result = interpret(src);
        assert_eq!(result, Value::I64(0));

        let src = "
        comptime fn main() -> pub i64 {
            64 >> 255
        }
        ";
        let result = interpret_expect_error(src);
        assert!(matches!(result, InterpreterError::BinaryOperationOverflow { operator: ">>", .. }));

        let src = "
        comptime fn main() -> pub i32 {
            1360887544 >> 141
        }
        ";
        let result = interpret_expect_error(src);
        assert!(matches!(result, InterpreterError::BinaryOperationOverflow { operator: ">>", .. }));
    }

    #[test]
    fn shr_signed() {
        let src = r#"
            comptime fn main() -> pub i64 {
                -64 >> 1
            }
        "#;
        let result = interpret(src);
        assert_eq!(result, Value::I64(-32));
    }

    #[test]
    fn div_zero_field() {
        let src = r#"
            comptime fn main() -> pub Field {
                32 / 0
            }
        "#;
        let result = interpret_expect_error(src);
        assert!(matches!(result, InterpreterError::InvalidValuesForBinary { operator: "/", .. }));
    }

    #[test]
    fn div_zero_int() {
        let src = r#"
            comptime fn main() -> pub i32 {
                32 / 0
            }
        "#;
        let result = interpret_expect_error(src);
        assert!(matches!(result, InterpreterError::InvalidValuesForBinary { operator: "/", .. }));
    }

    #[test]
    fn div() {
        let src = r#"
            comptime fn main() {
                let x_field = 8;
                let x_i32: i32 = -7;

                // Field division is weird so I'm not testing with a remainder here
                assert_eq(x_field / 2, 4);
                assert_eq(x_i32 / 2, -3);
            }
        "#;
        let result = interpret(src);
        assert_eq!(result, Value::Unit);
    }
}
