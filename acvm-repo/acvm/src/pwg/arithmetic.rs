use std::collections::HashMap;

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
    /// Derives the rest of the witness in the provided expression based on the known witness values
    /// 1. First we simplify the expression based on the known values and try to reduce the multiplication and linear terms
    /// 2. If we end up with only the constant term;
    ///     - if it is 0 then the opcode is solved, if not,
    ///     - the assert_zero opcode is not satisfied and we return an error
    /// 3. If we end up with only linear terms on the same witness 'w',
    ///    we can regroup them and solve 'a*w+c = 0':
    ///    - If 'a' is zero in the above expression;
    ///      - if c is also 0 then the opcode is solved
    ///      - if not that means the assert_zero opcode is not satisfied and we return an error
    ///    - If 'a' is not zero, we can solve it by setting the value of w: 'w = -c/a'
    pub(crate) fn solve<F: AcirField>(
        initial_witness: &mut WitnessMap<F>,
        opcode: &Expression<F>,
    ) -> Result<(), OpcodeResolutionError<F>> {
        let opcode = &ExpressionSolver::evaluate(opcode, initial_witness);
        // Evaluate multiplication term
        let mul_result =
            ExpressionSolver::solve_mul_term(opcode, initial_witness).map_err(|_| {
                OpcodeResolutionError::OpcodeNotSolvable(
                    OpcodeNotSolvable::ExpressionHasTooManyUnknowns(opcode.clone()),
                )
            })?;
        // Evaluate the fan-in terms
        let opcode_status = ExpressionSolver::solve_fan_in_term(opcode, initial_witness);

        match (mul_result, opcode_status) {
            (MulTerm::TooManyUnknowns, _) | (_, OpcodeStatus::OpcodeUnsolvable) => {
                Err(OpcodeResolutionError::OpcodeNotSolvable(
                    OpcodeNotSolvable::ExpressionHasTooManyUnknowns(opcode.clone()),
                ))
            }
            (MulTerm::OneUnknown(q, w1), OpcodeStatus::OpcodeSolvable(a, (b, w2))) => {
                if w1 == w2 {
                    // We have one unknown so we can solve the equation
                    let total_sum = a + opcode.q_c;
                    if (q + b).is_zero() {
                        if !total_sum.is_zero() {
                            Err(OpcodeResolutionError::UnsatisfiedConstrain {
                                opcode_location: ErrorLocation::Unresolved,
                                payload: None,
                            })
                        } else {
                            Ok(())
                        }
                    } else {
                        let assignment = -quick_invert(total_sum, q + b);
                        insert_value(&w1, assignment, initial_witness)
                    }
                } else {
                    // TODO(https://github.com/noir-lang/noir/issues/10191): can we be more specific with this error?
                    Err(OpcodeResolutionError::OpcodeNotSolvable(
                        OpcodeNotSolvable::ExpressionHasTooManyUnknowns(opcode.clone()),
                    ))
                }
            }
            (
                MulTerm::OneUnknown(partial_prod, unknown_var),
                OpcodeStatus::OpcodeSatisfied(sum),
            ) => {
                // We have one unknown in the mul term and the fan-in terms are solved.
                // Hence the equation is solvable, since there is a single unknown
                // The equation is: partial_prod * unknown_var + sum + qC = 0

                let total_sum = sum + opcode.q_c;
                if partial_prod.is_zero() {
                    if !total_sum.is_zero() {
                        Err(OpcodeResolutionError::UnsatisfiedConstrain {
                            opcode_location: ErrorLocation::Unresolved,
                            payload: None,
                        })
                    } else {
                        Ok(())
                    }
                } else {
                    let assignment = -quick_invert(total_sum, partial_prod);
                    insert_value(&unknown_var, assignment, initial_witness)
                }
            }
            (MulTerm::Solved(a), OpcodeStatus::OpcodeSatisfied(b)) => {
                // All the variables in the MulTerm are solved and the Fan-in is also solved
                // There is nothing to solve
                if !(a + b + opcode.q_c).is_zero() {
                    Err(OpcodeResolutionError::UnsatisfiedConstrain {
                        opcode_location: ErrorLocation::Unresolved,
                        payload: None,
                    })
                } else {
                    Ok(())
                }
            }
            (
                MulTerm::Solved(total_prod),
                OpcodeStatus::OpcodeSolvable(partial_sum, (coeff, unknown_var)),
            ) => {
                // The variables in the MulTerm are solved nad there is one unknown in the Fan-in
                // Hence the equation is solvable, since we have one unknown
                // The equation is total_prod + partial_sum + coeff * unknown_var + q_C = 0
                let total_sum = total_prod + partial_sum + opcode.q_c;
                if coeff.is_zero() {
                    if !total_sum.is_zero() {
                        Err(OpcodeResolutionError::UnsatisfiedConstrain {
                            opcode_location: ErrorLocation::Unresolved,
                            payload: None,
                        })
                    } else {
                        Ok(())
                    }
                } else {
                    let assignment = -quick_invert(total_sum, coeff);
                    insert_value(&unknown_var, assignment, initial_witness)
                }
            }
        }
    }

    /// Try to reduce the multiplication terms of the given expression to a known value or to a linear term,
    /// using the provided witness mapping.
    /// If there are 2 or more multiplication terms it returns the OpcodeUnsolvable error.
    /// If no witnesses value is in the provided 'witness_assignments' map,
    /// it returns MulTerm::TooManyUnknowns
    fn solve_mul_term<F: AcirField>(
        arith_opcode: &Expression<F>,
        witness_assignments: &WitnessMap<F>,
    ) -> Result<MulTerm<F>, OpcodeStatus<F>> {
        // First note that the mul term can only contain one/zero term,
        // e.g. that it has been optimized, or else we're returning OpcodeUnsolvable
        match arith_opcode.mul_terms.len() {
            0 => Ok(MulTerm::Solved(F::zero())),
            1 => Ok(ExpressionSolver::solve_mul_term_helper(
                &arith_opcode.mul_terms[0],
                witness_assignments,
            )),
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
        arith_opcode: &Expression<F>,
        witness_assignments: &WitnessMap<F>,
    ) -> OpcodeStatus<F> {
        // If the fan-in has more than 0 num_unknowns:

        // This is the variable that we want to assign the value to
        let mut unknown_variable = (F::zero(), Witness::default());
        let mut num_unknowns = 0;
        // This is the sum of all of the known variables
        let mut result = F::zero();

        for term in arith_opcode.linear_combinations.iter() {
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
        let mut linear_combinations = HashMap::new();

        for &(c, w1, w2) in &expr.mul_terms {
            let mul_result = ExpressionSolver::solve_mul_term_helper(&(c, w1, w2), initial_witness);
            match mul_result {
                MulTerm::OneUnknown(v, w) => {
                    if !v.is_zero() {
                        linear_combinations.entry(w).and_modify(|value| *value += v).or_insert(c);
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
                linear_combinations.entry(w).and_modify(|value| *value += c).or_insert(c);
            }
        }

        result.linear_combinations = linear_combinations
            .into_iter()
            .filter_map(|(w, c)| if c.is_zero() { None } else { Some((c, w)) })
            .collect();

        result.q_c += expr.q_c;
        result
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
}
