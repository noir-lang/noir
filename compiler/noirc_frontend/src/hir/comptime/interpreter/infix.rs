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
                    Ok(Value::$res_var(($expr).ok_or(error($op))?))
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
        (($lhs_value:ident as $lhs:ident $op:literal $rhs_value:ident as $rhs:ident) { field: $field_expr:expr, int: $int_expr:expr, }) => {
            match_values! {
                ($lhs_value as $lhs $op $rhs_value as $rhs) {
                    (Field, Field) to Field => Some($field_expr),
                    (I8,  I8)      to I8    => $int_expr,
                    (I16, I16)     to I16   => $int_expr,
                    (I32, I32)     to I32   => $int_expr,
                    (I64, I64)     to I64   => $int_expr,
                    (U8,  U8)      to U8    => $int_expr,
                    (U16, U16)     to U16   => $int_expr,
                    (U32, U32)     to U32   => $int_expr,
                    (U64, U64)     to U64   => $int_expr,
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
                    (U8,  U8)      to Bool => Some($expr),
                    (U16, U16)     to Bool => Some($expr),
                    (U32, U32)     to Bool => Some($expr),
                    (U64, U64)     to Bool => Some($expr),
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
                    (U8,  U8)      to U8   => Some($expr),
                    (U16, U16)     to U16  => Some($expr),
                    (U32, U32)     to U32  => Some($expr),
                    (U64, U64)     to U64  => Some($expr),
                }
            }
        };
    }

    /// Generate matches for operations on just integer values.
    macro_rules! match_integer {
        (($lhs_value:ident as $lhs:ident $op:literal $rhs_value:ident as $rhs:ident) => $expr:expr) => {
            match_values! {
                ($lhs_value as $lhs $op $rhs_value as $rhs) {
                    (I8,  I8)      to I8   => $expr,
                    (I16, I16)     to I16  => $expr,
                    (I32, I32)     to I32  => $expr,
                    (I64, I64)     to I64  => $expr,
                    (U8,  U8)      to U8   => $expr,
                    (U16, U16)     to U16  => $expr,
                    (U32, U32)     to U32  => $expr,
                    (U64, U64)     to U64  => $expr,
                }
            }
        };
    }

    /// Generate matches for bit shifting, which in Noir only accepts `u8` for RHS.
    macro_rules! match_bitshift {
        (($lhs_value:ident as $lhs:ident $op:literal $rhs_value:ident as $rhs:ident) => $expr:expr) => {
            match_values! {
                ($lhs_value as $lhs $op $rhs_value as $rhs) {
                    (I8,  U8)      to I8   => $expr,
                    (I16, U8)      to I16  => $expr,
                    (I32, U8)      to I32  => $expr,
                    (I64, U8)      to I64  => $expr,
                    (U8,  U8)      to U8   => $expr,
                    (U16, U8)      to U16  => $expr,
                    (U32, U8)      to U32  => $expr,
                    (U64, U8)      to U64  => $expr,
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
            }
        },
        BinaryOpKind::Subtract => match_arithmetic! {
            (lhs_value as lhs "-" rhs_value as rhs) {
                field: lhs - rhs,
                int: lhs.checked_sub(rhs),
            }
        },
        BinaryOpKind::Multiply => match_arithmetic! {
            (lhs_value as lhs "*" rhs_value as rhs) {
                field: lhs * rhs,
                int: lhs.checked_mul(rhs),
            }
        },
        BinaryOpKind::Divide => match_arithmetic! {
            (lhs_value as lhs "/" rhs_value as rhs) {
                field: lhs / rhs,
                int: lhs.checked_div(rhs),
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
        BinaryOpKind::ShiftRight => match_bitshift! {
            (lhs_value as lhs ">>" rhs_value as rhs) => lhs.checked_shr(rhs.into())
        },
        BinaryOpKind::ShiftLeft => match_bitshift! {
            (lhs_value as lhs "<<" rhs_value as rhs) => lhs.checked_shl(rhs.into())
        },
        BinaryOpKind::Modulo => match_integer! {
            (lhs_value as lhs "%" rhs_value as rhs) => lhs.checked_rem(rhs)
        },
    }
}
