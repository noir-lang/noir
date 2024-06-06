use crate::native_types::Witness;
use acir_field::AcirField;
use std::{
    cmp::Ordering,
    ops::{Add, Mul, Neg, Sub},
};

use super::Expression;

// Negation

impl<F: AcirField> Neg for &Expression<F> {
    type Output = Expression<F>;
    fn neg(self) -> Self::Output {
        // XXX(med) : Implement an efficient way to do this

        let mul_terms: Vec<_> =
            self.mul_terms.iter().map(|(q_m, w_l, w_r)| (-*q_m, *w_l, *w_r)).collect();

        let linear_combinations: Vec<_> =
            self.linear_combinations.iter().map(|(q_k, w_k)| (-*q_k, *w_k)).collect();
        let q_c = -self.q_c;

        Expression { mul_terms, linear_combinations, q_c }
    }
}

// FieldElement

impl<F: AcirField> Add<F> for Expression<F> {
    type Output = Self;
    fn add(self, rhs: F) -> Self::Output {
        // Increase the constant
        let q_c = self.q_c + rhs;

        Expression { mul_terms: self.mul_terms, q_c, linear_combinations: self.linear_combinations }
    }
}

impl<F: AcirField> Sub<F> for Expression<F> {
    type Output = Self;
    fn sub(self, rhs: F) -> Self::Output {
        // Increase the constant
        let q_c = self.q_c - rhs;

        Expression { mul_terms: self.mul_terms, q_c, linear_combinations: self.linear_combinations }
    }
}

impl<F: AcirField> Mul<F> for &Expression<F> {
    type Output = Expression<F>;
    fn mul(self, rhs: F) -> Self::Output {
        // Scale the mul terms
        let mul_terms: Vec<_> =
            self.mul_terms.iter().map(|(q_m, w_l, w_r)| (*q_m * rhs, *w_l, *w_r)).collect();

        // Scale the linear combinations terms
        let lin_combinations: Vec<_> =
            self.linear_combinations.iter().map(|(q_l, w_l)| (*q_l * rhs, *w_l)).collect();

        // Scale the constant
        let q_c = self.q_c * rhs;

        Expression { mul_terms, q_c, linear_combinations: lin_combinations }
    }
}

// Witness

impl<F: AcirField> Add<Witness> for &Expression<F> {
    type Output = Expression<F>;
    fn add(self, rhs: Witness) -> Self::Output {
        self + &Expression::from(rhs)
    }
}

impl<F: AcirField> Add<&Expression<F>> for Witness {
    type Output = Expression<F>;
    #[inline]
    fn add(self, rhs: &Expression<F>) -> Self::Output {
        rhs + self
    }
}

impl<F: AcirField> Sub<Witness> for &Expression<F> {
    type Output = Expression<F>;
    fn sub(self, rhs: Witness) -> Self::Output {
        self - &Expression::from(rhs)
    }
}

impl<F: AcirField> Sub<&Expression<F>> for Witness {
    type Output = Expression<F>;
    #[inline]
    fn sub(self, rhs: &Expression<F>) -> Self::Output {
        rhs - self
    }
}

// Mul<Witness> is not implemented as this could result in degree 3 terms.

// Expression

impl<F: AcirField> Add<&Expression<F>> for &Expression<F> {
    type Output = Expression<F>;
    fn add(self, rhs: &Expression<F>) -> Self::Output {
        self.add_mul(F::one(), rhs)
    }
}

impl<F: AcirField> Sub<&Expression<F>> for &Expression<F> {
    type Output = Expression<F>;
    fn sub(self, rhs: &Expression<F>) -> Self::Output {
        self.add_mul(-F::one(), rhs)
    }
}

impl<F: AcirField> Mul<&Expression<F>> for &Expression<F> {
    type Output = Option<Expression<F>>;
    fn mul(self, rhs: &Expression<F>) -> Self::Output {
        if self.is_const() {
            return Some(rhs * self.q_c);
        } else if rhs.is_const() {
            return Some(self * rhs.q_c);
        } else if !(self.is_linear() && rhs.is_linear()) {
            // `Expression`s can only represent terms which are up to degree 2.
            // We then disallow multiplication of `Expression`s which have degree 2 terms.
            return None;
        }

        let mut output = Expression::from_field(self.q_c * rhs.q_c);

        //TODO to optimize...
        for lc in &self.linear_combinations {
            let single = single_mul(lc.1, rhs);
            output = output.add_mul(lc.0, &single);
        }

        //linear terms
        let mut i1 = 0; //a
        let mut i2 = 0; //b
        while i1 < self.linear_combinations.len() && i2 < rhs.linear_combinations.len() {
            let (a_c, a_w) = self.linear_combinations[i1];
            let (b_c, b_w) = rhs.linear_combinations[i2];

            // Apply scaling from multiplication
            let a_c = rhs.q_c * a_c;
            let b_c = self.q_c * b_c;

            let (coeff, witness) = match a_w.cmp(&b_w) {
                Ordering::Greater => {
                    i2 += 1;
                    (b_c, b_w)
                }
                Ordering::Less => {
                    i1 += 1;
                    (a_c, a_w)
                }
                Ordering::Equal => {
                    // Here we're taking both terms as the witness indices are equal.
                    // We then advance both `i1` and `i2`.
                    i1 += 1;
                    i2 += 1;
                    (a_c + b_c, a_w)
                }
            };

            if !coeff.is_zero() {
                output.linear_combinations.push((coeff, witness));
            }
        }
        while i1 < self.linear_combinations.len() {
            let (a_c, a_w) = self.linear_combinations[i1];
            let coeff = rhs.q_c * a_c;
            if !coeff.is_zero() {
                output.linear_combinations.push((coeff, a_w));
            }
            i1 += 1;
        }
        while i2 < rhs.linear_combinations.len() {
            let (b_c, b_w) = rhs.linear_combinations[i2];
            let coeff = self.q_c * b_c;
            if !coeff.is_zero() {
                output.linear_combinations.push((coeff, b_w));
            }
            i2 += 1;
        }

        Some(output)
    }
}

/// Returns `w*b.linear_combinations`
fn single_mul<F: AcirField>(w: Witness, b: &Expression<F>) -> Expression<F> {
    Expression {
        mul_terms: b
            .linear_combinations
            .iter()
            .map(|(a, wit)| {
                let (wl, wr) = if w < *wit { (w, *wit) } else { (*wit, w) };
                (*a, wl, wr)
            })
            .collect(),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use acir_field::{AcirField, FieldElement};

    #[test]
    fn add_smoke_test() {
        let a = Expression {
            mul_terms: vec![],
            linear_combinations: vec![(FieldElement::from(2u128), Witness(2))],
            q_c: FieldElement::from(2u128),
        };

        let b = Expression {
            mul_terms: vec![],
            linear_combinations: vec![(FieldElement::from(4u128), Witness(4))],
            q_c: FieldElement::one(),
        };

        assert_eq!(
            &a + &b,
            Expression {
                mul_terms: vec![],
                linear_combinations: vec![
                    (FieldElement::from(2u128), Witness(2)),
                    (FieldElement::from(4u128), Witness(4))
                ],
                q_c: FieldElement::from(3u128)
            }
        );

        // Enforce commutativity
        assert_eq!(&a + &b, &b + &a);
    }

    #[test]
    fn mul_smoke_test() {
        let a = Expression {
            mul_terms: vec![],
            linear_combinations: vec![(FieldElement::from(2u128), Witness(2))],
            q_c: FieldElement::from(2u128),
        };

        let b = Expression {
            mul_terms: vec![],
            linear_combinations: vec![(FieldElement::from(4u128), Witness(4))],
            q_c: FieldElement::one(),
        };

        assert_eq!(
            (&a * &b).unwrap(),
            Expression {
                mul_terms: vec![(FieldElement::from(8u128), Witness(2), Witness(4)),],
                linear_combinations: vec![
                    (FieldElement::from(2u128), Witness(2)),
                    (FieldElement::from(8u128), Witness(4))
                ],
                q_c: FieldElement::from(2u128)
            }
        );

        // Enforce commutativity
        assert_eq!(&a * &b, &b * &a);
    }
}
