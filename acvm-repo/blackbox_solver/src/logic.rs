use acir::AcirField;

pub fn bit_and<F: AcirField>(lhs: F, rhs: F, num_bits: u32) -> F {
    bitwise_op(lhs, rhs, num_bits, |lhs_byte, rhs_byte| lhs_byte & rhs_byte)
}

pub fn bit_xor<F: AcirField>(lhs: F, rhs: F, num_bits: u32) -> F {
    bitwise_op(lhs, rhs, num_bits, |lhs_byte, rhs_byte| lhs_byte ^ rhs_byte)
}

/// Performs a bitwise operation on two field elements by treating them as byte arrays.
///
/// Both field elements are converted to little-endian byte arrays and masked to keep only
/// the lowest `num_bits` bits. The provided operation `op` is then applied byte-by-byte,
/// and the result is converted back to a field element.
/// This function works for any `num_bits` value and does not assume it to be a multiple of 8.
fn bitwise_op<F: AcirField>(lhs: F, rhs: F, num_bits: u32, op: fn(u8, u8) -> u8) -> F {
    // We could explicitly expect `num_bits` to be a multiple of 8 as most backends assume bytes:
    // assert!(num_bits % 8 == 0, "num_bits is not a multiple of 8, it is {num_bits}");

    let lhs_bytes = mask_to_le_bytes(lhs, num_bits);
    let rhs_bytes = mask_to_le_bytes(rhs, num_bits);

    let and_byte_arr: Vec<_> =
        lhs_bytes.into_iter().zip(rhs_bytes).map(|(left, right)| op(left, right)).collect();

    F::from_le_bytes_reduce(&and_byte_arr)
}

// mask_to methods will not remove any bytes from the field
// they are simply zeroed out
// Whereas truncate_to will remove those bits and make the byte array smaller
fn mask_to_le_bytes<F: AcirField>(field: F, num_bits: u32) -> Vec<u8> {
    let mut bytes = field.to_le_bytes();
    mask_vector_le(&mut bytes, num_bits as usize);
    bytes
}

/// Mask a byte array in-place to only keep the lowest `num_bits`
fn mask_vector_le(bytes: &mut [u8], num_bits: usize) {
    let total_bits = bytes.len() * 8;
    if num_bits >= total_bits {
        // nothing to mask, all bits are used
        return;
    }

    // Find which byte contains the last bit we want to keep
    let array_mask_index = num_bits / 8;
    // Find how many bits to keep in that byte (0-7)
    let mask_power = num_bits % 8;

    // If `mask_power` is non-zero, this keeps only the lower `mask_power` bits of the byte.
    // If `mask_power` is zero (when `num_bits` is a multiple of 8), this zeros out the byte,
    // which is correct since that byte is the first one beyond what we want to keep.
    bytes[array_mask_index] &= 2u8.pow(mask_power as u32) - 1;

    // Zero out all remaining bytes
    for byte in &mut bytes[(array_mask_index + 1)..] {
        *byte = 0;
    }
}

#[cfg(test)]
mod tests {
    use acir::FieldElement;
    use proptest::prelude::*;

    use crate::{bit_and, bit_xor};

    proptest! {
        #[test]
        fn matches_bitwise_and_on_u128s(x in 0..=u128::MAX, y in 0..=u128::MAX, bit_size in 128u32..) {
            let x_as_field = FieldElement::from(x);
            let y_as_field = FieldElement::from(y);

            let x_and_y = x & y;
            let x_and_y_as_field = bit_and(x_as_field, y_as_field, bit_size);

            prop_assert_eq!(x_and_y_as_field, FieldElement::from(x_and_y), "AND on fields should match that on integers");
        }

        #[test]
        fn matches_bitwise_xor_on_u128s(x in 0..=u128::MAX, y in 0..=u128::MAX, bit_size in 128u32..) {
            let x_as_field = FieldElement::from(x);
            let y_as_field = FieldElement::from(y);

            let x_xor_y = x ^ y;
            let x_xor_y_as_field = bit_xor(x_as_field, y_as_field, bit_size);

            prop_assert_eq!(x_xor_y_as_field, FieldElement::from(x_xor_y), "XOR on fields should match that on integers");
        }
    }
}
