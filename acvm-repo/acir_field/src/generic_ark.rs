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
    // + From<u32>
    // + From<u16>
    // + From<u8>
    + From<bool>
    + std::hash::Hash
    + Eq
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

    fn try_to_u32(&self) -> Option<u32>;

    /// Computes the inverse or returns zero if the inverse does not exist
    /// Before using this FieldElement, please ensure that this behavior is necessary
    fn inverse(&self) -> Self;

    fn to_hex(self) -> String;

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
