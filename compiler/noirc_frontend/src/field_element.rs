//! Field element types for Sensei
//! 
//! This module provides the core numeric types used throughout the Sensei compiler.
//! - `FieldElement`: An unsigned 256-bit integer (U256)
//! - `SignedFieldElement`: A signed 256-bit integer (I256)

use alloy_primitives::{U256, I256};

/// The primary field element type - a wrapper around U256
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct FieldElement(pub U256);

impl FieldElement {
    /// Maximum value for FieldElement
    pub const MAX: Self = FieldElement(U256::MAX);
    
    /// Zero value for FieldElement
    pub const ZERO: Self = FieldElement(U256::ZERO);
    
    /// Create from big-endian bytes
    pub fn from_be_bytes(bytes: [u8; 32]) -> Self {
        FieldElement(U256::from_be_bytes(bytes))
    }
}

/// A signed field element type - a signed 256-bit integer  
pub type SignedFieldElement = I256;

/// Extension trait for FieldElement to provide compatibility methods
pub trait FieldElementExt {
    fn is_zero(&self) -> bool;
    fn is_one(&self) -> bool;
    fn zero() -> Self;
    fn one() -> Self;
    fn from_hex(hex: &str) -> Option<Self> where Self: Sized;
    fn to_hex(&self) -> String;
    fn to_be_bytes(&self) -> [u8; 32];
    fn from_be_bytes(bytes: &[u8]) -> Self where Self: Sized;
    fn try_into_u128(&self) -> Option<u128>;
    fn try_to_u64(&self) -> Option<u64>;
    fn try_to_u32(&self) -> Option<u32>;
}

impl FieldElementExt for FieldElement {
    fn is_zero(&self) -> bool {
        self.0 == U256::ZERO
    }

    fn is_one(&self) -> bool {
        self.0 == U256::from(1)
    }

    fn zero() -> Self {
        FieldElement(U256::ZERO)
    }

    fn one() -> Self {
        FieldElement(U256::from(1))
    }

    fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.strip_prefix("0x").unwrap_or(hex);
        U256::from_str_radix(hex, 16).ok().map(FieldElement)
    }

    fn to_hex(&self) -> String {
        format!("0x{}", self.0)
    }

    fn to_be_bytes(&self) -> [u8; 32] {
        self.0.to_be_bytes()
    }

    fn from_be_bytes(bytes: &[u8]) -> Self {
        if bytes.len() > 32 {
            return FieldElement(U256::ZERO);
        }
        let mut padded = [0u8; 32];
        padded[32 - bytes.len()..].copy_from_slice(bytes);
        FieldElement(U256::from_be_bytes(padded))
    }

    fn try_into_u128(&self) -> Option<u128> {
        // Check if the value fits in a u128
        if self.0 <= U256::from(u128::MAX) {
            // Safe to convert - we know it fits
            let bytes: [u8; 32] = self.0.to_le_bytes();
            let mut u128_bytes = [0u8; 16];
            u128_bytes.copy_from_slice(&bytes[..16]);
            Some(u128::from_le_bytes(u128_bytes))
        } else {
            None
        }
    }
    
    fn try_to_u64(&self) -> Option<u64> {
        self.try_into_u128().and_then(|v| if v <= u64::MAX as u128 { Some(v as u64) } else { None })
    }
    
    fn try_to_u32(&self) -> Option<u32> {
        self.try_into_u128().and_then(|v| if v <= u32::MAX as u128 { Some(v as u32) } else { None })
    }
}

/// Extension trait for SignedFieldElement
pub trait SignedFieldElementExt {
    fn is_negative(&self) -> bool;
    fn abs(&self) -> U256;
}

impl SignedFieldElementExt for SignedFieldElement {
    fn is_negative(&self) -> bool {
        self.is_negative()
    }

    fn abs(&self) -> U256 {
        self.unsigned_abs()
    }
}

/// Helper functions for creating field elements
pub mod field_helpers {
    use super::*;
    
    pub fn field_from_u32(value: u32) -> FieldElement {
        FieldElement(U256::from(value))
    }
    
    pub fn field_from_u64(value: u64) -> FieldElement {
        FieldElement(U256::from(value))
    }
    
    pub fn field_from_u128(value: u128) -> FieldElement {
        FieldElement(U256::from(value))
    }
    
    pub fn field_from_i128(value: i128) -> FieldElement {
        if value < 0 {
            // For negative values, we need to handle the two's complement representation
            let abs_value = value.unsigned_abs();
            // In a field, -x is equivalent to MODULUS - x
            // For now, we'll just store the absolute value
            // This will need proper field arithmetic later
            FieldElement(U256::from(abs_value))
        } else {
            FieldElement(U256::from(value as u128))
        }
    }
    
    pub fn signed_from_i32(value: i32) -> SignedFieldElement {
        I256::try_from(value as i128).expect("i32 should always fit in I256")
    }
    
    pub fn signed_from_i64(value: i64) -> SignedFieldElement {
        I256::try_from(value as i128).expect("i64 should always fit in I256")
    }
    
    pub fn signed_from_i128(value: i128) -> SignedFieldElement {
        I256::try_from(value).expect("i128 should always fit in I256")
    }
    
    pub fn field_to_signed(value: FieldElement) -> SignedFieldElement {
        I256::from_raw(value.0)
    }
    
    pub fn signed_to_field(value: SignedFieldElement) -> FieldElement {
        FieldElement(value.into_raw())
    }
}

// Implement required traits for FieldElement
impl std::fmt::Display for FieldElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::LowerHex for FieldElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

impl From<usize> for FieldElement {
    fn from(value: usize) -> Self {
        FieldElement(U256::from(value))
    }
}

impl From<u128> for FieldElement {
    fn from(value: u128) -> Self {
        FieldElement(U256::from(value))
    }
}

impl From<u32> for FieldElement {
    fn from(value: u32) -> Self {
        FieldElement(U256::from(value))
    }
}

impl From<bool> for FieldElement {
    fn from(value: bool) -> Self {
        FieldElement(if value { U256::from(1) } else { U256::ZERO })
    }
}

impl From<u64> for FieldElement {
    fn from(value: u64) -> Self {
        FieldElement(U256::from(value))
    }
}

impl From<i128> for FieldElement {
    fn from(value: i128) -> Self {
        if value < 0 {
            // For negative values, we need to handle the modular arithmetic
            // This is a stub implementation - proper modular arithmetic would be needed
            FieldElement(U256::from(value.unsigned_abs()))
        } else {
            FieldElement(U256::from(value as u128))
        }
    }
}

impl std::ops::Neg for FieldElement {
    type Output = Self;
    fn neg(self) -> Self::Output {
        todo!("neg implementation for FieldElement")
    }
}

impl std::ops::Add for FieldElement {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        FieldElement(self.0 + rhs.0)
    }
}

impl std::ops::Sub for FieldElement {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        FieldElement(self.0 - rhs.0)
    }
}

impl std::ops::Mul for FieldElement {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        FieldElement(self.0 * rhs.0)
    }
}

impl std::ops::Div for FieldElement {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        FieldElement(self.0 / rhs.0)
    }
}

impl std::ops::Rem for FieldElement {
    type Output = Self;
    fn rem(self, rhs: Self) -> Self::Output {
        FieldElement(self.0 % rhs.0)
    }
}

impl std::ops::AddAssign for FieldElement {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl std::ops::SubAssign for FieldElement {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}


// Implement TryFrom for common conversions
impl TryFrom<FieldElement> for u128 {
    type Error = &'static str;
    
    fn try_from(value: FieldElement) -> Result<Self, Self::Error> {
        value.try_into_u128().ok_or("Value too large for u128")
    }
}

impl TryFrom<FieldElement> for u64 {
    type Error = &'static str;
    
    fn try_from(value: FieldElement) -> Result<Self, Self::Error> {
        value.try_to_u64().ok_or("Value too large for u64")
    }
}

impl TryFrom<FieldElement> for u32 {
    type Error = &'static str;
    
    fn try_from(value: FieldElement) -> Result<Self, Self::Error> {
        value.try_to_u32().ok_or("Value too large for u32")
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::field_helpers::*;

    #[test]
    fn test_field_element_basics() {
        let zero = FieldElement::zero();
        let one = FieldElement::one();
        
        assert!(zero.is_zero());
        assert!(!one.is_zero());
        assert!(one.is_one());
        assert!(!zero.is_one());
    }

    #[test]
    fn test_conversions() {
        let fe = field_from_u32(42);
        assert_eq!(fe.try_into_u128(), Some(42u128));
        
        let big_fe = FieldElement(field_from_u128(u128::MAX).0 * U256::from(2));
        assert_eq!(big_fe.try_into_u128(), None);
    }

    #[test]
    fn test_hex_conversion() {
        let fe = field_from_u32(255);
        let hex = fe.to_hex();
        let fe2 = FieldElement::from_hex(&hex).unwrap();
        assert_eq!(fe, fe2);
    }

    #[test]
    fn test_signed_conversions() {
        let positive = signed_from_i32(42);
        let negative = signed_from_i32(-42);
        
        assert!(!positive.is_negative());
        assert!(negative.is_negative());
        
        assert_eq!(positive.abs(), U256::from(42));
        assert_eq!(negative.abs(), U256::from(42));
    }
}