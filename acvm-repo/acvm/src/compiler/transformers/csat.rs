use std::{cmp::Ordering, collections::HashSet};

use acir::{
    native_types::{Expression, Witness},
    AcirField,
};
use indexmap::IndexMap;

/// Minimum width accepted by the `CSatTransformer`.
pub const MIN_EXPRESSION_WIDTH: usize = 3;

/// A transformer which processes any [`Expression`]s to break them up such that they
/// fit within the [`ProofSystemCompiler`][crate::ProofSystemCompiler]'s width.
///
/// This transformer is only used when targeting the [`Bounded`][crate::ExpressionWidth::Bounded] configuration.
///
/// This is done by creating intermediate variables to hold partial calculations and then combining them
/// to calculate the original expression.
// Should we give it all of the opcodes?
// Have a single transformer that you instantiate with a width, then pass many opcodes through
pub(crate) struct CSatTransformer {
    width: usize,
    /// Track the witness that can be solved
    solvable_witness: HashSet<Witness>,
}

impl CSatTransformer {
    /// Create an optimizer with a given width.
    ///
    /// Panics if `width` is less than `MIN_EXPRESSION_WIDTH`.
    pub(crate) fn new(width: usize) -> CSatTransformer {
        assert!(width >= MIN_EXPRESSION_WIDTH, "width has to be at least {MIN_EXPRESSION_WIDTH}");

        CSatTransformer { width, solvable_witness: HashSet::new() }
    }

    /// Check if the equation 'expression=0' can be solved, and if yes, add the solved witness to set of solvable witness
    fn try_solve<F>(&mut self, opcode: &Expression<F>) {
        let mut unresolved = Vec::new();
        for (_, w1, w2) in &opcode.mul_terms {
            if !self.solvable_witness.contains(w1) {
                unresolved.push(w1);
                if !self.solvable_witness.contains(w2) {
                    return;
                }
            }
            if !self.solvable_witness.contains(w2) {
                unresolved.push(w2);
                if !self.solvable_witness.contains(w1) {
                    return;
                }
            }
        }
        for (_, w) in &opcode.linear_combinations {
            if !self.solvable_witness.contains(w) {
                unresolved.push(w);
            }
        }
        if unresolved.len() == 1 {
            self.mark_solvable(*unresolved[0]);
        }
    }

    /// Adds the witness to set of solvable witness
    pub(crate) fn mark_solvable(&mut self, witness: Witness) {
        self.solvable_witness.insert(witness);
    }

    // Still missing dead witness optimization.
    // To do this, we will need the whole set of assert-zero opcodes
    // I think it can also be done before the local optimization seen here, as dead variables will come from the user
    pub(crate) fn transform<F: AcirField>(
        &mut self,
        opcode: Expression<F>,
        intermediate_variables: &mut IndexMap<Expression<F>, (F, Witness)>,
        num_witness: &mut u32,
    ) -> Expression<F> {
        // Here we create intermediate variables and constrain them to be equal to any subset of the polynomial that can be represented as a full opcode
        let opcode =
            self.full_opcode_scan_optimization(opcode, intermediate_variables, num_witness);
        // The last optimization to do is to create intermediate variables in order to flatten the fan-in and the amount of mul terms
        // If a opcode has more than one mul term. We may need an intermediate variable for each one. Since not every variable will need to link to
        // the mul term, we could possibly do it that way.
        // We wil call this a partial opcode scan optimization which will result in the opcodes being able to fit into the correct width
        let mut opcode =
            self.partial_opcode_scan_optimization(opcode, intermediate_variables, num_witness);
        opcode.sort();
        self.try_solve(&opcode);
        opcode
    }

    // This optimization will search for combinations of terms which can be represented in a single assert-zero opcode
    // Case 1 : qM * wL * wR + qL * wL + qR * wR + qO * wO + qC
    // This polynomial does not require any further optimizations, it can be safely represented in one opcode
    // ie a polynomial with 1 mul(bi-variate) term and 3 (univariate) terms where 2 of those terms match the bivariate term
    // wL and wR, we can represent it in one opcode
    // GENERALIZED for WIDTH: instead of the number 3, we use `WIDTH`
    //
    //
    // Case 2: qM * wL * wR + qL * wL + qR * wR + qO * wO + qC + qM2 * wL2 * wR2 + qL * wL2 + qR * wR2 + qO * wO2 + qC2
    // This polynomial cannot be represented using one assert-zero opcode.
    //
    // This algorithm will first extract the first full opcode(if possible):
    // t = qM * wL * wR + qL * wL + qR * wR + qO * wO + qC
    //
    // The polynomial now looks like so t + qM2 * wL2 * wR2 + qL * wL2 + qR * wR2 + qO * wO2 + qC2
    // This polynomial cannot be represented using one assert-zero opcode.
    //
    // This algorithm will then extract the second full opcode(if possible):
    // t2 = qM2 * wL2 * wR2 + qL * wL2 + qR * wR2 + qO * wO2 + qC2
    //
    // The polynomial now looks like so t + t2
    // We can no longer extract another full opcode, hence the algorithm terminates. Creating two intermediate variables t and t2.
    // This stage of preprocessing does not guarantee that all polynomials can fit into a opcode. It only guarantees that all full opcodes have been extracted from each polynomial
    fn full_opcode_scan_optimization<F: AcirField>(
        &mut self,
        mut opcode: Expression<F>,
        intermediate_variables: &mut IndexMap<Expression<F>, (F, Witness)>,
        num_witness: &mut u32,
    ) -> Expression<F> {
        // We pass around this intermediate variable IndexMap, so that we do not create intermediate variables that we have created before
        // One instance where this might happen is t1 = wL * wR and t2 = wR * wL

        // First check that this is not a simple opcode which does not need optimization
        //
        // If the opcode only has one mul term, then this algorithm cannot optimize it any further
        // Either it can be represented in a single arithmetic equation or it's fan-in is too large and we need intermediate variables for those
        // large-fan-in optimization is not this algorithms purpose.
        // If the opcode has 0 mul terms, then it is an add opcode and similarly it can either fit into a single assert-zero opcode or it has a large fan-in
        if opcode.mul_terms.len() <= 1 {
            return opcode;
        }

        // We now know that this opcode has multiple mul terms and can possibly be simplified into multiple full opcodes
        // We need to create a (wl, wr) IndexMap and then check the simplified fan-in to verify if we have terms both with wl and wr
        // In general, we can then push more terms into the opcode until we are at width-1 then the last variable will be the intermediate variable
        //

        // This will be our new opcode which will be equal to `self` except we will have intermediate variables that will be constrained to any
        // subset of the terms that can be represented as full opcodes
        let mut new_opcode = Expression::default();
        let mut remaining_mul_terms = Vec::with_capacity(opcode.mul_terms.len());
        for pair in opcode.mul_terms {
            // We want to layout solvable intermediate variable, if we cannot solve one of the witness
            // that means the intermediate opcode will not be immediately solvable
            if !self.solvable_witness.contains(&pair.1) || !self.solvable_witness.contains(&pair.2)
            {
                remaining_mul_terms.push(pair);
                continue;
            }

            // Check if this pair is present in the simplified fan-in
            // We are assuming that the fan-in/fan-out has been simplified.
            // Note this function is not public, and can only be called within the optimize method, so this guarantee will always hold
            let index_wl =
                opcode.linear_combinations.iter().position(|(_scale, witness)| *witness == pair.1);
            let index_wr =
                opcode.linear_combinations.iter().position(|(_scale, witness)| *witness == pair.2);

            match (index_wl, index_wr) {
                (None, _) => {
                    // This means that the polynomial does not contain both terms
                    // Just push the Qm term as it cannot form a full opcode
                    new_opcode.mul_terms.push(pair);
                }
                (_, None) => {
                    // This means that the polynomial does not contain both terms
                    // Just push the Qm term as it cannot form a full opcode
                    new_opcode.mul_terms.push(pair);
                }
                (Some(x), Some(y)) => {
                    // This means that we can form a full opcode with this Qm term

                    // First fetch the left and right wires which match the mul term
                    let left_wire_term = opcode.linear_combinations[x];
                    let right_wire_term = opcode.linear_combinations[y];

                    // Lets create an intermediate opcode to store this full opcode
                    //
                    let mut intermediate_opcode = Expression::default();
                    intermediate_opcode.mul_terms.push(pair);

                    // Add the left and right wires
                    intermediate_opcode.linear_combinations.push(left_wire_term);
                    intermediate_opcode.linear_combinations.push(right_wire_term);
                    // Remove the left and right wires so we do not re-add them
                    match x.cmp(&y) {
                        Ordering::Greater => {
                            opcode.linear_combinations.remove(x);
                            opcode.linear_combinations.remove(y);
                        }
                        Ordering::Less => {
                            opcode.linear_combinations.remove(y);
                            opcode.linear_combinations.remove(x);
                        }
                        Ordering::Equal => {
                            opcode.linear_combinations.remove(x);
                            intermediate_opcode.linear_combinations.pop();
                        }
                    }

                    // Now we have used up 2 spaces in our assert-zero opcode. The width now dictates, how many more we can add
                    let mut remaining_space = self.width - 2 - 1; // We minus 1 because we need an extra space to contain the intermediate variable
                                                                  // Keep adding terms until we have no more left, or we reach the width
                    let mut remaining_linear_terms =
                        Vec::with_capacity(opcode.linear_combinations.len());
                    while remaining_space > 0 {
                        if let Some(wire_term) = opcode.linear_combinations.pop() {
                            // Add this element into the new opcode
                            if self.solvable_witness.contains(&wire_term.1) {
                                intermediate_opcode.linear_combinations.push(wire_term);
                                remaining_space -= 1;
                            } else {
                                remaining_linear_terms.push(wire_term);
                            }
                        } else {
                            // No more usable elements left in the old opcode
                            break;
                        }
                    }
                    opcode.linear_combinations.extend(remaining_linear_terms);

                    // Constraint this intermediate_opcode to be equal to the temp variable by adding it into the IndexMap
                    // We need a unique name for our intermediate variable
                    // XXX: Another optimization, which could be applied in another algorithm
                    // If two opcodes have a large fan-in/out and they share a few common terms, then we should create intermediate variables for them
                    // Do some sort of subset matching algorithm for this on the terms of the polynomial
                    let inter_var = Self::get_or_create_intermediate_vars(
                        intermediate_variables,
                        intermediate_opcode,
                        num_witness,
                    );

                    // Add intermediate variable to the new opcode instead of the full opcode
                    self.mark_solvable(inter_var.1);
                    new_opcode.linear_combinations.push(inter_var);
                }
            };
        }
        opcode.mul_terms = remaining_mul_terms;

        // Add the rest of the elements back into the new_opcode
        new_opcode.mul_terms.extend(opcode.mul_terms);
        new_opcode.linear_combinations.extend(opcode.linear_combinations);
        new_opcode.q_c = opcode.q_c;
        new_opcode.sort();
        new_opcode
    }

    /// Normalize an expression by dividing it by its first coefficient
    /// The first coefficient here means coefficient of the first linear term, or of the first quadratic term if no linear terms exist.
    /// The function panic if the input expression is constant
    fn normalize<F: AcirField>(mut expr: Expression<F>) -> (F, Expression<F>) {
        expr.sort();
        let a = if !expr.linear_combinations.is_empty() {
            expr.linear_combinations[0].0
        } else {
            expr.mul_terms[0].0
        };
        (a, &expr * a.inverse())
    }

    /// Get or generate a scaled intermediate witness which is equal to the provided expression
    /// The sets of previously generated witness and their (normalized) expression is cached in the intermediate_variables map
    /// If there is no cache hit, we generate a new witness (and add the expression to the cache)
    /// else, we return the cached witness along with the scaling factor so it is equal to the provided expression
    fn get_or_create_intermediate_vars<F: AcirField>(
        intermediate_variables: &mut IndexMap<Expression<F>, (F, Witness)>,
        expr: Expression<F>,
        num_witness: &mut u32,
    ) -> (F, Witness) {
        let (k, normalized_expr) = Self::normalize(expr);

        if intermediate_variables.contains_key(&normalized_expr) {
            let (l, iv) = intermediate_variables[&normalized_expr];
            (k / l, iv)
        } else {
            let inter_var = Witness(*num_witness);
            *num_witness += 1;
            // Add intermediate opcode and variable to map
            intermediate_variables.insert(normalized_expr, (k, inter_var));
            (F::one(), inter_var)
        }
    }

    // A partial opcode scan optimization aim to create intermediate variables in order to compress the polynomial
    // So that it fits within the given width
    // Note that this opcode follows the full opcode scan optimization.
    // We define the partial width as equal to the full width - 2.
    // This is because two of our variables cannot be used as they are linked to the multiplication terms
    // Example: qM1 * wL1 * wR2 + qL1 * wL3 + qR1 * wR4+ qR2 * wR5 + qO1 * wO5 + qC
    // One thing to note is that the multiplication wires do not match any of the fan-in/out wires. This is guaranteed as we have
    // just completed the full opcode optimization algorithm.
    //
    //Actually we can optimize in two ways here: We can create an intermediate variable which is equal to the fan-in terms
    // t = qL1 * wL3 + qR1 * wR4 -> width = 3
    // This `t` value can only use width - 1 terms
    // The opcode now looks like: qM1 * wL1 * wR2 + t + qR2 * wR5+ qO1 * wO5 + qC
    // But this is still not acceptable since wR5 is not wR2, so we need another intermediate variable
    // t2 = t + qR2 * wR5
    //
    // The opcode now looks like: qM1 * wL1 * wR2 + t2 + qO1 * wO5 + qC
    // This is still not good, so we do it one more time:
    // t3 = t2 + qO1 * wO5
    // The opcode now looks like: qM1 * wL1 * wR2 + t3 + qC
    //
    // Another strategy is to create a temporary variable for the multiplier term and then we can see it as a term in the fan-in
    //
    // Same Example: qM1 * wL1 * wR2 + qL1 * wL3 + qR1 * wR4+ qR2 * wR5 + qO1 * wO5 + qC
    // t = qM1 * wL1 * wR2
    // The opcode now looks like: t + qL1 * wL3 + qR1 * wR4+ qR2 * wR5 + qO1 * wO5 + qC
    // Still assuming width3, we still need to use width-1 terms for the intermediate variables, however we can stop at an earlier stage because
    // the opcode does not need the multiplier term to match with any of the fan-in terms
    // t2 = t + qL1 * wL3
    // The opcode now looks like: t2 + qR1 * wR4+ qR2 * wR5 + qO1 * wO5 + qC
    // t3 = t2 + qR1 * wR4
    // The opcode now looks like: t3 + qR2 * wR5 + qO1 * wO5 + qC
    // This took the same amount of opcodes, but which one is better when the width increases? Compute this and maybe do both optimizations
    // naming : partial_opcode_mul_first_opt and partial_opcode_fan_first_opt
    // Also remember that since we did full opcode scan, there is no way we can have a non-zero mul term along with the wL and wR terms being non-zero
    //
    // Cases, a lot of mul terms, a lot of fan-in terms, 50/50
    fn partial_opcode_scan_optimization<F: AcirField>(
        &mut self,
        mut opcode: Expression<F>,
        intermediate_variables: &mut IndexMap<Expression<F>, (F, Witness)>,
        num_witness: &mut u32,
    ) -> Expression<F> {
        // We will go for the easiest route, which is to convert all multiplications into additions using intermediate variables
        // Then use intermediate variables again to squash the fan-in, so that it can fit into the appropriate width

        // First check if this polynomial actually needs a partial opcode optimization
        // There is the chance that it fits perfectly within the assert-zero opcode
        if fits_in_one_identity(&opcode, self.width) {
            return opcode;
        }

        // 2. Create Intermediate variables for the multiplication opcodes
        let mut remaining_mul_terms = Vec::with_capacity(opcode.mul_terms.len());
        for mul_term in opcode.mul_terms {
            if self.solvable_witness.contains(&mul_term.1)
                && self.solvable_witness.contains(&mul_term.2)
            {
                let mut intermediate_opcode = Expression::default();

                // Push mul term into the opcode
                intermediate_opcode.mul_terms.push(mul_term);
                // Get an intermediate variable which squashes the multiplication term
                let inter_var = Self::get_or_create_intermediate_vars(
                    intermediate_variables,
                    intermediate_opcode,
                    num_witness,
                );

                // Add intermediate variable as a part of the fan-in for the original opcode
                opcode.linear_combinations.push(inter_var);
                self.mark_solvable(inter_var.1);
            } else {
                remaining_mul_terms.push(mul_term);
            }
        }

        // Remove all of the mul terms as we have intermediate variables to represent them now
        opcode.mul_terms = remaining_mul_terms;

        // We now only have a polynomial with only fan-in/fan-out terms i.e. terms of the form Ax + By + Cd + ...
        // Lets create intermediate variables if all of them cannot fit into the width
        //
        // If the polynomial fits perfectly within the given width, we are finished
        if opcode.linear_combinations.len() <= self.width {
            return opcode;
        }

        // Stores the intermediate variables that are used to
        // reduce the fan in.
        let mut added = Vec::new();

        while opcode.linear_combinations.len() > self.width {
            // Collect as many terms up to the given width-1 and constrain them to an intermediate variable
            let mut intermediate_opcode = Expression::default();

            let mut remaining_linear_terms = Vec::with_capacity(opcode.linear_combinations.len());

            for term in opcode.linear_combinations {
                if self.solvable_witness.contains(&term.1)
                    && intermediate_opcode.linear_combinations.len() < self.width - 1
                {
                    intermediate_opcode.linear_combinations.push(term);
                } else {
                    remaining_linear_terms.push(term);
                }
            }
            opcode.linear_combinations = remaining_linear_terms;
            let not_full = intermediate_opcode.linear_combinations.len() < self.width - 1;
            if intermediate_opcode.linear_combinations.len() > 1 {
                let inter_var = Self::get_or_create_intermediate_vars(
                    intermediate_variables,
                    intermediate_opcode,
                    num_witness,
                );
                self.mark_solvable(inter_var.1);
                added.push(inter_var);
            }
            // The intermediate opcode is not full, but the opcode still has too many terms
            if not_full && opcode.linear_combinations.len() > self.width {
                unreachable!("Could not reduce the expression");
            }
        }

        // Add back the intermediate variables to
        // keep consistency with the original equation.
        opcode.linear_combinations.extend(added);
        self.partial_opcode_scan_optimization(opcode, intermediate_variables, num_witness)
    }
}

/// Checks if this expression can fit into one arithmetic identity
fn fits_in_one_identity<F: AcirField>(expr: &Expression<F>, width: usize) -> bool {
    // A Polynomial with more than one mul term cannot fit into one opcode
    if expr.mul_terms.len() > 1 {
        return false;
    };

    expr.width() <= width
}

#[cfg(test)]
mod tests {
    use super::*;
    use acir::{AcirField, FieldElement};

    #[test]
    fn simple_reduction_smoke_test() {
        let a = Witness(0);
        let b = Witness(1);
        let c = Witness(2);
        let d = Witness(3);

        // a = b + c + d;
        let opcode_a = Expression {
            mul_terms: vec![],
            linear_combinations: vec![
                (FieldElement::one(), a),
                (-FieldElement::one(), b),
                (-FieldElement::one(), c),
                (-FieldElement::one(), d),
            ],
            q_c: FieldElement::zero(),
        };

        let mut intermediate_variables: IndexMap<
            Expression<FieldElement>,
            (FieldElement, Witness),
        > = IndexMap::new();

        let mut num_witness = 4;

        let mut optimizer = CSatTransformer::new(3);
        optimizer.mark_solvable(b);
        optimizer.mark_solvable(c);
        optimizer.mark_solvable(d);
        let got_optimized_opcode_a =
            optimizer.transform(opcode_a, &mut intermediate_variables, &mut num_witness);

        // a = b + c + d => a - b - c - d = 0
        // For width3, the result becomes:
        // a - d + e = 0
        // - c - b  - e = 0
        //
        // a - b + e = 0
        let e = Witness(4);
        let expected_optimized_opcode_a = Expression {
            mul_terms: vec![],
            linear_combinations: vec![
                (FieldElement::one(), a),
                (-FieldElement::one(), d),
                (FieldElement::one(), e),
            ],
            q_c: FieldElement::zero(),
        };
        assert_eq!(expected_optimized_opcode_a, got_optimized_opcode_a);

        assert_eq!(intermediate_variables.len(), 1);

        // e = - c - b
        let expected_intermediate_opcode = Expression {
            mul_terms: vec![],
            linear_combinations: vec![(-FieldElement::one(), c), (-FieldElement::one(), b)],
            q_c: FieldElement::zero(),
        };
        let (_, normalized_opcode) = CSatTransformer::normalize(expected_intermediate_opcode);
        assert!(intermediate_variables.contains_key(&normalized_opcode));
        assert_eq!(intermediate_variables[&normalized_opcode].1, e);
    }

    #[test]
    fn stepwise_reduction_test() {
        let a = Witness(0);
        let b = Witness(1);
        let c = Witness(2);
        let d = Witness(3);
        let e = Witness(4);

        // a = b + c + d + e;
        let opcode_a = Expression {
            mul_terms: vec![],
            linear_combinations: vec![
                (-FieldElement::one(), a),
                (FieldElement::one(), b),
                (FieldElement::one(), c),
                (FieldElement::one(), d),
                (FieldElement::one(), e),
            ],
            q_c: FieldElement::zero(),
        };

        let mut intermediate_variables: IndexMap<
            Expression<FieldElement>,
            (FieldElement, Witness),
        > = IndexMap::new();

        let mut num_witness = 4;

        let mut optimizer = CSatTransformer::new(3);
        optimizer.mark_solvable(a);
        optimizer.mark_solvable(c);
        optimizer.mark_solvable(d);
        optimizer.mark_solvable(e);
        let got_optimized_opcode_a =
            optimizer.transform(opcode_a, &mut intermediate_variables, &mut num_witness);

        // Since b is not known, it cannot be put inside intermediate opcodes, so it must belong to the transformed opcode.
        let contains_b = got_optimized_opcode_a.linear_combinations.iter().any(|(_, w)| *w == b);
        assert!(contains_b);
    }

    #[test]
    fn recognize_expr_with_single_shared_witness_which_fits_in_single_identity() {
        // Regression test for an expression which Zac found which should have been preserved but
        // was being split into two expressions.
        let expr = Expression {
            mul_terms: vec![(-FieldElement::from(555u128), Witness(8), Witness(10))],
            linear_combinations: vec![
                (FieldElement::one(), Witness(10)),
                (FieldElement::one(), Witness(11)),
                (-FieldElement::one(), Witness(13)),
            ],
            q_c: FieldElement::zero(),
        };
        assert!(fits_in_one_identity(&expr, 4));
    }
}
