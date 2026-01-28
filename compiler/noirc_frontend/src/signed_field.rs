use acvm::{AcirField, FieldElement};
use num_bigint::{BigInt, BigUint, Sign};
use num_traits::{One, ToPrimitive, Zero};

/// A signed integer type that can represent arbitrarily large values.
/// This is field-agnostic and uses BigUint internally instead of FieldElement.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SignedField {
    integer: BigUint,
    is_negative: bool,
}

impl SignedField {
    pub fn new(integer: BigUint, mut is_negative: bool) -> Self {
        if integer.is_zero() {
            is_negative = false;
        }
        Self { integer, is_negative }
    }

    pub fn positive(integer: impl Into<BigUint>) -> Self {
        Self { integer: integer.into(), is_negative: false }
    }

    pub fn negative(integer: impl Into<BigUint>) -> Self {
        Self::new(integer.into(), true)
    }

    pub fn zero() -> SignedField {
        Self { integer: BigUint::zero(), is_negative: false }
    }

    pub fn one() -> SignedField {
        Self { integer: BigUint::one(), is_negative: false }
    }

    /// Returns the inner BigUint which will always be positive (the absolute value)
    pub fn absolute_value(&self) -> BigUint {
        self.integer.clone()
    }

    pub fn is_negative(&self) -> bool {
        self.is_negative
    }

    pub fn is_positive(&self) -> bool {
        !self.is_negative
    }

    pub fn is_zero(&self) -> bool {
        self.integer.is_zero()
    }

    pub fn is_one(&self) -> bool {
        self.is_positive() && self.integer.is_one()
    }

    /// Convert a signed integer to a SignedField, carefully handling
    /// INT_MIN in the process. Note that to convert an unsigned integer
    /// you can call `SignedField::positive`.
    #[inline]
    pub fn from_signed<T>(value: T) -> Self
    where
        T: num_traits::Signed + AbsU128,
    {
        let negative = value.is_negative();
        let value = value.abs_u128();
        SignedField::new(value.into(), negative)
    }

    fn integer_to_u128(&self) -> Option<u128> {
        self.integer.to_u128()
    }

    /// Convert a SignedField into an unsigned integer type (up to u128),
    /// returning None if the value does not fit (e.g. if it is negative).
    #[inline]
    pub fn try_to_unsigned<T: TryFrom<u128>>(self) -> Option<T> {
        if self.is_negative {
            return None;
        }

        assert!(size_of::<T>() <= size_of::<u128>());
        let u128_value = self.integer_to_u128()?;
        u128_value.try_into().ok()
    }

    /// Convert a SignedField into a signed integer type (up to i128),
    /// returning None if the value does not fit. This function is more complex
    /// for handling negative values, specifically INT_MIN which we can't cast from
    /// a u128 to i128 without wrapping it.
    #[inline]
    pub fn try_to_signed<T>(self) -> Option<T>
    where
        T: TryFrom<u128> + TryFrom<i128> + num_traits::Signed + num_traits::Bounded + AbsU128,
        u128: TryFrom<T>,
    {
        let u128_value = self.integer_to_u128()?;

        if self.is_negative {
            // The positive version of the minimum value of this type.
            // E.g. 128 for i8.
            let positive_min = T::min_value().abs_u128();

            // If it is the min value, we can't negate it without overflowing
            // so test for it and return it directly
            if u128_value == positive_min {
                Some(T::min_value())
            } else {
                let i128_value = -(u128_value as i128);
                T::try_from(i128_value).ok()
            }
        } else {
            T::try_from(u128_value).ok()
        }
    }

    /// Convert to a BigInt (signed arbitrary precision integer)
    pub fn to_bigint(&self) -> BigInt {
        if self.is_negative {
            BigInt::new(Sign::Minus, self.integer.to_u32_digits())
        } else {
            BigInt::from(self.integer.clone())
        }
    }

    /// Convert to a FieldElement. Note: this may lose precision for values
    /// larger than the field modulus.
    pub fn to_field_element(&self) -> FieldElement {
        let fe = FieldElement::from_be_bytes_reduce(&self.integer.to_bytes_be());
        if self.is_negative { -fe } else { fe }
    }

    /// Create from a FieldElement (always positive)
    pub fn from_field_element(field_element: FieldElement) -> Self {
        Self::new(BigUint::from_bytes_be(&field_element.to_be_bytes()), false)
    }

    /// Get the underlying BigUint
    pub fn to_biguint(&self) -> BigUint {
        self.integer.clone()
    }

    pub fn to_u128(self) -> u128 {
        assert!(self.is_positive());
        self.integer.to_u128().expect("Value too large for u128")
    }

    pub fn to_i128(self) -> i128 {
        if self.is_negative() {
            let value = self.integer.to_u128().expect("Value too large for u128");
            if value == ((i128::MAX as u128) + 1) { i128::MIN } else { -(value as i128) }
        } else {
            self.integer.to_u128().expect("Value too large for u128") as i128
        }
    }

    pub fn from_bigint(value: &BigInt) -> Self {
        let is_negative = value.sign() == Sign::Minus;
        SignedField::new(value.magnitude().clone(), is_negative)
    }
}

impl std::ops::Add for SignedField {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        if self.is_negative == rhs.is_negative {
            Self::new(self.integer + rhs.integer, self.is_negative)
        } else if self.is_negative && !rhs.is_negative {
            if self.integer > rhs.integer {
                // For example "-4 + 3", so "-(4 - 3)"
                Self::new(self.integer - rhs.integer, true)
            } else {
                // For example "-4 + 5", so "5 - 4"
                Self::new(rhs.integer - self.integer, false)
            }
        } else if rhs.integer > self.integer {
            // For example "4 + (-5)", so "-(5 - 4)"
            Self::new(rhs.integer - self.integer, true)
        } else {
            // For example "4 + (-3)", so "4 - 3"
            Self::new(self.integer - rhs.integer, false)
        }
    }
}

impl std::ops::Sub for SignedField {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self + -rhs
    }
}

impl std::ops::Mul for SignedField {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        if self.integer.is_zero() || rhs.integer.is_zero() {
            return Self::zero();
        }

        Self::new(&self.integer * &rhs.integer, self.is_negative ^ rhs.is_negative)
    }
}

impl std::ops::Div for SignedField {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        if self.integer.is_zero() {
            return Self::zero();
        }

        Self::new(&self.integer / &rhs.integer, self.is_negative ^ rhs.is_negative)
    }
}

impl std::ops::Neg for SignedField {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(self.integer, !self.is_negative)
    }
}

impl Ord for SignedField {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.is_negative != other.is_negative {
            if self.is_negative { std::cmp::Ordering::Less } else { std::cmp::Ordering::Greater }
        } else if self.is_negative {
            // Negative comparisons should be reversed so that -2 < -1
            other.integer.cmp(&self.integer)
        } else {
            self.integer.cmp(&other.integer)
        }
    }
}

impl PartialOrd for SignedField {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl From<bool> for SignedField {
    fn from(value: bool) -> Self {
        if value { Self::one() } else { Self::zero() }
    }
}

impl From<u8> for SignedField {
    fn from(value: u8) -> Self {
        Self::new(value.into(), false)
    }
}

impl From<u16> for SignedField {
    fn from(value: u16) -> Self {
        Self::new(value.into(), false)
    }
}

impl From<u32> for SignedField {
    fn from(value: u32) -> Self {
        Self::new(value.into(), false)
    }
}

impl From<u64> for SignedField {
    fn from(value: u64) -> Self {
        Self::new(value.into(), false)
    }
}

impl From<u128> for SignedField {
    fn from(value: u128) -> Self {
        Self::new(value.into(), false)
    }
}

impl From<i8> for SignedField {
    fn from(value: i8) -> Self {
        i128::from(value).into()
    }
}

impl From<i16> for SignedField {
    fn from(value: i16) -> Self {
        i128::from(value).into()
    }
}

impl From<i32> for SignedField {
    fn from(value: i32) -> Self {
        i128::from(value).into()
    }
}

impl From<i64> for SignedField {
    fn from(value: i64) -> Self {
        i128::from(value).into()
    }
}

impl From<i128> for SignedField {
    fn from(value: i128) -> Self {
        if value == i128::MIN {
            Self::new(BigUint::from((i128::MAX as u128) + 1), true)
        } else if value < 0 {
            Self::new(BigUint::from((-value) as u128), true)
        } else {
            Self::new(BigUint::from(value as u128), false)
        }
    }
}

impl From<usize> for SignedField {
    fn from(value: usize) -> Self {
        Self::new((value as u128).into(), false)
    }
}

impl From<FieldElement> for SignedField {
    fn from(value: FieldElement) -> Self {
        Self::from_field_element(value)
    }
}

impl From<BigUint> for SignedField {
    fn from(value: BigUint) -> Self {
        Self::new(value, false)
    }
}

impl From<SignedField> for BigInt {
    fn from(value: SignedField) -> Self {
        value.to_bigint()
    }
}

impl<T> From<&T> for SignedField
where
    T: Clone,
    SignedField: From<T>,
{
    fn from(value: &T) -> Self {
        value.clone().into()
    }
}

impl std::fmt::Display for SignedField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_negative {
            write!(f, "-")?;
        }
        write!(f, "{}", self.integer)
    }
}

impl rangemap::StepLite for SignedField {
    fn add_one(&self) -> Self {
        if self.is_negative {
            if self.integer.is_one() {
                Self::new(BigUint::zero(), false)
            } else {
                Self::new(&self.integer - BigUint::one(), self.is_negative)
            }
        } else {
            Self::new(&self.integer + BigUint::one(), self.is_negative)
        }
    }

    fn sub_one(&self) -> Self {
        if self.is_negative {
            Self::new(&self.integer + BigUint::one(), self.is_negative)
        } else if self.integer.is_zero() {
            Self::new(BigUint::one(), true)
        } else {
            Self::new(&self.integer - BigUint::one(), self.is_negative)
        }
    }
}

// Implement serde manually since BigUint doesn't derive Copy
impl serde::Serialize for SignedField {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Serialize as a tuple of (bytes, is_negative)
        let bytes = self.integer.to_bytes_be();
        (bytes, self.is_negative).serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for SignedField {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let (bytes, is_negative): (Vec<u8>, bool) = serde::Deserialize::deserialize(deserializer)?;
        Ok(Self::new(BigUint::from_bytes_be(&bytes), is_negative))
    }
}

pub trait AbsU128 {
    /// Necessary to handle casting to unsigned generically without overflowing on INT_MIN.
    fn abs_u128(self) -> u128;
}

macro_rules! impl_unsigned_abs_for {
    ($typ:ty) => {
        impl AbsU128 for $typ {
            fn abs_u128(self) -> u128 {
                self.unsigned_abs().into()
            }
        }
    };
}

impl_unsigned_abs_for!(i8);
impl_unsigned_abs_for!(i16);
impl_unsigned_abs_for!(i32);
impl_unsigned_abs_for!(i64);
impl_unsigned_abs_for!(i128);

#[cfg(test)]
mod tests {
    use super::SignedField;

    #[test]
    fn int_min_to_signed_field_roundtrip() {
        let x = i128::MIN;
        let field = SignedField::from_signed(x);
        assert_eq!(field.try_to_signed(), Some(x));
    }

    #[test]
    fn comparisons() {
        let neg_two = SignedField::negative(2u32);
        let neg_one = SignedField::negative(1u32);
        let zero = SignedField::positive(0u32);
        let one = SignedField::positive(1u32);
        let two = SignedField::positive(2u32);

        assert!(one < two);
        assert!(zero < one);
        assert!(neg_one < zero);
        assert!(neg_two < neg_one);

        assert!(two > neg_two);
    }

    #[test]
    fn addition() {
        let zero = SignedField::zero();
        let one = SignedField::one();
        let two = SignedField::positive(2_u32);
        let three = SignedField::positive(3_u32);
        assert_eq!(one.clone() + two.clone(), three.clone()); // positive + positive

        let minus_one = SignedField::negative(1_u32);
        let minus_two = SignedField::negative(2_u32);
        let minus_three = SignedField::negative(3_u32);
        assert_eq!(two.clone() + minus_one.clone(), one.clone()); // positive + negative

        assert_eq!(minus_three.clone() + one.clone(), minus_two.clone()); // negative + positive

        assert_eq!(minus_one.clone() + minus_two.clone(), minus_three.clone()); // negative + negative

        assert_eq!(one.clone() + zero.clone(), one.clone());
        assert_eq!(zero.clone() + one.clone(), one.clone());
        assert_eq!(minus_one.clone() + zero.clone(), minus_one.clone());
        assert_eq!(zero.clone() + minus_one.clone(), minus_one.clone());
    }

    #[test]
    fn subtraction() {
        let zero = SignedField::zero();
        let one = SignedField::one();
        let two = SignedField::positive(2_u32);
        let three = SignedField::positive(3_u32);
        assert_eq!(three.clone() - two.clone(), one.clone()); // positive - positive

        let minus_one = SignedField::negative(1_u32);
        let minus_three = SignedField::negative(3_u32);
        assert_eq!(two.clone() - minus_one.clone(), three.clone()); // positive - negative

        assert_eq!(minus_one.clone() - two.clone(), minus_three.clone()); // negative - positive

        assert_eq!(minus_one.clone() - minus_three.clone(), two.clone()); // negative - negative

        assert_eq!(one.clone() - zero.clone(), one.clone());
        assert_eq!(minus_one.clone() - zero.clone(), minus_one.clone());
        assert_eq!(zero.clone() - one.clone(), minus_one.clone());
        assert_eq!(zero.clone() - minus_one.clone(), one.clone());
    }

    #[test]
    fn multiplication() {
        let zero = SignedField::zero();
        let two = SignedField::positive(2_u32);
        let three = SignedField::positive(3_u32);
        let six = SignedField::positive(6_u32);
        let minus_two = SignedField::negative(2_u32);
        let minus_three = SignedField::negative(3_u32);
        let minus_six = SignedField::negative(6_u32);

        assert_eq!(two.clone() * three.clone(), six.clone()); // positive * positive
        assert_eq!(two.clone() * minus_three.clone(), minus_six.clone()); // positive * negative
        assert_eq!(minus_two.clone() * three.clone(), minus_six.clone()); // negative * positive
        assert_eq!(minus_two.clone() * minus_three.clone(), six.clone()); // negative * negative
        assert_eq!(two.clone() * zero.clone(), zero.clone());
        assert_eq!(minus_two.clone() * zero.clone(), zero.clone());
        assert_eq!(zero.clone() * two.clone(), zero.clone());
        assert_eq!(zero.clone() * minus_two.clone(), zero.clone());
    }

    #[test]
    fn division() {
        let zero = SignedField::zero();
        let two = SignedField::positive(2_u32);
        let three = SignedField::positive(3_u32);
        let six = SignedField::positive(6_u32);
        let minus_two = SignedField::negative(2_u32);
        let minus_three = SignedField::negative(3_u32);
        let minus_six = SignedField::negative(6_u32);

        assert_eq!(six.clone() / two.clone(), three.clone()); // positive / positive
        assert_eq!(six.clone() / minus_three.clone(), minus_two.clone()); // positive / negative
        assert_eq!(minus_six.clone() / three.clone(), minus_two.clone()); // negative / positive
        assert_eq!(minus_six.clone() / minus_three.clone(), two.clone()); // negative / negative
        assert_eq!(zero.clone() / two.clone(), zero.clone());
        assert_eq!(zero.clone() / minus_two.clone(), zero.clone());
    }

    #[test]
    fn is_zero() {
        assert!(SignedField::zero().is_zero());
        assert!(SignedField::negative(0u32).is_zero());
        assert!(!SignedField::one().is_zero());
    }

    #[test]
    fn is_one() {
        assert!(SignedField::one().is_one());
        assert!(!SignedField::negative(1u32).is_one());
        assert!(!SignedField::zero().is_one());
    }

    #[test]
    fn to_i128() {
        assert_eq!(SignedField::positive(i128::MAX as u128).to_i128(), i128::MAX);
        assert_eq!(SignedField::negative((i128::MAX as u128) + 1).to_i128(), i128::MIN);
    }

    #[test]
    fn from_i128() {
        assert_eq!(SignedField::from(i128::MAX).to_i128(), i128::MAX);
        assert_eq!(SignedField::from(i128::MIN).to_i128(), i128::MIN);
        assert_eq!(SignedField::from(i128::MIN + 1).to_i128(), i128::MIN + 1);
    }

    #[test]
    fn equality() {
        let a = SignedField::negative(1u32);
        let b = SignedField::negative(1u32);
        assert_eq!(a.clone(), a.clone());
        assert_eq!(b.clone(), b.clone());
        assert_eq!(a.clone(), b.clone());
        assert_eq!(b.clone(), a.clone());
    }
}
