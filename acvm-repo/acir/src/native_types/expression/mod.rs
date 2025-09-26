use crate::native_types::Witness;
use acir_field::AcirField;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
mod operators;
mod ordering;

/// An expression representing a quadratic polynomial.
///
/// This struct is primarily used to express arithmetic relations between variables.
/// It includes multiplication terms, linear combinations, and a constant term.
///
/// # Addition polynomial
/// - Unlike standard plonk constraints with fixed wire assignments (wL, wR, wO),
///   we allow arbitrary fan-in and fan-out. This means we need a more flexible representation
///   and we need more than wL, wR, and wO.
/// - When looking at the quotient polynomial for the assert-zero opcode in standard plonk,
///   you can interpret the structure in two ways:
///   1. Fan-in 2 and fan-out 1
///   2. Fan-in 1 and fan-out 2
///
/// # Multiplication polynomial
/// - If we were allow the degree of the quotient polynomial to be arbitrary, then we will need a vector of wire values.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[cfg_attr(feature = "arb", derive(proptest_derive::Arbitrary))]
pub struct Expression<F> {
    /// Collection of multiplication terms.
    ///
    /// To avoid having to create intermediate variables pre-optimization
    /// We collect all of the multiplication terms in the assert-zero opcode
    /// A multiplication term is of the form q_M * wL * wR
    /// Hence this vector represents the following sum: q_M1 * wL1 * wR1 + q_M2 * wL2 * wR2 + .. +
    pub mul_terms: Vec<(F, Witness, Witness)>,

    /// Collection of linear terms in the expression.
    ///
    /// Each term follows the form: `q_L * w`, where `q_L` is a coefficient
    /// and `w` is a witness.
    pub linear_combinations: Vec<(F, Witness)>,
    /// A constant term in the expression
    // TODO: rename q_c to `constant` moreover q_X is not clear to those who
    // TODO: are not familiar with PLONK
    pub q_c: F,
}

impl<F: AcirField> Default for Expression<F> {
    fn default() -> Self {
        Expression { mul_terms: Vec::new(), linear_combinations: Vec::new(), q_c: F::zero() }
    }
}

impl<F: AcirField> std::fmt::Display for Expression<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        display_expression(self, false, f)
    }
}

impl<F> Expression<F> {
    /// Returns the number of multiplication terms
    pub fn num_mul_terms(&self) -> usize {
        self.mul_terms.len()
    }

    /// Adds a new linear term to the `Expression`.
    pub fn push_addition_term(&mut self, coefficient: F, variable: Witness) {
        self.linear_combinations.push((coefficient, variable));
    }

    /// Adds a new quadratic term to the `Expression`.
    pub fn push_multiplication_term(&mut self, coefficient: F, lhs: Witness, rhs: Witness) {
        self.mul_terms.push((coefficient, lhs, rhs));
    }

    /// Returns `true` if the expression represents a constant polynomial.
    ///
    /// Examples:
    /// -  f(x,y) = x + y would return false
    /// -  f(x,y) = xy would return false, the degree here is 2
    /// -  f(x,y) = 5 would return true, the degree is 0
    pub fn is_const(&self) -> bool {
        self.mul_terms.is_empty() && self.linear_combinations.is_empty()
    }

    /// Returns a `FieldElement` if the expression represents a constant polynomial.
    /// Otherwise returns `None`.
    ///
    /// Examples:
    /// - f(x,y) = x would return `None`
    /// - f(x,y) = x + 6 would return `None`
    /// - f(x,y) = 2*y + 6 would return `None`
    /// - f(x,y) = x + y would return `None`
    /// - f(x,y) = 5 would return `FieldElement(5)`
    pub fn to_const(&self) -> Option<&F> {
        self.is_const().then_some(&self.q_c)
    }

    /// Returns `true` if highest degree term in the expression is one or less.
    ///
    /// - `mul_term` in an expression contains degree-2 terms
    /// - `linear_combinations` contains degree-1 terms
    ///
    /// Hence, it is sufficient to check that there are no `mul_terms`
    ///
    /// Examples:
    /// -  f(x,y) = x + y would return true
    /// -  f(x,y) = xy would return false, the degree here is 2
    /// -  f(x,y) = 0 would return true, the degree is 0
    pub fn is_linear(&self) -> bool {
        self.mul_terms.is_empty()
    }

    /// Returns `true` if the expression can be seen as a degree-1 univariate polynomial
    ///
    /// - `mul_terms` in an expression can be univariate, however unless the coefficient
    ///   is zero, it is always degree-2.
    /// - `linear_combinations` contains the sum of degree-1 terms, these terms do not
    ///   need to contain the same variable and so it can be multivariate. However, we
    ///   have thus far only checked if `linear_combinations` contains one term, so this
    ///   method will return false, if the `Expression` has not been simplified.
    ///
    /// Hence, we check in the simplest case if an expression is a degree-1 univariate,
    /// by checking if it contains no `mul_terms` and it contains one `linear_combination` term.
    ///
    /// Examples:
    /// - f(x,y) = x would return true
    /// - f(x,y) = x + 6 would return true
    /// - f(x,y) = 2*y + 6 would return true
    /// - f(x,y) = x + y would return false
    /// - f(x,y) = x + x should return true, but we return false *** (we do not simplify)
    /// - f(x,y) = 5 would return false
    pub fn is_degree_one_univariate(&self) -> bool {
        self.is_linear() && self.linear_combinations.len() == 1
    }

    /// Sorts opcode in a deterministic order
    /// XXX: We can probably make this more efficient by sorting on each phase. We only care if it is deterministic
    pub fn sort(&mut self) {
        self.mul_terms.sort_by(|a, b| a.1.cmp(&b.1).then(a.2.cmp(&b.2)));
        self.linear_combinations.sort_by(|a, b| a.1.cmp(&b.1));
    }
}

impl<F: AcirField> Expression<F> {
    pub fn from_field(q_c: F) -> Self {
        Self { q_c, ..Default::default() }
    }

    pub fn zero() -> Self {
        Self::default()
    }

    pub fn is_zero(&self) -> bool {
        *self == Self::zero()
    }

    pub fn one() -> Self {
        Self::from_field(F::one())
    }

    pub fn is_one(&self) -> bool {
        *self == Self::one()
    }

    /// Returns a `Witness` if the `Expression` can be represented as a degree-1
    /// univariate polynomial. Otherwise returns `None`.
    ///
    /// Note that `Witness` is only capable of expressing polynomials of the form
    /// f(x) = x and not polynomials of the form f(x) = mx+c , so this method has
    /// extra checks to ensure that m=1 and c=0
    pub fn to_witness(&self) -> Option<Witness> {
        if self.is_degree_one_univariate() {
            // If we get here, we know that our expression is of the form `f(x) = mx+c`
            // We want to now restrict ourselves to expressions of the form f(x) = x
            // ie where the constant term is 0 and the coefficient in front of the variable is
            // one.
            let (coefficient, variable) = self.linear_combinations[0];
            let constant = self.q_c;

            if coefficient.is_one() && constant.is_zero() {
                return Some(variable);
            }
        }
        None
    }

    /// Returns `self + k*b`
    pub fn add_mul(&self, k: F, b: &Self) -> Self {
        if k.is_zero() {
            return self.clone();
        } else if self.is_const() {
            let kb = b * k;
            return kb + self.q_c;
        } else if b.is_const() {
            return self.clone() + (k * b.q_c);
        }

        let mut mul_terms: Vec<(F, Witness, Witness)> =
            Vec::with_capacity(self.mul_terms.len() + b.mul_terms.len());
        let mut linear_combinations: Vec<(F, Witness)> =
            Vec::with_capacity(self.linear_combinations.len() + b.linear_combinations.len());
        let q_c = self.q_c + k * b.q_c;

        //linear combinations
        let mut i1 = 0; //a
        let mut i2 = 0; //b
        while i1 < self.linear_combinations.len() && i2 < b.linear_combinations.len() {
            let (a_c, a_w) = self.linear_combinations[i1];
            let (b_c, b_w) = b.linear_combinations[i2];

            let (coeff, witness) = match a_w.cmp(&b_w) {
                Ordering::Greater => {
                    i2 += 1;
                    (k * b_c, b_w)
                }
                Ordering::Less => {
                    i1 += 1;
                    (a_c, a_w)
                }
                Ordering::Equal => {
                    // Here we're taking both witnesses as the witness indices are equal.
                    // We then advance both `i1` and `i2`.
                    i1 += 1;
                    i2 += 1;
                    (a_c + k * b_c, a_w)
                }
            };

            if !coeff.is_zero() {
                linear_combinations.push((coeff, witness));
            }
        }

        // Finally process all the remaining terms which we didn't handle in the above loop.
        while i1 < self.linear_combinations.len() {
            linear_combinations.push(self.linear_combinations[i1]);
            i1 += 1;
        }
        while i2 < b.linear_combinations.len() {
            let (b_c, b_w) = b.linear_combinations[i2];
            let coeff = b_c * k;
            if !coeff.is_zero() {
                linear_combinations.push((coeff, b_w));
            }
            i2 += 1;
        }

        //mul terms

        i1 = 0; //a
        i2 = 0; //b
        while i1 < self.mul_terms.len() && i2 < b.mul_terms.len() {
            let (a_c, a_wl, a_wr) = self.mul_terms[i1];
            let (b_c, b_wl, b_wr) = b.mul_terms[i2];

            let (coeff, wl, wr) = match (a_wl, a_wr).cmp(&(b_wl, b_wr)) {
                Ordering::Greater => {
                    i2 += 1;
                    (k * b_c, b_wl, b_wr)
                }
                Ordering::Less => {
                    i1 += 1;
                    (a_c, a_wl, a_wr)
                }
                Ordering::Equal => {
                    // Here we're taking both terms as the witness indices are equal.
                    // We then advance both `i1` and `i2`.
                    i2 += 1;
                    i1 += 1;
                    (a_c + k * b_c, a_wl, a_wr)
                }
            };

            if !coeff.is_zero() {
                mul_terms.push((coeff, wl, wr));
            }
        }

        // Finally process all the remaining terms which we didn't handle in the above loop.
        while i1 < self.mul_terms.len() {
            mul_terms.push(self.mul_terms[i1]);
            i1 += 1;
        }
        while i2 < b.mul_terms.len() {
            let (b_c, b_wl, b_wr) = b.mul_terms[i2];
            let coeff = b_c * k;
            if coeff != F::zero() {
                mul_terms.push((coeff, b_wl, b_wr));
            }
            i2 += 1;
        }

        Expression { mul_terms, linear_combinations, q_c }
    }

    /// Determine the width of this expression.
    /// The width meaning the number of unique witnesses needed for this expression.
    pub fn width(&self) -> usize {
        let mut width = 0;

        for mul_term in &self.mul_terms {
            // The coefficient should be non-zero, as this method is ran after the compiler removes all zero coefficient terms
            assert_ne!(mul_term.0, F::zero());

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

            // If the multiplication is a squaring then we must assign the two witnesses to separate wires and so we
            // can never get a zero contribution to the width.
            let multiplication_is_squaring = mul_term.1 == mul_term.2;

            let mul_term_width_contribution = if !multiplication_is_squaring && (found_x & found_y)
            {
                // Both witnesses involved in the multiplication exist elsewhere in the expression.
                // They both do not contribute to the width of the expression as this would be double-counting
                // due to their appearance in the linear terms.
                0
            } else if found_x || found_y {
                // One of the witnesses involved in the multiplication exists elsewhere in the expression.
                // The multiplication then only contributes 1 new witness to the width.
                1
            } else {
                // Worst case scenario, the multiplication is using completely unique witnesses so has a contribution of 2.
                2
            };

            width += mul_term_width_contribution;
        }

        width += self.linear_combinations.len();

        width
    }
}

impl<F: AcirField> From<F> for Expression<F> {
    fn from(constant: F) -> Self {
        Expression { q_c: constant, linear_combinations: Vec::new(), mul_terms: Vec::new() }
    }
}

impl<F: AcirField> From<Witness> for Expression<F> {
    /// Creates an Expression from a Witness.
    ///
    /// This is infallible since an `Expression` is
    /// a multi-variate polynomial and a `Witness`
    /// can be seen as a univariate polynomial
    fn from(wit: Witness) -> Self {
        Expression {
            q_c: F::zero(),
            linear_combinations: vec![(F::one(), wit)],
            mul_terms: Vec::new(),
        }
    }
}

/// Displays an expression as a quadratic polynomial.
/// If `as_equal_to_zero` is true, the expression is displayed as equaling zero,
/// where it's tried to shown as a polynomial equal to the largest witness, if possible.
pub(crate) fn display_expression<F: AcirField>(
    expr: &Expression<F>,
    as_equal_to_zero: bool,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    // This is set to an index if we show this expression "as a witness assignment", meaning
    // that the linear combination at this index must not be printed again.
    let mut assignment_witness: Option<usize> = None;

    // If true, negate all coefficients when printing.
    // This is set to true if we show this expression "as a witness assignment", and the witness
    // had a coefficient of 1 and we "moved" everything to the right of the equal sign.
    let mut negate_coefficients = false;

    // Find a linear combination with a coefficient of 1 or -1 and, if there are many,
    // keep the one with the largest witness.
    let linear_witness_one = if as_equal_to_zero {
        expr.linear_combinations
            .iter()
            .enumerate()
            .filter(|(_, (coefficient, _))| coefficient.is_one() || (-*coefficient).is_one())
            .max_by_key(|(_, (_, witness))| witness)
    } else {
        None
    };

    // If we find one, show the expression as equaling this witness to everything else
    // (this is likely to happen as in ACIR gen we tend to equate a witness to previous expressions)
    if let Some((index, (coefficient, witness))) = linear_witness_one {
        assignment_witness = Some(index);
        negate_coefficients = coefficient.is_one();
        write!(f, "{witness} = ")?;
    } else if as_equal_to_zero {
        write!(f, "0 = ")?;
    }

    let mut printed_term = false;

    for (coefficient, witness1, witness2) in &expr.mul_terms {
        let witnesses = [*witness1, *witness2];
        display_term(*coefficient, witnesses, printed_term, negate_coefficients, f)?;
        printed_term = true;
    }

    for (index, (coefficient, witness)) in expr.linear_combinations.iter().enumerate() {
        if assignment_witness
            .is_some_and(|show_as_assignment_index| show_as_assignment_index == index)
        {
            // We already printed this term as part of the assignment
            continue;
        }

        let witnesses = [*witness];
        display_term(*coefficient, witnesses, printed_term, negate_coefficients, f)?;
        printed_term = true;
    }

    if expr.q_c.is_zero() {
        if as_equal_to_zero && !printed_term {
            write!(f, "0")?;
        }
    } else {
        let coefficient = expr.q_c;
        let coefficient = if negate_coefficients { -coefficient } else { coefficient };
        let coefficient_as_string = coefficient.to_string();
        let coefficient_is_negative = coefficient_as_string.starts_with('-');

        if printed_term {
            if coefficient_is_negative {
                write!(f, " - ")?;
            } else {
                write!(f, " + ")?;
            }
        }

        let coefficient =
            if printed_term && coefficient_is_negative { -coefficient } else { coefficient };
        write!(f, "{coefficient}")?;
    }

    Ok(())
}

fn display_term<F: AcirField, const N: usize>(
    coefficient: F,
    witnesses: [Witness; N],
    printed_term: bool,
    negate_coefficients: bool,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    let coefficient = if negate_coefficients { -coefficient } else { coefficient };
    let coefficient_as_string = coefficient.to_string();
    let coefficient_is_negative = coefficient_as_string.starts_with('-');

    if printed_term {
        if coefficient_is_negative {
            write!(f, " - ")?;
        } else {
            write!(f, " + ")?;
        }
    }

    let coefficient =
        if printed_term && coefficient_is_negative { -coefficient } else { coefficient };

    if coefficient.is_one() {
        // Don't print the coefficient
    } else if (-coefficient).is_one() {
        write!(f, "-")?;
    } else {
        write!(f, "{coefficient}*")?;
    }

    for (index, witness) in witnesses.iter().enumerate() {
        if index != 0 {
            write!(f, "*")?;
        }
        write!(f, "{witness}")?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use acir_field::{AcirField, FieldElement};

    #[test]
    fn add_mul_smoke_test() {
        let a = Expression {
            mul_terms: vec![(FieldElement::from(2u128), Witness(1), Witness(2))],
            ..Default::default()
        };

        let k = FieldElement::from(10u128);

        let b = Expression {
            mul_terms: vec![
                (FieldElement::from(3u128), Witness(0), Witness(2)),
                (FieldElement::from(3u128), Witness(1), Witness(2)),
                (FieldElement::from(4u128), Witness(4), Witness(5)),
            ],
            linear_combinations: vec![(FieldElement::from(4u128), Witness(4))],
            q_c: FieldElement::one(),
        };

        let result = a.add_mul(k, &b);
        assert_eq!(
            result,
            Expression {
                mul_terms: vec![
                    (FieldElement::from(30u128), Witness(0), Witness(2)),
                    (FieldElement::from(32u128), Witness(1), Witness(2)),
                    (FieldElement::from(40u128), Witness(4), Witness(5)),
                ],
                linear_combinations: vec![(FieldElement::from(40u128), Witness(4))],
                q_c: FieldElement::from(10u128)
            }
        );
    }
}
