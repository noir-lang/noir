use acir::{
    AcirField,
    native_types::{Expression, Witness, WitnessMap},
};

use super::{ErrorLocation, OpcodeNotSolvable, OpcodeResolutionError, insert_value};

/// An Expression solver will take a Circuit's assert-zero opcodes with witness assignments
/// and create the other witness variables
pub(crate) struct ExpressionSolver;

#[allow(clippy::enum_variant_names)]
pub(super) enum OpcodeStatus<F> {
    OpcodeSatisfied(F),
    OpcodeSolvable(F, (F, Witness)),
    OpcodeUnsolvable,
}

pub(crate) enum MulTerm<F> {
    OneUnknown(F, Witness), // (qM * known_witness, unknown_witness)
    TooManyUnknowns,
    Solved(F),
}

impl ExpressionSolver {
    /// Derives the rest of the witness based on known witness values in a single pass.
    ///
    /// For expressions with 0 or 1 multiplication terms (the common case), this avoids
    /// allocating an intermediate `Expression` and eliminates redundant witness map lookups.
    /// Falls back to the general evaluate-based approach for 2+ mul terms or when
    /// linear term combining is needed.
    pub(crate) fn solve<F: AcirField>(
        initial_witness: &mut WitnessMap<F>,
        opcode: &Expression<F>,
    ) -> Result<(), OpcodeResolutionError<F>> {
        // Evaluate the multiplication term contribution.
        // Most expressions have 0 mul terms; at most 1 is solvable without combining.
        let (mul_constant, mut unknown) = match opcode.mul_terms.len() {
            0 => (F::zero(), None),
            1 => {
                match Self::solve_mul_term_helper(&opcode.mul_terms[0], initial_witness) {
                    MulTerm::Solved(val) => (val, None),
                    MulTerm::OneUnknown(coeff, witness) => {
                        let unknown = if coeff.is_zero() { None } else { Some((coeff, witness)) };
                        (F::zero(), unknown)
                    }
                    MulTerm::TooManyUnknowns => {
                        let (c, _, _) = opcode.mul_terms[0];
                        if c.is_zero() {
                            // Zero-coefficient mul term contributes nothing.
                            (F::zero(), None)
                        } else {
                            // Both witnesses unknown — always unsolvable for a single mul term.
                            return Err(OpcodeResolutionError::OpcodeNotSolvable(
                                OpcodeNotSolvable::ExpressionHasTooManyUnknowns(opcode.clone()),
                            ));
                        }
                    }
                }
            }
            // 2+ mul terms may cancel via combining; use the general solver.
            _ => return Self::solve_via_evaluate(initial_witness, opcode),
        };

        // Single pass over all linear terms (original + extra from partially-evaluated mul).
        let mut sum = opcode.q_c + mul_constant;

        for &(coeff, witness) in &opcode.linear_combinations {
            if let Some(value) = initial_witness.get(&witness) {
                sum += coeff * *value;
            } else if !coeff.is_zero() {
                if unknown.is_some() {
                    // Multiple unknowns — need to try combining duplicate witnesses.
                    return Self::solve_via_evaluate(initial_witness, opcode);
                }
                unknown = Some((coeff, witness));
            }
        }

        if let Some((coeff, witness)) = unknown {
            Self::solve_single_unknown(sum, coeff, witness, initial_witness)
        } else {
            Self::verify_satisfied(sum)
        }
    }

    /// Verify that the fully-evaluated expression equals zero.
    fn verify_satisfied<F: AcirField>(sum: F) -> Result<(), OpcodeResolutionError<F>> {
        if sum.is_zero() {
            Ok(())
        } else {
            Err(OpcodeResolutionError::UnsatisfiedConstrain {
                opcode_location: ErrorLocation::Unresolved,
                payload: None,
            })
        }
    }

    /// Solve `sum + coeff * witness = 0` for the witness.
    fn solve_single_unknown<F: AcirField>(
        sum: F,
        coeff: F,
        witness: Witness,
        initial_witness: &mut WitnessMap<F>,
    ) -> Result<(), OpcodeResolutionError<F>> {
        if coeff.is_zero() {
            Self::verify_satisfied(sum)
        } else {
            let assignment = -quick_invert(sum, coeff);
            insert_value(&witness, assignment, initial_witness)
        }
    }

    /// General solver that allocates an intermediate evaluated `Expression`.
    /// Used as a fallback when the single-pass approach cannot handle the expression
    /// (2+ mul terms, or linear terms that need combining).
    fn solve_via_evaluate<F: AcirField>(
        initial_witness: &mut WitnessMap<F>,
        opcode: &Expression<F>,
    ) -> Result<(), OpcodeResolutionError<F>> {
        let opcode = &ExpressionSolver::evaluate(opcode, initial_witness);

        // Evaluate multiplication terms
        let mul_result = ExpressionSolver::solve_mul_term(&opcode.mul_terms, initial_witness);

        // If we can't solve the multiplication terms, try again by combining multiplication terms
        // with the same witnesses to see if they all cancel out.
        let mul_result = if mul_result.is_err() {
            let mul_terms = ExpressionSolver::combine_mul_terms(&opcode.mul_terms);
            ExpressionSolver::solve_mul_term(&mul_terms, initial_witness)
        } else {
            mul_result
        };

        let mul_result = mul_result.map_err(|_| {
            OpcodeResolutionError::OpcodeNotSolvable(
                OpcodeNotSolvable::ExpressionHasTooManyUnknowns(opcode.clone()),
            )
        })?;

        // Evaluate the fan-in terms
        let opcode_status =
            ExpressionSolver::solve_fan_in_term(&opcode.linear_combinations, initial_witness);

        // If we can solve the multiplication terms but not the linear terms,
        // try again by combining linear terms with the same witness.
        let opcode_status = if matches!(
            (&mul_result, &opcode_status),
            (MulTerm::Solved(..), OpcodeStatus::OpcodeUnsolvable)
        ) {
            let linear_combinations =
                ExpressionSolver::combine_linear_terms(&opcode.linear_combinations);
            ExpressionSolver::solve_fan_in_term(&linear_combinations, initial_witness)
        } else {
            opcode_status
        };

        match (mul_result, opcode_status) {
            // Mul terms solved, one unknown in linear terms.
            (
                MulTerm::Solved(total_prod),
                OpcodeStatus::OpcodeSolvable(partial_sum, (coeff, witness)),
            ) => Self::solve_single_unknown(
                total_prod + partial_sum + opcode.q_c,
                coeff,
                witness,
                initial_witness,
            ),
            // Everything solved — just verify the constraint holds.
            (MulTerm::Solved(a), OpcodeStatus::OpcodeSatisfied(b)) => {
                Self::verify_satisfied(a + b + opcode.q_c)
            }
            // One unknown in the mul term, linear terms fully solved.
            (MulTerm::OneUnknown(coeff, witness), OpcodeStatus::OpcodeSatisfied(sum)) => {
                Self::solve_single_unknown(sum + opcode.q_c, coeff, witness, initial_witness)
            }
            // One unknown appears in both mul and linear terms for the same witness.
            // Combine coefficients: solve (q + b) * w = -(a + q_c)
            (MulTerm::OneUnknown(q, w1), OpcodeStatus::OpcodeSolvable(a, (b, w2))) => {
                if w1 == w2 {
                    Self::solve_single_unknown(a + opcode.q_c, q + b, w1, initial_witness)
                } else {
                    // TODO(https://github.com/noir-lang/noir/issues/10191): can we be more specific with this error?
                    Err(OpcodeResolutionError::OpcodeNotSolvable(
                        OpcodeNotSolvable::ExpressionHasTooManyUnknowns(opcode.clone()),
                    ))
                }
            }
            (MulTerm::TooManyUnknowns, _) | (_, OpcodeStatus::OpcodeUnsolvable) => {
                Err(OpcodeResolutionError::OpcodeNotSolvable(
                    OpcodeNotSolvable::ExpressionHasTooManyUnknowns(opcode.clone()),
                ))
            }
        }
    }

    /// Try to reduce the multiplication terms of the given expression's mul terms to a known value or to a linear term,
    /// using the provided witness mapping.
    /// If there are 2 or more multiplication terms it returns the OpcodeUnsolvable error.
    /// If no witnesses value is in the provided 'witness_assignments' map,
    /// it returns MulTerm::TooManyUnknowns
    fn solve_mul_term<F: AcirField>(
        mul_terms: &[(F, Witness, Witness)],
        witness_assignments: &WitnessMap<F>,
    ) -> Result<MulTerm<F>, OpcodeStatus<F>> {
        // First note that the mul term can only contain one/zero term,
        // e.g. that it has been optimized, or else we're returning OpcodeUnsolvable
        match mul_terms.len() {
            0 => Ok(MulTerm::Solved(F::zero())),
            1 => Ok(ExpressionSolver::solve_mul_term_helper(&mul_terms[0], witness_assignments)),
            _ => Err(OpcodeStatus::OpcodeUnsolvable),
        }
    }

    /// Try to solve a multiplication term of the form q*a*b, where
    /// q is a constant and a,b are witnesses
    /// If both a and b have known values (in the provided map), it returns the value q*a*b
    /// If only one of a or b has a known value, it returns the linear term c*w where c is a constant and w is the unknown witness
    /// If both a and b are unknown, it returns MulTerm::TooManyUnknowns
    fn solve_mul_term_helper<F: AcirField>(
        term: &(F, Witness, Witness),
        witness_assignments: &WitnessMap<F>,
    ) -> MulTerm<F> {
        let (q_m, w_l, w_r) = term;
        // Check if these values are in the witness assignments
        let w_l_value = witness_assignments.get(w_l);
        let w_r_value = witness_assignments.get(w_r);

        match (w_l_value, w_r_value) {
            (None, None) => MulTerm::TooManyUnknowns,
            (Some(w_l), Some(w_r)) => MulTerm::Solved(*q_m * *w_l * *w_r),
            (None, Some(w_r)) => MulTerm::OneUnknown(*q_m * *w_r, *w_l),
            (Some(w_l), None) => MulTerm::OneUnknown(*q_m * *w_l, *w_r),
        }
    }

    /// Reduce a linear term to its value if the witness assignment is known
    /// If the witness value is not known in the provided map, it returns None.
    fn solve_fan_in_term_helper<F: AcirField>(
        term: &(F, Witness),
        witness_assignments: &WitnessMap<F>,
    ) -> Option<F> {
        let (q_l, w_l) = term;
        // Check if we have w_l
        let w_l_value = witness_assignments.get(w_l);
        w_l_value.map(|a| *q_l * *a)
    }

    /// Returns the summation of all of the variables, plus the unknown variable
    /// Returns [`OpcodeStatus::OpcodeUnsolvable`], if there is more than one unknown variable
    pub(super) fn solve_fan_in_term<F: AcirField>(
        linear_combinations: &[(F, Witness)],
        witness_assignments: &WitnessMap<F>,
    ) -> OpcodeStatus<F> {
        // If the fan-in has more than 0 num_unknowns:

        // This is the variable that we want to assign the value to
        let mut unknown_variable = (F::zero(), Witness::default());
        let mut num_unknowns = 0;
        // This is the sum of all of the known variables
        let mut result = F::zero();

        for term in linear_combinations {
            let value = ExpressionSolver::solve_fan_in_term_helper(term, witness_assignments);
            match value {
                Some(a) => result += a,
                None => {
                    unknown_variable = *term;
                    num_unknowns += 1;
                }
            }

            // If we have more than 1 unknown, then we cannot solve this equation
            if num_unknowns > 1 {
                return OpcodeStatus::OpcodeUnsolvable;
            }
        }

        if num_unknowns == 0 {
            return OpcodeStatus::OpcodeSatisfied(result);
        }

        OpcodeStatus::OpcodeSolvable(result, unknown_variable)
    }

    // Partially evaluate the opcode using the known witnesses
    // For instance if values of witness 'a' and 'b' are known, then
    // the multiplication 'a*b' is removed and their multiplied values are added to the constant term
    // If only witness 'a' is known, then the multiplication 'a*b' is replaced by the linear term '(value of b)*a'
    // etc ...
    // If all values are known, the partial evaluation gives a constant expression
    // If no value is known, the partial evaluation returns the original expression
    pub(crate) fn evaluate<F: AcirField>(
        expr: &Expression<F>,
        initial_witness: &WitnessMap<F>,
    ) -> Expression<F> {
        let mut result = Expression::default();
        for &(c, w1, w2) in &expr.mul_terms {
            let mul_result = ExpressionSolver::solve_mul_term_helper(&(c, w1, w2), initial_witness);
            match mul_result {
                MulTerm::OneUnknown(v, w) => {
                    if !v.is_zero() {
                        result.linear_combinations.push((v, w));
                    }
                }
                MulTerm::TooManyUnknowns => {
                    if !c.is_zero() {
                        result.mul_terms.push((c, w1, w2));
                    }
                }
                MulTerm::Solved(f) => result.q_c += f,
            }
        }
        for &(c, w) in &expr.linear_combinations {
            if let Some(f) = ExpressionSolver::solve_fan_in_term_helper(&(c, w), initial_witness) {
                result.q_c += f;
            } else if !c.is_zero() {
                result.linear_combinations.push((c, w));
            }
        }
        result.q_c += expr.q_c;
        result
    }

    /// Combines linear terms with the same witness by summing their coefficients.
    /// For example `w1 + 2*w1` becomes `3*w1`.
    pub(crate) fn combine_linear_terms<F: AcirField>(
        linear_combinations: &[(F, Witness)],
    ) -> Vec<(F, Witness)> {
        let mut combined_linear_combinations = std::collections::HashMap::new();

        for (c, w) in linear_combinations {
            let existing_c = combined_linear_combinations.entry(*w).or_insert(F::zero());
            *existing_c += *c;
        }

        combined_linear_combinations
            .into_iter()
            .filter_map(
                |(witness, coeff)| {
                    if !coeff.is_zero() { Some((coeff, witness)) } else { None }
                },
            )
            .collect()
    }

    /// Combines multiplication terms with the same witnesses by summing their coefficients.
    /// For example `w1*w2 + 2*w2*w1` becomes `3*w1*w2`. If a coefficient ends up being zero,
    /// the term is removed.
    pub(crate) fn combine_mul_terms<F: AcirField>(
        mul_terms: &[(F, Witness, Witness)],
    ) -> Vec<(F, Witness, Witness)> {
        // This is similar to GeneralOptimizer::simplify_mul_terms but it's duplicated because
        // we don't have access to the acvm crate here.
        let mut hash_map = std::collections::HashMap::new();

        // Canonicalize the ordering of the multiplication, lets just order by variable name
        for (scale, w_l, w_r) in mul_terms.iter().copied() {
            let mut pair = [w_l, w_r];
            pair.sort();

            *hash_map.entry((pair[0], pair[1])).or_insert_with(F::zero) += scale;
        }

        hash_map
            .into_iter()
            .filter(|(_, scale)| !scale.is_zero())
            .map(|((w_l, w_r), scale)| (scale, w_l, w_r))
            .collect()
    }
}

/// A wrapper around field division which skips the inversion if the denominator
/// is ±1.
///
/// Field inversion is the most significant cost of solving [`Opcode::AssertZero`][acir::circuit::opcodes::Opcode::AssertZero]
/// opcodes, which we can avoid when the denominator is ±1.
fn quick_invert<F: AcirField>(numerator: F, denominator: F) -> F {
    if denominator == F::one() {
        numerator
    } else if denominator == -F::one() {
        -numerator
    } else {
        assert!(
            denominator != F::zero(),
            "quick_invert: attempting to divide numerator by F::zero()"
        );
        numerator / denominator
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use acir::FieldElement;

    #[test]
    /// Sanity check for the special cases of [`quick_invert`]
    fn quick_invert_matches_slow_invert() {
        let numerator = FieldElement::from_be_bytes_reduce("hello_world".as_bytes());
        assert_eq!(quick_invert(numerator, FieldElement::one()), numerator / FieldElement::one());
        assert_eq!(quick_invert(numerator, -FieldElement::one()), numerator / -FieldElement::one());
    }

    #[test]
    #[should_panic(expected = "quick_invert: attempting to divide numerator by F::zero()")]
    fn quick_invert_zero_denominator() {
        quick_invert(FieldElement::one(), FieldElement::zero());
    }

    #[test]
    fn solves_simple_assignment() {
        let a = Witness(0);

        // a - 1 == 0;
        let opcode_a = Expression::from_str(&format!("{a} - 1")).unwrap();

        let mut values = WitnessMap::new();
        assert_eq!(ExpressionSolver::solve(&mut values, &opcode_a), Ok(()));

        assert_eq!(values.get(&a).unwrap(), &FieldElement::from(1_i128));
    }

    #[test]
    fn solves_unknown_in_mul_term() {
        let a = Witness(0);
        let b = Witness(1);
        let c = Witness(2);
        let d = Witness(3);

        // a * b - b - c - d == 0;
        let opcode_a = Expression::from_str(&format!("{a}*{b} - {b} - {c} - {d}")).unwrap();

        let mut values = WitnessMap::new();
        values.insert(b, FieldElement::from(2_i128));
        values.insert(c, FieldElement::from(1_i128));
        values.insert(d, FieldElement::from(1_i128));

        assert_eq!(ExpressionSolver::solve(&mut values, &opcode_a), Ok(()));

        assert_eq!(values.get(&a).unwrap(), &FieldElement::from(2_i128));
    }

    #[test]
    fn solves_unknown_in_linear_term() {
        let a = Witness(0);
        let b = Witness(1);
        let c = Witness(2);
        let d = Witness(3);

        // a = b + c + d;
        let opcode_a = Expression::from_str(&format!("{a} - {b} - {c} - {d}")).unwrap();

        let e = Witness(4);
        let opcode_b = Expression::from_str(&format!("{e} - {a} - {b}")).unwrap();

        let mut values = WitnessMap::new();
        values.insert(b, FieldElement::from(2_i128));
        values.insert(c, FieldElement::from(1_i128));
        values.insert(d, FieldElement::from(1_i128));

        assert_eq!(ExpressionSolver::solve(&mut values, &opcode_a), Ok(()));
        assert_eq!(ExpressionSolver::solve(&mut values, &opcode_b), Ok(()));

        assert_eq!(values.get(&a).unwrap(), &FieldElement::from(4_i128));
    }

    #[test]
    fn solves_by_combining_linear_terms_after_they_have_been_multiplied_by_known_witnesses() {
        let expr = Expression::from_str("w1 + w1*w0 - 4").unwrap();
        let mut values = WitnessMap::new();
        values.insert(Witness(0), FieldElement::from(1_i128));

        let res = ExpressionSolver::solve(&mut values, &expr);
        assert!(res.is_ok());

        assert_eq!(values.get(&Witness(1)).unwrap(), &FieldElement::from(2_i128));
    }

    #[test]
    fn solves_by_combining_mul_terms() {
        let expr = Expression::from_str("w1*w2 - w2*w1 + w3 - 2").unwrap();
        let mut values = WitnessMap::new();

        let res = ExpressionSolver::solve(&mut values, &expr);
        assert!(res.is_ok());

        assert_eq!(values.get(&Witness(3)).unwrap(), &FieldElement::from(2_i128));
    }
}
