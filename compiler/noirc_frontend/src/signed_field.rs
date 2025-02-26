use acvm::{AcirField, FieldElement};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SignedField {
    pub field: FieldElement,
    pub is_negative: bool,
}

impl SignedField {
    pub fn new(field: FieldElement, is_negative: bool) -> Self {
        Self { field, is_negative }
    }

    pub fn positive(field: impl Into<FieldElement>) -> Self {
        Self { field: field.into(), is_negative: false }
    }

    pub fn negative(field: impl Into<FieldElement>) -> Self {
        Self { field: field.into(), is_negative: true }
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
}

impl std::ops::Neg for SignedField {
    type Output = Self;

    fn neg(mut self) -> Self::Output {
        self.is_negative = !self.is_negative;
        self
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
}
