use std::cmp::Ordering;

use crate::{
    Type,
    ast::IntegerBitSize,
    hir::comptime::{InterpreterError, Value, errors::IResult},
    shared::Signedness,
};
use acvm::{AcirField, FieldElement, acir::acir_field::truncate_to};
use noirc_errors::Location;

fn bit_size(typ: &Type) -> u32 {
    match typ {
        Type::FieldElement => FieldElement::max_num_bits(),
        Type::Integer(_, bit_size) => u32::from(bit_size.bit_size()),
        Type::Bool => 1,
        _ => FieldElement::max_num_bits(),
    }
}

#[derive(Debug)]
enum CastType {
    Truncate {
        new_bit_size: u32,
    },
    SignExtend {
        old_bit_size: u32,
        new_bit_size: u32,
    },
    /// No-op also covers the zero-extend case since we convert between
    /// field elements rather than concrete bit sizes
    ///
    /// This is also the case for casting signed integers to fields.
    /// We represent negatives with two's complement, so e.g.
    /// `-1 as i8` is stored as the field value for `255`, and `255`
    /// is also the expected result of casting these to a field.
    Noop,
}

fn classify_cast(input: &Type, output: &Type) -> CastType {
    let input = input.follow_bindings_shallow();
    let output = output.follow_bindings_shallow();

    let input_signed = input.is_signed();
    let input_size = bit_size(&input);
    let output_size = bit_size(&output);

    match input_size.cmp(&output_size) {
        Ordering::Less => {
            if input_signed {
                if output.is_field() {
                    CastType::Noop // We always zero-extend when casting to a field
                } else {
                    CastType::SignExtend { old_bit_size: input_size, new_bit_size: output_size }
                }
            } else {
                CastType::Noop //zero-extend
            }
        }
        Ordering::Equal => CastType::Noop,
        Ordering::Greater => CastType::Truncate { new_bit_size: output_size },
    }
}

fn perform_cast(kind: CastType, lhs: FieldElement) -> FieldElement {
    match kind {
        CastType::Truncate { new_bit_size } => truncate_to(&lhs, new_bit_size),
        CastType::SignExtend { old_bit_size, new_bit_size } => {
            assert!(new_bit_size <= 128);
            let max_positive_value = 2u128.pow(old_bit_size - 1) - 1;
            let is_negative = lhs > max_positive_value.into();

            if is_negative {
                let max_target =
                    if new_bit_size == 128 { u128::MAX } else { 2u128.pow(new_bit_size) - 1 };
                let max_input = 2u128.pow(old_bit_size) - 1;

                // Subtracting these should give ones for each of the extension bits: `11111111 00000000`
                let mask = max_target - max_input;
                lhs + mask.into()
            } else {
                lhs
            }
        }
        CastType::Noop => lhs,
    }
}

/// Convert the input value to a field.
///
/// Negatives of `U{N}` and `I{N}` types in the field are represented in two's
/// complement instead of the corresponding field value.
fn convert_to_2s_complement_field(value: Value, location: Location) -> IResult<FieldElement> {
    Ok(match value {
        Value::Integer(int) => int.as_field_twos_complement(),
        Value::Bool(value) => value.into(),
        value => {
            let typ = value.get_type().into_owned();
            return Err(InterpreterError::NonNumericCasted { typ, location });
        }
    })
}

/// evaluate_cast without recursion
pub(super) fn evaluate_cast_one_step(
    output_type: &Type,
    location: Location,
    evaluated_lhs: Value,
) -> IResult<Value> {
    let lhs_type = evaluated_lhs.get_type().into_owned();
    let lhs = convert_to_2s_complement_field(evaluated_lhs, location)?;

    let cast_kind = classify_cast(&lhs_type, output_type);
    let lhs = perform_cast(cast_kind, lhs);

    // Now just wrap the Result in a Value
    match output_type.follow_bindings() {
        Type::FieldElement => Ok(Value::field(lhs)),
        typ @ Type::Integer(sign, bit_size) => match (sign, bit_size) {
            // These casts are expected to be no-ops
            (Signedness::Unsigned, IntegerBitSize::One) => Ok(Value::u1(lhs.to_u128() != 0)),
            (Signedness::Unsigned, IntegerBitSize::Eight) => Ok(Value::u8(lhs.to_u128() as u8)),
            (Signedness::Unsigned, IntegerBitSize::Sixteen) => Ok(Value::u16(lhs.to_u128() as u16)),
            (Signedness::Unsigned, IntegerBitSize::ThirtyTwo) => {
                Ok(Value::u32(lhs.to_u128() as u32))
            }
            (Signedness::Unsigned, IntegerBitSize::SixtyFour) => {
                Ok(Value::u64(lhs.to_u128() as u64))
            }
            (Signedness::Unsigned, IntegerBitSize::HundredTwentyEight) => {
                Ok(Value::u128(lhs.to_u128()))
            }
            (Signedness::Signed, IntegerBitSize::One) => {
                Err(InterpreterError::TypeUnsupported { typ, location })
            }
            (Signedness::Signed, IntegerBitSize::Eight) => Ok(Value::i8(lhs.to_u128() as i8)),
            (Signedness::Signed, IntegerBitSize::Sixteen) => Ok(Value::i16(lhs.to_u128() as i16)),
            (Signedness::Signed, IntegerBitSize::ThirtyTwo) => Ok(Value::i32(lhs.to_u128() as i32)),
            (Signedness::Signed, IntegerBitSize::SixtyFour) => Ok(Value::i64(lhs.to_u128() as i64)),
            (Signedness::Signed, IntegerBitSize::HundredTwentyEight) => {
                Err(InterpreterError::TypeUnsupported { typ, location })
            }
        },
        Type::Bool if lhs_type == Type::Bool => Ok(Value::Bool(!lhs.is_zero())),
        // Numeric conversions to booleans must use `!= 0`
        Type::Bool => Err(InterpreterError::CannotCastNumericToBool { typ: lhs_type, location }),
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
            Value::field(FieldElement::one()),
            Value::Bool(true),
            Value::u1(true),
            Value::u8(1),
            Value::u16(1),
            Value::u32(1),
            Value::u64(1),
            Value::u128(1),
            Value::i8(1),
            Value::i16(1),
            Value::i32(1),
            Value::i64(1),
        ];

        for lhs in lhs_values {
            assert_eq!(
                evaluate_cast_one_step(&typ, location, lhs),
                Ok(Value::field(FieldElement::one()))
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
            (Value::u8(255), unsigned(SixtyFour), Value::u64(255)),
            (Value::u8(255), signed(SixtyFour), Value::i64(255)),
            (Value::u64(u64::MAX), unsigned(HundredTwentyEight), Value::u128(u128::from(u64::MAX))),
            // Reinterpret as negative
            (Value::u8(255), signed(Eight), Value::i8(-1)),
            (Value::field(255u32.into()), signed(Eight), Value::i8(-1)),
            // Truncate
            (Value::u16(300), unsigned(Eight), Value::u8(44)),
            (Value::u16(300), signed(Eight), Value::i8(44)),
            (Value::u16(255), signed(Eight), Value::i8(-1)),
            (Value::field(300u32.into()), unsigned(Eight), Value::u8(44)),
            (Value::field(300u32.into()), signed(Eight), Value::i8(44)),
            (Value::field(10u32.into()), unsigned(Sixteen), Value::u16(10)),
            (Value::field(256u32.into()), unsigned(Eight), Value::u8(0)),
            (Value::field(255u32.into()), unsigned(Eight), Value::u8(255)),
            (Value::u128(u128::MAX), unsigned(SixtyFour), Value::u64(u64::MAX)),
            // Casting Field -> Field should be a no-op
            (Value::field(4u32.into()), Type::FieldElement, Value::field(4u32.into())),
            (
                Value::field(-FieldElement::from(4u32)),
                Type::FieldElement,
                Value::field(-FieldElement::from(4u32)),
            ),
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
            // Widen
            (Value::i8(127), unsigned(SixtyFour), Value::u64(127)),
            (Value::i8(127), signed(SixtyFour), Value::i64(127)),
            // Widen signed->unsigned: sign extend
            (Value::i8(-1), unsigned(Sixteen), Value::u16(65535)),
            (Value::i8(-100), unsigned(Sixteen), Value::u16(65436)),
            // Casting a negative integer to a field always results in a positive value
            // This is the only case we zero-extend signed integers instead of sign-extending them
            (Value::i8(-1), Type::FieldElement, Value::field(255u32.into())),
            // Widen negative: sign extend
            (Value::i8(-1), signed(Sixteen), Value::i16(-1)),
            (Value::i8(-100), signed(Sixteen), Value::i16(-100)),
            // Reinterpret as positive
            (Value::i8(-100), unsigned(Eight), Value::u8(156)),
            // Truncate
            (Value::i16(300), unsigned(Eight), Value::u8(44)),
            (Value::i16(300), signed(Eight), Value::i8(44)),
            (Value::i16(255), signed(Eight), Value::i8(-1)),
            (Value::i16(i16::MIN + 5), signed(Eight), Value::i8(5)),
            (Value::i16(i16::MIN + 5), unsigned(Eight), Value::u8(5)),
            (Value::field(-FieldElement::from(1u32)), unsigned(Eight), Value::u8(0)),
            (Value::field(-FieldElement::from(1u32)), signed(Eight), Value::i8(0)),
            (Value::field(-FieldElement::from(2u32)), unsigned(Sixteen), Value::u16(65535)),
            (Value::field(-FieldElement::from(2u32)), signed(Sixteen), Value::i16(-1)),
            (Value::field(u128::MAX.into()), signed(Eight), Value::i8(-1)),
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
    fn bool_cast() {
        let location = Location::dummy();
        let lhs = Value::field(0u32.into());
        let actual = evaluate_cast_one_step(&Type::Bool, location, lhs);
        assert!(matches!(actual, Err(InterpreterError::CannotCastNumericToBool { .. })));
    }
}
