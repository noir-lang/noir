use ark_ff::PrimeField;
use ark_ff::Zero;
use ark_std::io::Write;
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

use crate::AcirField;

/// The value 2^127, which represents the boundary between positive and negative
/// values in i128 representation. Values greater this are treated as negative when
/// converting to signed integers.
const I128_SIGN_BOUNDARY: u128 = 1_u128 << 127;

// XXX: Include a trait-based design with field-specific implementations.
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
    fn from(a: i128) -> FieldElement<F> {
        // Optimized: Convert directly without string conversion
        if a >= 0 {
            // Positive case: convert via u128
            FieldElement(F::from(a as u128))
        } else {
            // Negative case: handle i128::MIN specially to avoid overflow
            let abs_value = a.wrapping_neg() as u128;
            FieldElement(-F::from(abs_value))
        }
    }
}

impl<T: PrimeField> Serialize for FieldElement<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_be_bytes().serialize(serializer)
    }
}

impl<'de, T: PrimeField> Deserialize<'de> for FieldElement<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: Cow<'de, [u8]> = Deserialize::deserialize(deserializer)?;
        Ok(Self::from_be_bytes_reduce(&s))
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
        if boolean { FieldElement::one() } else { FieldElement::zero() }
    }
}

impl<F: PrimeField> FieldElement<F> {
    /// Constructs a `FieldElement` from the underlying prime field representation.
    ///
    /// This wraps an `ark_ff::PrimeField` element into a `FieldElement`.
    pub fn from_repr(field: F) -> Self {
        Self(field)
    }

    /// Extracts the underlying prime field representation.
    ///
    /// This returns the wrapped `ark_ff::PrimeField` element.
    pub fn into_repr(self) -> F {
        self.0
    }

    /// Returns true if this field element can be represented as a u128.
    ///
    /// A field element fits in u128 if it requires at most 128 bits to represent,
    /// i.e., if its value is in the range [0, 2^128 - 1].
    pub fn fits_in_u128(&self) -> bool {
        self.num_bits() <= 128
    }

    /// Returns true if this field element can be represented as an i128.
    ///
    /// An i128 can represent values in the range [i128::MIN, i128::MAX], which corresponds
    /// to field elements in [0, 2^127 - 1] (positive) and [p - 2^127, p - 1] (negative),
    /// where p is the field modulus. Note that 2^127 itself cannot be represented as i128
    /// since it is not in the negative range.
    pub fn fits_in_i128(&self) -> bool {
        let num_bits = u32::min(self.neg().num_bits(), self.num_bits());
        num_bits <= 127 && self != &FieldElement::from(I128_SIGN_BOUNDARY)
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

    /// Assume this field element holds a signed integer of the given `bit_size` and format
    /// it as a string. The range of valid values for this field element is `0..2^bit_size`
    /// with `0..2^(bit_size - 1)` representing positive values and `2^(bit_size - 1)..2^bit_size`
    /// representing negative values (as is commonly done for signed integers).
    /// `2^(bit_size - 1)` is the lowest negative value, so for example if bit_size is 8 then
    /// `0..127` map to `0..127`, `128` maps to `-128`, `129` maps to `-127` and `255` maps to `-1`.
    /// If `self` falls outside of the valid range it's formatted as-is.
    pub fn to_string_as_signed_integer(self, bit_size: u32) -> String {
        assert!(bit_size <= 128);
        if self.num_bits() > bit_size {
            return self.to_string();
        }

        // Compute the maximum value that is considered a positive value
        let max = if bit_size == 128 { i128::MAX as u128 } else { (1 << (bit_size - 1)) - 1 };
        if self.to_u128() > max {
            let f = FieldElement::from(2u32).pow(&bit_size.into()) - self;
            format!("-{f}")
        } else {
            self.to_string()
        }
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
        if Self::max_num_bits() % 8 == 0 { num_bytes } else { num_bytes + 1 }
    }

    fn modulus() -> BigUint {
        F::MODULUS.into()
    }

    /// This is the number of bits required to represent this specific field element
    fn num_bits(&self) -> u32 {
        let mut bit_counter = BitCounter::default();
        self.0.serialize_uncompressed(&mut bit_counter).unwrap();
        bit_counter.bits()
    }

    fn to_u128(self) -> u128 {
        if !self.fits_in_u128() {
            panic!("field element too large for u128");
        }
        let as_bigint = self.0.into_bigint();
        let limbs = as_bigint.as_ref();

        let mut result = u128::from(limbs[0]);
        if limbs.len() > 1 {
            let high_limb = u128::from(limbs[1]);
            result += high_limb << 64;
        }

        result
    }

    fn try_into_u128(self) -> Option<u128> {
        self.fits_in_u128().then(|| self.to_u128())
    }

    fn to_i128(self) -> i128 {
        if !self.fits_in_i128() {
            panic!("field element too large for i128");
        }
        // Negative integers are represented by the range [p + i128::MIN, p) while
        // positive integers are represented by the range [0, i128::MAX).
        // We can then differentiate positive from negative values by their MSB.
        if self.neg().num_bits() < self.num_bits() {
            let bytes = self.neg().to_be_bytes();
            i128::from_be_bytes(bytes[16..32].try_into().unwrap()).neg()
        } else {
            let bytes = self.to_be_bytes();
            i128::from_be_bytes(bytes[16..32].try_into().unwrap())
        }
    }

    fn try_into_i128(self) -> Option<i128> {
        self.fits_in_i128().then(|| self.to_i128())
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
        let bytes = self.to_be_bytes();
        hex::encode(bytes)
    }

    fn to_short_hex(self) -> String {
        if self.is_zero() {
            return "0x00".to_owned();
        }

        // Work directly with bytes
        let bytes = self.to_be_bytes();

        // Find the first non-zero byte
        let first_nonzero = bytes.iter().position(|&b| b != 0).unwrap_or(bytes.len());
        let trimmed = &bytes[first_nonzero..];

        // Build the hex string directly
        // Pre-allocate: "0x" + at least 2 chars per byte
        let mut result = String::with_capacity(2 + trimmed.len() * 2);
        result.push_str("0x");

        // Format the first byte - use {:x} to avoid leading zero if byte >= 0x10
        use std::fmt::Write;
        write!(&mut result, "{:x}", trimmed[0]).unwrap();

        // Ensure even length by padding if necessary
        if result.len() % 2 != 0 {
            // Insert '0' after "0x" to make it even
            result.insert(2, '0');
        }

        // Format remaining bytes with padding
        for byte in &trimmed[1..] {
            write!(&mut result, "{byte:02x}").unwrap();
        }

        result
    }

    fn from_hex(hex_str: &str) -> Option<FieldElement<F>> {
        let value = hex_str.strip_prefix("0x").unwrap_or(hex_str);

        // Decode directly, handling even length efficiently
        let hex_as_bytes = if value.len() % 2 == 0 {
            hex::decode(value).ok()?
        } else {
            // For odd length, prepend '0' to the string view only for decoding
            let mut padded = String::with_capacity(value.len() + 1);
            padded.push('0');
            padded.push_str(value);
            hex::decode(padded).ok()?
        };

        Some(FieldElement::from_be_bytes_reduce(&hex_as_bytes))
    }

    fn to_be_bytes(self) -> Vec<u8> {
        let mut bytes = self.to_le_bytes();
        bytes.reverse();
        bytes
    }

    /// Converts the field element to a vector of bytes in little-endian order
    fn to_le_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::new();
        self.0.serialize_uncompressed(&mut bytes).unwrap();
        bytes
    }

    /// Converts bytes into a FieldElement and applies a
    /// reduction if needed.
    fn from_be_bytes_reduce(bytes: &[u8]) -> FieldElement<F> {
        FieldElement(F::from_be_bytes_mod_order(bytes))
    }

    /// Converts bytes in little-endian order into a FieldElement and applies a
    /// reduction if needed.
    fn from_le_bytes_reduce(bytes: &[u8]) -> FieldElement<F> {
        FieldElement(F::from_le_bytes_mod_order(bytes))
    }

    /// Returns the closest number of bytes to the bits specified
    /// This method truncates
    fn fetch_nearest_bytes(&self, num_bits: usize) -> Vec<u8> {
        fn nearest_bytes(num_bits: usize) -> usize {
            num_bits.div_ceil(8) * 8
        }

        let num_bytes = nearest_bytes(num_bits);
        let num_elements = num_bytes / 8;

        let bytes = self.to_le_bytes();

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
        self.add_assign(rhs);
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
        self.sub_assign(rhs);
        FieldElement(self.0)
    }
}
impl<F: PrimeField> SubAssign for FieldElement<F> {
    fn sub_assign(&mut self, rhs: FieldElement<F>) {
        self.0.sub_assign(&rhs.0);
    }
}

#[derive(Default, Debug)]
struct BitCounter {
    /// Total number of non-zero bytes we found.
    count: usize,
    /// Total bytes we found.
    total: usize,
    /// The last non-zero byte we found.
    head_byte: u8,
}

impl BitCounter {
    fn bits(&self) -> u32 {
        // If we don't have a non-zero byte then the field element is zero,
        // which we consider to require a zero bits to represent.
        if self.count == 0 {
            return 0;
        }

        let num_bits_for_head_byte = self.head_byte.ilog2();

        // Each remaining byte in the byte decomposition requires 8 bits.
        //
        // Note: count will panic if it goes over usize::MAX.
        // This may not be suitable for devices whose usize < u16
        let tail_length = (self.count - 1) as u32;
        8 * tail_length + num_bits_for_head_byte + 1
    }
}

impl Write for BitCounter {
    fn write(&mut self, buf: &[u8]) -> ark_std::io::Result<usize> {
        for byte in buf {
            self.total += 1;
            if *byte != 0 {
                self.count = self.total;
                self.head_byte = *byte;
            }
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> ark_std::io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{AcirField, FieldElement};
    use proptest::prelude::*;

    #[test]
    fn requires_zero_bit_to_hold_zero() {
        let field = FieldElement::<ark_bn254::Fr>::zero();
        assert_eq!(field.num_bits(), 0);
    }

    #[test]
    fn requires_one_bit_to_hold_one() {
        let field = FieldElement::<ark_bn254::Fr>::one();
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
    fn test_to_u128_basic() {
        type F = FieldElement<ark_bn254::Fr>;

        // Test zero
        assert_eq!(F::zero().to_u128(), 0);

        // Test small values
        assert_eq!(F::from(1_u128).to_u128(), 1);
        assert_eq!(F::from(42_u128).to_u128(), 42);
        assert_eq!(F::from(1000_u128).to_u128(), 1000);

        // Test u128::MAX
        assert_eq!(F::from(u128::MAX).to_u128(), u128::MAX);

        // Test power of 2 boundaries
        assert_eq!(F::from(1_u128 << 127).to_u128(), 1_u128 << 127);
        assert_eq!(F::from((1_u128 << 127) - 1).to_u128(), (1_u128 << 127) - 1);
    }

    #[test]
    #[should_panic(expected = "field element too large for u128")]
    fn test_to_u128_panics_on_overflow() {
        type F = FieldElement<ark_bn254::Fr>;

        // Create a field element larger than u128::MAX
        let too_large = F::from(u128::MAX) + F::one();
        too_large.to_u128(); // Should panic
    }

    #[test]
    fn test_try_into_u128() {
        type F = FieldElement<ark_bn254::Fr>;

        // Valid conversions
        assert_eq!(F::zero().try_into_u128(), Some(0));
        assert_eq!(F::from(42_u128).try_into_u128(), Some(42));
        assert_eq!(F::from(u128::MAX).try_into_u128(), Some(u128::MAX));

        // Invalid conversion
        let too_large = F::from(u128::MAX) + F::one();
        assert_eq!(too_large.try_into_u128(), None);
    }

    #[test]
    fn test_fits_in_i128() {
        type F = FieldElement<ark_bn254::Fr>;

        // Positive values that fit
        assert!(F::zero().fits_in_i128());
        assert!(F::from(1_i128).fits_in_i128());
        assert!(F::from(42_i128).fits_in_i128());
        assert!(F::from(i128::MAX).fits_in_i128());

        // Negative values that fit (except i128::MIN)
        assert!(F::from(-1_i128).fits_in_i128());
        assert!(F::from(-42_i128).fits_in_i128());
        assert!(F::from(i128::MIN + 1).fits_in_i128());

        // Boundary: 2^127 - 1 fits (i128::MAX)
        assert!(F::from((1_u128 << 127) - 1).fits_in_i128());

        // Boundary: 2^127 does NOT fit (exceeds i128::MAX, not negative)
        // Note: This also means i128::MIN doesn't fit, as it converts to a field element
        // that when interpreted as unsigned equals 2^127
        assert!(!F::from(1_u128 << 127).fits_in_i128());
        assert!(!F::from(i128::MIN).fits_in_i128());

        // Values that don't fit
        let too_large = F::from(u128::MAX);
        assert!(!too_large.fits_in_i128());
    }

    #[test]
    fn test_to_i128_positive() {
        type F = FieldElement<ark_bn254::Fr>;

        // Test positive values
        assert_eq!(F::zero().to_i128(), 0);
        assert_eq!(F::from(1_i128).to_i128(), 1);
        assert_eq!(F::from(42_i128).to_i128(), 42);
        assert_eq!(F::from(1000_i128).to_i128(), 1000);
        assert_eq!(F::from(i128::MAX).to_i128(), i128::MAX);
    }

    #[test]
    fn test_to_i128_negative() {
        type F = FieldElement<ark_bn254::Fr>;

        // Test negative values
        assert_eq!(F::from(-1_i128).to_i128(), -1);
        assert_eq!(F::from(-42_i128).to_i128(), -42);
        assert_eq!(F::from(-1000_i128).to_i128(), -1000);

        // Test boundary values
        assert_eq!(F::from(-i128::MAX).to_i128(), -i128::MAX);
        assert_eq!(F::from(i128::MIN + 1).to_i128(), i128::MIN + 1);

        // i128::MIN doesn't fit
    }

    #[test]
    fn test_to_i128_roundtrip() {
        type F = FieldElement<ark_bn254::Fr>;

        // Test roundtrip for various values
        // i128::MIN doesn't fit
        let test_values =
            vec![0_i128, 1, -1, 42, -42, i128::MAX, i128::MAX - 1, i128::MIN + 1, -i128::MAX];

        for value in test_values {
            let field = F::from(value);
            assert!(field.fits_in_i128(), "Value {value} should fit in i128");
            assert_eq!(field.to_i128(), value, "Roundtrip failed for {value}");
        }
    }

    #[test]
    #[should_panic(expected = "field element too large for i128")]
    fn test_to_i128_panics_on_positive_overflow() {
        type F = FieldElement<ark_bn254::Fr>;

        // 2^127 is too large (exceeds i128::MAX)
        let too_large = F::from(1_u128 << 127);
        too_large.to_i128(); // Should panic
    }

    #[test]
    #[should_panic(expected = "field element too large for i128")]
    fn test_to_i128_panics_on_large_value() {
        type F = FieldElement<ark_bn254::Fr>;

        // Large positive value that doesn't fit
        let too_large = F::from(u128::MAX);
        too_large.to_i128(); // Should panic
    }

    #[test]
    fn test_try_into_i128() {
        type F = FieldElement<ark_bn254::Fr>;

        // Valid positive conversions
        assert_eq!(F::zero().try_into_i128(), Some(0));
        assert_eq!(F::from(42_i128).try_into_i128(), Some(42));
        assert_eq!(F::from(i128::MAX).try_into_i128(), Some(i128::MAX));

        // Valid negative conversions
        assert_eq!(F::from(-1_i128).try_into_i128(), Some(-1));
        assert_eq!(F::from(-42_i128).try_into_i128(), Some(-42));
        assert_eq!(F::from(i128::MIN + 1).try_into_i128(), Some(i128::MIN + 1));

        // Invalid conversions
        assert_eq!(F::from(1_u128 << 127).try_into_i128(), None);
        assert_eq!(F::from(u128::MAX).try_into_i128(), None);
        // i128::MIN doesn't fit due to implementation
        assert_eq!(F::from(i128::MIN).try_into_i128(), None);
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
        #[test]
        fn test_endianness_prop(value in any::<u64>()) {
            let field = FieldElement::<ark_bn254::Fr>::from(value);
            // Test serialization consistency
            let le_bytes = field.to_le_bytes();
            let be_bytes = field.to_be_bytes();

            let mut reversed_le = le_bytes.clone();
            reversed_le.reverse();
            prop_assert_eq!(&be_bytes, &reversed_le, "BE bytes should be reverse of LE bytes");

            // Test deserialization consistency
            let from_le = FieldElement::from_le_bytes_reduce(&le_bytes);
            let from_be = FieldElement::from_be_bytes_reduce(&be_bytes);
            prop_assert_eq!(from_le, from_be, "Deserialization should be consistent between LE and BE");
            prop_assert_eq!(from_le, field, "Deserialized value should match original");
        }
    }

    #[test]
    fn test_endianness() {
        let field = FieldElement::<ark_bn254::Fr>::from(0x1234_5678_u32);
        let le_bytes = field.to_le_bytes();
        let be_bytes = field.to_be_bytes();

        // Check that the bytes are reversed between BE and LE
        let mut reversed_le = le_bytes.clone();
        reversed_le.reverse();
        assert_eq!(&be_bytes, &reversed_le);

        // Verify we can reconstruct the same field element from either byte order
        let from_le = FieldElement::from_le_bytes_reduce(&le_bytes);
        let from_be = FieldElement::from_be_bytes_reduce(&be_bytes);
        assert_eq!(from_le, from_be);
        assert_eq!(from_le, field);

        // Additional test with a larger number to ensure proper byte handling
        let large_field = FieldElement::<ark_bn254::Fr>::from(0x0123_4567_89AB_CDEF_u64); // cSpell:disable-line
        let large_le = large_field.to_le_bytes();
        let reconstructed = FieldElement::from_le_bytes_reduce(&large_le);
        assert_eq!(reconstructed, large_field);
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

    #[test]
    fn test_to_hex() {
        type F = FieldElement<ark_bn254::Fr>;
        assert_eq!(
            F::zero().to_hex(),
            "0000000000000000000000000000000000000000000000000000000000000000"
        );
        assert_eq!(
            F::one().to_hex(),
            "0000000000000000000000000000000000000000000000000000000000000001"
        );
        assert_eq!(
            F::from(0x123_u128).to_hex(),
            "0000000000000000000000000000000000000000000000000000000000000123"
        );
        assert_eq!(
            F::from(0x1234_u128).to_hex(),
            "0000000000000000000000000000000000000000000000000000000000001234"
        );
    }

    #[test]
    fn test_to_short_hex() {
        type F = FieldElement<ark_bn254::Fr>;
        assert_eq!(F::zero().to_short_hex(), "0x00");
        assert_eq!(F::one().to_short_hex(), "0x01");
        assert_eq!(F::from(0x123_u128).to_short_hex(), "0x0123");
        assert_eq!(F::from(0x1234_u128).to_short_hex(), "0x1234");
    }

    #[test]
    fn to_string_as_signed_integer() {
        type F = FieldElement<ark_bn254::Fr>;
        assert_eq!(F::zero().to_string_as_signed_integer(8), "0");
        assert_eq!(F::one().to_string_as_signed_integer(8), "1");
        assert_eq!(F::from(127_u128).to_string_as_signed_integer(8), "127");
        assert_eq!(F::from(128_u128).to_string_as_signed_integer(8), "-128");
        assert_eq!(F::from(129_u128).to_string_as_signed_integer(8), "-127");
        assert_eq!(F::from(255_u128).to_string_as_signed_integer(8), "-1");
        assert_eq!(F::from(32767_u128).to_string_as_signed_integer(16), "32767");
        assert_eq!(F::from(32768_u128).to_string_as_signed_integer(16), "-32768");
        assert_eq!(F::from(65535_u128).to_string_as_signed_integer(16), "-1");
    }
}
