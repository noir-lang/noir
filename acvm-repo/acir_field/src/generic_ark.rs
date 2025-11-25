use num_bigint::BigUint;

/// This trait is extremely unstable and WILL have breaking changes.
pub trait AcirField:
    Sized
    + std::fmt::Display
    + std::fmt::Debug
    + Default
    + Clone
    + Copy
    + std::ops::Neg<Output = Self>
    + std::ops::Add<Self, Output = Self>
    + std::ops::Sub<Self, Output = Self>
    + std::ops::Mul<Self, Output = Self>
    + std::ops::Div<Self, Output = Self>
    + std::ops::AddAssign<Self>
    + std::ops::SubAssign<Self>
    + PartialOrd
    + From<usize>
    + From<u128>
    // + From<u64>
    + From<u32>
    // + From<u16>
    // + From<u8>
    + From<bool>
    + std::hash::Hash
    + Eq
    + 'static
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

    /// Downcast the field into a `u128`.
    /// Panic if the value does not fit
    fn to_u128(self) -> u128;

    /// Downcast the field into a `u128` if it fits into 128 bits, otherwise return `None`.
    fn try_into_u128(self) -> Option<u128>;

    /// Downcast the field into a `i128`.
    /// Panic if the value does not fit
    fn to_i128(self) -> i128;

    fn try_into_i128(self) -> Option<i128>;

    fn try_to_u64(&self) -> Option<u64>;

    fn try_to_u32(&self) -> Option<u32>;

    /// Computes the inverse or returns zero if the inverse does not exist
    /// Before using this FieldElement, please ensure that this behavior is necessary
    fn inverse(&self) -> Self;

    /// Returns the vale of this field as a hex string without the `0x` prefix.
    /// The returned string will have a length equal to the maximum number of hex
    /// digits needed to represent the maximum value of this field.
    fn to_hex(self) -> String;

    /// Returns the value of this field as a hex string with leading zeroes removed,
    /// prepended with `0x`.
    /// A singular '0' will be prepended as well if the trimmed string has an odd length.
    fn to_short_hex(self) -> String;

    fn from_hex(hex_str: &str) -> Option<Self>;

    fn to_be_bytes(self) -> Vec<u8>;

    /// Converts bytes into a FieldElement and applies a reduction if needed.
    fn from_be_bytes_reduce(bytes: &[u8]) -> Self;

    /// Converts bytes in little-endian order into a FieldElement and applies a reduction if needed.
    fn from_le_bytes_reduce(bytes: &[u8]) -> Self;

    /// Converts the field element to a vector of bytes in little-endian order
    fn to_le_bytes(self) -> Vec<u8>;

    /// Returns the closest number of bytes to the bits specified
    /// This method truncates
    fn fetch_nearest_bytes(&self, num_bits: usize) -> Vec<u8>;
}

/// Define a _newtype_ wrapper around an `AcirField` by implementing all the
/// boilerplate for forwarding the field operations.
///
/// This allows the wrapper to implement traits such as `Arbitrary`, and then
/// be used by code that is generic in `F: AcirField`.
///
/// # Example
/// ```ignore
/// field_wrapper!(TestField, FieldElement);
/// ```
#[macro_export]
macro_rules! field_wrapper {
    ($wrapper:ident, $field:ident) => {
        #[derive(
            Clone,
            Debug,
            PartialEq,
            Eq,
            Hash,
            PartialOrd,
            Copy,
            Default,
            serde::Serialize,
            serde::Deserialize,
        )]
        struct $wrapper(pub $field);

        impl $crate::AcirField for $wrapper {
            fn one() -> Self {
                Self($field::one())
            }

            fn zero() -> Self {
                Self($field::zero())
            }

            fn is_zero(&self) -> bool {
                self.0.is_zero()
            }

            fn is_one(&self) -> bool {
                self.0.is_one()
            }

            fn pow(&self, exponent: &Self) -> Self {
                Self(self.0.pow(&exponent.0))
            }

            fn max_num_bits() -> u32 {
                $field::max_num_bits()
            }

            fn max_num_bytes() -> u32 {
                $field::max_num_bytes()
            }

            fn modulus() -> ::num_bigint::BigUint {
                $field::modulus()
            }

            fn num_bits(&self) -> u32 {
                self.0.num_bits()
            }

            fn to_u128(self) -> u128 {
                self.0.to_u128()
            }

            fn try_into_u128(self) -> Option<u128> {
                self.0.try_into_u128()
            }

            fn try_into_i128(self) -> Option<i128> {
                self.0.try_into_i128()
            }

            fn to_i128(self) -> i128 {
                self.0.to_i128()
            }

            fn try_to_u64(&self) -> Option<u64> {
                self.0.try_to_u64()
            }

            fn try_to_u32(&self) -> Option<u32> {
                self.0.try_to_u32()
            }

            fn inverse(&self) -> Self {
                Self(self.0.inverse())
            }

            fn to_hex(self) -> String {
                self.0.to_hex()
            }

            fn to_short_hex(self) -> String {
                self.0.to_short_hex()
            }

            fn from_hex(hex_str: &str) -> Option<Self> {
                $field::from_hex(hex_str).map(Self)
            }

            fn to_be_bytes(self) -> Vec<u8> {
                self.0.to_be_bytes()
            }

            fn from_be_bytes_reduce(bytes: &[u8]) -> Self {
                Self($field::from_be_bytes_reduce(bytes))
            }

            fn from_le_bytes_reduce(bytes: &[u8]) -> Self {
                Self($field::from_le_bytes_reduce(bytes))
            }

            fn to_le_bytes(self) -> Vec<u8> {
                self.0.to_le_bytes()
            }

            fn fetch_nearest_bytes(&self, num_bits: usize) -> Vec<u8> {
                self.0.fetch_nearest_bytes(num_bits)
            }
        }

        impl From<bool> for $wrapper {
            fn from(value: bool) -> Self {
                Self($field::from(value))
            }
        }

        impl From<u128> for $wrapper {
            fn from(value: u128) -> Self {
                Self($field::from(value))
            }
        }

        impl From<u32> for $wrapper {
            fn from(value: u32) -> Self {
                Self($field::from(value))
            }
        }

        impl From<usize> for $wrapper {
            fn from(value: usize) -> Self {
                Self($field::from(value))
            }
        }

        impl std::ops::SubAssign<$wrapper> for $wrapper {
            fn sub_assign(&mut self, rhs: $wrapper) {
                self.0.sub_assign(rhs.0);
            }
        }

        impl std::ops::AddAssign<$wrapper> for $wrapper {
            fn add_assign(&mut self, rhs: $wrapper) {
                self.0.add_assign(rhs.0);
            }
        }

        impl std::ops::Add<$wrapper> for $wrapper {
            type Output = Self;

            fn add(self, rhs: $wrapper) -> Self::Output {
                Self(self.0.add(rhs.0))
            }
        }

        impl std::ops::Sub<$wrapper> for $wrapper {
            type Output = Self;

            fn sub(self, rhs: $wrapper) -> Self::Output {
                Self(self.0.sub(rhs.0))
            }
        }

        impl std::ops::Mul<$wrapper> for $wrapper {
            type Output = Self;

            fn mul(self, rhs: $wrapper) -> Self::Output {
                Self(self.0.mul(rhs.0))
            }
        }

        impl std::ops::Div<$wrapper> for $wrapper {
            type Output = Self;

            fn div(self, rhs: $wrapper) -> Self::Output {
                Self(self.0.div(rhs.0))
            }
        }

        impl std::ops::Neg for $wrapper {
            type Output = Self;

            fn neg(self) -> Self::Output {
                Self(self.0.neg())
            }
        }

        impl std::fmt::Display for $wrapper {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}
