use std::ops::Mul;

use serde::{Deserialize, Serialize};

/// Represents the length of an array or vector as seen from a user's perspective.
/// For example in the array `[(u8, u16, [u32; 4]); 8]`, the semantic length is 8.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub struct SemanticLength(pub usize);

impl Mul<ElementsLength> for SemanticLength {
    type Output = SemiFlattenedLength;

    /// Computes the semi-flattened length by multiplying the semantic length
    /// by the elements length.
    fn mul(self, rhs: ElementsLength) -> Self::Output {
        SemiFlattenedLength(self.0 * rhs.0)
    }
}

impl std::fmt::Display for SemanticLength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents the number of elements inside a vectors' or array's type, without
/// taking into account the vector or array length.
/// For example, in the array `[(u8, u16, [u32; 4]); 8]`, the elements length is 3:
/// 1. u8
/// 2. u16
/// 3. [u32; 4]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub struct ElementsLength(pub usize);

impl Mul<SemanticLength> for ElementsLength {
    type Output = SemiFlattenedLength;

    /// Computes the semi-flattened length by multiplying the semantic length
    /// by the elements length.
    fn mul(self, rhs: SemanticLength) -> Self::Output {
        SemiFlattenedLength(self.0 * rhs.0)
    }
}

impl std::fmt::Display for ElementsLength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents the number of value/memory slots required to represent an array or vector.
/// The semi-flattened length can be computed by multiplying the semantic length by
/// the elements length.
/// For example in the array `[(u8, u16, [u32; 4]); 8]`:
/// - The semantic length is 8
/// - The elements length is 3
/// - The semi-flattened length is 24 (8 * 3)
/// Note that this is different from the fully flattened length, which would be 8 * (1 + 1 + 4) = 48.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Hash)]
#[cfg_attr(feature = "arb", derive(proptest_derive::Arbitrary))]
pub struct SemiFlattenedLength(pub usize);

impl std::fmt::Display for SemiFlattenedLength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
