use crate::circuit::Witness;
use crate::polynomial::arithmetic::Arithmetic;
use rasa_field::FieldElement;

use std::ops::{Add, Mul, Neg, Sub};

#[derive(Clone, Debug)]
pub struct Linear {
    pub mul_scale: FieldElement,
    pub witness: Witness,
    pub add_scale: FieldElement,
}

impl Linear {
    pub fn is_unit(&self) -> bool {
        self.mul_scale.is_one() && self.add_scale.is_zero()
    }
    pub fn from_witness(witness: Witness) -> Linear {
        Linear {
            mul_scale: FieldElement::one(),
            witness: witness,
            add_scale: FieldElement::zero(),
        }
    }
}

impl From<Witness> for Linear {
    fn from(w: Witness) -> Linear {
        Linear {
            add_scale: FieldElement::zero(),
            witness: w,
            mul_scale: FieldElement::one(),
        }
    }
}
impl From<FieldElement> for Linear {
    fn from(element: FieldElement) -> Linear {
        Linear {
            add_scale: element,
            witness: Witness::default(),
            mul_scale: FieldElement::zero(),
        }
    }
}

impl Add<&Linear> for &Linear {
    type Output = Arithmetic;
    fn add(self, rhs: &Linear) -> Self::Output {
        // (Ax+B) + ( Cx + D) = (Ax + Cx) + ( B+D)
        // (Ax + B) + (Cy + D) = Ax + Cy + (B+D)
        Arithmetic {
            mul_terms: Vec::new(),
            fan_in: vec![
                (self.mul_scale, self.witness.clone()),
                (rhs.mul_scale, rhs.witness.clone()),
            ],
            fan_out: Vec::new(),
            simplified_fan: Vec::new(),
            q_C: self.add_scale + rhs.add_scale,
        }
    }
}

impl Neg for &Linear {
    type Output = Linear;
    fn neg(self) -> Self::Output {
        // -(Ax + B) = -Ax - B
        Linear {
            add_scale: -self.add_scale,
            witness: self.witness.clone(),
            mul_scale: -self.mul_scale,
        }
    }
}

impl Mul<&Linear> for &Linear {
    type Output = Arithmetic;
    fn mul(self, rhs: &Linear) -> Self::Output {
        // (Ax+B)(Cy+D) = ACxy + ADx + BCy + BD
        let a = self.mul_scale;
        let b = self.add_scale;
        let x = self.witness.clone();

        let c = rhs.mul_scale;
        let d = rhs.add_scale;
        let y = rhs.witness.clone();

        let ac = a * c;
        let ad = a * d;
        let bc = b * c;
        let bd = b * d;

        Arithmetic {
            mul_terms: vec![(ac, x.clone(), y.clone())],
            fan_in: vec![(ad, x), (bc, y)],
            fan_out: Vec::new(),
            simplified_fan: Vec::new(),
            q_C: bd,
        }
    }
}
impl Mul<&FieldElement> for &Linear {
    type Output = Linear;
    fn mul(self, rhs: &FieldElement) -> Self::Output {
        Linear {
            mul_scale: self.mul_scale * *rhs,
            witness: self.witness.clone(),
            add_scale: self.add_scale * *rhs,
        }
    }
}
impl Add<&FieldElement> for &Linear {
    type Output = Linear;
    fn add(self, rhs: &FieldElement) -> Self::Output {
        Linear {
            mul_scale: self.mul_scale,
            witness: self.witness.clone(),
            add_scale: self.add_scale + *rhs,
        }
    }
}

/// Convenience Trait implementations
impl Add<Linear> for Linear {
    type Output = Arithmetic;
    fn add(self, rhs: Linear) -> Self::Output {
        &self + &rhs
    }
}
impl Mul<Linear> for Linear {
    type Output = Arithmetic;
    fn mul(self, rhs: Linear) -> Self::Output {
        &self + &rhs
    }
}
impl Add<&Linear> for Linear {
    type Output = Arithmetic;
    fn add(self, rhs: &Linear) -> Self::Output {
        &self + rhs
    }
}
impl Mul<&Linear> for Linear {
    type Output = Arithmetic;
    fn mul(self, rhs: &Linear) -> Self::Output {
        &self + rhs
    }
}
impl Sub<&Linear> for &Linear {
    type Output = Arithmetic;
    fn sub(self, rhs: &Linear) -> Self::Output {
        self + &-rhs
    }
}
