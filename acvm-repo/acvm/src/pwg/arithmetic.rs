use super::{ErrorLocation, OpcodeNotSolvable, OpcodeResolutionError, insert_value};
use acir::{
    AcirField,
    native_types::{Expression, Witness, WitnessMap},
};
/// An Expression solver will take a Circuit's assert-zero opcodes with witness assignments
/// and create the other witness variables
pub(crate) struct ExpressionSolver;

#[derive(Clone, Debug)]
pub(crate) struct Pending_Arithmetic_Opcodes<F: AcirField> {
    pending_witness_write: Vec<PendingOp<F>>,
    failures: u32,
}

impl<F: AcirField> Pending_Arithmetic_Opcodes<F> {
    pub(crate) fn new() -> Self {
        Self { pending_witness_write: vec![], failures: 0 }
    }

    pub(crate) fn add_pending_op(
        &mut self,
        neumerator: F,
        denominator: F,
        witness: Witness,
    ) -> Result<(), OpcodeResolutionError<F>> {
        // note that there might be multiple witness assignments in this list
        // however when pending_ops is called, this would cause an error
        self.pending_witness_write.push(PendingOp { neumerator, denominator, witness });
        Ok(())
    }

    pub(crate) fn write_pending_ops(
        &mut self,
        initial_witness: &mut WitnessMap<F>,
    ) -> Result<(), OpcodeResolutionError<F>> {
        // get the list of denominator
        let mut denominator_list: Vec<F> = self
            .pending_witness_write
            .clone()
            .into_iter()
            .map(|pending_op| pending_op.denominator)
            .collect();
        batch_invert::<F>(&mut denominator_list);
        for i in 0..denominator_list.len() {
            let pending_op = &self.pending_witness_write[i];
            let assignment = pending_op.neumerator * denominator_list[i];
            let res = insert_value(&pending_op.witness, assignment, initial_witness);
            if res.is_err() {
                return Err(OpcodeResolutionError::UnsatisfiedConstrain {
                    opcode_location: ErrorLocation::Unresolved,
                    payload: None,
                });
            }
        }
        self.pending_witness_write.clear();
        self.failures = 0;
        Ok(())
    }
}

// this is the same function as in arkworks
pub(crate) fn batch_invert<F: AcirField>(v: &mut Vec<F>) {
    // First pass: compute [a, ab, abc, ...]
    // we're never adding elements that are zero to this list
    let mut prod = Vec::with_capacity(v.len());
    let mut tmp = F::one();
    for f in v.iter() {
        tmp = tmp * *f;
        prod.push(tmp);
    }

    // Invert `tmp`.
    tmp = tmp.inverse(); // Guaranteed to be nonzero.

    // Second pass: iterate backwards to compute inverses
    for (f, s) in v
        .iter_mut()
        // Backwards
        .rev()
        // Ignore normalized elements
        .filter(|f| !f.is_zero())
        // Backwards, skip last element, fill in one for last term.
        .zip(prod.into_iter().rev().skip(1).chain(Some(F::one())))
    {
        // tmp := tmp * f; f := tmp * s = 1/f
        let new_tmp = tmp * *f;
        *f = tmp * s;
        tmp = new_tmp;
    }
}

#[derive(Clone, Debug)]
pub(crate) struct PendingOp<F: AcirField> {
    neumerator: F,
    denominator: F,
    witness: Witness,
}

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
    /// the solver method along side the the optimization with the pending arithmetic opcodes
    /// Derives the rest of the witness based on the initial low level variables
    pub(crate) fn solve_optimized<F: AcirField>(
        initial_witness: &mut WitnessMap<F>,
        opcode: &Expression<F>,
        pending_arithmetic_opcodes: &mut Pending_Arithmetic_Opcodes<F>,
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
                // if we have too many unknowns, and there are no pending ops, then we can return an error
                if pending_arithmetic_opcodes.pending_witness_write.is_empty() {
                    return Err(OpcodeResolutionError::OpcodeNotSolvable(
                        OpcodeNotSolvable::ExpressionHasTooManyUnknowns(opcode.clone()),
                    ));
                }
                // there might be too many unknowns that are in the pending witness list that is not written down yet.
                // so we write down the pending witness lists and solve again
                // pending_arithmetic_opcodes.failures += 1;
                let write_output = pending_arithmetic_opcodes.write_pending_ops(initial_witness);
                write_output.map_err(|_| {
                    OpcodeResolutionError::OpcodeNotSolvable(
                        OpcodeNotSolvable::ExpressionHasTooManyUnknowns(opcode.clone()),
                    )
                })?;
                // no we have to solve again to see if the opcode is solvable
                ExpressionSolver::solve_optimized(
                    initial_witness,
                    opcode,
                    pending_arithmetic_opcodes,
                )
            }

            (MulTerm::OneUnknown(q, w1), OpcodeStatus::OpcodeSolvable(a, (b, w2))) => {
                if w1 == w2 {
                    // We have one unknown so we can solve the equation
                    let total_sum = a + opcode.q_c;
                    match q + b {
                        x if x.is_zero() => {
                            if !total_sum.is_zero() {
                                Err(OpcodeResolutionError::UnsatisfiedConstrain {
                                    opcode_location: ErrorLocation::Unresolved,
                                    payload: None,
                                })
                            } else {
                                Ok(())
                            }
                        }
                        x if x == F::one() => insert_value(&w1, total_sum, initial_witness),
                        x if x == -F::one() => insert_value(&w1, -total_sum, initial_witness),
                        x => {
                            // normally we would do
                            // let assignment = -total_sum / (q + b);
                            // insert_value(&w1, assignment, initial_witness)
                            // but we want to add this to pending_arithmetic_opcodes
                            pending_arithmetic_opcodes.add_pending_op(-total_sum, x, w1)
                        }
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

                match partial_prod {
                    x if x.is_zero() => {
                        if !total_sum.is_zero() {
                            Err(OpcodeResolutionError::UnsatisfiedConstrain {
                                opcode_location: ErrorLocation::Unresolved,
                                payload: None,
                            })
                        } else {
                            Ok(())
                        }
                    }
                    x if x == F::one() => insert_value(&unknown_var, -total_sum, initial_witness),
                    x if x == -F::one() => insert_value(&unknown_var, total_sum, initial_witness),
                    _ => {
                        // normally we would do
                        // let assignment = -(total_sum / partial_prod);
                        // insert_value(&unknown_var, assignment, initial_witness)
                        // but we want to add this to pending_arithmetic_opcodes
                        pending_arithmetic_opcodes.add_pending_op(
                            -total_sum,
                            partial_prod,
                            unknown_var,
                        )
                    }
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
                // The variables in the MulTerm are solved and there is one unknown in the Fan-in
                // Hence the equation is solvable, since we have one unknown
                // The equation is total_prod + partial_sum + coeff * unknown_var + q_C = 0
                let total_sum = total_prod + partial_sum + opcode.q_c;
                match coeff {
                    x if x.is_zero() => {
                        if !total_sum.is_zero() {
                            Err(OpcodeResolutionError::UnsatisfiedConstrain {
                                opcode_location: ErrorLocation::Unresolved,
                                payload: None,
                            })
                        } else {
                            Ok(())
                        }
                    }
                    x if x == F::one() => insert_value(&unknown_var, -total_sum, initial_witness),
                    x if x == -F::one() => insert_value(&unknown_var, total_sum, initial_witness),
                    _ => {
                        // normally we would do
                        // let assignment = -(total_sum / coeff);
                        // insert_value(&unknown_var, assignment, initial_witness)
                        // but we want to add this to pending_arithmetic_opcodes
                        pending_arithmetic_opcodes.add_pending_op(-total_sum, coeff, unknown_var)
                    }
                }
            }
        }
    }

    /// Derives the rest of the witness based on the initial low level variables
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

    /// Returns the evaluation of the multiplication term in the expression
    /// If the witness values are not known, then the function returns a None
    /// XXX: Do we need to account for the case where 5xy + 6x = 0 ? We do not know y, but it can be solved given x . But I believe x can be solved with another opcode
    /// XXX: What about making a mul opcode = a constant 5xy + 7 = 0 ? This is the same as the above.
    fn solve_mul_term<F: AcirField>(
        arith_opcode: &Expression<F>,
        witness_assignments: &WitnessMap<F>,
    ) -> Result<MulTerm<F>, OpcodeStatus<F>> {
        // First note that the mul term can only contain one/zero term
        // We are assuming it has been optimized.
        match arith_opcode.mul_terms.len() {
            0 => Ok(MulTerm::Solved(F::zero())),
            1 => Ok(ExpressionSolver::solve_mul_term_helper(
                &arith_opcode.mul_terms[0],
                witness_assignments,
            )),
            _ => Err(OpcodeStatus::OpcodeUnsolvable),
        }
    }

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
    /// Returns None, if there is more than one unknown variable
    /// We cannot assign
    pub(super) fn solve_fan_in_term<F: AcirField>(
        arith_opcode: &Expression<F>,
        witness_assignments: &WitnessMap<F>,
    ) -> OpcodeStatus<F> {
        // This is assuming that the fan-in is more than 0

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
}

/// A wrapper around field division which skips the inversion if the denominator
/// is ±1.
///
/// Field inversion is the most significant cost of solving [`Opcode::AssertZero`][acir::circuit::opcodes::Opcode::AssertZero]
/// opcodes, we can avoid this in the situation
fn quick_invert<F: AcirField>(numerator: F, denominator: F) -> F {
    if denominator == F::one() {
        numerator
    } else if denominator == -F::one() {
        -numerator
    } else {
        numerator / denominator
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Mul;

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
    fn solves_simple_assignment() {
        let a = Witness(0);

        // a - 1 == 0;
        let opcode_a = Expression {
            mul_terms: vec![],
            linear_combinations: vec![(FieldElement::one(), a)],
            q_c: -FieldElement::one(),
        };

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
        let opcode_a = Expression {
            mul_terms: vec![(FieldElement::one(), a, b)],
            linear_combinations: vec![
                (-FieldElement::one(), b),
                (-FieldElement::one(), c),
                (-FieldElement::one(), d),
            ],
            q_c: FieldElement::zero(),
        };

        let mut values = WitnessMap::new();
        values.insert(b, FieldElement::from(2_i128));
        values.insert(c, FieldElement::from(1_i128));
        values.insert(d, FieldElement::from(1_i128));

        assert_eq!(ExpressionSolver::solve(&mut values, &opcode_a), Ok(()));

        assert_eq!(values.get(&a).unwrap(), &FieldElement::from(2_i128));
    }

    #[test]
    fn test_batch_invert() {
        let mut v = vec![
            FieldElement::from(1_i128),
            FieldElement::from(2_i128),
            FieldElement::from(3_i128),
        ];
        batch_invert::<FieldElement>(&mut v);
        // assert_eq!(v, vec![FieldElement::from(1_i128), FieldElement::from(2_i128), FieldElement::from(3_i128)]);
        assert_eq!(
            [
                v[0] * FieldElement::from(1_i128),
                v[1] * FieldElement::from(2_i128),
                v[2] * FieldElement::from(3_i128)
            ],
            [FieldElement::from(1_i128), FieldElement::from(1_i128), FieldElement::from(1_i128)]
        );
    }

    #[test]
    fn solves_unknown_in_linear_term() {
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

        assert_eq!(ExpressionSolver::solve(&mut values, &opcode_a), Ok(()));
        assert_eq!(ExpressionSolver::solve(&mut values, &opcode_b), Ok(()));

        assert_eq!(values.get(&a).unwrap(), &FieldElement::from(4_i128));
    }

    #[test]
    fn test_pending_ops() {
        let mut pending_ops: Pending_Arithmetic_Opcodes<FieldElement> =
            Pending_Arithmetic_Opcodes::new();
        let a = Witness(0);
        let opcode_a = Expression {
            mul_terms: vec![],
            linear_combinations: vec![(FieldElement::one() + FieldElement::one(), a)],
            q_c: -FieldElement::one(),
        };
        let mut initial_witness: WitnessMap<FieldElement> = WitnessMap::new();
        let _ =
            ExpressionSolver::solve_optimized(&mut initial_witness, &opcode_a, &mut pending_ops);
        println!("pending_ops: {:?}", pending_ops);
    }
    #[test]
    fn test_pending_ops_2() {
        let mut pending_ops: Pending_Arithmetic_Opcodes<FieldElement> =
            Pending_Arithmetic_Opcodes::new();
        let a = Witness(0);
        let opcode_a = Expression {
            mul_terms: vec![],
            linear_combinations: vec![(FieldElement::one() + FieldElement::one(), a)],
            q_c: -FieldElement::one(),
        };
        let b: Witness = Witness(1);
        let opcode_b = Expression {
            mul_terms: vec![],
            linear_combinations: vec![
                (FieldElement::one() + FieldElement::one(), a),
                (FieldElement::one(), b),
            ],
            q_c: -FieldElement::one(),
        };
        let mut initial_witness: WitnessMap<FieldElement> = WitnessMap::new();
        let _ =
            ExpressionSolver::solve_optimized(&mut initial_witness, &opcode_a, &mut pending_ops);
        println!("pending_ops: {:?}", pending_ops);
        println!("initial_witness: {:?}", initial_witness);
        let _ =
            ExpressionSolver::solve_optimized(&mut initial_witness, &opcode_b, &mut pending_ops);
        println!("pending_ops: {:?}", pending_ops);
        println!("initial_witness: {:?}", initial_witness);
    }

    #[test]
    fn test_pending_ops_batching_linear_combinations() {
        // we want the following scenario
        // w0 = 2
        // 9w1 = 3 + w0
        // w1 1/9 should be added to pending_ops,
        // 5w2 = 3 * w0
        // w2 1/5 should be added to the pending ops
        // w3 = 4 + w1
        // a failure should happen so w1 and w2 should be written
        // w4 = 5 * w2
        // there's no unknowns here so the pending ops should be empty
        let mut pending_ops: Pending_Arithmetic_Opcodes<FieldElement> =
            Pending_Arithmetic_Opcodes::new();
        let w0 = Witness(0);
        let w1 = Witness(1);
        let w2 = Witness(2);
        let w3 = Witness(3);
        let w4 = Witness(4);
        // opcode0 : w0 - 15 = 0
        // w0 = 15
        let opcode0 = Expression {
            mul_terms: vec![],
            linear_combinations: vec![(FieldElement::one(), w0)],
            q_c: -FieldElement::from(15_i128),
        };
        // opcode1 : 9w1 - 3 - w0 = 0
        // w1 = 2
        let opcode1 = Expression {
            mul_terms: vec![],
            linear_combinations: vec![(FieldElement::from(9_i128), w1), (-FieldElement::one(), w0)],
            q_c: -FieldElement::from(3_i128),
        };
        // opcode2 : 5w2 - 3 * w0 = 0
        // w2 = 9
        let opcode2 = Expression {
            mul_terms: vec![],
            linear_combinations: vec![
                (FieldElement::from(5_i128), w2),
                (-FieldElement::from(3_i128), w0),
            ],
            q_c: FieldElement::zero(),
        };
        // opcode3 : w3 - 4 - w1 = 0
        // w3 = 6
        let opcode3 = Expression {
            mul_terms: vec![],
            linear_combinations: vec![(FieldElement::one(), w3), (-FieldElement::one(), w1)],
            q_c: -FieldElement::from(4_i128),
        };
        // opcode4:  w4 - 5 * w2 = 0
        // w4 = 45
        let opcode4 = Expression {
            mul_terms: vec![],
            linear_combinations: vec![(FieldElement::one(), w4), (-FieldElement::from(5_i128), w2)],
            q_c: FieldElement::zero(),
        };
        let mut initial_witness: WitnessMap<FieldElement> = WitnessMap::new();
        let _ = ExpressionSolver::solve_optimized(&mut initial_witness, &opcode0, &mut pending_ops);
        let _ = ExpressionSolver::solve_optimized(&mut initial_witness, &opcode1, &mut pending_ops);
        let _ = ExpressionSolver::solve_optimized(&mut initial_witness, &opcode2, &mut pending_ops);
        let _ = ExpressionSolver::solve_optimized(&mut initial_witness, &opcode3, &mut pending_ops);
        let _ = ExpressionSolver::solve_optimized(&mut initial_witness, &opcode4, &mut pending_ops);

        assert_eq!(initial_witness.get(&w0).unwrap(), &FieldElement::from(15_i128));
        assert_eq!(initial_witness.get(&w1).unwrap(), &FieldElement::from(2_i128));
        assert_eq!(initial_witness.get(&w2).unwrap(), &FieldElement::from(9_i128));
        assert_eq!(initial_witness.get(&w3).unwrap(), &FieldElement::from(6_i128));
        assert_eq!(initial_witness.get(&w4).unwrap(), &FieldElement::from(45_i128));
    }
    #[test]
    fn test_pending_ops_batching_multiplication_terms() {
        // 5 * w0 = 15 => w0 = 3 // opcode0 = 5 * w0 - 15 = 0
        // 3 * w1 = 12 => w1 = 4 // opcode1 = 3 * w1 - 12 = 0
        // w2 * w0 = 15 => w2 = 5 // opcode2 = w2 * w0 - 15 = 0
        // w3 + 2 = w2  => true // opcode3 = w3 + 2 - w2 = 0
        //
        let mut pending_ops: Pending_Arithmetic_Opcodes<FieldElement> =
            Pending_Arithmetic_Opcodes::new();
        let w0 = Witness(0);
        let w1 = Witness(1);
        let w2 = Witness(2);

        // opcode0 : 5 * w0 - 15 = 0
        let opcode0 = Expression {
            mul_terms: vec![],
            linear_combinations: vec![(FieldElement::from(5_i128), w0)],
            q_c: -FieldElement::from(15_i128),
        };
        // opcode1 = 3 * w1 - 12 = 0
        let opcode1 = Expression {
            mul_terms: vec![],
            linear_combinations: vec![(FieldElement::from(3_i128), w1)],
            q_c: -FieldElement::from(12_i128),
        };
        // opcode2 = w2 * w0 - 15 = 0
        let opcode2 = Expression {
            mul_terms: vec![(FieldElement::one(), w0, w2)],
            linear_combinations: vec![],
            q_c: -FieldElement::from(15_i128),
        };
        // opcode3 = w0 + 2 - w2 = 0
        let opcode3 = Expression {
            mul_terms: vec![],
            linear_combinations: vec![(FieldElement::one(), w0), (-FieldElement::one(), w2)],
            q_c: FieldElement::from(2_i128),
        };
        // set up an empty witness map
        let mut initial_witness: WitnessMap<FieldElement> = WitnessMap::new();
        // now we run the opcodes
        let _ = ExpressionSolver::solve_optimized(&mut initial_witness, &opcode0, &mut pending_ops);
        let _ = ExpressionSolver::solve_optimized(&mut initial_witness, &opcode1, &mut pending_ops);
        let _ = ExpressionSolver::solve_optimized(&mut initial_witness, &opcode2, &mut pending_ops);
        let _ = ExpressionSolver::solve_optimized(&mut initial_witness, &opcode3, &mut pending_ops);

        // empty the pending writes
        let _ = pending_ops.write_pending_ops(&mut initial_witness);
        assert_eq!(initial_witness.get(&w0).unwrap(), &FieldElement::from(3_i128));
        assert_eq!(initial_witness.get(&w1).unwrap(), &FieldElement::from(4_i128));
        assert_eq!(initial_witness.get(&w2).unwrap(), &FieldElement::from(5_i128));
    }
}
