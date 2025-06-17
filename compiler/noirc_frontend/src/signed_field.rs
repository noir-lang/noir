use crate::field_element::{FieldElement, FieldElementExt, field_helpers};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SignedField {
    field: FieldElement,
    is_negative: bool,
}

impl SignedField {
    pub fn new(field: FieldElement, mut is_negative: bool) -> Self {
        if field.is_zero() {
            is_negative = false;
        }
        Self { field, is_negative }
    }

    pub fn positive(field: FieldElement) -> Self {
        Self { field, is_negative: false }
    }

    pub fn negative(field: FieldElement) -> Self {
        Self::new(field, true)
    }

    pub fn zero() -> SignedField {
        Self { field: FieldElement::zero(), is_negative: false }
    }

    pub fn one() -> SignedField {
        Self { field: FieldElement::one(), is_negative: false }
    }

    /// Returns the inner FieldElement which will always be positive
    pub fn absolute_value(&self) -> FieldElement {
        self.field
    }
    
    /// Alias for absolute_value to match new I256 interface
    pub fn abs(&self) -> FieldElement {
        self.field
    }

    pub fn is_negative(&self) -> bool {
        self.is_negative
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
        SignedField::new(field_helpers::field_from_u128(value), negative)
    }

    /// Convert a SignedField into an unsigned integer type (up to u128),
    /// returning None if the value does not fit (e.g. if it is negative).
    #[inline]
    pub fn try_to_unsigned<T: TryFrom<u128>>(self) -> Option<T> {
        if self.is_negative {
            return None;
        }

        assert!(std::mem::size_of::<T>() <= std::mem::size_of::<u128>());
        let u128_value = self.field.try_into_u128()?;
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
        let u128_value = self.field.try_into_u128()?;

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

    pub fn to_field_element(self) -> FieldElement {
        if self.is_negative { -self.field } else { self.field }
    }
}

impl std::ops::Add for SignedField {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        if self.is_negative == rhs.is_negative {
            Self::new(self.field + rhs.field, self.is_negative)
        } else if self.is_negative && !rhs.is_negative {
            if self.field > rhs.field {
                // For example "-4 + 3", so "-(4 - 3)"
                Self::new(self.field - rhs.field, true)
            } else {
                // For example "-4 + 5", so "5 - 4"
                Self::new(rhs.field - self.field, false)
            }
        } else if rhs.field > self.field {
            // For example "4 + (-5)", so "-(5 - 4)"
            Self::new(rhs.field - self.field, true)
        } else {
            // For example "4 + (-3)", so "4 - 3"
            Self::new(self.field - rhs.field, false)
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
        if self.field.is_zero() || rhs.field.is_zero() {
            return Self::zero();
        }

        Self::new(self.field * rhs.field, self.is_negative ^ rhs.is_negative)
    }
}

impl std::ops::Div for SignedField {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        if self.field.is_zero() {
            return Self::zero();
        }

        Self::new(self.field / rhs.field, self.is_negative ^ rhs.is_negative)
    }
}

impl std::ops::Neg for SignedField {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(self.field, !self.is_negative)
    }
}

impl Ord for SignedField {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.is_negative != other.is_negative {
            if self.is_negative { std::cmp::Ordering::Less } else { std::cmp::Ordering::Greater }
        } else if self.is_negative {
            // Negative comparisons should be reversed so that -2 < -1
            other.field.cmp(&self.field)
        } else {
            self.field.cmp(&other.field)
        }
    }
}

impl PartialOrd for SignedField {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl From<FieldElement> for SignedField {
    fn from(value: FieldElement) -> Self {
        Self::new(value, false)
    }
}

impl From<SignedField> for FieldElement {
    fn from(value: SignedField) -> Self {
        if value.is_negative { -value.field } else { value.field }
    }
}

impl std::fmt::Display for SignedField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_negative {
            write!(f, "-")?;
        }
        write!(f, "{}", self.field)
    }
}

impl rangemap::StepLite for SignedField {
    fn add_one(&self) -> Self {
        if self.is_negative {
            if self.field.is_one() {
                Self::new(FieldElement::zero(), false)
            } else {
                Self::new(self.field - FieldElement::one(), self.is_negative)
            }
        } else {
            Self::new(self.field + FieldElement::one(), self.is_negative)
        }
    }

    fn sub_one(&self) -> Self {
        if self.is_negative {
            Self::new(self.field + FieldElement::one(), self.is_negative)
        } else if self.field.is_zero() {
            Self::new(FieldElement::one(), true)
        } else {
            Self::new(self.field - FieldElement::one(), self.is_negative)
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
        assert_eq!(one + two, three); // positive + positive

        let minus_one = SignedField::negative(1_u32);
        let minus_two = SignedField::negative(2_u32);
        let minus_three = SignedField::negative(3_u32);
        assert_eq!(two + minus_one, one); // positive + negative

        assert_eq!(minus_three + one, minus_two); // negative + positive

        assert_eq!(minus_one + minus_two, minus_three); // negative + negative

        assert_eq!(one + zero, one);
        assert_eq!(zero + one, one);
        assert_eq!(minus_one + zero, minus_one);
        assert_eq!(zero + minus_one, minus_one);
    }

    #[test]
    fn subtraction() {
        let zero = SignedField::zero();
        let one = SignedField::one();
        let two = SignedField::positive(2_u32);
        let three = SignedField::positive(3_u32);
        assert_eq!(three - two, one); // positive - positive

        let minus_one = SignedField::negative(1_u32);
        let minus_three = SignedField::negative(3_u32);
        assert_eq!(two - minus_one, three); // positive - negative

        assert_eq!(minus_one - two, minus_three); // negative - positive

        assert_eq!(minus_one - minus_three, two); // negative - negative

        assert_eq!(one - zero, one);
        assert_eq!(minus_one - zero, minus_one);
        assert_eq!(zero - one, minus_one);
        assert_eq!(zero - minus_one, one);
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

        assert_eq!(two * three, six); // positive * positive
        assert_eq!(two * minus_three, minus_six); // positive * negative
        assert_eq!(minus_two * three, minus_six); // negative * positive
        assert_eq!(minus_two * minus_three, six); // negative * negative
        assert_eq!(two * zero, zero);
        assert_eq!(minus_two * zero, zero);
        assert_eq!(zero * two, zero);
        assert_eq!(zero * minus_two, zero);
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

        assert_eq!(six / two, three); // positive / positive
        assert_eq!(six / minus_three, minus_two); // positive / negative
        assert_eq!(minus_six / three, minus_two); // negative / positive
        assert_eq!(minus_six / minus_three, two); // negative / negative
        assert_eq!(zero / two, zero);
        assert_eq!(zero / minus_two, zero);
    }
}
