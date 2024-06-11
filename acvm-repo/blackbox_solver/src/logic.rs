use acir::AcirField;

pub fn bit_and<F: AcirField>(lhs: F, rhs: F, num_bits: u32) -> F {
    bitwise_op(lhs, rhs, num_bits, |lhs_byte, rhs_byte| lhs_byte & rhs_byte)
}

pub fn bit_xor<F: AcirField>(lhs: F, rhs: F, num_bits: u32) -> F {
    bitwise_op(lhs, rhs, num_bits, |lhs_byte, rhs_byte| lhs_byte ^ rhs_byte)
}

fn bitwise_op<F: AcirField>(lhs: F, rhs: F, num_bits: u32, op: fn(u8, u8) -> u8) -> F {
    // XXX: Gadgets like SHA256 need to have their input be a multiple of 8
    // This is not a restriction caused by SHA256, as it works on bits
    // but most backends assume bytes.
    // We could implicitly pad, however this may not be intuitive for users.
    // assert!(
    //     num_bits % 8 == 0,
    //     "num_bits is not a multiple of 8, it is {}",
    //     num_bits
    // );

    let lhs_bytes = mask_to_be_bytes(lhs, num_bits);
    let rhs_bytes = mask_to_be_bytes(rhs, num_bits);

    let and_byte_arr: Vec<_> =
        lhs_bytes.into_iter().zip(rhs_bytes).map(|(left, right)| op(left, right)).collect();

    F::from_be_bytes_reduce(&and_byte_arr)
}

// mask_to methods will not remove any bytes from the field
// they are simply zeroed out
// Whereas truncate_to will remove those bits and make the byte array smaller
fn mask_to_be_bytes<F: AcirField>(field: F, num_bits: u32) -> Vec<u8> {
    let mut bytes = field.to_be_bytes();
    mask_vector_le(&mut bytes, num_bits as usize);
    bytes
}

fn mask_vector_le(bytes: &mut [u8], num_bits: usize) {
    // reverse to big endian format
    bytes.reverse();

    let mask_power = num_bits % 8;
    let array_mask_index = num_bits / 8;

    for (index, byte) in bytes.iter_mut().enumerate() {
        match index.cmp(&array_mask_index) {
            std::cmp::Ordering::Less => {
                // do nothing if the current index is less than
                // the array index.
            }
            std::cmp::Ordering::Equal => {
                let mask = 2u8.pow(mask_power as u32) - 1;
                // mask the byte
                *byte &= mask;
            }
            std::cmp::Ordering::Greater => {
                // Anything greater than the array index
                // will be set to zero
                *byte = 0;
            }
        }
    }
    // reverse back to little endian
    bytes.reverse();
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
