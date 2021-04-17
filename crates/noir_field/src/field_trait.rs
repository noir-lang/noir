use std::{
    fmt::{Debug, Display},
    hash::Hash,
    ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign},
};

pub trait FieldElement:
    Copy
    + Clone
    + Debug
    + Display
    + Default
    + Send
    + Sync
    + Eq
    + PartialEq
    + Hash
    + Sized
    + Add<Self, Output = Self>
    + Sub<Self, Output = Self>
    + AddAssign
    + SubAssign
    + Mul<Self, Output = Self>
    + Div<Self, Output = Self>
    + Neg<Output = Self>
    + Ord
    + From<u128>
    + From<u64>
    + From<u32>
    + From<u16>
    + From<u8>
{
    /// Converts field element to bytes
    fn to_bytes(&self) -> Vec<u8>;

    /// Converts bytes into a FieldElement. Does not reduce
    /// and panics if non-canonical
    // TODO: replace this with a for<'a> TryFrom<&'a [u8]> bound
    fn from_bytes(bytes: &[u8]) -> Self;

    /// Converts bytes into a FieldElement.
    /// Reducing modulo the field order
    fn from_bytes_reduce(bytes: &[u8]) -> Self;

    // XXX: -- Start: check usage of these functions

    // mask_to methods will not remove any bytes from the field
    // they are simply zeroed out
    // Whereas truncate_to will remove those bits and make the byte array smaller
    fn mask_to_field(&self, num_bits: u32) -> Self;

    fn mask_to_bytes(&self, num_bits: u32) -> Vec<u8>;

    fn bits(&self) -> Vec<bool>;

    fn mask_to_bits(&self, num_bits: u32) -> Vec<bool>;

    fn truncate_to_bits(&self, num_bits: u32) -> Vec<bool>;

    fn truncate_to_bytes(&self, num_bits: u32) -> Vec<u8>;

    /// Returns the closest number of bytes to the bits specified
    fn fetch_nearest_bytes(&self, num_bits: usize) -> Vec<u8>;

    fn and_xor(&self, rhs: &Self, num_bits: u32, is_xor: bool) -> Self;

    fn and(&self, rhs: &Self, num_bits: u32) -> Self;
    fn xor(&self, rhs: &Self, num_bits: u32) -> Self;

    // XXX: -- End

    /// Returns the representation of the number 1
    /// in the field
    fn one() -> Self;

    /// Returns the representation of the number 0
    /// in the field
    fn zero() -> Self;

    fn is_one(&self) -> bool {
        self == &Self::one()
    }
    fn is_zero(&self) -> bool {
        self == &Self::zero()
    }

    /// Maximum number of bits _needed_ to represent a field element
    /// This is not the amount of bits being _used_ to represent a field element
    /// Example, you only need 254 bits to represent a field element in BN256
    /// But the representation uses 256 bits, so the top two bits are always zero
    /// This method would return 254
    const MAX_NUM_BITS: u32;

    /// Returns None, if the string is not a canonical
    /// representation of a field element; less than the order
    /// or if the hex string is invalid.
    /// This method can be used for both hex and decimal representations.
    fn try_from_str(input: &str) -> Option<Self>;

    /// This is the amount of bits that are always zero,
    /// In BN256, every element can be represented with 254 bits.
    /// However this representation uses 256 bits, hence 2 wasted bits
    /// Note: This has nothing to do with saturated field elements.
    fn wasted_bits() -> u32;

    /// This is the number of bits required to represent this specific field element
    fn num_bits(&self) -> u32;

    /// Returns true if this number fits within a u128
    fn fits_in_u128(&self) -> bool {
        self.num_bits() <= 128
    }

    /// Casts a Field element as a u128
    //XXX: Change this to return Option, incase it cannot fit in a u128?
    fn to_u128(&self) -> u128;

    fn from_i128(a: i128) -> Self;

    /// Computes the inverse or returns zero if the inverse does not exist
    /// Do not panic, as we do not want the compiler to need to catch the unwind
    fn inverse(&self) -> Self;

    /// Returns the field element as a hex string
    fn to_hex(&self) -> String;

    /// Converts a hex string to a Field element
    fn from_hex(hex_str: &str) -> Option<Self>;
}
