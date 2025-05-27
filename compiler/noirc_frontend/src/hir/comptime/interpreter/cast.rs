use std::cmp::Ordering;

use acvm::{AcirField, FieldElement};
use noirc_errors::Location;

use crate::{
    Type,
    ast::IntegerBitSize,
    hir::comptime::{InterpreterError, Value, errors::IResult},
    shared::Signedness,
    signed_field::SignedField,
};

fn bit_size(typ: &Type) -> u32 {
    match typ {
        Type::FieldElement => FieldElement::max_num_bits(),
        Type::Integer(_, bit_size) => bit_size.bit_size() as u32,
        Type::Bool => 2,
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
    /// SignedField makes casting signed values more difficult since we need
    /// to add an offset to make the signed value positive if it is negative,
    /// and need to store a boolean to remember it is negative
    SignedToField {
        old_bit_size: u32,
    },
    /// No-op also covers the zero-extend case since we convert between
    /// field elements rather than concrete bit sizes
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
                    CastType::SignedToField { old_bit_size: input_size }
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
        CastType::Truncate { new_bit_size } => {
            // This performs a truncation to u128 but all types should be <= 128 bits anyway
            let lhs = lhs.to_u128();
            if new_bit_size == 128 {
                return lhs.into();
            }
            let mask = 2u128.pow(new_bit_size) - 1;
            FieldElement::from(lhs & mask)
        }
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
        CastType::SignedToField { old_bit_size } => {
            assert!(old_bit_size < 128, "i128 and above are not supported");
            let max_positive_value = 2u128.pow(old_bit_size - 1) - 1;
            let is_negative = lhs > max_positive_value.into();

            if !is_negative {
                lhs
            } else {
                // We only return the FieldElement component of the SignedField, the caller
                // needs to add the negative sign themselves
                let max_negative_value = FieldElement::from(2u128.pow(old_bit_size));
                // E.g. `256 - 255 = 1` or `256 - 128 = 128`
                max_negative_value - lhs
            }
        }
        CastType::Noop => lhs,
    }
}

/// Convert the input value to a (field, sign) pair.
/// Crucially, this is _not_ equivalent to a `SignedField` because negatives
/// in the field component are represented in two's complement instead of their
/// positive absolute values.
fn convert_to_field(value: Value, location: Location) -> IResult<(FieldElement, bool)> {
    Ok(match value {
        Value::Field(value) if value.is_negative() => (-value.absolute_value(), true),
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
    })
}

/// evaluate_cast without recursion
pub(super) fn evaluate_cast_one_step(
    output_type: &Type,
    location: Location,
    evaluated_lhs: Value,
) -> IResult<Value> {
    let lhs_type = evaluated_lhs.get_type().into_owned();
    let (lhs, lhs_is_negative) = convert_to_field(evaluated_lhs, location)?;

    let cast_kind = classify_cast(&lhs_type, output_type);
    let lhs = perform_cast(cast_kind, lhs);

    // Now just wrap the Result in a Value
    match output_type.follow_bindings() {
        Type::FieldElement => Ok(Value::Field(SignedField::new(lhs, lhs_is_negative))),
        typ @ Type::Integer(sign, bit_size) => match (sign, bit_size) {
            (Signedness::Unsigned, IntegerBitSize::One) => {
                Err(InterpreterError::TypeUnsupported { typ: output_type.clone(), location })
            }
            // These casts are expected to be no-ops
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
                Err(InterpreterError::TypeUnsupported { typ, location })
            }
            (Signedness::Signed, IntegerBitSize::Eight) => Ok(Value::I8(lhs.to_u128() as i8)),
            (Signedness::Signed, IntegerBitSize::Sixteen) => Ok(Value::I16(lhs.to_u128() as i16)),
            (Signedness::Signed, IntegerBitSize::ThirtyTwo) => Ok(Value::I32(lhs.to_u128() as i32)),
            (Signedness::Signed, IntegerBitSize::SixtyFour) => Ok(Value::I64(lhs.to_u128() as i64)),
            (Signedness::Signed, IntegerBitSize::HundredTwentyEight) => {
                Err(InterpreterError::TypeUnsupported { typ, location })
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
            (Value::U8(255), signed(SixtyFour), Value::I64(255)),
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
            (Value::I8(-100), unsigned(Eight), Value::U8(156)),
            // Truncate
            (Value::I16(300), unsigned(Eight), Value::U8(44)),
            (Value::I16(300), signed(Eight), Value::I8(44)),
            (Value::I16(255), signed(Eight), Value::I8(-1)),
            (Value::I16(i16::MIN + 5), signed(Eight), Value::I8(5)),
            (Value::I16(i16::MIN + 5), unsigned(Eight), Value::U8(5)),
            (Value::Field(SignedField::negative(1u32)), unsigned(Eight), Value::U8(0)),
            (Value::Field(SignedField::negative(1u32)), signed(Eight), Value::I8(0)),
            (Value::Field(SignedField::negative(2u32)), unsigned(Sixteen), Value::U16(65535)),
            (Value::Field(SignedField::negative(2u32)), signed(Sixteen), Value::I16(-1)),
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
