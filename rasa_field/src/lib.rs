// XXX: Switch out for a trait and proper implementations
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FieldElement(pub i128);

impl FieldElement {
    pub fn one() -> FieldElement {
        FieldElement(1)
    }
    pub fn zero() -> FieldElement {
        FieldElement(0)
    }
    pub fn is_one(&self) -> bool {
        self == &FieldElement::one()
    }
    pub fn is_zero(&self) -> bool {
        self == &FieldElement::zero()
    }
}

use std::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

impl Neg for FieldElement {
    type Output = FieldElement;

    fn neg(self) -> Self::Output {
        FieldElement(-self.0)
    }
}

impl Mul for FieldElement {
    type Output = FieldElement;
    fn mul(self, rhs: FieldElement) -> Self::Output {
        FieldElement(self.0 * rhs.0)
    }
}
impl Add for FieldElement {
    type Output = FieldElement;
    fn add(self, rhs: FieldElement) -> Self::Output {
        FieldElement(self.0 + rhs.0)
    }
}
impl AddAssign for FieldElement {
    fn add_assign(&mut self, rhs: FieldElement) {
        *self = *self + rhs;
    }
}

impl Sub for FieldElement {
    type Output = FieldElement;
    fn sub(self, rhs: FieldElement) -> Self::Output {
        FieldElement(self.0 - rhs.0)
    }
}
impl SubAssign for FieldElement {
    fn sub_assign(&mut self, rhs: FieldElement) {
        *self = *self - rhs;
    }
}
