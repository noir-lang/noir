use acvm::{AcirField, FieldElement};
use noirc_errors::Location;
use num_bigint::{ToBigInt, ToBigUint};

use crate::{
    Type,
    ast::IntegerBitSize,
    hir::comptime::{InterpreterError, Value, errors::IResult},
    shared::Signedness,
    signed_field::SignedField,
};

/// Returns the truncate offset for casting to or from field values.
/// When we do this we need to know the size of the input/output type.
fn bit_size(typ: &Type) -> u32 {
    match typ {
        Type::FieldElement => 0,
        Type::Integer(_, bit_size) => bit_size.bit_size() as u32,
        Type::Bool => 2,
        _ => 0,
    }
}

enum CastType {
    Truncate { new_bit_size: u32 },
    Reinterpret,
    SignExtend { new_bit_size: u32 },
    ZeroExtend { new_bit_size: u32 },
    Noop,
    ToField,
}

fn classify_cast(input: &Type, output: &Type) -> CastType {
    let input = input.follow_bindings();
    let output = output.follow_bindings();

    if input == output {
        return CastType::Noop;
    }

    let input_signed = input.is_signed();
    let input_size = bit_size(&input);
    let output_size = bit_size(&input);

    if input.is_field() {
        return CastType::Truncate { new_bit_size: output_size };
    }

    if output.is_field() {
        return CastType::ToField;
    }

    if input_size < output_size {
        if input_signed {
            CastType::SignExtend { new_bit_size: output_size }
        } else {
            CastType::ZeroExtend { new_bit_size: output_size }
        }
    } else if input_size == output_size {
        CastType::Reinterpret
    } else {
        CastType::Truncate { new_bit_size: output_size }
    }
}

fn perform_cast(kind: CastType, lhs: FieldElement) -> FieldElement {
    match kind {
        CastType::Truncate { new_bit_size } => {}
        CastType::Reinterpret => todo!(),
        CastType::SignExtend { new_bit_size } => todo!(),
        CastType::ZeroExtend { new_bit_size } => todo!(),
        CastType::Noop => lhs,
        CastType::ToField => lhs,
    }
}

/// evaluate_cast without recursion
pub(super) fn evaluate_cast_one_step(
    typ: &Type,
    location: Location,
    evaluated_lhs: Value,
) -> IResult<Value> {
    let lhs_type = evaluated_lhs.get_type();
    let input_is_signed = lhs_type.is_signed();
    let input_bit_size = truncate_bit_size(&lhs_type);
    let target_bit_size = truncate_bit_size(typ);

    let (mut lhs, lhs_is_negative) = match evaluated_lhs {
        Value::Field(value) if value.is_negative() => {
            // Shift negative field values into the positive range
            let offset = 2u32.pow(target_bit_size);
            let value = -value.absolute_value() + offset.into();
            (value, true)
        }
        Value::Field(value) => (value.absolute_value(), false),
        Value::U1(value) => ((value as u128).into(), false),
        Value::U8(value) => ((value as u128).into(), false),
        Value::U16(value) => ((value as u128).into(), false),
        Value::U32(value) => ((value as u128).into(), false),
        Value::U64(value) => ((value as u128).into(), false),
        Value::U128(value) => (value.into(), false),
        // Shared logic from ssa::interpreter::Value::convert_to_field
        Value::I8(value) => (FieldElement::from(value as u8 as i128), value < 0),
        Value::I16(value) => (FieldElement::from(value as u16 as i128), value < 0),
        Value::I32(value) => (FieldElement::from(value as u32 as i128), value < 0),
        Value::I64(value) => (FieldElement::from(value as u64 as i128), value < 0),
        Value::Bool(value) => (FieldElement::from(value), false),
        value => {
            let typ = value.get_type().into_owned();
            return Err(InterpreterError::NonNumericCasted { typ, location });
        }
    };

    // Perform a sign extension if we need to
    if lhs_is_negative && input_is_signed && target_bit_size > input_bit_size {
        let max_target = 2u32.to_biguint().unwrap().pow(target_bit_size) - 1u8;
        let max_input = 2u32.to_biguint().unwrap().pow(input_bit_size) - 1u8;

        // Subtracting these should give ones for each of the extension bits: `11111111 00000000`
        let mask = max_target - max_input;
        lhs += u128::try_from(mask).expect("This should always fit in a u128").into();
    }

    // Now actually cast the lhs, bit casting and wrapping as necessary
    match typ.follow_bindings() {
        Type::FieldElement => {
            let value = if lhs_is_negative {
                let offset = 2u32.pow(input_bit_size);
                let value = lhs + offset.into();
                // `lhs` is already negative and the absolute value is stored internally
                // so we need to undo it
                SignedField::negative(value)
            } else {
                SignedField::positive(lhs)
            };
            Ok(Value::Field(value))
        }
        Type::Integer(sign, bit_size) => match (sign, bit_size) {
            (Signedness::Unsigned, IntegerBitSize::One) => {
                Err(InterpreterError::TypeUnsupported { typ: typ.clone(), location })
            }
            (Signedness::Unsigned, IntegerBitSize::Eight) => Ok(Value::U8(lhs.to_u128() as u8)),
            (Signedness::Unsigned, IntegerBitSize::Sixteen) => Ok(Value::U16(lhs.to_u128() as u16)),
            (Signedness::Unsigned, IntegerBitSize::ThirtyTwo) => {
                Ok(Value::U32(lhs.to_u128() as u32))
            }
            (Signedness::Unsigned, IntegerBitSize::SixtyFour) => {
                Ok(Value::U64(lhs.to_u128() as u64))
            }
            (Signedness::Unsigned, IntegerBitSize::HundredTwentyEight) => {
                Ok(Value::U128(lhs.to_u128()))
            }
            (Signedness::Signed, IntegerBitSize::One) => {
                Err(InterpreterError::TypeUnsupported { typ: typ.clone(), location })
            }
            (Signedness::Signed, IntegerBitSize::Eight) => Ok(Value::I8(lhs.to_u128() as i8)),
            (Signedness::Signed, IntegerBitSize::Sixteen) => Ok(Value::I16(lhs.to_u128() as i16)),
            (Signedness::Signed, IntegerBitSize::ThirtyTwo) => Ok(Value::I32(lhs.to_u128() as i32)),
            (Signedness::Signed, IntegerBitSize::SixtyFour) => Ok(Value::I64(lhs.to_u128() as i64)),
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
            Value::Field(SignedField::one()),
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
                Ok(Value::Field(SignedField::one()))
            );
        }
    }

    #[test]
    fn unsigned_casts() {
        let location = Location::dummy();
        let signed = |size| Type::Integer(Signedness::Signed, size);
        let unsigned = |size| Type::Integer(Signedness::Unsigned, size);

        use IntegerBitSize::*;
        let tests = [
            // Widen
            (Value::U8(255), unsigned(SixtyFour), Value::U64(255)),
            (Value::U8(255), unsigned(SixtyFour), Value::U64(255)),
            // Reinterpret as negative
            (Value::U8(255), signed(Eight), Value::I8(-1)),
            (Value::Field(SignedField::positive(255u32)), signed(Eight), Value::I8(-1)),
            // Truncate
            (Value::U16(300), unsigned(Eight), Value::U8(44)),
            (Value::U16(300), signed(Eight), Value::I8(44)),
            (Value::U16(255), signed(Eight), Value::I8(-1)),
            (Value::Field(SignedField::positive(300u32)), unsigned(Eight), Value::U8(44)),
            (Value::Field(SignedField::positive(300u32)), signed(Eight), Value::I8(44)),
            (Value::Field(SignedField::positive(10u32)), unsigned(Sixteen), Value::U16(10)),
        ];

        for (lhs, typ, expected) in tests {
            let actual = evaluate_cast_one_step(&typ, location, lhs.clone());
            assert_eq!(
                actual,
                Ok(expected.clone()),
                "{lhs:?} as {typ}, expected {expected:?}, got {actual:?}"
            );
        }
    }

    #[test]
    fn signed_casts() {
        let location = Location::dummy();
        let signed = |size| Type::Integer(Signedness::Signed, size);
        let unsigned = |size| Type::Integer(Signedness::Unsigned, size);

        use IntegerBitSize::*;
        let tests = [
            (Value::Field(SignedField::negative(1u32)), unsigned(Eight), Value::U8(255)),
            (Value::Field(SignedField::negative(1u32)), signed(Eight), Value::I8(-1)),
            // Widen
            (Value::I8(127), unsigned(SixtyFour), Value::U64(127)),
            (Value::I8(127), signed(SixtyFour), Value::I64(127)),
            // Widen negative: zero extend
            (Value::I8(-1), unsigned(Sixteen), Value::U16(65535)),
            (Value::I8(-100), unsigned(Sixteen), Value::U16(65436)),
            (Value::I8(-100), Type::FieldElement, Value::Field(SignedField::negative(100u32))),
            // Widen negative: sign extend
            (Value::I8(-1), signed(Sixteen), Value::I16(-1)),
            (Value::I8(-100), signed(Sixteen), Value::I16(-100)),
            // Reinterpret as positive
            (Value::I8(-100), signed(Eight), Value::U8(156)),
            (Value::Field(SignedField::negative(1u32)), unsigned(Eight), Value::U8(255)),
            (Value::Field(SignedField::negative(1u32)), unsigned(Sixteen), Value::U16(25535)),
            // Truncate
            (Value::I16(300), unsigned(Eight), Value::I8(44)),
            (Value::I16(300), signed(Eight), Value::I8(44)),
            (Value::I16(255), signed(Eight), Value::I8(-1)),
            (Value::I16(i16::MIN + 5), signed(Eight), Value::I8(5)),
            (Value::I16(i16::MIN + 5), unsigned(Eight), Value::U8(5)),
            (Value::Field(SignedField::negative(1u32)), signed(Eight), Value::I8(0)),
        ];

        for (lhs, typ, expected) in tests {
            let actual = evaluate_cast_one_step(&typ, location, lhs.clone());
            assert_eq!(
                actual,
                Ok(expected.clone()),
                "{lhs:?} as {typ}, expected {expected:?}, got {actual:?}"
            );
        }
    }
}
