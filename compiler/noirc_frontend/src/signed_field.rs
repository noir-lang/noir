use acvm::{AcirField, FieldElement};

#[derive(Debug, Copy, Clone, Hash, serde::Deserialize, serde::Serialize)]
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

    pub fn positive(field: impl Into<FieldElement>) -> Self {
        Self { field: field.into(), is_negative: false }
    }

    pub fn negative(field: impl Into<FieldElement>) -> Self {
        Self::new(field.into(), true)
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

    pub fn is_negative(&self) -> bool {
        self.is_negative
    }

    pub fn is_zero(&self) -> bool {
        self.field.is_zero()
    }

    pub fn is_one(&self) -> bool {
        !self.is_negative && self.field.is_one()
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

    /// Convert a SignedField into an unsigned integer type (up to u128),
    /// returning None if the value does not fit (e.g. if it is negative).
    #[inline]
    pub fn try_to_unsigned<T: TryFrom<u128>>(self) -> Option<T> {
        if self.is_negative {
            return None;
        }

        assert!(size_of::<T>() <= size_of::<u128>());
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

    pub fn to_u128(self) -> u128 {
        assert!(!self.is_negative());
        self.to_field_element().to_u128()
    }

    pub fn to_i128(self) -> i128 {
        if self.is_negative() {
            let value = self.field.to_u128();
            if value == ((i128::MAX as u128) + 1) { i128::MIN } else { -(value as i128) }
        } else {
            self.field.to_u128() as i128
        }
    }
}

impl PartialEq for SignedField {
    fn eq(&self, other: &Self) -> bool {
        if self.is_negative == other.is_negative {
            self.field == other.field
        } else {
            self.field == -other.field
        }
    }
}

impl Eq for SignedField {}

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

impl From<u32> for SignedField {
    fn from(value: u32) -> Self {
        Self::new(value.into(), false)
    }
}

impl From<u128> for SignedField {
    fn from(value: u128) -> Self {
        Self::new(value.into(), false)
    }
}

impl From<i128> for SignedField {
    fn from(value: i128) -> Self {
        if value == i128::MIN {
            Self::new(FieldElement::from((i128::MAX as u128) + 1), true)
        } else if value < 0 {
            Self::new((-value).into(), true)
        } else {
            Self::new(value.into(), false)
        }
    }
}

impl From<usize> for SignedField {
    fn from(value: usize) -> Self {
        Self::new(value.into(), false)
    }
}

impl From<FieldElement> for SignedField {
    fn from(value: FieldElement) -> Self {
        Self::new(value, false)
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
    use acvm::{AcirField, FieldElement};

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

    #[test]
    fn is_zero() {
        assert!(SignedField::zero().is_zero());
        assert!(SignedField::negative(FieldElement::zero()).is_zero());
        assert!(!SignedField::one().is_zero());
    }

    #[test]
    fn is_one() {
        assert!(SignedField::one().is_one());
        assert!(!SignedField::negative(FieldElement::one()).is_one());
        assert!(!SignedField::zero().is_one());
    }

    #[test]
    fn to_i128() {
        assert_eq!(SignedField::positive(FieldElement::from(i128::MAX)).to_i128(), i128::MAX);
        assert_eq!(
            SignedField::negative(FieldElement::from((i128::MAX as u128) + 1)).to_i128(),
            i128::MIN
        );
    }

    #[test]
    fn from_i128() {
        assert_eq!(SignedField::from(i128::MAX).to_i128(), i128::MAX);
        assert_eq!(SignedField::from(i128::MIN).to_i128(), i128::MIN);
        assert_eq!(SignedField::from(i128::MIN + 1).to_i128(), i128::MIN + 1);
    }

    #[test]
    fn equality() {
        let a = SignedField::negative(FieldElement::one());
        let b = SignedField::positive(-FieldElement::one());
        assert_eq!(a, a);
        assert_eq!(b, b);
        assert_eq!(a, b);
    }
}
