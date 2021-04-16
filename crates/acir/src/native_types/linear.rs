// turn off linting related to operator usage (this
// file contains /implementations/)
#![allow(clippy::op_ref)]

use crate::native_types::{Arithmetic, Witness};
use noir_field::FieldElement;

use std::ops::{Add, Mul, Neg, Sub};

#[derive(Clone, Copy, Debug)]
pub struct Linear<F> {
    pub mul_scale: F,
    pub witness: Witness,
    pub add_scale: F,
}

impl<F: FieldElement> Linear<F> {
    pub fn is_unit(&self) -> bool {
        self.mul_scale.is_one() && self.add_scale.is_zero()
    }
    pub fn from_witness(witness: Witness) -> Linear<F> {
        Linear {
            mul_scale: FieldElement::one(),
            witness,
            add_scale: FieldElement::zero(),
        }
    }
    // XXX: This is true for the NPC languages that we use, are there any where this is not true?
    pub fn can_defer_constraint(&self) -> bool {
        true
    }
}

impl<F: FieldElement> From<Witness> for Linear<F> {
    fn from(w: Witness) -> Linear<F> {
        Linear::from_witness(w)
    }
}
impl<F: FieldElement> From<F> for Linear<F> {
    fn from(element: F) -> Linear<F> {
        Linear {
            add_scale: element,
            witness: Witness::default(),
            mul_scale: FieldElement::zero(),
        }
    }
}

impl<F: FieldElement> Add<&Linear<F>> for &Linear<F> {
    type Output = Arithmetic<F>;
    fn add(self, rhs: &Linear<F>) -> Self::Output {
        // (Ax+B) + ( Cx + D) = (Ax + Cx) + ( B+D)
        // (Ax + B) + (Cy + D) = Ax + Cy + (B+D)
        Arithmetic {
            mul_terms: Vec::new(),
            linear_combinations: vec![(self.mul_scale, self.witness), (rhs.mul_scale, rhs.witness)],
            q_c: self.add_scale + rhs.add_scale,
        }
    }
}

impl<F: FieldElement> Neg for &Linear<F> {
    type Output = Linear<F>;
    fn neg(self) -> Self::Output {
        // -(Ax + B) = -Ax - B
        Linear {
            add_scale: -self.add_scale,
            witness: self.witness,
            mul_scale: -self.mul_scale,
        }
    }
}

impl<F: FieldElement> Mul<&Linear<F>> for &Linear<F> {
    type Output = Arithmetic<F>;
    #[allow(clippy::many_single_char_names)]
    fn mul(self, rhs: &Linear<F>) -> Self::Output {
        // (Ax+B)(Cy+D) = ACxy + ADx + BCy + BD
        let a = self.mul_scale;
        let b = self.add_scale;
        let x = self.witness;

        let c = rhs.mul_scale;
        let d = rhs.add_scale;
        let y = rhs.witness;

        let ac = a * c;
        let ad = a * d;
        let bc = b * c;
        let bd = b * d;

        let mul_terms = {
            let mut mt = Vec::with_capacity(1);
            if ac != FieldElement::zero() {
                mt.push((ac, x, y))
            }
            mt
        };

        let linear_combinations = {
            let mut lc = Vec::with_capacity(2);

            if ad != FieldElement::zero() {
                lc.push((ad, x));
            }
            if bc != FieldElement::zero() {
                lc.push((bc, y));
            }
            lc
        };

        Arithmetic {
            mul_terms,
            linear_combinations,
            q_c: bd,
        }
    }
}
impl<F: FieldElement> Mul<&F> for &Linear<F> {
    type Output = Linear<F>;
    fn mul(self, rhs: &F) -> Self::Output {
        Linear {
            mul_scale: self.mul_scale * *rhs,
            witness: self.witness,
            add_scale: self.add_scale * *rhs,
        }
    }
}
impl<F: FieldElement> Add<&F> for &Linear<F> {
    type Output = Linear<F>;
    fn add(self, rhs: &F) -> Self::Output {
        Linear {
            mul_scale: self.mul_scale,
            witness: self.witness,
            add_scale: self.add_scale + *rhs,
        }
    }
}

// Convenience Trait implementations
impl<F: FieldElement> Add<Linear<F>> for Linear<F> {
    type Output = Arithmetic<F>;
    fn add(self, rhs: Linear<F>) -> Self::Output {
        &self + &rhs
    }
}
impl<F: FieldElement> Mul<Linear<F>> for Linear<F> {
    type Output = Arithmetic<F>;
    fn mul(self, rhs: Linear<F>) -> Self::Output {
        &self * &rhs
    }
}
impl<F: FieldElement> Add<&Linear<F>> for Linear<F> {
    type Output = Arithmetic<F>;
    fn add(self, rhs: &Linear<F>) -> Self::Output {
        &self + rhs
    }
}
impl<F: FieldElement> Mul<&Linear<F>> for Linear<F> {
    type Output = Arithmetic<F>;
    fn mul(self, rhs: &Linear<F>) -> Self::Output {
        &self * rhs
    }
}
impl<F: FieldElement> Sub<&Linear<F>> for &Linear<F> {
    type Output = Arithmetic<F>;
    fn sub(self, rhs: &Linear<F>) -> Self::Output {
        self + &-rhs
    }
}
impl<F: FieldElement> Sub<&F> for &Linear<F> {
    type Output = Linear<F>;
    fn sub(self, rhs: &F) -> Self::Output {
        self + &-*rhs
    }
}
