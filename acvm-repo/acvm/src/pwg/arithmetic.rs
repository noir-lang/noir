use acir::{
    native_types::{Expression, Witness, WitnessMap},
    FieldElement,
};

use super::{insert_value, ErrorLocation, OpcodeNotSolvable, OpcodeResolutionError};

/// An Arithmetic solver will take a Circuit's arithmetic opcodes with witness assignments
/// and create the other witness variables
pub(super) struct ArithmeticSolver;

#[allow(clippy::enum_variant_names)]
pub(super) enum OpcodeStatus {
    OpcodeSatisfied(FieldElement),
    OpcodeSolvable(FieldElement, (FieldElement, Witness)),
    OpcodeUnsolvable,
}

pub(crate) enum MulTerm {
    OneUnknown(FieldElement, Witness), // (qM * known_witness, unknown_witness)
    TooManyUnknowns,
    Solved(FieldElement),
}

impl ArithmeticSolver {
    /// Derives the rest of the witness based on the initial low level variables
    pub(super) fn solve(
        initial_witness: &mut WitnessMap,
        opcode: &Expression,
    ) -> Result<(), OpcodeResolutionError> {
        let opcode = &ArithmeticSolver::evaluate(opcode, initial_witness);
        // Evaluate multiplication term
        let mul_result = ArithmeticSolver::solve_mul_term(opcode, initial_witness);
        // Evaluate the fan-in terms
        let opcode_status = ArithmeticSolver::solve_fan_in_term(opcode, initial_witness);

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
                            })
                        } else {
                            Ok(())
                        }
                    } else {
                        let assignment = -total_sum / (q + b);
                        // Add this into the witness assignments
                        insert_value(&w1, assignment, initial_witness)?;
                        Ok(())
                    }
                } else {
                    // TODO: can we be more specific with this error?
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
                        })
                    } else {
                        Ok(())
                    }
                } else {
                    let assignment = -(total_sum / partial_prod);
                    // Add this into the witness assignments
                    insert_value(&unknown_var, assignment, initial_witness)?;
                    Ok(())
                }
            }
            (MulTerm::Solved(a), OpcodeStatus::OpcodeSatisfied(b)) => {
                // All the variables in the MulTerm are solved and the Fan-in is also solved
                // There is nothing to solve
                if !(a + b + opcode.q_c).is_zero() {
                    Err(OpcodeResolutionError::UnsatisfiedConstrain {
                        opcode_location: ErrorLocation::Unresolved,
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
                        })
                    } else {
                        Ok(())
                    }
                } else {
                    let assignment = -(total_sum / coeff);
                    // Add this into the witness assignments
                    insert_value(&unknown_var, assignment, initial_witness)?;
                    Ok(())
                }
            }
        }
    }

    /// Returns the evaluation of the multiplication term in the arithmetic opcode
    /// If the witness values are not known, then the function returns a None
    /// XXX: Do we need to account for the case where 5xy + 6x = 0 ? We do not know y, but it can be solved given x . But I believe x can be solved with another opcode
    /// XXX: What about making a mul opcode = a constant 5xy + 7 = 0 ? This is the same as the above.
    fn solve_mul_term(arith_opcode: &Expression, witness_assignments: &WitnessMap) -> MulTerm {
        // First note that the mul term can only contain one/zero term
        // We are assuming it has been optimized.
        match arith_opcode.mul_terms.len() {
            0 => MulTerm::Solved(FieldElement::zero()),
            1 => ArithmeticSolver::solve_mul_term_helper(
                &arith_opcode.mul_terms[0],
                witness_assignments,
            ),
            _ => panic!("Mul term in the arithmetic opcode must contain either zero or one term"),
        }
    }

    fn solve_mul_term_helper(
        term: &(FieldElement, Witness, Witness),
        witness_assignments: &WitnessMap,
    ) -> MulTerm {
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

    fn solve_fan_in_term_helper(
        term: &(FieldElement, Witness),
        witness_assignments: &WitnessMap,
    ) -> Option<FieldElement> {
        let (q_l, w_l) = term;
        // Check if we have w_l
        let w_l_value = witness_assignments.get(w_l);
        w_l_value.map(|a| *q_l * *a)
    }

    /// Returns the summation of all of the variables, plus the unknown variable
    /// Returns None, if there is more than one unknown variable
    /// We cannot assign
    pub(super) fn solve_fan_in_term(
        arith_opcode: &Expression,
        witness_assignments: &WitnessMap,
    ) -> OpcodeStatus {
        // This is assuming that the fan-in is more than 0

        // This is the variable that we want to assign the value to
        let mut unknown_variable = (FieldElement::zero(), Witness::default());
        let mut num_unknowns = 0;
        // This is the sum of all of the known variables
        let mut result = FieldElement::zero();

        for term in arith_opcode.linear_combinations.iter() {
            let value = ArithmeticSolver::solve_fan_in_term_helper(term, witness_assignments);
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
    pub(super) fn evaluate(expr: &Expression, initial_witness: &WitnessMap) -> Expression {
        let mut result = Expression::default();
        for &(c, w1, w2) in &expr.mul_terms {
            let mul_result = ArithmeticSolver::solve_mul_term_helper(&(c, w1, w2), initial_witness);
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
            if let Some(f) = ArithmeticSolver::solve_fan_in_term_helper(&(c, w), initial_witness) {
                result.q_c += f;
            } else if !c.is_zero() {
                result.linear_combinations.push((c, w));
            }
        }
        result.q_c += expr.q_c;
        result
    }
}

#[test]
fn arithmetic_smoke_test() {
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

    let e = Witness(4);
    let opcode_b = Expression {
        mul_terms: vec![],
        linear_combinations: vec![
            (FieldElement::one(), e),
            (-FieldElement::one(), a),
            (-FieldElement::one(), b),
        ],
        q_c: FieldElement::zero(),
    };

    let mut values = WitnessMap::new();
    values.insert(b, FieldElement::from(2_i128));
    values.insert(c, FieldElement::from(1_i128));
    values.insert(d, FieldElement::from(1_i128));

    assert_eq!(ArithmeticSolver::solve(&mut values, &opcode_a), Ok(()));
    assert_eq!(ArithmeticSolver::solve(&mut values, &opcode_b), Ok(()));

    assert_eq!(values.get(&a).unwrap(), &FieldElement::from(4_i128));
}
