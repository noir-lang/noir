use crate::native_types::{Linear, Witness};
use noir_field::FieldElement;
use std::ops::{Add, Mul, Neg, Sub};

// In the addition polynomial
// We can have arbitrary fan-in/out, so we need more than wL,wR and wO
// When looking at the arithmetic gate for the quotient polynomial in standard plonk
// You can think of it as fan-in 2 and fan out-1 , or you can think of it as fan-in 1 and fan-out 2
//
// In the multiplication polynomial
// XXX: If we allow the degree of the quotient polynomial to be arbitrary, then we will need a vector of wire values
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Arithmetic<F: FieldElement> {
    // To avoid having to create intermediate variables pre-optimisation
    // We collect all of the multiplication terms in the arithmetic gate
    // A multiplication term if of the form q_M * wL * wR
    // Hence this vector represents the following sum: q_M1 * wL1 * wR1 + q_M2 * wL2 * wR2 + .. +
    pub mul_terms: Vec<(F, Witness, Witness)>,

    pub linear_combinations: Vec<(F, Witness)>,
    pub q_c: F,
}

impl<F: FieldElement> Default for Arithmetic<F> {
    fn default() -> Arithmetic<F> {
        Arithmetic {
            mul_terms: Vec::new(),
            linear_combinations: Vec::new(),
            q_c: FieldElement::zero(),
        }
    }
}

impl<F: FieldElement> Arithmetic<F> {
    pub fn can_defer_constraint(&self) -> bool {
        false
    }
    pub fn num_mul_terms(&self) -> usize {
        self.mul_terms.len()
    }
}

impl<F: FieldElement> Mul<&F> for &Arithmetic<F> {
    type Output = Arithmetic<F>;
    fn mul(self, rhs: &F) -> Self::Output {
        // Scale the mul terms
        let mul_terms: Vec<_> = self
            .mul_terms
            .iter()
            .map(|(q_m, w_l, w_r)| (*q_m * *rhs, *w_l, *w_r))
            .collect();

        // Scale the linear combinations terms
        let lin_combinations: Vec<_> = self
            .linear_combinations
            .iter()
            .map(|(q_l, w_l)| (*q_l * *rhs, *w_l))
            .collect();

        // Scale the constant
        let q_c = self.q_c * *rhs;

        Arithmetic {
            mul_terms,
            q_c,
            linear_combinations: lin_combinations,
        }
    }
}
impl<F: FieldElement> Add<&F> for Arithmetic<F> {
    type Output = Arithmetic<F>;
    fn add(self, rhs: &F) -> Self::Output {
        // Increase the constant
        let q_c = self.q_c + *rhs;

        Arithmetic {
            mul_terms: self.mul_terms,
            q_c,
            linear_combinations: self.linear_combinations,
        }
    }
}
impl<F: FieldElement> Sub<&F> for Arithmetic<F> {
    type Output = Arithmetic<F>;
    fn sub(self, rhs: &F) -> Self::Output {
        // Increase the constant
        let q_c = self.q_c - *rhs;

        Arithmetic {
            mul_terms: self.mul_terms,
            q_c,
            linear_combinations: self.linear_combinations,
        }
    }
}

impl<F: FieldElement> Add<&Arithmetic<F>> for &Arithmetic<F> {
    type Output = Arithmetic<F>;
    fn add(self, rhs: &Arithmetic<F>) -> Arithmetic<F> {
        // XXX(med) : Implement an efficient way to do this

        let mul_terms: Vec<_> = self
            .mul_terms
            .iter()
            .cloned()
            .chain(rhs.mul_terms.iter().cloned())
            .collect();

        let linear_combinations: Vec<_> = self
            .linear_combinations
            .iter()
            .cloned()
            .chain(rhs.linear_combinations.iter().cloned())
            .collect();
        let q_c = self.q_c + rhs.q_c;

        Arithmetic {
            mul_terms,
            linear_combinations,
            q_c,
        }
    }
}

impl<F: FieldElement> Neg for &Arithmetic<F> {
    type Output = Arithmetic<F>;
    fn neg(self) -> Self::Output {
        // XXX(med) : Implement an efficient way to do this

        let mul_terms: Vec<_> = self
            .mul_terms
            .iter()
            .map(|(q_m, w_l, w_r)| (-*q_m, *w_l, *w_r))
            .collect();

        let linear_combinations: Vec<_> = self
            .linear_combinations
            .iter()
            .map(|(q_k, w_k)| (-*q_k, *w_k))
            .collect();
        let q_c = -self.q_c;

        Arithmetic {
            mul_terms,
            linear_combinations,
            q_c,
        }
    }
}

impl<F: FieldElement> Sub<&Arithmetic<F>> for &Arithmetic<F> {
    type Output = Arithmetic<F>;
    fn sub(self, rhs: &Arithmetic<F>) -> Arithmetic<F> {
        self + &-rhs
    }
}

impl<F: FieldElement> From<&F> for Arithmetic<F> {
    fn from(constant: &F) -> Arithmetic<F> {
        Arithmetic {
            q_c: *constant,
            linear_combinations: Vec::new(),
            mul_terms: Vec::new(),
        }
    }
}
impl<F: FieldElement> From<&Linear<F>> for Arithmetic<F> {
    fn from(lin: &Linear<F>) -> Arithmetic<F> {
        Arithmetic {
            q_c: lin.add_scale,
            linear_combinations: vec![(lin.mul_scale, lin.witness)],
            mul_terms: Vec::new(),
        }
    }
}
impl<F: FieldElement> From<Linear<F>> for Arithmetic<F> {
    fn from(lin: Linear<F>) -> Arithmetic<F> {
        Arithmetic::from(&lin)
    }
}
impl<F: FieldElement> From<&Witness> for Arithmetic<F> {
    fn from(wit: &Witness) -> Arithmetic<F> {
        Linear::from_witness(*wit).into()
    }
}

impl<F: FieldElement> Add<&Arithmetic<F>> for &Linear<F> {
    type Output = Arithmetic<F>;
    fn add(self, rhs: &Arithmetic<F>) -> Arithmetic<F> {
        &Arithmetic::from(self) + rhs
    }
}
impl<F: FieldElement> Add<&Linear<F>> for &Arithmetic<F> {
    type Output = Arithmetic<F>;
    fn add(self, rhs: &Linear<F>) -> Arithmetic<F> {
        &Arithmetic::from(rhs) + self
    }
}
impl<F: FieldElement> Sub<&Witness> for &Arithmetic<F> {
    type Output = Arithmetic<F>;
    fn sub(self, rhs: &Witness) -> Arithmetic<F> {
        self - &Arithmetic::from(rhs)
    }
}

impl<F: FieldElement> Arithmetic<F> {
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
