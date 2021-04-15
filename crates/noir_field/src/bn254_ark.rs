use crate::field_trait::FieldElement;
use ark_bn254::Fr;
use ark_ff::to_bytes;
use ark_ff::{BitIteratorBE, PrimeField};
use std::str::FromStr;

impl FieldElement for Fr {
    fn to_bytes(&self) -> Vec<u8> {
        hex::decode(self.to_hex()).unwrap()
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let hex_str = hex::encode(bytes);
        let dec_str = hex_to_decimal(&hex_str);
        Fr::from_str(&dec_str).unwrap()
    }

    fn from_bytes_reduce(bytes: &[u8]) -> Self {
        Fr::from_be_bytes_mod_order(bytes)
    }

    fn mask_to_field(&self, num_bits: u32) -> Self {
        let bit_iter = self.mask_to_bits(num_bits);
        let byte_arr = pack_bits_into_bytes(bit_iter);
        Fr::from_bytes(&byte_arr)
    }

    fn mask_to_bytes(&self, num_bits: u32) -> Vec<u8> {
        let bit_iter = self.mask_to_bits(num_bits);
        pack_bits_into_bytes(bit_iter)
    }

    fn bits(&self) -> Vec<bool> {
        BitIteratorBE::new(self.into_repr()).collect()
    }

    fn mask_to_bits(&self, num_bits: u32) -> Vec<bool> {
        let max_bits = Self::max_num_bits() + Self::wasted_bits();

        let bit_iter: Vec<_> = BitIteratorBE::new(self.into_repr())
            .enumerate()
            .map(|(i, bit)| {
                if i < (max_bits - num_bits) as usize {
                    false
                } else {
                    bit
                }
            })
            .collect();

        bit_iter
    }

    fn truncate_to_bits(&self, num_bits: u32) -> Vec<bool> {
        let max_bits = Self::max_num_bits() + Self::wasted_bits();

        let bit_iter: Vec<_> = BitIteratorBE::new(self.into_repr())
            .enumerate()
            .filter(|(i, _)| *i >= (max_bits - num_bits) as usize)
            .map(|(_, bit)| bit)
            .collect();

        bit_iter
    }

    fn truncate_to_bytes(&self, num_bits: u32) -> Vec<u8> {
        let bit_iter = self.truncate_to_bits(num_bits);
        pack_bits_into_bytes(bit_iter)
    }

    fn fetch_nearest_bytes(&self, num_bits: usize) -> Vec<u8> {
        fn nearest_bytes(num_bits: usize) -> usize {
            ((num_bits + 7) / 8) * 8
        }

        let num_bytes = nearest_bytes(num_bits);
        let num_elements = num_bytes / 8;

        let mut bytes = self.to_bytes();
        bytes.reverse(); // put it in big endian format. XXX(next refactor): we should be explicit about endianess.

        bytes[0..num_elements].to_vec()
    }

    fn and_xor(&self, rhs: &Self, num_bits: u32, is_xor: bool) -> Self {
        let lhs = self.mask_to_field(num_bits);
        let lhs_bit_iter = BitIteratorBE::new(lhs.into_repr());
        let rhs = rhs.mask_to_field(num_bits);
        let rhs_bit_iter = BitIteratorBE::new(rhs.into_repr());

        let and_iter: Vec<_> = lhs_bit_iter
            .zip(rhs_bit_iter)
            .map(
                |(bit_a, bit_b)| {
                    if is_xor {
                        bit_a ^ bit_b
                    } else {
                        bit_a & bit_b
                    }
                },
            )
            .collect();

        let byte_arr = pack_bits_into_bytes(and_iter);
        Self::from_bytes(&byte_arr)
    }

    fn and(&self, rhs: &Self, num_bits: u32) -> Self {
        self.and_xor(rhs, num_bits, false)
    }
    fn xor(&self, rhs: &Self, num_bits: u32) -> Self {
        self.and_xor(rhs, num_bits, true)
    }

    fn one() -> Self {
        ark_ff::One::one()
    }

    fn zero() -> Self {
        ark_ff::Zero::zero()
    }

    fn max_num_bits() -> u32 {
        254
    }

    fn try_from_str(input: &str) -> Option<Self> {
        if input.contains('x') {
            return Self::from_hex(input);
        }

        let fr = Fr::from_str(input).ok()?;
        Some(fr)
    }

    fn wasted_bits() -> u32 {
        let one: Fr = ark_ff::One::one();
        let vec: Vec<_> = BitIteratorBE::new(one.into_repr()).collect();

        let num_bits_used = vec.len() as u32;
        let num_bits_needed = Self::max_num_bits();
        num_bits_used - num_bits_needed
    }

    fn num_bits(&self) -> u32 {
        let non_zero_index = BitIteratorBE::new(self.into_repr()).position(|x| x);

        match non_zero_index {
            None => 0,
            Some(index) => {
                // The most significant bit was found at index.
                // The index tells us how many elements came before the most significant bit

                // We need to compute the offset as the representation may have wasted bits
                let offset = Self::wasted_bits();

                // This is now the amount of significant elements that came before the most significant bit
                let msb_index_offset = (index as u32) - offset;

                Self::max_num_bits() - msb_index_offset
            }
        }
    }

    fn to_u128(&self) -> u128 {
        use std::convert::TryInto;

        let bytes = self.to_bytes();
        u128::from_be_bytes(bytes[16..32].try_into().unwrap())
    }

    fn inverse(&self) -> Self {
        ark_ff::Field::inverse(self).unwrap_or_else(Fr::zero)
    }

    fn to_hex(&self) -> String {
        let mut bytes = to_bytes!(self.0).unwrap();
        bytes.reverse();
        hex::encode(bytes)
    }

    fn from_hex(hex_str: &str) -> Option<Self> {
        let dec_str = hex_to_decimal(hex_str);
        Fr::from_str(&dec_str).ok()
    }
}

// This is needed because arkworks only accepts arbitrary sized
// decimal strings and not hex strings
pub fn hex_to_decimal(value: &str) -> String {
    let value = value.strip_prefix("0x").unwrap_or(value);

    use num_bigint::BigInt;
    BigInt::parse_bytes(value.as_bytes(), 16)
        .unwrap()
        .to_str_radix(10)
}

// Taken from matter-labs: https://github.com/matter-labs/zksync/blob/6bfe1c06f5c00519ce14adf9827086119a50fae2/core/models/src/primitives.rs#L243
fn pack_bits_into_bytes(bits: Vec<bool>) -> Vec<u8> {
    // XXX(FIXME): Passing in just a field element
    // will trigger this panic for bn254.
    // The evaluator will need to pad the number of bits
    // accordingly.
    assert_eq!(
        bits.len() % 8,
        0,
        "input is not a multiple of 8, len is {}",
        bits.len()
    );
    let mut message_bytes: Vec<u8> = vec![];

    let byte_chunks = bits.chunks(8);
    for byte_chunk in byte_chunks {
        let mut byte = 0u8;
        for (i, bit) in byte_chunk.iter().enumerate() {
            if *bit {
                byte |= 1 << i;
            }
        }
        message_bytes.push(byte);
    }

    message_bytes
}
