use crate::native_types::{Linear, Witness};
use noir_field::FieldElement;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::ops::{Add, Mul, Neg, Sub};

use super::witness::UnknownWitness;

// In the addition polynomial
// We can have arbitrary fan-in/out, so we need more than wL,wR and wO
// When looking at the arithmetic gate for the quotient polynomial in standard plonk
// You can think of it as fan-in 2 and fan out-1 , or you can think of it as fan-in 1 and fan-out 2
//
// In the multiplication polynomial
// XXX: If we allow the degree of the quotient polynomial to be arbitrary, then we will need a vector of wire values
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Expression {
    // To avoid having to create intermediate variables pre-optimisation
    // We collect all of the multiplication terms in the arithmetic gate
    // A multiplication term if of the form q_M * wL * wR
    // Hence this vector represents the following sum: q_M1 * wL1 * wR1 + q_M2 * wL2 * wR2 + .. +
    pub mul_terms: Vec<(FieldElement, Witness, Witness)>,

    pub linear_combinations: Vec<(FieldElement, Witness)>,
    pub q_c: FieldElement,
}

impl Default for Expression {
    fn default() -> Expression {
        Expression {
            mul_terms: Vec::new(),
            linear_combinations: Vec::new(),
            q_c: FieldElement::zero(),
        }
    }
}

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.mul_terms.is_empty() && self.linear_combinations.len() == 1 && self.q_c.is_zero() {
            write!(f, "x{}", self.linear_combinations[0].1.witness_index())
        } else {
            write!(f, "%{:?}%", crate::circuit::gate::Gate::Arithmetic(self.clone()))
        }
    }
}

impl Ord for Expression {
    fn cmp(&self, other: &Self) -> Ordering {
        let mut i1 = self.get_max_idx();
        let mut i2 = other.get_max_idx();
        let mut result = Ordering::Equal;
        while result == Ordering::Equal {
            let m1 = self.get_max_term(&mut i1);
            let m2 = other.get_max_term(&mut i2);
            if m1.is_none() && m2.is_none() {
                return Ordering::Equal;
            }
            result = Expression::cmp_max(m1, m2);
        }
        result
    }
}

impl PartialOrd for Expression {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

struct WitnessIdx {
    linear: usize,
    mul: usize,
    second_term: bool,
}

impl Expression {
    pub const fn can_defer_constraint(&self) -> bool {
        false
    }
    pub fn num_mul_terms(&self) -> usize {
        self.mul_terms.len()
    }

    pub fn from_field(q_c: FieldElement) -> Expression {
        Self { q_c, ..Default::default() }
    }

    pub fn one() -> Expression {
        Self::from_field(FieldElement::one())
    }

    pub fn zero() -> Expression {
        Self::default()
    }

    pub fn is_linear(&self) -> bool {
        self.mul_terms.is_empty()
    }

    pub fn is_const(&self) -> bool {
        self.mul_terms.is_empty() && self.linear_combinations.is_empty()
    }

    fn get_max_idx(&self) -> WitnessIdx {
        WitnessIdx {
            linear: self.linear_combinations.len(),
            mul: self.mul_terms.len(),
            second_term: true,
        }
    }
    // Returns the maximum witness at the provided position, and decrement the position
    // This function assumes the gate is sorted
    fn get_max_term(&self, idx: &mut WitnessIdx) -> Option<Witness> {
        if idx.linear > 0 {
            if idx.mul > 0 {
                let mul_term = if idx.second_term {
                    self.mul_terms[idx.mul - 1].2
                } else {
                    self.mul_terms[idx.mul - 1].1
                };
                if self.linear_combinations[idx.linear - 1].1 > mul_term {
                    idx.linear -= 1;
                    Some(self.linear_combinations[idx.linear].1)
                } else {
                    if idx.second_term {
                        idx.second_term = false;
                    } else {
                        idx.mul -= 1;
                    }
                    Some(mul_term)
                }
            } else {
                idx.linear -= 1;
                Some(self.linear_combinations[idx.linear].1)
            }
        } else if idx.mul > 0 {
            if idx.second_term {
                idx.second_term = false;
                Some(self.mul_terms[idx.mul - 1].2)
            } else {
                idx.mul -= 1;
                Some(self.mul_terms[idx.mul].1)
            }
        } else {
            None
        }
    }

    fn cmp_max(m1: Option<Witness>, m2: Option<Witness>) -> Ordering {
        if let Some(m1) = m1 {
            if let Some(m2) = m2 {
                m1.cmp(&m2)
            } else {
                Ordering::Greater
            }
        } else if m2.is_some() {
            Ordering::Less
        } else {
            Ordering::Equal
        }
    }

    /// Sorts gate in a deterministic order
    /// XXX: We can probably make this more efficient by sorting on each phase. We only care if it is deterministic
    pub fn sort(&mut self) {
        self.mul_terms.sort_by(|a, b| a.1.cmp(&b.1).then(a.2.cmp(&b.2)));
        self.linear_combinations.sort_by(|a, b| a.1.cmp(&b.1));
    }
}

impl Mul<&FieldElement> for &Expression {
    type Output = Expression;
    fn mul(self, rhs: &FieldElement) -> Self::Output {
        // Scale the mul terms
        let mul_terms: Vec<_> =
            self.mul_terms.iter().map(|(q_m, w_l, w_r)| (*q_m * *rhs, *w_l, *w_r)).collect();

        // Scale the linear combinations terms
        let lin_combinations: Vec<_> =
            self.linear_combinations.iter().map(|(q_l, w_l)| (*q_l * *rhs, *w_l)).collect();

        // Scale the constant
        let q_c = self.q_c * *rhs;

        Expression { mul_terms, q_c, linear_combinations: lin_combinations }
    }
}
impl Add<&FieldElement> for Expression {
    type Output = Expression;
    fn add(self, rhs: &FieldElement) -> Self::Output {
        // Increase the constant
        let q_c = self.q_c + *rhs;

        Expression { mul_terms: self.mul_terms, q_c, linear_combinations: self.linear_combinations }
    }
}
impl Sub<&FieldElement> for Expression {
    type Output = Expression;
    fn sub(self, rhs: &FieldElement) -> Self::Output {
        // Increase the constant
        let q_c = self.q_c - *rhs;

        Expression { mul_terms: self.mul_terms, q_c, linear_combinations: self.linear_combinations }
    }
}

impl Add<&Expression> for &Expression {
    type Output = Expression;
    fn add(self, rhs: &Expression) -> Expression {
        // XXX(med) : Implement an efficient way to do this

        let mul_terms: Vec<_> =
            self.mul_terms.iter().cloned().chain(rhs.mul_terms.iter().cloned()).collect();

        let linear_combinations: Vec<_> = self
            .linear_combinations
            .iter()
            .cloned()
            .chain(rhs.linear_combinations.iter().cloned())
            .collect();
        let q_c = self.q_c + rhs.q_c;

        Expression { mul_terms, linear_combinations, q_c }
    }
}

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

impl Sub<&Expression> for &Expression {
    type Output = Expression;
    fn sub(self, rhs: &Expression) -> Expression {
        self + &-rhs
    }
}

impl From<&FieldElement> for Expression {
    fn from(constant: &FieldElement) -> Expression {
        Expression { q_c: *constant, linear_combinations: Vec::new(), mul_terms: Vec::new() }
    }
}
impl From<&Linear> for Expression {
    fn from(lin: &Linear) -> Expression {
        Expression {
            q_c: lin.add_scale,
            linear_combinations: vec![(lin.mul_scale, lin.witness)],
            mul_terms: Vec::new(),
        }
    }
}
impl From<Linear> for Expression {
    fn from(lin: Linear) -> Expression {
        Expression::from(&lin)
    }
}
impl From<&Witness> for Expression {
    fn from(wit: &Witness) -> Expression {
        Linear::from_witness(*wit).into()
    }
}

impl Add<&Expression> for &Linear {
    type Output = Expression;
    fn add(self, rhs: &Expression) -> Expression {
        &Expression::from(self) + rhs
    }
}
impl Add<&Linear> for &Expression {
    type Output = Expression;
    fn add(self, rhs: &Linear) -> Expression {
        &Expression::from(rhs) + self
    }
}
impl Sub<&Witness> for &Expression {
    type Output = Expression;
    fn sub(self, rhs: &Witness) -> Expression {
        self - &Expression::from(rhs)
    }
}
impl Sub<&UnknownWitness> for &Expression {
    type Output = Expression;
    fn sub(self, rhs: &UnknownWitness) -> Expression {
        let mut cloned = self.clone();
        cloned.linear_combinations.insert(0, (-FieldElement::one(), rhs.as_witness()));
        cloned
    }
}

impl Expression {
    // Checks if this polynomial can fit into one arithmetic identity
    pub fn fits_in_one_identity(&self, width: usize) -> bool {
        // A Polynomial with more than one mul term cannot fit into one gate
        if self.mul_terms.len() > 1 {
            return false;
        };
        // A Polynomial with more terms than fan-in cannot fit within a single gate
        if self.linear_combinations.len() > width {
            return false;
        }

        // A polynomial with no mul term and a fan-in that fits inside of the width can fit into a single gate
        if self.mul_terms.is_empty() {
            return true;
        }

        // A polynomial with width-2 fan-in terms and a single non-zero mul term can fit into one gate
        // Example: Axy + Dz . Notice, that the mul term places a constraint on the first two terms, but not the last term
        // XXX: This would change if our arithmetic polynomial equation was changed to Axyz for example, but for now it is not.
        if self.linear_combinations.len() <= (width - 2) {
            return true;
        }

        // We now know that we have a single mul term. We also know that the mul term must match up with two other terms
        // A polynomial whose mul terms are non zero which do not match up with two terms in the fan-in cannot fit into one gate
        // An example of this is: Axy + Bx + Cy + ...
        // Notice how the bivariate monomial xy has two univariate monomials with their respective coefficients
        // XXX: note that if x or y is zero, then we could apply a further optimisation, but this would be done in another algorithm.
        // It would be the same as when we have zero coefficients - Can only work if wire is constrained to be zero publicly
        let mul_term = &self.mul_terms[0];

        // The coefficient should be non-zero, as this method is ran after the compiler removes all zero coefficient terms
        assert_ne!(mul_term.0, FieldElement::zero());

        let mut found_x = false;
        let mut found_y = false;

        for term in self.linear_combinations.iter() {
            let witness = &term.1;
            let x = &mul_term.1;
            let y = &mul_term.2;
            if witness == x {
                found_x = true;
            };
            if witness == y {
                found_y = true;
            };
            if found_x & found_y {
                break;
            }
        }

        found_x & found_y
    }
}
