use acvm::{AcirField, FieldElement};
use num_bigint::{BigInt, Sign};
use num_traits::{One, Signed, ToPrimitive, Zero};

/// A signed, arbitrary-precision integer carrying numeric literals and
/// compile-time constants through the frontend. The field is only consulted at
/// [`Self::to_field_element`], [`Self::absolute_value`] and
/// [`Self::from_field_element`].
#[derive(
    Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, serde::Deserialize, serde::Serialize,
)]
pub struct SignedField(BigInt);

/// Interpret a field element as a non-negative integer in `[0, p)`.
fn field_to_bigint(field: FieldElement) -> BigInt {
    BigInt::from_bytes_be(Sign::Plus, &field.to_be_bytes())
}

/// Reduce a signed integer to its field element, negating if negative.
fn bigint_to_field(value: &BigInt) -> FieldElement {
    let (sign, magnitude) = value.to_bytes_be();
    let field = FieldElement::from_be_bytes_reduce(&magnitude);
    if sign == Sign::Minus { -field } else { field }
}

impl SignedField {
    /// Construct from a field-element magnitude and a sign.
    pub fn new(field: FieldElement, is_negative: bool) -> Self {
        let magnitude = field_to_bigint(field);
        Self(if is_negative { -magnitude } else { magnitude })
    }

    pub fn positive(value: impl Into<BigInt>) -> Self {
        Self(value.into())
    }

    pub fn negative(value: impl Into<BigInt>) -> Self {
        Self(-value.into().abs())
    }

    /// Construct directly from a field element (a genuine field constant),
    /// interpreting it as a non-negative integer in `[0, p)`.
    pub fn from_field_element(field: FieldElement) -> Self {
        Self(field_to_bigint(field))
    }

    pub fn zero() -> SignedField {
        Self(BigInt::zero())
    }

    pub fn one() -> SignedField {
        Self(BigInt::one())
    }

    /// The magnitude as a field element (reduced mod p if it exceeds the modulus).
    pub fn absolute_value(&self) -> FieldElement {
        FieldElement::from_be_bytes_reduce(&self.0.magnitude().to_bytes_be())
    }

    pub fn is_negative(&self) -> bool {
        self.0.sign() == Sign::Minus
    }

    pub fn is_positive(&self) -> bool {
        !self.is_negative()
    }

    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    pub fn is_one(&self) -> bool {
        self.0.is_one()
    }

    /// Convert a signed integer to a SignedField, carefully handling
    /// INT_MIN in the process. Note that to convert an unsigned integer
    /// you can call `SignedField::positive`.
    #[inline]
    pub fn from_signed<T>(value: T) -> Self
    where
        T: Signed + AbsU128,
    {
        let negative = value.is_negative();
        let magnitude = BigInt::from(value.abs_u128());
        Self(if negative { -magnitude } else { magnitude })
    }

    /// Convert a SignedField into an unsigned integer type (up to u128),
    /// returning None if the value does not fit (e.g. if it is negative).
    #[inline]
    pub fn try_to_unsigned<T: TryFrom<u128>>(&self) -> Option<T> {
        if self.is_negative() {
            return None;
        }

        assert!(size_of::<T>() <= size_of::<u128>());
        let u128_value = self.0.to_u128()?;
        u128_value.try_into().ok()
    }

    /// Convert a SignedField into a signed integer type (up to i128),
    /// returning None if the value does not fit.
    #[inline]
    pub fn try_to_signed<T>(&self) -> Option<T>
    where
        T: TryFrom<i128>,
    {
        let i128_value = self.0.to_i128()?;
        T::try_from(i128_value).ok()
    }

    pub fn to_field_element(&self) -> FieldElement {
        bigint_to_field(&self.0)
    }

    pub fn to_u128(&self) -> u128 {
        assert!(self.is_positive());
        self.0.magnitude().to_u128().expect("value does not fit in u128")
    }

    pub fn to_i128(&self) -> i128 {
        if self.is_negative() {
            let value = self.0.magnitude().to_u128().expect("value does not fit in i128");
            if value == ((i128::MAX as u128) + 1) { i128::MIN } else { -(value as i128) }
        } else {
            self.0.magnitude().to_u128().expect("value does not fit in i128") as i128
        }
    }
}

impl std::ops::Add for SignedField {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl std::ops::Sub for SignedField {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl std::ops::Mul for SignedField {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl std::ops::Div for SignedField {
    type Output = Self;

    /// Division is a field operation (multiplication by the modular inverse), so
    /// it is only meaningful for `Field`-typed values. The magnitudes are divided
    /// in the field and the signs are combined.
    fn div(self, rhs: Self) -> Self::Output {
        if self.0.is_zero() {
            return Self::zero();
        }

        let quotient = self.absolute_value() / rhs.absolute_value();
        Self::new(quotient, self.is_negative() ^ rhs.is_negative())
    }
}

impl std::ops::Neg for SignedField {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl From<bool> for SignedField {
    fn from(value: bool) -> Self {
        if value { Self::one() } else { Self::zero() }
    }
}

macro_rules! impl_from_integer_for_signed_field {
    ($typ:ty) => {
        impl From<$typ> for SignedField {
            fn from(value: $typ) -> Self {
                Self(BigInt::from(value))
            }
        }
    };
}

impl_from_integer_for_signed_field!(u8);
impl_from_integer_for_signed_field!(u16);
impl_from_integer_for_signed_field!(u32);
impl_from_integer_for_signed_field!(u64);
impl_from_integer_for_signed_field!(u128);
impl_from_integer_for_signed_field!(i8);
impl_from_integer_for_signed_field!(i16);
impl_from_integer_for_signed_field!(i32);
impl_from_integer_for_signed_field!(i64);
impl_from_integer_for_signed_field!(i128);
impl_from_integer_for_signed_field!(usize);

impl From<FieldElement> for SignedField {
    fn from(value: FieldElement) -> Self {
        Self::from_field_element(value)
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
        write!(f, "{}", self.0)
    }
}

impl rangemap::StepLite for SignedField {
    fn add_one(&self) -> Self {
        Self(&self.0 + 1)
    }

    fn sub_one(&self) -> Self {
        Self(&self.0 - 1)
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
        assert_eq!(one.clone() + two.clone(), three.clone()); // positive + positive

        let minus_one = SignedField::negative(1_u32);
        let minus_two = SignedField::negative(2_u32);
        let minus_three = SignedField::negative(3_u32);
        assert_eq!(two + minus_one.clone(), one.clone()); // positive + negative

        assert_eq!(minus_three.clone() + one.clone(), minus_two); // negative + positive

        assert_eq!(minus_one.clone() + minus_two, minus_three); // negative + negative

        assert_eq!(one.clone() + zero.clone(), one.clone());
        assert_eq!(zero.clone() + one.clone(), one.clone());
        assert_eq!(minus_one.clone() + zero.clone(), minus_one.clone());
        assert_eq!(zero + minus_one.clone(), minus_one);
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
        assert_eq!(two.clone() - minus_one.clone(), three); // positive - negative

        assert_eq!(minus_one.clone() - two.clone(), minus_three.clone()); // negative - positive

        assert_eq!(minus_one.clone() - minus_three, two); // negative - negative

        assert_eq!(one.clone() - zero.clone(), one.clone());
        assert_eq!(minus_one.clone() - zero.clone(), minus_one.clone());
        assert_eq!(zero.clone() - one.clone(), minus_one.clone());
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

        assert_eq!(two.clone() * three.clone(), six.clone()); // positive * positive
        assert_eq!(two.clone() * minus_three.clone(), minus_six.clone()); // positive * negative
        assert_eq!(minus_two.clone() * three, minus_six); // negative * positive
        assert_eq!(minus_two.clone() * minus_three, six); // negative * negative
        assert_eq!(two * zero.clone(), zero.clone());
        assert_eq!(minus_two * zero.clone(), zero.clone());
        assert_eq!(zero.clone() * SignedField::positive(2_u32), zero.clone());
        assert_eq!(zero.clone() * SignedField::negative(2_u32), zero);
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
        assert_eq!(six / minus_three.clone(), minus_two.clone()); // positive / negative
        assert_eq!(minus_six.clone() / three, minus_two); // negative / positive
        assert_eq!(minus_six / minus_three, two.clone()); // negative / negative
        assert_eq!(zero.clone() / two, zero.clone());
        assert_eq!(zero / SignedField::negative(2_u32), SignedField::zero());
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
        assert_eq!(SignedField::from(i128::MAX).to_i128(), i128::MAX);
        assert_eq!(SignedField::from(i128::MIN).to_i128(), i128::MIN);
    }

    #[test]
    fn from_i128() {
        assert_eq!(SignedField::from(i128::MAX).to_i128(), i128::MAX);
        assert_eq!(SignedField::from(i128::MIN).to_i128(), i128::MIN);
        assert_eq!(SignedField::from(i128::MIN + 1).to_i128(), i128::MIN + 1);
    }

    // negative(1) and from_field_element(-1) (p-1) share a field element but are
    // distinct integers.
    #[test]
    fn de_conflated_field_equality() {
        let neg_one = SignedField::negative(1u32);
        let field_neg_one = SignedField::from_field_element(-FieldElement::one());

        assert_ne!(neg_one, field_neg_one);
        assert_eq!(neg_one.to_field_element(), field_neg_one.to_field_element());
    }
}
