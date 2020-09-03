use crate::circuit::Witness;
use crate::Linear;
use noir_field::FieldElement;
use std::collections::HashMap;
use std::ops::{Add, Mul, Neg, Sub};
// In the addition polynomial
// We can have arbitrary fan-in/out, so we need more than wL,wR and wO
// When looking at the arithmetic gate for the quotient polynomial in standard plonk
// You can think of it as fan-in 2 and fan out-1 , or you can think of it as fan-in 1 and fan-out 2
//
// In the multiplication polynomial
// XXX: If we allow the degree of the quotient polynomial to be arbitrary, then we will need a vector of wire values
#[derive(Clone, Debug)]
pub struct Arithmetic {
    // To avoid having to create intermediate variables pre-optimisation
    // We collect all of the multiplication terms in the arithmetic gate
    // A multiplication term if of the form q_M * wL * wR
    // Hence this vector represents the following sum: q_M1 * wL1 * wR1 + q_M2 * wL2 * wR2 + .. +
    pub mul_terms: Vec<(FieldElement, Witness, Witness)>,

    // XXX(low) : Remove fan_in and fan_out and just have simplified_fan
    pub fan_in: Vec<(FieldElement, Witness)>,
    pub fan_out: Vec<(FieldElement, Witness)>,
    // Upon optimising, we simplify the gates by merging the like terms in the fan-in and fan-out
    // They are then stored in this gate
    pub simplified_fan: Vec<(FieldElement, Witness)>,

    pub q_C: FieldElement,
}

impl Default for Arithmetic {
    fn default() -> Arithmetic {
        Arithmetic {
            mul_terms: Vec::new(),
            fan_in: Vec::new(),
            fan_out: Vec::new(),
            simplified_fan: Vec::new(),
            q_C: FieldElement::zero(),
        }
    }
}

impl Arithmetic {
    pub(crate) fn simplify_fan(&mut self) {
        let max_terms = std::cmp::max(self.fan_in.len(), self.fan_out.len());
        let mut hash_map: HashMap<Witness, FieldElement> = HashMap::with_capacity(max_terms);

        // Add the fan-in terms
        for (scale, witness) in self.fan_in.clone().into_iter() {
            *hash_map.entry(witness).or_insert(FieldElement::zero()) += scale;
        }
        // Add the fan-out terms
        for (scale, witness) in self.fan_out.clone().into_iter() {
            *hash_map.entry(witness).or_insert(FieldElement::zero()) -= scale;
        }

        // Convert hashmap back to a vector
        self.simplified_fan = hash_map
            .into_iter()
            .map(|(witness, scale)| (scale, witness))
            .collect();

        // Clear fan out and fan in
        self.fan_in.clear();
        self.fan_out.clear();
    }
}

impl Mul<&FieldElement> for &Arithmetic {
    type Output = Arithmetic;
    fn mul(self, rhs: &FieldElement) -> Self::Output {
        // Scale the mul terms
        let mul_terms: Vec<_> = self
            .mul_terms
            .iter()
            .map(|(qM, wL, wR)| (*qM * *rhs, wL.clone(), wR.clone()))
            .collect();

        // Scale the fan-in terms
        let fan_in: Vec<_> = self
            .fan_in
            .iter()
            .map(|(qL, wL)| (*qL * *rhs, wL.clone()))
            .collect();

        // Scale the fan-out terms
        let fan_out: Vec<_> = self
            .fan_out
            .iter()
            .map(|(qO, wO)| (*qO * *rhs, wO.clone()))
            .collect();

        // Scale the constant
        let q_C = self.q_C * *rhs;

        Arithmetic {
            mul_terms,
            fan_in,
            fan_out,
            q_C,
            simplified_fan: Vec::new(),
        }
    }
}
impl Add<&FieldElement> for Arithmetic {
    type Output = Arithmetic;
    fn add(self, rhs: &FieldElement) -> Self::Output {
        // Increase the constant
        let q_C = self.q_C + *rhs;

        Arithmetic {
            mul_terms: self.mul_terms,
            fan_in: self.fan_in,
            fan_out: self.fan_out,
            q_C,
            simplified_fan: self.simplified_fan,
        }
    }
}
impl Sub<&FieldElement> for Arithmetic {
    type Output = Arithmetic;
    fn sub(self, rhs: &FieldElement) -> Self::Output {
        // Increase the constant
        let q_C = *rhs - self.q_C;

        Arithmetic {
            mul_terms: self.mul_terms,
            fan_in: self.fan_in,
            fan_out: self.fan_out,
            q_C,
            simplified_fan: self.simplified_fan,
        }
    }
}

impl Add<&Arithmetic> for &Arithmetic {
    type Output = Arithmetic;
    fn add(self, rhs: &Arithmetic) -> Arithmetic {
        // XXX(med) : Implement an efficient way to do this

        let mul_terms: Vec<_> = self
            .mul_terms
            .iter()
            .cloned()
            .chain(rhs.mul_terms.iter().cloned())
            .collect();
        let fan_in: Vec<_> = self
            .fan_in
            .iter()
            .cloned()
            .chain(rhs.fan_in.iter().cloned())
            .collect();
        let fan_out: Vec<_> = self
            .fan_out
            .iter()
            .cloned()
            .chain(rhs.fan_out.iter().cloned())
            .collect();
        let simplified_fan: Vec<_> = self
            .simplified_fan
            .iter()
            .cloned()
            .chain(rhs.simplified_fan.iter().cloned())
            .collect();
        let q_C = self.q_C + rhs.q_C;

        Arithmetic {
            mul_terms,
            fan_in,
            fan_out,
            simplified_fan,
            q_C,
        }
    }
}

impl Neg for &Arithmetic {
    type Output = Arithmetic;
    fn neg(self) -> Self::Output {
        // XXX(med) : Implement an efficient way to do this

        let mul_terms: Vec<_> = self
            .mul_terms
            .iter()
            .map(|(qM, wL, wR)| (-*qM, wL.clone(), wR.clone()))
            .collect();
        let fan_in: Vec<_> = self
            .fan_in
            .iter()
            .map(|(qL, wL)| (-*qL, wL.clone()))
            .collect();
        let fan_out: Vec<_> = self
            .fan_out
            .iter()
            .map(|(qO, wO)| (-*qO, wO.clone()))
            .collect();
        let simplified_fan: Vec<_> = self
            .simplified_fan
            .iter()
            .map(|(qK, wK)| (-*qK, wK.clone()))
            .collect();
        let q_C = -self.q_C;

        Arithmetic {
            mul_terms,
            fan_in,
            fan_out,
            simplified_fan,
            q_C,
        }
    }
}

impl Sub<&Arithmetic> for &Arithmetic {
    type Output = Arithmetic;
    fn sub(self, rhs: &Arithmetic) -> Arithmetic {
        self + &-rhs
    }
}

impl From<&Linear> for Arithmetic {
    fn from(lin: &Linear) -> Arithmetic {
        Arithmetic {
            q_C: lin.add_scale,
            fan_in: vec![(lin.mul_scale, lin.witness.clone())],
            fan_out: Vec::new(),
            simplified_fan: Vec::new(),
            mul_terms: Vec::new(),
        }
    }
}
impl From<Linear> for Arithmetic {
    fn from(lin: Linear) -> Arithmetic {
        Arithmetic::from(&lin)
    }
}

impl Arithmetic {
    // Checks if this polynomial can fit into one arithmetic identity
    // Should I put this on Arithmetic struct as method?
    pub fn fits_in_one_identity(&self, width: usize) -> bool {
        // A Polynomial with more than one mul term cannot fit into one gate
        if self.mul_terms.len() > 1 {
            return false;
        };
        // A Polynomial with more terms than fan-in cannot fit within a single gate
        if self.simplified_fan.len() > width {
            return false;
        }

        // A polynomial with no mul term and a fan-in that fits inside of the width can fit into a single gate
        if self.mul_terms.len() == 0 {
            return true;
        }

        // A polynomial with width-2 fan-in terms and a single non-zero mul term can fit into one gate
        // Example: Axy + Dz . Notice, that the mul term places a constraint on the first two terms, but not the last term
        // XXX: This would change if our arithmetic polynomial equation was changed to Axyz for example, but for now it is not.
        if self.simplified_fan.len() <= (width - 2) {
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

        for term in self.simplified_fan.iter() {
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

        return found_x & found_y;
    }
}
