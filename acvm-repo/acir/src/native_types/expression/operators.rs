use crate::native_types::Witness;
use acir_field::AcirField;
use std::ops::{Add, Mul, Neg, Sub};

use super::Expression;

// Negation

impl<F: AcirField> Neg for &Expression<F> {
    type Output = Expression<F>;
    fn neg(self) -> Self::Output {
        let mut mul_terms = self.mul_terms.clone();
        for (q_m, _, _) in &mut mul_terms {
            *q_m = -*q_m;
        }

        let mut linear_combinations = self.linear_combinations.clone();
        for (q_k, _) in &mut linear_combinations {
            *q_k = -*q_k;
        }

        Expression { mul_terms, linear_combinations, q_c: -self.q_c }
    }
}

impl<F: AcirField> Neg for Expression<F> {
    type Output = Expression<F>;
    fn neg(mut self) -> Self::Output {
        for (q_m, _, _) in &mut self.mul_terms {
            *q_m = -*q_m;
        }

        for (q_k, _) in &mut self.linear_combinations {
            *q_k = -*q_k;
        }

        self.q_c = -self.q_c;

        self
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

        // Start with the constant term: q_c_self * q_c_rhs
        let mut output = Expression::from_field(self.q_c * rhs.q_c);

        // 'each linear term in self' * 'each linear term in rhs'
        // XXX: This has a quadratic cost that can be improved, but for now we favor simplicity.
        for lc in &self.linear_combinations {
            let single = single_mul(lc.1, rhs);
            output = output.add_mul(lc.0, &single);
        }

        // Add linear terms from self scaled by rhs's constant: self.linear * rhs.q_c
        if !rhs.q_c.is_zero() {
            let self_linear = Expression {
                mul_terms: Vec::new(),
                linear_combinations: self.linear_combinations.clone(),
                q_c: F::zero(),
            };
            output = output.add_mul(rhs.q_c, &self_linear);
        }

        // Add linear terms from rhs scaled by self's constant: rhs.linear * self.q_c
        if !self.q_c.is_zero() {
            let rhs_linear = Expression {
                mul_terms: Vec::new(),
                linear_combinations: rhs.linear_combinations.clone(),
                q_c: F::zero(),
            };
            output = output.add_mul(self.q_c, &rhs_linear);
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
    use crate::native_types::Expression;
    use acir_field::{AcirField, FieldElement};

    #[test]
    fn add_smoke_test() {
        let a = Expression::from_str("2*w2 + 2").unwrap();
        let b = Expression::from_str("4*w4 + 1").unwrap();
        let result = Expression::from_str("2*w2 + 4*w4 + 3").unwrap();
        assert_eq!(&a + &b, result);

        // Enforce commutativity
        assert_eq!(&a + &b, &b + &a);
    }

    #[test]
    fn mul_smoke_test() {
        let a = Expression::from_str("2*w2 + 2").unwrap();
        let b = Expression::from_str("4*w4 + 1").unwrap();
        let result = Expression::from_str("8*w2*w4 + 2*w2 + 8*w4 + 2").unwrap();
        assert_eq!((&a * &b).unwrap(), result);

        // Enforce commutativity
        assert_eq!(&a * &b, &b * &a);
    }

    #[test]
    fn mul_by_zero_constant() {
        // Multiplying by zero should give zero (with zero coefficients)
        // Note: The implementation may leave zero-coefficient terms in place
        let a = Expression::from_str("3*w1 + 5*w2 + 7").unwrap();
        let zero: Expression<FieldElement> = Expression::zero();

        let result = (&a * &zero).unwrap();
        // All terms should have zero coefficients and the constant should be zero
        assert!(result.mul_terms.is_empty());
        assert!(result.q_c.is_zero());
        for (coeff, _) in &result.linear_combinations {
            assert!(coeff.is_zero());
        }

        // Enforce commutativity
        assert_eq!(&a * &zero, &zero * &a);
    }

    #[test]
    fn mul_by_one_constant() {
        // Multiplying by one should give the same expression
        let a = Expression::from_str("3*w1 + 5*w2 + 7").unwrap();
        let one: Expression<FieldElement> = Expression::one();

        let result = (&a * &one).unwrap();
        assert_eq!(result, a);

        // Enforce commutativity
        assert_eq!(&a * &one, &one * &a);
    }

    #[test]
    fn mul_by_scalar_constant() {
        // Multiplying by a constant should scale all terms
        let a = Expression::from_str("2*w1 + 3*w2 + 4").unwrap();
        let scalar = Expression::from_field(FieldElement::from(5u128));

        let result = (&a * &scalar).unwrap();
        assert_eq!(result.to_string(), "10*w1 + 15*w2 + 20");

        // Enforce commutativity
        assert_eq!(&a * &scalar, &scalar * &a);
    }

    #[test]
    fn mul_two_constants() {
        // Multiplying two constants
        let a = Expression::from_field(FieldElement::from(3u128));
        let b = Expression::from_field(FieldElement::from(7u128));

        let result = (&a * &b).unwrap();
        assert_eq!(result, Expression::from_field(FieldElement::from(21u128)));

        // Enforce commutativity
        assert_eq!(&a * &b, &b * &a);
    }

    #[test]
    fn mul_linear_expressions() {
        // Test multiplication of two linear expressions (no constants)
        let a = Expression::from_str("2*w1 + 3*w2").unwrap();
        let b = Expression::from_str("4*w3 + 5*w4").unwrap();

        let result = (&a * &b).unwrap();
        // (2*w1 + 3*w2) * (4*w3 + 5*w4) = 8*w1*w3 + 10*w1*w4 + 12*w2*w3 + 15*w2*w4
        assert_eq!(result.to_string(), "8*w1*w3 + 10*w1*w4 + 12*w2*w3 + 15*w2*w4");

        // Enforce commutativity
        assert_eq!(&a * &b, &b * &a);
    }

    #[test]
    fn mul_with_shared_witness() {
        // Test multiplication where both expressions share a witness
        let a = Expression::from_str("2*w1 + 3*w2").unwrap();
        let b = Expression::from_str("4*w1 + 5*w3").unwrap();

        let result = (&a * &b).unwrap();
        // (2*w1 + 3*w2) * (4*w1 + 5*w3) = 8*w1*w1 + 10*w1*w3 + 12*w1*w2 + 15*w2*w3
        assert_eq!(result.to_string(), "8*w1*w1 + 12*w1*w2 + 10*w1*w3 + 15*w2*w3");

        // Enforce commutativity
        assert_eq!(&a * &b, &b * &a);
    }

    #[test]
    fn mul_single_witness() {
        // Test squaring a single witness: (w1) * (w1) = w1*w1
        let a = Expression::from_str("w1").unwrap();
        let b = Expression::from_str("w1").unwrap();

        let result = (&a * &b).unwrap();
        assert_eq!(result.to_string(), "w1*w1");
    }

    #[test]
    fn mul_with_constant_term() {
        // Test multiplication where one expression has a constant term
        let a = Expression::from_str("2*w1 + 3").unwrap();
        let b = Expression::from_str("4*w2 + 5").unwrap();

        let result = (&a * &b).unwrap();
        // (2*w1 + 3) * (4*w2 + 5) = 8*w1*w2 + 10*w1 + 12*w2 + 15
        assert_eq!(result.to_string(), "8*w1*w2 + 10*w1 + 12*w2 + 15");

        // Enforce commutativity
        assert_eq!(&a * &b, &b * &a);
    }

    #[test]
    fn mul_degree_two_fails() {
        // Multiplying expressions that would result in degree > 2 should return None
        let a = Expression::from_str("2*w1*w2 + 3*w1").unwrap();
        let b = Expression::from_str("4*w3 + 5").unwrap();

        let result = &a * &b;
        assert!(result.is_none(), "Multiplication should fail for degree > 2");

        // Enforce commutativity
        assert_eq!(&a * &b, &b * &a);
    }

    #[test]
    fn mul_both_degree_two_fails() {
        // Multiplying two degree-2 expressions should fail
        let a = Expression::from_str("w1*w2").unwrap();
        let b = Expression::from_str("w3*w4").unwrap();

        let result = &a * &b;
        assert!(result.is_none(), "Multiplication of two degree-2 expressions should fail");

        // Enforce commutativity
        assert_eq!(&a * &b, &b * &a);
    }

    #[test]
    fn mul_complex_linear_expressions() {
        // Test a more complex multiplication
        let a = Expression::from_str("2*w1 + 3*w2 + 4*w3 + 5").unwrap();
        let b = Expression::from_str("6*w4 + 7*w5 + 8").unwrap();

        let result = (&a * &b).unwrap();
        // (2*w1 + 3*w2 + 4*w3 + 5) * (6*w4 + 7*w5 + 8)
        // = 12*w1*w4 + 14*w1*w5 + 18*w2*w4 + 21*w2*w5 + 24*w3*w4 + 28*w3*w5
        //   + 16*w1 + 24*w2 + 32*w3 + 30*w4 + 35*w5 + 40
        assert_eq!(
            result.to_string(),
            "12*w1*w4 + 14*w1*w5 + 18*w2*w4 + 21*w2*w5 + 24*w3*w4 + 28*w3*w5 + 16*w1 + 24*w2 + 32*w3 + 30*w4 + 35*w5 + 40"
        );

        // Enforce commutativity
        assert_eq!(&a * &b, &b * &a);
    }

    #[test]
    fn mul_witness_ordering() {
        // Test that witness pairs are ordered correctly (smaller index first)
        let a = Expression::from_str("w5").unwrap();
        let b = Expression::from_str("w2").unwrap();

        let result = (&a * &b).unwrap();
        // Should be w2*w5, not w5*w2
        assert_eq!(result.to_string(), "w2*w5");

        // Enforce commutativity
        assert_eq!(&a * &b, &b * &a);
    }

    #[test]
    fn mul_result_is_sorted() {
        // Verify the witness ordering in mul_terms is correct
        let a = Expression::from_str("w3 + w1").unwrap();
        let b = Expression::from_str("w4 + w2").unwrap();

        let result = (&a * &b).unwrap();
        // Verify that each mul_term has properly ordered witnesses (smaller first)
        for (_, wl, wr) in &result.mul_terms {
            assert!(wl <= wr, "Witnesses in mul_terms should be ordered");
        }
    }

    #[test]
    fn neg_reference() {
        // Test negation of a reference (uses clone + in-place negate)
        let a = Expression::from_str("2*w1*w2 + 3*w1 + 5*w2 + 7").unwrap();
        let result = -&a;

        assert_eq!(result.to_string(), "-2*w1*w2 - 3*w1 - 5*w2 - 7");

        // Original should be unchanged
        assert_eq!(a.to_string(), "2*w1*w2 + 3*w1 + 5*w2 + 7");
    }

    #[test]
    fn neg_owned() {
        // Test negation of an owned expression (in-place, no clone)
        let a = Expression::from_str("2*w1*w2 + 3*w1 + 5*w2 + 7").unwrap();
        let result = -a;

        assert_eq!(result.to_string(), "-2*w1*w2 - 3*w1 - 5*w2 - 7");
    }

    #[test]
    fn neg_zero() {
        // Negating zero should give zero
        let zero: Expression<FieldElement> = Expression::zero();
        let result = -&zero;

        assert_eq!(result, Expression::zero());
    }

    #[test]
    fn neg_constant() {
        // Negating a constant expression
        let a = Expression::from_field(FieldElement::from(42u128));
        let result = -a;

        assert_eq!(result.q_c, FieldElement::from(-42i128));
        assert!(result.mul_terms.is_empty());
        assert!(result.linear_combinations.is_empty());
    }

    #[test]
    fn neg_linear_only() {
        // Negating an expression with only linear terms
        let a = Expression::from_str("3*w1 + 5*w2 + 7").unwrap();
        let result = -a;

        assert_eq!(result.to_string(), "-3*w1 - 5*w2 - 7");
    }

    #[test]
    fn neg_mul_only() {
        // Negating an expression with only multiplication terms
        let a = Expression::from_str("2*w1*w2 + 4*w3*w4").unwrap();
        let result = -a;

        assert_eq!(result.to_string(), "-2*w1*w2 - 4*w3*w4");
    }

    #[test]
    fn double_neg() {
        // Double negation should give back the original
        let a = Expression::from_str("2*w1*w2 + 3*w1 + 5").unwrap();
        let result = -(-a.clone());

        assert_eq!(result, a);
    }

    #[test]
    fn neg_preserves_structure() {
        // Negation should preserve the structure (number of terms)
        let a = Expression::from_str("2*w1*w2 + 3*w3*w4 + 5*w1 + 7*w2 + 11").unwrap();
        let result = -&a;

        assert_eq!(result.mul_terms.len(), a.mul_terms.len());
        assert_eq!(result.linear_combinations.len(), a.linear_combinations.len());
    }
}
