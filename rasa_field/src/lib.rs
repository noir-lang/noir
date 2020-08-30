
    use ff::{Field, PrimeField, PrimeFieldRepr};
    use pairing::bn256::Fr;

// XXX: Switch out for a trait and proper implementations
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FieldElement(Fr);

impl std::hash::Hash for FieldElement {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write(&self.to_bytes())
    }
}

impl From<i128> for FieldElement {
    fn from(mut a : i128) -> FieldElement {
        let mut negative = false;
        if a < 0 {
            a = -a;
            negative = true;
        }
        

        let mut result = Fr::from_str(&a.to_string()).expect("Cannot convert i128 as a string to a field element");
        
        if negative {
            result.negate();
        }
        return FieldElement(result)
    }
}

impl FieldElement {

    pub fn one() -> FieldElement {
        FieldElement(Fr::one())
    }
    pub fn zero() -> FieldElement {
        FieldElement(Fr::zero())
    }
    pub fn is_one(&self) -> bool {
        self == &FieldElement::one()
    }
    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
    /// Computes the inverse or returns zero if the inverse does not exist
    /// Before using this FieldElement, please ensure that this behaviour is necessary
    pub fn inverse(&self) -> FieldElement {
        let inv = self.0.inverse().unwrap_or(Fr::zero());
        FieldElement(inv)
    }

    pub fn to_hex(&self) -> String {
        self.0.to_hex()
    }

    // XXX: 100% sure there is a better way to do this. Check API for it.
    pub fn to_bytes(&self) -> [u8;32] {
        let mut buf = [0;32];
        hex::decode_to_slice(self.to_hex(), &mut buf).unwrap();
        buf
    }

}

use std::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign, Div};

impl Neg for FieldElement {
    type Output = FieldElement;

    fn neg(mut self) -> Self::Output {
        self.0.negate();
        FieldElement(self.0)
    }
}

impl Mul for FieldElement {
    type Output = FieldElement;
    fn mul(mut self, rhs: FieldElement) -> Self::Output {
        self.0.mul_assign(&rhs.0);
        FieldElement(self.0)
    }
}
impl Div for FieldElement {
    type Output = FieldElement;
    fn div(self, rhs: FieldElement) -> Self::Output {
        self * rhs.inverse()
    }
}
impl Add for FieldElement {
    type Output = FieldElement;
    fn add(mut self, rhs: FieldElement) -> Self::Output {
        self.0.add_assign(&rhs.0);
        FieldElement(self.0)
    }
}
impl AddAssign for FieldElement {
    fn add_assign(&mut self, rhs: FieldElement) {
        self.0.add_assign(&rhs.0);
    }
}

impl Sub for FieldElement {
    type Output = FieldElement;
    fn sub(mut self, rhs: FieldElement) -> Self::Output {
        self.0.sub_assign(&rhs.0);
        FieldElement(self.0)
    }
}
impl SubAssign for FieldElement {
    fn sub_assign(&mut self, rhs: FieldElement) {
        self.0.sub_assign(&rhs.0);
    }
}