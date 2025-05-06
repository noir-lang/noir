use acvm::{AcirField, FieldElement};
use noirc_errors::Location;

use crate::{
    Type,
    ast::IntegerBitSize,
    hir::comptime::{InterpreterError, Value, errors::IResult},
    shared::Signedness,
};

/// evaluate_cast without recursion
pub(super) fn evaluate_cast_one_step(
    typ: &Type,
    location: Location,
    evaluated_lhs: Value,
) -> IResult<Value> {
    macro_rules! signed_int_to_field {
        ($x:expr) => {{
            // Need to convert the signed integer to an i128 before
            // we negate it to preserve the MIN value.
            let mut value = $x as i128;
            let is_negative = value < 0;
            if is_negative {
                value = -value;
            }
            ((value as u128).into(), is_negative)
        }};
    }

    let (mut lhs, lhs_is_negative) = match evaluated_lhs {
        Value::Field(value) => (value, false),
        Value::U1(value) => ((value as u128).into(), false),
        Value::U8(value) => ((value as u128).into(), false),
        Value::U16(value) => ((value as u128).into(), false),
        Value::U32(value) => ((value as u128).into(), false),
        Value::U64(value) => ((value as u128).into(), false),
        Value::U128(value) => (value.into(), false),
        Value::I8(value) => signed_int_to_field!(value),
        Value::I16(value) => signed_int_to_field!(value),
        Value::I32(value) => signed_int_to_field!(value),
        Value::I64(value) => signed_int_to_field!(value),
        Value::Bool(value) => (FieldElement::from(value), false),
        value => {
            let typ = value.get_type().into_owned();
            return Err(InterpreterError::NonNumericCasted { typ, location });
        }
    };

    macro_rules! cast_to_int {
        ($x:expr, $method:ident, $typ:ty, $f:ident) => {{
            let mut value = $x.$method() as $typ;
            if lhs_is_negative {
                value = 0 - value;
            }
            Ok(Value::$f(value))
        }};
    }

    // Now actually cast the lhs, bit casting and wrapping as necessary
    match typ.follow_bindings() {
        Type::FieldElement => {
            if lhs_is_negative {
                lhs = FieldElement::zero() - lhs;
            }
            Ok(Value::Field(lhs))
        }
        Type::Integer(sign, bit_size) => match (sign, bit_size) {
            (Signedness::Unsigned, IntegerBitSize::One) => {
                Err(InterpreterError::TypeUnsupported { typ: typ.clone(), location })
            }
            (Signedness::Unsigned, IntegerBitSize::Eight) => cast_to_int!(lhs, to_u128, u8, U8),
            (Signedness::Unsigned, IntegerBitSize::Sixteen) => {
                cast_to_int!(lhs, to_u128, u16, U16)
            }
            (Signedness::Unsigned, IntegerBitSize::ThirtyTwo) => {
                cast_to_int!(lhs, to_u128, u32, U32)
            }
            (Signedness::Unsigned, IntegerBitSize::SixtyFour) => {
                cast_to_int!(lhs, to_u128, u64, U64)
            }
            (Signedness::Unsigned, IntegerBitSize::HundredTwentyEight) => {
                cast_to_int!(lhs, to_u128, u128, U128)
            }
            (Signedness::Signed, IntegerBitSize::One) => {
                Err(InterpreterError::TypeUnsupported { typ: typ.clone(), location })
            }
            (Signedness::Signed, IntegerBitSize::Eight) => cast_to_int!(lhs, to_i128, i8, I8),
            (Signedness::Signed, IntegerBitSize::Sixteen) => {
                cast_to_int!(lhs, to_i128, i16, I16)
            }
            (Signedness::Signed, IntegerBitSize::ThirtyTwo) => {
                cast_to_int!(lhs, to_i128, i32, I32)
            }
            (Signedness::Signed, IntegerBitSize::SixtyFour) => {
                cast_to_int!(lhs, to_i128, i64, I64)
            }
            (Signedness::Signed, IntegerBitSize::HundredTwentyEight) => {
                todo!()
            }
        },
        Type::Bool => Ok(Value::Bool(!lhs.is_zero() || lhs_is_negative)),
        typ => Err(InterpreterError::CastToNonNumericType { typ, location }),
    }
}

#[cfg(test)]
mod tests {
    use noirc_errors::Location;

    use super::*;

    #[test]
    fn smoke_test() {
        let location = Location::dummy();
        let typ = Type::FieldElement;

        let lhs_values = [
            Value::Field(FieldElement::one()),
            Value::Bool(true),
            Value::U1(true),
            Value::U8(1),
            Value::U16(1),
            Value::U32(1),
            Value::U64(1),
            Value::U128(1),
            Value::I8(1),
            Value::I16(1),
            Value::I32(1),
            Value::I64(1),
        ];
        for lhs in lhs_values {
            assert_eq!(
                evaluate_cast_one_step(&typ, location, lhs),
                Ok(Value::Field(FieldElement::one()))
            );
        }
    }
}
