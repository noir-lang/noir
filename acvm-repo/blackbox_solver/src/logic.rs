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
    use acir::{AcirField, FieldElement};

    use crate::bit_and;

    #[test]
    fn and() {
        let max = 10_000u32;

        let num_bits = (std::mem::size_of::<u32>() * 8) as u32 - max.leading_zeros();

        for x in 0..max {
            let x = FieldElement::from(x as i128);
            let res = bit_and(x, x, num_bits);
            assert_eq!(res.to_be_bytes(), x.to_be_bytes());
        }
    }
}
