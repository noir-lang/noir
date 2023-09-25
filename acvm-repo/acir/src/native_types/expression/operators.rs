use crate::native_types::Witness;
use acir_field::FieldElement;
use std::{
    cmp::Ordering,
    ops::{Add, Mul, Neg, Sub},
};

use super::Expression;

// Negation

impl Neg for &Expression {
    type Output = Expression;
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

impl Add<FieldElement> for Expression {
    type Output = Expression;
    fn add(self, rhs: FieldElement) -> Self::Output {
        // Increase the constant
        let q_c = self.q_c + rhs;

        Expression { mul_terms: self.mul_terms, q_c, linear_combinations: self.linear_combinations }
    }
}

impl Add<Expression> for FieldElement {
    type Output = Expression;
    #[inline]
    fn add(self, rhs: Expression) -> Self::Output {
        rhs + self
    }
}

impl Sub<FieldElement> for Expression {
    type Output = Expression;
    fn sub(self, rhs: FieldElement) -> Self::Output {
        // Increase the constant
        let q_c = self.q_c - rhs;

        Expression { mul_terms: self.mul_terms, q_c, linear_combinations: self.linear_combinations }
    }
}

impl Sub<Expression> for FieldElement {
    type Output = Expression;
    #[inline]
    fn sub(self, rhs: Expression) -> Self::Output {
        rhs - self
    }
}

impl Mul<FieldElement> for &Expression {
    type Output = Expression;
    fn mul(self, rhs: FieldElement) -> Self::Output {
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

impl Mul<&Expression> for FieldElement {
    type Output = Expression;
    #[inline]
    fn mul(self, rhs: &Expression) -> Self::Output {
        rhs * self
    }
}

// Witness

impl Add<Witness> for &Expression {
    type Output = Expression;
    fn add(self, rhs: Witness) -> Expression {
        self + &Expression::from(rhs)
    }
}

impl Add<&Expression> for Witness {
    type Output = Expression;
    #[inline]
    fn add(self, rhs: &Expression) -> Expression {
        rhs + self
    }
}

impl Sub<Witness> for &Expression {
    type Output = Expression;
    fn sub(self, rhs: Witness) -> Expression {
        self - &Expression::from(rhs)
    }
}

impl Sub<&Expression> for Witness {
    type Output = Expression;
    #[inline]
    fn sub(self, rhs: &Expression) -> Expression {
        rhs - self
    }
}

// Mul<Witness> is not implemented as this could result in degree 3 terms.

// Expression

impl Add<&Expression> for &Expression {
    type Output = Expression;
    fn add(self, rhs: &Expression) -> Expression {
        self.add_mul(FieldElement::one(), rhs)
    }
}

impl Sub<&Expression> for &Expression {
    type Output = Expression;
    fn sub(self, rhs: &Expression) -> Expression {
        self.add_mul(-FieldElement::one(), rhs)
    }
}

impl Mul<&Expression> for &Expression {
    type Output = Option<Expression>;
    fn mul(self, rhs: &Expression) -> Option<Expression> {
        if self.is_const() {
            return Some(self.q_c * rhs);
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
fn single_mul(w: Witness, b: &Expression) -> Expression {
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

#[test]
fn add_smoketest() {
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
fn mul_smoketest() {
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
