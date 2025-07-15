use acvm::{AcirField, FieldElement};
use num_bigint::{BigInt, BigUint, Sign};
use num_traits::{FromBytes, One, ToPrimitive, Zero};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SignedInteger {
    integer: BigUint,
    is_negative: bool,
}

impl SignedInteger {
    pub fn new(integer: BigUint, mut is_negative: bool) -> Self {
        if integer.is_zero() {
            is_negative = false;
        }
        Self { integer, is_negative }
    }

    pub fn positive(integer: impl Into<BigUint>) -> Self {
        Self { integer: integer.into(), is_negative: false }
    }

    pub fn negative(integer: impl Into<BigInt>) -> Self {
        Self::new(integer.into().magnitude().clone(), true)
    }

    pub fn zero() -> SignedInteger {
        Self { integer: BigUint::zero(), is_negative: false }
    }

    pub fn one() -> SignedInteger {
        Self { integer: BigUint::one(), is_negative: false }
    }

    /// Returns the inner FieldElement which will always be positive
    pub fn absolute_value(&self) -> BigUint {
        self.integer.clone()
    }

    pub fn is_negative(&self) -> bool {
        self.is_negative
    }

    /// Convert a signed integer to a SignedInteger, carefully handling
    /// INT_MIN in the process. Note that to convert an unsigned integer
    /// you can call `SignedInteger::positive`.
    #[inline]
    pub fn from_signed<T>(value: T) -> Self
    where
        T: num_traits::Signed + AbsU128,
    {
        let negative = value.is_negative();
        let value = value.abs_u128();
        SignedInteger::new(value.into(), negative)
    }

    fn integer_to_u128(&self) -> Option<u128> {
        self.integer.to_u128()
    }

    /// Convert a SignedInteger into an unsigned integer type (up to u128),
    /// returning None if the value does not fit (e.g. if it is negative).
    #[inline]
    pub fn try_to_unsigned<T: TryFrom<u128>>(self) -> Option<T> {
        if self.is_negative {
            return None;
        }

        assert!(std::mem::size_of::<T>() <= std::mem::size_of::<u128>());
        let u128_value = self.integer_to_u128()?;
        u128_value.try_into().ok()
    }

    /// Convert a SignedInteger into a signed integer type (up to i128),
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

    pub fn to_integer(self) -> BigInt {
        if self.is_negative {
            BigInt::new(Sign::Minus, self.integer.to_u32_digits())
        } else {
            BigInt::from(self.integer)
        }
    }

    pub fn to_field_element(self) -> FieldElement {
        FieldElement::from_be_bytes_reduce(&self.integer.to_bytes_be())
    }

    pub fn from_field_element(field_element: FieldElement) -> Self {
        Self::new(BigUint::from_be_bytes(&field_element.to_be_bytes()), false)
    }
}

impl std::ops::Add for SignedInteger {
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

impl std::ops::Sub for SignedInteger {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self + -rhs
    }
}

impl std::ops::Mul for SignedInteger {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        if self.integer.is_zero() || rhs.integer.is_zero() {
            return Self::zero();
        }

        Self::new(self.integer * rhs.integer, self.is_negative ^ rhs.is_negative)
    }
}

impl std::ops::Div for SignedInteger {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        if self.integer.is_zero() {
            return Self::zero();
        }

        Self::new(self.integer / rhs.integer, self.is_negative ^ rhs.is_negative)
    }
}

impl std::ops::Neg for SignedInteger {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(self.integer, !self.is_negative)
    }
}

impl Ord for SignedInteger {
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

impl PartialOrd for SignedInteger {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl From<BigUint> for SignedInteger {
    fn from(value: BigUint) -> Self {
        Self::new(value, false)
    }
}

impl From<SignedInteger> for BigInt {
    fn from(value: SignedInteger) -> Self {
        if value.is_negative {
            BigInt::new(Sign::Minus, value.integer.to_u32_digits())
        } else {
            BigInt::from(value.integer)
        }
    }
}

impl std::fmt::Display for SignedInteger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_negative {
            write!(f, "-")?;
        }
        write!(f, "{}", self.integer)
    }
}

impl rangemap::StepLite for SignedInteger {
    fn add_one(&self) -> Self {
        if self.is_negative {
            if self.integer.is_one() {
                Self::new(BigUint::zero(), false)
            } else {
                Self::new(self.integer.clone() - BigUint::one(), self.is_negative)
            }
        } else {
            Self::new(self.integer.clone() + BigUint::one(), self.is_negative)
        }
    }

    fn sub_one(&self) -> Self {
        if self.is_negative {
            Self::new(self.integer.clone() + BigUint::one(), self.is_negative)
        } else if self.integer.is_zero() {
            Self::new(BigUint::one(), true)
        } else {
            Self::new(self.integer.clone() - BigUint::one(), self.is_negative)
        }
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
                self.unsigned_abs() as u128
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
    use super::SignedInteger;

    #[test]
    fn int_min_to_signed_field_roundtrip() {
        let x = i128::MIN;
        let field = SignedInteger::from_signed(x);
        assert_eq!(field.try_to_signed(), Some(x));
    }

    #[test]
    fn comparisons() {
        let neg_two = SignedInteger::negative(2u32);
        let neg_one = SignedInteger::negative(1u32);
        let zero = SignedInteger::positive(0u32);
        let one = SignedInteger::positive(1u32);
        let two = SignedInteger::positive(2u32);

        assert!(one < two);
        assert!(zero < one);
        assert!(neg_one < zero);
        assert!(neg_two < neg_one);

        assert!(two > neg_two);
    }

    #[test]
    fn addition() {
        let zero = SignedInteger::zero();
        let one = SignedInteger::one();
        let two = SignedInteger::positive(2_u32);
        let three = SignedInteger::positive(3_u32);
        assert_eq!(one.clone() + two.clone(), three.clone()); // positive + positive

        let minus_one = SignedInteger::negative(1_u32);
        let minus_two = SignedInteger::negative(2_u32);
        let minus_three = SignedInteger::negative(3_u32);
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
        let zero = SignedInteger::zero();
        let one = SignedInteger::one();
        let two = SignedInteger::positive(2_u32);
        let three = SignedInteger::positive(3_u32);
        assert_eq!(three.clone() - two.clone(), one.clone()); // positive - positive

        let minus_one = SignedInteger::negative(1_u32);
        let minus_three = SignedInteger::negative(3_u32);
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
        let zero = SignedInteger::zero();
        let two = SignedInteger::positive(2_u32);
        let three = SignedInteger::positive(3_u32);
        let six = SignedInteger::positive(6_u32);
        let minus_two = SignedInteger::negative(2_u32);
        let minus_three = SignedInteger::negative(3_u32);
        let minus_six = SignedInteger::negative(6_u32);

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
        let zero = SignedInteger::zero();
        let two = SignedInteger::positive(2_u32);
        let three = SignedInteger::positive(3_u32);
        let six = SignedInteger::positive(6_u32);
        let minus_two = SignedInteger::negative(2_u32);
        let minus_three = SignedInteger::negative(3_u32);
        let minus_six = SignedInteger::negative(6_u32);

        assert_eq!(six.clone() / two.clone(), three.clone()); // positive / positive
        assert_eq!(six.clone() / minus_three.clone(), minus_two.clone()); // positive / negative
        assert_eq!(minus_six.clone() / three.clone(), minus_two.clone()); // negative / positive
        assert_eq!(minus_six.clone() / minus_three.clone(), two.clone()); // negative / negative
        assert_eq!(zero.clone() / two.clone(), zero.clone());
        assert_eq!(zero.clone() / minus_two.clone(), zero.clone());
    }
}
