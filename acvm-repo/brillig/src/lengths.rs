use std::{
    iter::Sum,
    ops::{Add, AddAssign, Div, Mul},
};

use serde::{Deserialize, Serialize};

/// Represents the length of an array or vector as seen from a user's perspective.
/// For example in the array `[(u8, u16, [u32; 4]); 8]`, the semantic length is 8.
#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
pub struct SemanticLength(pub u32);

impl Add<SemanticLength> for SemanticLength {
    type Output = SemanticLength;

    /// Computes the sum of two semantic lengths.
    fn add(self, rhs: SemanticLength) -> Self::Output {
        SemanticLength(self.0 + rhs.0)
    }
}

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
#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
pub struct ElementsLength(pub u32);

impl<T> From<&[T]> for ElementsLength {
    fn from(elements: &[T]) -> Self {
        Self(assert_u32(elements.len()))
    }
}

impl<T> From<&Vec<T>> for ElementsLength {
    fn from(elements: &Vec<T>) -> Self {
        Self(assert_u32(elements.len()))
    }
}

impl Mul<SemanticLength> for ElementsLength {
    type Output = SemiFlattenedLength;

    /// Computes the semi-flattened length by multiplying the semantic length
    /// by the elements length.
    fn mul(self, rhs: SemanticLength) -> Self::Output {
        SemiFlattenedLength(self.0 * rhs.0)
    }
}

impl Mul<ElementsFlattenedLength> for SemanticLength {
    type Output = FlattenedLength;

    /// Computes the flattened length by multiplying the semantic length
    /// by the elements flattened length.
    fn mul(self, rhs: ElementsFlattenedLength) -> Self::Output {
        FlattenedLength(self.0 * rhs.0)
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
///
/// For example in the array `[(u8, u16, [u32; 4]); 8]`:
/// - The semantic length is 8
/// - The elements length is 3
/// - The semi-flattened length is 24 (8 * 3)
///
/// The reason the semi-flattened length is required, and different than the semantic length,
/// is that in our SSA tuples are flattened so the number of value slots needed to represent an
/// array is different than the semantic length
///
/// Note that this is different from the fully flattened length, which would be 8 * (1 + 1 + 4) = 48.
#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
#[cfg_attr(feature = "arb", derive(proptest_derive::Arbitrary))]
pub struct SemiFlattenedLength(pub u32);

impl std::fmt::Display for SemiFlattenedLength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T> From<&[T]> for SemiFlattenedLength {
    fn from(elements: &[T]) -> Self {
        Self(assert_u32(elements.len()))
    }
}

impl<T> From<&Vec<T>> for SemiFlattenedLength {
    fn from(elements: &Vec<T>) -> Self {
        Self(assert_u32(elements.len()))
    }
}

impl Div<ElementsLength> for SemiFlattenedLength {
    type Output = SemanticLength;

    fn div(self, rhs: ElementsLength) -> Self::Output {
        SemanticLength(self.0 / rhs.0)
    }
}

/// Represents the total number of fields required to represent a single entry of an array or vector.
/// For example in the array `[(u8, u16, [u32; 4]); 8]` the elements flattened length is 6:
/// 1. u8 (1)
/// 2. u16 (1)
/// 3. [u32; 4] (4)
#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
pub struct ElementsFlattenedLength(pub u32);

impl std::fmt::Display for ElementsFlattenedLength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Mul<SemanticLength> for ElementsFlattenedLength {
    type Output = FlattenedLength;

    /// Computes the flattened length by multiplying the semantic length
    /// by the elements flattened length.
    fn mul(self, rhs: SemanticLength) -> Self::Output {
        FlattenedLength(self.0 * rhs.0)
    }
}

impl From<FlattenedLength> for ElementsFlattenedLength {
    /// Assumes this flattened length represents a single entry in an array or vector,
    fn from(flattened_length: FlattenedLength) -> Self {
        Self(flattened_length.0)
    }
}

/// Represents the total number of fields required to represent the entirety of an array or vector.
/// For example in the array `[(u8, u16, [u32; 4]); 8]` the flattened length is 48: 8 * (1 + 1 + 4).
#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
pub struct FlattenedLength(pub u32);

impl std::fmt::Display for FlattenedLength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T> From<&[T]> for FlattenedLength {
    fn from(elements: &[T]) -> Self {
        Self(assert_u32(elements.len()))
    }
}

impl<T> From<&Vec<T>> for FlattenedLength {
    fn from(elements: &Vec<T>) -> Self {
        Self(assert_u32(elements.len()))
    }
}

impl Add for FlattenedLength {
    type Output = FlattenedLength;

    fn add(self, rhs: Self) -> Self::Output {
        FlattenedLength(self.0 + rhs.0)
    }
}

impl AddAssign for FlattenedLength {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Sum for FlattenedLength {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(FlattenedLength(0), |acc, x| acc + x)
    }
}

impl Div<ElementsFlattenedLength> for FlattenedLength {
    type Output = SemanticLength;

    fn div(self, rhs: ElementsFlattenedLength) -> Self::Output {
        SemanticLength(self.0 / rhs.0)
    }
}

/// Converts a usize value to u32, panicking if the conversion fails.
pub(crate) fn assert_u32(value: usize) -> u32 {
    value.try_into().expect("Failed conversion from usize to u32")
}
