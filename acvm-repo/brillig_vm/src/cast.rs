//! Implementation for the [cast operation][acir::brillig::Opcode::Cast].
use acir::{
    AcirField,
    acir_field::truncate_to,
    brillig::{BitSize, IntegerBitSize},
};

use crate::MemoryValue;

/// Casts a value to a different bit size.
pub(crate) fn cast<F: AcirField>(
    source_value: MemoryValue<F>,
    target_bit_size: BitSize,
) -> MemoryValue<F> {
    use MemoryValue::*;

    match (source_value, target_bit_size) {
        // Field downcast to arbitrary bit size
        (Field(field), BitSize::Integer(target_bit_size)) => {
            let as_u128 = truncate_to(&field, 128).to_u128();
            match target_bit_size {
                IntegerBitSize::U1 => U1(as_u128 & 0x01 == 1),
                IntegerBitSize::U8 => U8(as_u128 as u8),
                IntegerBitSize::U16 => U16(as_u128 as u16),
                IntegerBitSize::U32 => U32(as_u128 as u32),
                IntegerBitSize::U64 => U64(as_u128 as u64),
                IntegerBitSize::U128 => U128(as_u128),
            }
        }

        (U1(value), BitSize::Integer(IntegerBitSize::U8)) => U8(value.into()),
        (U1(value), BitSize::Integer(IntegerBitSize::U16)) => U16(value.into()),
        (U1(value), BitSize::Integer(IntegerBitSize::U32)) => U32(value.into()),
        (U1(value), BitSize::Integer(IntegerBitSize::U64)) => U64(value.into()),
        (U1(value), BitSize::Integer(IntegerBitSize::U128)) => U128(value.into()),
        (U1(value), BitSize::Field) => Field(value.into()),

        (U8(value), BitSize::Integer(IntegerBitSize::U1)) => U1(value & 0x01 == 1),
        (U8(value), BitSize::Integer(IntegerBitSize::U16)) => U16(value.into()),
        (U8(value), BitSize::Integer(IntegerBitSize::U32)) => U32(value.into()),
        (U8(value), BitSize::Integer(IntegerBitSize::U64)) => U64(value.into()),
        (U8(value), BitSize::Integer(IntegerBitSize::U128)) => U128(value.into()),
        (U8(value), BitSize::Field) => Field(u128::from(value).into()),

        (U16(value), BitSize::Integer(IntegerBitSize::U1)) => U1(value & 0x01 == 1),
        (U16(value), BitSize::Integer(IntegerBitSize::U8)) => U8(value as u8),
        (U16(value), BitSize::Integer(IntegerBitSize::U32)) => U32(value.into()),
        (U16(value), BitSize::Integer(IntegerBitSize::U64)) => U64(value.into()),
        (U16(value), BitSize::Integer(IntegerBitSize::U128)) => U128(value.into()),
        (U16(value), BitSize::Field) => Field(u128::from(value).into()),

        (U32(value), BitSize::Integer(IntegerBitSize::U1)) => U1(value & 0x01 == 1),
        (U32(value), BitSize::Integer(IntegerBitSize::U8)) => U8(value as u8),
        (U32(value), BitSize::Integer(IntegerBitSize::U16)) => U16(value as u16),
        (U32(value), BitSize::Integer(IntegerBitSize::U64)) => U64(value.into()),
        (U32(value), BitSize::Integer(IntegerBitSize::U128)) => U128(value.into()),
        (U32(value), BitSize::Field) => Field(u128::from(value).into()),

        (U64(value), BitSize::Integer(IntegerBitSize::U1)) => U1(value & 0x01 == 1),
        (U64(value), BitSize::Integer(IntegerBitSize::U8)) => U8(value as u8),
        (U64(value), BitSize::Integer(IntegerBitSize::U16)) => U16(value as u16),
        (U64(value), BitSize::Integer(IntegerBitSize::U32)) => U32(value as u32),
        (U64(value), BitSize::Integer(IntegerBitSize::U128)) => U128(value.into()),
        (U64(value), BitSize::Field) => Field(u128::from(value).into()),

        (U128(value), BitSize::Integer(IntegerBitSize::U1)) => U1(value & 0x01 == 1),
        (U128(value), BitSize::Integer(IntegerBitSize::U8)) => U8(value as u8),
        (U128(value), BitSize::Integer(IntegerBitSize::U16)) => U16(value as u16),
        (U128(value), BitSize::Integer(IntegerBitSize::U32)) => U32(value as u32),
        (U128(value), BitSize::Integer(IntegerBitSize::U64)) => U64(value as u64),
        (U128(value), BitSize::Field) => Field(value.into()),

        // no ops
        (Field(_), BitSize::Field) => source_value,
        (U1(_), BitSize::Integer(IntegerBitSize::U1)) => source_value,
        (U8(_), BitSize::Integer(IntegerBitSize::U8)) => source_value,
        (U16(_), BitSize::Integer(IntegerBitSize::U16)) => source_value,
        (U32(_), BitSize::Integer(IntegerBitSize::U32)) => source_value,
        (U64(_), BitSize::Integer(IntegerBitSize::U64)) => source_value,
        (U128(_), BitSize::Integer(IntegerBitSize::U128)) => source_value,
    }
}
