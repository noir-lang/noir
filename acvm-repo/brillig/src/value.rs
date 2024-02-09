use acir_field::FieldElement;
use serde::{Deserialize, Serialize};
use std::ops::{Add, Div, Mul, Neg, Sub};

/// Types of values allowed in the VM
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub enum Typ {
    Field,
    Unsigned { bit_size: u32 },
    Signed { bit_size: u32 },
}

/// `Value` represents the base descriptor for a value in the VM.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Value {
    inner: FieldElement,
}

impl Value {
    /// Returns `true` if the `Value` represents `zero`
    pub fn is_zero(&self) -> bool {
        self.inner.is_zero()
    }

    /// Converts `Value` into a `FieldElement`.
    pub fn to_field(&self) -> FieldElement {
        self.inner
    }

    /// Converts `Value` into a `u128`.
    // TODO: Check what happens if `Value` cannot fit into a u128
    pub fn to_u128(&self) -> u128 {
        self.to_field().to_u128()
    }

    /// Converts `Value` into a u64 and then casts it into a usize.
    /// Panics: If `Value` cannot fit into a u64 or `Value` does
    //// not fit into a usize.
    pub fn to_usize(&self) -> usize {
        usize::try_from(self.inner.try_to_u64().expect("value does not fit into u64"))
            .expect("value does not fit into usize")
    }
}

impl From<usize> for Value {
    fn from(value: usize) -> Self {
        Value { inner: FieldElement::from(value as u128) }
    }
}

impl From<u128> for Value {
    fn from(value: u128) -> Self {
        Value { inner: FieldElement::from(value) }
    }
}

impl From<FieldElement> for Value {
    fn from(value: FieldElement) -> Self {
        Value { inner: value }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value { inner: FieldElement::from(value) }
    }
}

impl Add for Value {
    type Output = Value;

    fn add(self, rhs: Self) -> Self::Output {
        Value { inner: self.inner + rhs.inner }
    }
}
impl Sub for Value {
    type Output = Value;

    fn sub(self, rhs: Self) -> Self::Output {
        Value { inner: self.inner - rhs.inner }
    }
}
impl Mul for Value {
    type Output = Value;

    fn mul(self, rhs: Self) -> Self::Output {
        Value { inner: self.inner * rhs.inner }
    }
}
impl Div for Value {
    type Output = Value;

    fn div(self, rhs: Self) -> Self::Output {
        Value { inner: self.inner / rhs.inner }
    }
}
impl Neg for Value {
    type Output = Value;

    fn neg(self) -> Self::Output {
        Value { inner: -self.inner }
    }
}
