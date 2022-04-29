use ark_ff::to_bytes;
use ark_ff::FpParameters;
use ark_ff::One;
use ark_ff::PrimeField;
use ark_ff::Zero;
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};

// XXX: Switch out for a trait and proper implementations
// This implementation is in-efficient, can definitely remove hex usage and Iterator instances for trivial functionality
#[derive(Clone, Copy, Eq, PartialOrd, Ord)]
pub struct FieldElement<F: PrimeField>(F);

impl<F: PrimeField> std::fmt::Display for FieldElement<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let big_f = BigUint::from_bytes_be(&self.to_bytes());
        let s = big_f.bits();
        let big_s = BigUint::one() << s;
        if big_s == big_f {
            return write!(f, "2^{}", s);
        }
        if big_f == BigUint::zero() {
            return write!(f, "0");
        }
        let big_minus = BigUint::from_bytes_be(&(self.neg()).to_bytes());
        if big_minus.to_string().len() < big_f.to_string().len() {
            return write!(f, "-{}", big_minus);
        }
        write!(f, "{}", big_f)
    }
}

impl<F: PrimeField> std::fmt::Debug for FieldElement<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if *self == -FieldElement::one() {
            return write!(f, "-1");
        }
        if *self == FieldElement::one() {
            return write!(f, "1");
        }

        let big_f = BigUint::from_bytes_be(&self.to_bytes());
        if big_f == BigUint::zero() {
            return write!(f, "0");
        }

        let s = big_f.bits();
        let big_s = BigUint::one() << s;
        if big_s == big_f {
            return write!(f, "2^{}", s);
        }
        if big_f.clone() % BigUint::from(2_u128).pow(32) == BigUint::zero() {
            return write!(f, "2^32*{}", big_f / BigUint::from(2_u128).pow(32));
        }

        write!(f, "{}", self)
    }
}

impl<F: PrimeField> std::hash::Hash for FieldElement<F> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write(&self.to_bytes())
    }
}

impl<F: PrimeField> PartialEq for FieldElement<F> {
    fn eq(&self, other: &Self) -> bool {
        self.to_bytes() == other.to_bytes()
    }
}

impl<F: PrimeField> From<i128> for FieldElement<F> {
    fn from(mut a: i128) -> FieldElement<F> {
        let mut negative = false;
        if a < 0 {
            a = -a;
            negative = true;
        }

        let mut result = match F::from_str(&a.to_string()) {
            Ok(result) => result,
            Err(_) => panic!("Cannot convert i128 as a string to a field element"),
        };

        if negative {
            result = -result;
        }
        FieldElement(result)
    }
}

impl<T: ark_ff::PrimeField> Serialize for FieldElement<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_hex().serialize(serializer)
    }
}

impl<'de, T: ark_ff::PrimeField> Deserialize<'de> for FieldElement<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = <&str>::deserialize(deserializer)?;
        match Self::from_hex(s) {
            Some(value) => Ok(value),
            None => Err(serde::de::Error::custom(format!(
                "Invalid hex for FieldElement: {}",
                s
            ))),
        }
    }
}

impl<F: PrimeField> From<u128> for FieldElement<F> {
    fn from(a: u128) -> FieldElement<F> {
        let result = match F::from_str(&a.to_string()) {
            Ok(result) => result,
            Err(_) => panic!("Cannot convert u128 as a string to a field element"),
        };
        FieldElement(result)
    }
}

impl<F: PrimeField> FieldElement<F> {
    pub fn one() -> FieldElement<F> {
        FieldElement(F::one())
    }
    pub fn zero() -> FieldElement<F> {
        FieldElement(F::zero())
    }

    pub fn is_zero(&self) -> bool {
        self == &Self::zero()
    }
    pub fn is_one(&self) -> bool {
        self == &Self::one()
    }

    pub fn pow(&self, exponent: &Self) -> Self {
        FieldElement(self.0.pow(exponent.0.into_repr()))
    }

    /// Maximum number of bits needed to represent a field element
    /// This is not the amount of bits being used to represent a field element
    /// Example, you only need 254 bits to represent a field element in BN256
    /// But the representation uses 256 bits, so the top two bits are always zero
    /// This method would return 254
    pub fn max_num_bits() -> u32 {
        F::Params::MODULUS_BITS
    }
    /// Returns None, if the string is not a canonical
    /// representation of a field element; less than the order
    /// or if the hex string is invalid.
    /// This method can be used for both hex and decimal representations.
    pub fn try_from_str(input: &str) -> Option<FieldElement<F>> {
        if input.contains('x') {
            return FieldElement::from_hex(input);
        }

        let fr = F::from_str(input).ok()?;
        Some(FieldElement(fr))
    }

    /// This is the number of bits required to represent this specific field element
    pub fn num_bits(&self) -> u32 {
        let bits = self.bits();
        // Iterate the number of bits and pop off all leading zeroes
        let iter = bits.iter().skip_while(|x| !(**x));
        // Note: count will panic if it goes over usize::MAX.
        // This may not be suitable for devices whose usize < u16
        iter.count() as u32
    }

    pub fn fits_in_u128(&self) -> bool {
        self.num_bits() <= 128
    }

    pub fn to_u128(self) -> u128 {
        use std::convert::TryInto;

        let bytes = self.to_bytes();
        u128::from_be_bytes(bytes[16..32].try_into().unwrap())
    }
    /// Computes the inverse or returns zero if the inverse does not exist
    /// Before using this FieldElement, please ensure that this behaviour is necessary
    pub fn inverse(&self) -> FieldElement<F> {
        let inv = self.0.inverse().unwrap_or_else(F::zero);
        FieldElement(inv)
    }

    // XXX: This method is used while this field element
    // implementation is not generic.
    pub fn into_repr(self) -> F {
        self.0
    }

    pub fn to_hex(self) -> String {
        let mut bytes = to_bytes!(self.0).unwrap();
        bytes.reverse();
        hex::encode(bytes)
    }
    pub fn from_hex(hex_str: &str) -> Option<FieldElement<F>> {
        let value = hex_str.strip_prefix("0x").unwrap_or(hex_str);
        let hex_as_bytes = hex::decode(value).ok()?;
        Some(FieldElement::from_be_bytes_reduce(&hex_as_bytes))
    }

    pub fn to_bytes(self) -> Vec<u8> {
        let mut bytes = to_bytes!(self.0).unwrap();
        bytes.reverse();
        bytes
    }

    /// Converts bytes into a FieldElement and applies a
    /// reduction if needed.
    pub fn from_be_bytes_reduce(bytes: &[u8]) -> FieldElement<F> {
        FieldElement(F::from_be_bytes_mod_order(bytes))
    }

    pub fn bits(&self) -> Vec<bool> {
        let bytes = self.to_bytes();
        let mut bits = Vec::with_capacity(bytes.len() * 8);
        for byte in bytes {
            let _bits = FieldElement::<F>::byte_to_bit(byte);
            bits.extend(_bits);
        }
        bits
    }

    fn byte_to_bit(byte: u8) -> Vec<bool> {
        let mut bits = Vec::with_capacity(8);
        for index in (0..=7).rev() {
            bits.push((byte & (1 << index)) >> index == 1)
        }
        bits
    }

    /// Returns the closest number of bytes to the bits specified
    /// This method truncates
    pub fn fetch_nearest_bytes(&self, num_bits: usize) -> Vec<u8> {
        fn nearest_bytes(num_bits: usize) -> usize {
            ((num_bits + 7) / 8) * 8
        }

        let num_bytes = nearest_bytes(num_bits);
        let num_elements = num_bytes / 8;

        let mut bytes = self.to_bytes();
        bytes.reverse(); // put it in big endian format. XXX(next refactor): we should be explicit about endianess.

        bytes[0..num_elements].to_vec()
    }

    // mask_to methods will not remove any bytes from the field
    // they are simply zeroed out
    // Whereas truncate_to will remove those bits and make the byte array smaller
    fn mask_to_bytes(&self, num_bits: u32) -> Vec<u8> {
        let mut bytes = self.to_bytes();
        mask_vector_le(&mut bytes, num_bits as usize);
        bytes.to_vec()
    }

    fn and_xor(&self, rhs: &FieldElement<F>, num_bits: u32, is_xor: bool) -> FieldElement<F> {
        // XXX: Gadgets like SHA256 need to have their input be a multiple of 8
        // This is not a restriction caused by SHA256, as it works on bits
        // but most backends assume bytes.
        // We could implicitly pad, however this may not be intuitive for users.
        // assert!(
        //     num_bits % 8 == 0,
        //     "num_bits is not a multiple of 8, it is {}",
        //     num_bits
        // );

        let lhs_bytes = self.mask_to_bytes(num_bits);
        let rhs_bytes = rhs.mask_to_bytes(num_bits);

        let and_byte_arr: Vec<_> = lhs_bytes
            .into_iter()
            .zip(rhs_bytes.into_iter())
            .map(|(lhs, rhs)| if is_xor { lhs ^ rhs } else { lhs & rhs })
            .collect();

        FieldElement::from_be_bytes_reduce(&and_byte_arr)
    }
    pub fn and(&self, rhs: &FieldElement<F>, num_bits: u32) -> FieldElement<F> {
        self.and_xor(rhs, num_bits, false)
    }
    pub fn xor(&self, rhs: &FieldElement<F>, num_bits: u32) -> FieldElement<F> {
        self.and_xor(rhs, num_bits, true)
    }
}

use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

impl<F: PrimeField> Neg for FieldElement<F> {
    type Output = FieldElement<F>;

    fn neg(self) -> Self::Output {
        FieldElement(-self.0)
    }
}

impl<F: PrimeField> Mul for FieldElement<F> {
    type Output = FieldElement<F>;
    fn mul(mut self, rhs: FieldElement<F>) -> Self::Output {
        self.0.mul_assign(&rhs.0);
        FieldElement(self.0)
    }
}
impl<F: PrimeField> Div for FieldElement<F> {
    type Output = FieldElement<F>;
    #[allow(clippy::suspicious_arithmetic_impl)]
    fn div(self, rhs: FieldElement<F>) -> Self::Output {
        self * rhs.inverse()
    }
}
impl<F: PrimeField> Add for FieldElement<F> {
    type Output = FieldElement<F>;
    fn add(mut self, rhs: FieldElement<F>) -> Self::Output {
        self.0.add_assign(&rhs.0);
        FieldElement(self.0)
    }
}
impl<F: PrimeField> AddAssign for FieldElement<F> {
    fn add_assign(&mut self, rhs: FieldElement<F>) {
        self.0.add_assign(&rhs.0);
    }
}

impl<F: PrimeField> Sub for FieldElement<F> {
    type Output = FieldElement<F>;
    fn sub(mut self, rhs: FieldElement<F>) -> Self::Output {
        self.0.sub_assign(&rhs.0);
        FieldElement(self.0)
    }
}
impl<F: PrimeField> SubAssign for FieldElement<F> {
    fn sub_assign(&mut self, rhs: FieldElement<F>) {
        self.0.sub_assign(&rhs.0);
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn and() {
        let max = 10_000u32;

        let num_bits = (std::mem::size_of::<u32>() * 8) as u32 - max.leading_zeros();

        for x in 0..max {
            let x = crate::generic_ark::FieldElement::<ark_bn254::Fr>::from(x as i128);
            let res = x.and(&x, num_bits);
            assert_eq!(res.to_bytes(), x.to_bytes());
        }
    }
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
