use ark_ff::PrimeField;
use ark_ff::Zero;
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

use crate::AcirField;

// XXX: Switch out for a trait and proper implementations
// This implementation is inefficient, can definitely remove hex usage and Iterator instances for trivial functionality
#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

        write!(f, "{smaller_repr}")
    }
}

impl<F: PrimeField> std::fmt::Debug for FieldElement<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
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

impl<T: PrimeField> Serialize for FieldElement<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_hex().serialize(serializer)
    }
}

impl<'de, T: PrimeField> Deserialize<'de> for FieldElement<T> {
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
        FieldElement(F::from(a))
    }
}

impl<F: PrimeField> From<usize> for FieldElement<F> {
    fn from(a: usize) -> FieldElement<F> {
        FieldElement::from(a as u64)
    }
}

impl<F: PrimeField> From<u64> for FieldElement<F> {
    fn from(a: u64) -> FieldElement<F> {
        FieldElement(F::from(a))
    }
}

impl<F: PrimeField> From<u32> for FieldElement<F> {
    fn from(a: u32) -> FieldElement<F> {
        FieldElement(F::from(a))
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
        let bytes = self.to_be_bytes();

        // Iterate through the byte decomposition and pop off all leading zeroes
        let mut iter = bytes.iter().skip_while(|x| (**x) == 0);

        // The first non-zero byte in the decomposition may have some leading zero-bits.
        let Some(head_byte) = iter.next() else {
            // If we don't have a non-zero byte then the field element is zero,
            // which we consider to require a single bit to represent.
            return 1;
        };
        let num_bits_for_head_byte = head_byte.ilog2();

        // Each remaining byte in the byte decomposition requires 8 bits.
        //
        // Note: count will panic if it goes over usize::MAX.
        // This may not be suitable for devices whose usize < u16
        let tail_length = iter.count() as u32;

        8 * tail_length + num_bits_for_head_byte + 1
    }

    fn to_u128(self) -> u128 {
        let as_bigint = self.0.into_bigint();
        let limbs = as_bigint.as_ref();

        let mut result = limbs[0] as u128;
        if limbs.len() > 1 {
            let high_limb = limbs[1] as u128;
            result += high_limb << 64;
        }

        result
    }

    fn try_into_u128(self) -> Option<u128> {
        self.fits_in_u128().then(|| self.to_u128())
    }

    fn to_i128(self) -> i128 {
        // Negative integers are represented by the range [p + i128::MIN, p) whilst
        // positive integers are represented by the range [0, i128::MAX).
        // We can then differentiate positive from negative values by their MSB.
        let is_negative = self.neg().num_bits() < self.num_bits();
        let bytes = if is_negative { self.neg() } else { self }.to_be_bytes();
        i128::from_be_bytes(bytes[16..32].try_into().unwrap()) * if is_negative { -1 } else { 1 }
    }

    fn try_to_u64(&self) -> Option<u64> {
        (self.num_bits() <= 64).then(|| self.to_u128() as u64)
    }

    fn try_to_u32(&self) -> Option<u32> {
        (self.num_bits() <= 32).then(|| self.to_u128() as u32)
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
}

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
mod tests {
    use super::{AcirField, FieldElement};
    use proptest::prelude::*;

    #[test]
    fn requires_one_bit_to_hold_zero() {
        let field = FieldElement::<ark_bn254::Fr>::zero();
        assert_eq!(field.num_bits(), 1);
    }

    proptest! {
        #[test]
        fn num_bits_agrees_with_ilog2(num in 1u128..) {
            let field = FieldElement::<ark_bn254::Fr>::from(num);
            prop_assert_eq!(field.num_bits(), num.ilog2() + 1);
        }
    }

    #[test]
    fn test_fits_in_u128() {
        let field = FieldElement::<ark_bn254::Fr>::from(u128::MAX);
        assert_eq!(field.num_bits(), 128);
        assert!(field.fits_in_u128());
        let big_field = field + FieldElement::one();
        assert_eq!(big_field.num_bits(), 129);
        assert!(!big_field.fits_in_u128());
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
    fn max_num_bits_smoke() {
        let max_num_bits_bn254 = FieldElement::<ark_bn254::Fr>::max_num_bits();
        assert_eq!(max_num_bits_bn254, 254);
    }

    proptest! {
        // This currently panics due to the fact that we allow inputs which are greater than the field modulus,
        // automatically reducing them to fit within the canonical range.
        #[test]
        #[should_panic(expected = "serialized field element is not equal to input")]
        fn recovers_original_hex_string(hex in "[0-9a-f]{64}") {
            let fe: FieldElement::<ark_bn254::Fr> = FieldElement::from_hex(&hex).expect("should accept any 32 byte hex string");
            let output_hex = fe.to_hex();

            prop_assert_eq!(hex, output_hex, "serialized field element is not equal to input");
        }

        #[test]
        fn accepts_odd_length_hex_strings(hex in "(?:0x)[0-9a-fA-F]+") {
            // Here we inject a "0" immediately after the "0x" (if it exists) to construct an equivalent
            // hex string with the opposite parity length.
            let insert_index = if hex.starts_with("0x") { 2 } else { 0 };
            let mut opposite_parity_string = hex.to_string();
            opposite_parity_string.insert(insert_index, '0');

            let fe_1: FieldElement::<ark_bn254::Fr> = FieldElement::from_hex(&hex).unwrap();
            let fe_2: FieldElement::<ark_bn254::Fr> = FieldElement::from_hex(&opposite_parity_string).unwrap();

            prop_assert_eq!(fe_1, fe_2, "equivalent hex strings with opposite parity deserialized to different values");
        }
    }
}
