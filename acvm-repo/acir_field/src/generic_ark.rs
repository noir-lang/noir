use ark_ff::PrimeField;
use ark_ff::Zero;
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// This trait is extremely unstable and WILL have breaking changes.
pub trait AcirField:
    std::marker::Sized
    + Display
    + Debug
    + Default
    + Clone
    + Copy
    + Neg<Output = Self>
    + Add<Self, Output = Self>
    + Sub<Self, Output = Self>
    + Mul<Self, Output = Self>
    + Div<Self, Output = Self>
    + PartialOrd
    + AddAssign<Self>
    + SubAssign<Self>
    + From<usize>
    + From<u128>
    // + From<u64>
    // + From<u32>
    // + From<u16>
    // + From<u8>
    + From<bool>
    + Hash
    + std::cmp::Eq
{
    fn one() -> Self;
    fn zero() -> Self;

    fn is_zero(&self) -> bool;
    fn is_one(&self) -> bool;

    fn pow(&self, exponent: &Self) -> Self;

    /// Maximum number of bits needed to represent a field element
    /// This is not the amount of bits being used to represent a field element
    /// Example, you only need 254 bits to represent a field element in BN256
    /// But the representation uses 256 bits, so the top two bits are always zero
    /// This method would return 254
    fn max_num_bits() -> u32;

    /// Maximum numbers of bytes needed to represent a field element
    /// We are not guaranteed that the number of bits being used to represent a field element
    /// will always be divisible by 8. If the case that it is not, we add one to the max number of bytes
    /// For example, a max bit size of 254 would give a max byte size of 32.
    fn max_num_bytes() -> u32;

    fn modulus() -> BigUint;

    /// This is the number of bits required to represent this specific field element
    fn num_bits(&self) -> u32;

    fn to_u128(self) -> u128;

    fn try_into_u128(self) -> Option<u128>;

    fn to_i128(self) -> i128;

    fn try_to_u64(&self) -> Option<u64>;

    /// Computes the inverse or returns zero if the inverse does not exist
    /// Before using this FieldElement, please ensure that this behavior is necessary
    fn inverse(&self) -> Self;

    fn to_hex(self) -> String;

    fn from_hex(hex_str: &str) -> Option<Self>;

    fn to_be_bytes(self) -> Vec<u8>;

    /// Converts bytes into a FieldElement and applies a reduction if needed.
    fn from_be_bytes_reduce(bytes: &[u8]) -> Self;

    /// Returns the closest number of bytes to the bits specified
    /// This method truncates
    fn fetch_nearest_bytes(&self, num_bits: usize) -> Vec<u8>;

    fn and(&self, rhs: &Self, num_bits: u32) -> Self;
    fn xor(&self, rhs: &Self, num_bits: u32) -> Self;
}

// XXX: Switch out for a trait and proper implementations
// This implementation is in-efficient, can definitely remove hex usage and Iterator instances for trivial functionality
#[derive(Default, Clone, Copy, Eq, PartialOrd, Ord)]
pub struct FieldElement<F: PrimeField>(F);

impl<F: PrimeField> std::fmt::Display for FieldElement<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // First check if the number is zero
        //
        let number = BigUint::from_bytes_be(&self.to_be_bytes());
        if number == BigUint::zero() {
            return write!(f, "0");
        }
        // Check if the negative version is smaller to represent
        //
        let minus_number = BigUint::from_bytes_be(&(self.neg()).to_be_bytes());
        let (smaller_repr, is_negative) =
            if minus_number.to_string().len() < number.to_string().len() {
                (minus_number, true)
            } else {
                (number, false)
            };
        if is_negative {
            write!(f, "-")?;
        }

        // Number of bits needed to represent the smaller representation
        let num_bits = smaller_repr.bits();

        // Check if the number represents a power of 2
        if smaller_repr.count_ones() == 1 {
            let mut bit_index = 0;
            for i in 0..num_bits {
                if smaller_repr.bit(i) {
                    bit_index = i;
                    break;
                }
            }
            return match bit_index {
                0 => write!(f, "1"),
                1 => write!(f, "2"),
                2 => write!(f, "4"),
                3 => write!(f, "8"),
                _ => write!(f, "2{}", superscript(bit_index)),
            };
        }

        // Check if number is a multiple of a power of 2.
        // This is used because when computing the quotient
        // we usually have numbers in the form 2^t * q + r
        // We focus on 2^64, 2^32, 2^16, 2^8, 2^4 because
        // they are common. We could extend this to a more
        // general factorization strategy, but we pay in terms of CPU time
        let mul_sign = "×";
        for power in [64, 32, 16, 8, 4] {
            let power_of_two = BigUint::from(2_u128).pow(power);
            if &smaller_repr % &power_of_two == BigUint::zero() {
                return write!(
                    f,
                    "2{}{}{}",
                    superscript(power as u64),
                    mul_sign,
                    smaller_repr / &power_of_two,
                );
            }
        }
        write!(f, "{smaller_repr}")
    }
}

impl<F: PrimeField> std::fmt::Debug for FieldElement<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

impl<F: PrimeField> std::hash::Hash for FieldElement<F> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write(&self.to_be_bytes());
    }
}

impl<F: PrimeField> PartialEq for FieldElement<F> {
    fn eq(&self, other: &Self) -> bool {
        self.to_be_bytes() == other.to_be_bytes()
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
        let s: Cow<'de, str> = Deserialize::deserialize(deserializer)?;
        match Self::from_hex(&s) {
            Some(value) => Ok(value),
            None => Err(serde::de::Error::custom(format!("Invalid hex for FieldElement: {s}",))),
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

impl<F: PrimeField> From<usize> for FieldElement<F> {
    fn from(a: usize) -> FieldElement<F> {
        FieldElement::from(a as u128)
    }
}

impl<F: PrimeField> From<bool> for FieldElement<F> {
    fn from(boolean: bool) -> FieldElement<F> {
        if boolean {
            FieldElement::one()
        } else {
            FieldElement::zero()
        }
    }
}

impl<F: PrimeField> FieldElement<F> {
    pub fn from_repr(field: F) -> Self {
        Self(field)
    }

    // XXX: This method is used while this field element
    // implementation is not generic.
    pub fn into_repr(self) -> F {
        self.0
    }

    fn is_negative(&self) -> bool {
        self.neg().num_bits() < self.num_bits()
    }

    fn fits_in_u128(&self) -> bool {
        self.num_bits() <= 128
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

    // mask_to methods will not remove any bytes from the field
    // they are simply zeroed out
    // Whereas truncate_to will remove those bits and make the byte array smaller
    fn mask_to_be_bytes(&self, num_bits: u32) -> Vec<u8> {
        let mut bytes = self.to_be_bytes();
        mask_vector_le(&mut bytes, num_bits as usize);
        bytes
    }

    fn bits(&self) -> Vec<bool> {
        fn byte_to_bit(byte: u8) -> Vec<bool> {
            let mut bits = Vec::with_capacity(8);
            for index in (0..=7).rev() {
                bits.push((byte & (1 << index)) >> index == 1);
            }
            bits
        }

        let bytes = self.to_be_bytes();
        let mut bits = Vec::with_capacity(bytes.len() * 8);
        for byte in bytes {
            let _bits = byte_to_bit(byte);
            bits.extend(_bits);
        }
        bits
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

        let lhs_bytes = self.mask_to_be_bytes(num_bits);
        let rhs_bytes = rhs.mask_to_be_bytes(num_bits);

        let and_byte_arr: Vec<_> = lhs_bytes
            .into_iter()
            .zip(rhs_bytes)
            .map(|(lhs, rhs)| if is_xor { lhs ^ rhs } else { lhs & rhs })
            .collect();

        FieldElement::from_be_bytes_reduce(&and_byte_arr)
    }
}

impl<F: PrimeField> AcirField for FieldElement<F> {
    fn one() -> FieldElement<F> {
        FieldElement(F::one())
    }
    fn zero() -> FieldElement<F> {
        FieldElement(F::zero())
    }

    fn is_zero(&self) -> bool {
        self == &Self::zero()
    }
    fn is_one(&self) -> bool {
        self == &Self::one()
    }

    fn pow(&self, exponent: &Self) -> Self {
        FieldElement(self.0.pow(exponent.0.into_bigint()))
    }

    /// Maximum number of bits needed to represent a field element
    /// This is not the amount of bits being used to represent a field element
    /// Example, you only need 254 bits to represent a field element in BN256
    /// But the representation uses 256 bits, so the top two bits are always zero
    /// This method would return 254
    fn max_num_bits() -> u32 {
        F::MODULUS_BIT_SIZE
    }

    /// Maximum numbers of bytes needed to represent a field element
    /// We are not guaranteed that the number of bits being used to represent a field element
    /// will always be divisible by 8. If the case that it is not, we add one to the max number of bytes
    /// For example, a max bit size of 254 would give a max byte size of 32.
    fn max_num_bytes() -> u32 {
        let num_bytes = Self::max_num_bits() / 8;
        if Self::max_num_bits() % 8 == 0 {
            num_bytes
        } else {
            num_bytes + 1
        }
    }

    fn modulus() -> BigUint {
        F::MODULUS.into()
    }

    /// This is the number of bits required to represent this specific field element
    fn num_bits(&self) -> u32 {
        let bits = self.bits();
        // Iterate the number of bits and pop off all leading zeroes
        let iter = bits.iter().skip_while(|x| !(**x));
        // Note: count will panic if it goes over usize::MAX.
        // This may not be suitable for devices whose usize < u16
        iter.count() as u32
    }

    fn to_u128(self) -> u128 {
        let bytes = self.to_be_bytes();
        u128::from_be_bytes(bytes[16..32].try_into().unwrap())
    }

    fn try_into_u128(self) -> Option<u128> {
        self.fits_in_u128().then(|| self.to_u128())
    }

    fn to_i128(self) -> i128 {
        let is_negative = self.is_negative();
        let bytes = if is_negative { self.neg() } else { self }.to_be_bytes();
        i128::from_be_bytes(bytes[16..32].try_into().unwrap()) * if is_negative { -1 } else { 1 }
    }

    fn try_to_u64(&self) -> Option<u64> {
        (self.num_bits() <= 64).then(|| self.to_u128() as u64)
    }

    /// Computes the inverse or returns zero if the inverse does not exist
    /// Before using this FieldElement, please ensure that this behavior is necessary
    fn inverse(&self) -> FieldElement<F> {
        let inv = self.0.inverse().unwrap_or_else(F::zero);
        FieldElement(inv)
    }

    fn to_hex(self) -> String {
        let mut bytes = Vec::new();
        self.0.serialize_uncompressed(&mut bytes).unwrap();
        bytes.reverse();
        hex::encode(bytes)
    }
    fn from_hex(hex_str: &str) -> Option<FieldElement<F>> {
        let value = hex_str.strip_prefix("0x").unwrap_or(hex_str);
        // Values of odd length require an additional "0" prefix
        let sanitized_value =
            if value.len() % 2 == 0 { value.to_string() } else { format!("0{}", value) };
        let hex_as_bytes = hex::decode(sanitized_value).ok()?;
        Some(FieldElement::from_be_bytes_reduce(&hex_as_bytes))
    }

    fn to_be_bytes(self) -> Vec<u8> {
        // to_be_bytes! uses little endian which is why we reverse the output
        // TODO: Add a little endian equivalent, so the caller can use whichever one
        // TODO they desire
        let mut bytes = Vec::new();
        self.0.serialize_uncompressed(&mut bytes).unwrap();
        bytes.reverse();
        bytes
    }

    /// Converts bytes into a FieldElement and applies a
    /// reduction if needed.
    fn from_be_bytes_reduce(bytes: &[u8]) -> FieldElement<F> {
        FieldElement(F::from_be_bytes_mod_order(bytes))
    }

    /// Returns the closest number of bytes to the bits specified
    /// This method truncates
    fn fetch_nearest_bytes(&self, num_bits: usize) -> Vec<u8> {
        fn nearest_bytes(num_bits: usize) -> usize {
            ((num_bits + 7) / 8) * 8
        }

        let num_bytes = nearest_bytes(num_bits);
        let num_elements = num_bytes / 8;

        let mut bytes = self.to_be_bytes();
        bytes.reverse(); // put it in big endian format. XXX(next refactor): we should be explicit about endianness.

        bytes[0..num_elements].to_vec()
    }

    fn and(&self, rhs: &FieldElement<F>, num_bits: u32) -> FieldElement<F> {
        self.and_xor(rhs, num_bits, false)
    }
    fn xor(&self, rhs: &FieldElement<F>, num_bits: u32) -> FieldElement<F> {
        self.and_xor(rhs, num_bits, true)
    }
}

use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;
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

// For pretty printing powers
fn superscript(n: u64) -> String {
    if n == 0 {
        "⁰".to_owned()
    } else if n == 1 {
        "¹".to_owned()
    } else if n == 2 {
        "²".to_owned()
    } else if n == 3 {
        "³".to_owned()
    } else if n == 4 {
        "⁴".to_owned()
    } else if n == 5 {
        "⁵".to_owned()
    } else if n == 6 {
        "⁶".to_owned()
    } else if n == 7 {
        "⁷".to_owned()
    } else if n == 8 {
        "⁸".to_owned()
    } else if n == 9 {
        "⁹".to_owned()
    } else if n >= 10 {
        superscript(n / 10) + &superscript(n % 10)
    } else {
        panic!("{}", n.to_string() + " can't be converted to superscript.");
    }
}

#[cfg(test)]
mod tests {
    use super::{AcirField, FieldElement};

    #[test]
    fn and() {
        let max = 10_000u32;

        let num_bits = (std::mem::size_of::<u32>() * 8) as u32 - max.leading_zeros();

        for x in 0..max {
            let x = FieldElement::<ark_bn254::Fr>::from(x as i128);
            let res = x.and(&x, num_bits);
            assert_eq!(res.to_be_bytes(), x.to_be_bytes());
        }
    }

    #[test]
    fn serialize_fixed_test_vectors() {
        // Serialized field elements from of 0, -1, -2, -3
        let hex_strings = vec![
            "0000000000000000000000000000000000000000000000000000000000000000",
            "30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000000",
            "30644e72e131a029b85045b68181585d2833e84879b9709143e1f593efffffff",
            "30644e72e131a029b85045b68181585d2833e84879b9709143e1f593effffffe",
        ];

        for (i, string) in hex_strings.into_iter().enumerate() {
            let minus_i_field_element = -FieldElement::<ark_bn254::Fr>::from(i as i128);
            assert_eq!(minus_i_field_element.to_hex(), string);
        }
    }

    #[test]
    fn deserialize_even_and_odd_length_hex() {
        // Test cases of (odd, even) length hex strings
        let hex_strings =
            vec![("0x0", "0x00"), ("0x1", "0x01"), ("0x002", "0x0002"), ("0x00003", "0x000003")];
        for (i, case) in hex_strings.into_iter().enumerate() {
            let i_field_element = FieldElement::<ark_bn254::Fr>::from(i as i128);
            let odd_field_element = FieldElement::<ark_bn254::Fr>::from_hex(case.0).unwrap();
            let even_field_element = FieldElement::<ark_bn254::Fr>::from_hex(case.1).unwrap();

            assert_eq!(i_field_element, odd_field_element);
            assert_eq!(odd_field_element, even_field_element);
        }
    }

    #[test]
    fn max_num_bits_smoke() {
        let max_num_bits_bn254 = FieldElement::<ark_bn254::Fr>::max_num_bits();
        assert_eq!(max_num_bits_bn254, 254);
    }
}
