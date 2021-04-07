use acir::native_types::{Arithmetic, Witness};
use noir_field::FieldElement;
use std::collections::BTreeMap;

/// An Arithmetic solver will take a Circuit's arithmetic gates with witness assignments
/// and create the other witness variables
pub struct ArithmeticSolver;

#[allow(clippy::enum_variant_names)]
enum GateStatus {
    GateSatisfied(FieldElement),
    GateSolvable(FieldElement, (FieldElement, Witness)),
    GateUnsolvable,
}

enum MulTerm {
    OneUnknown(FieldElement, Witness), // (qM * known_witness, unknown_witness)
    TooManyUnknowns,
    Solved(FieldElement),
}

impl ArithmeticSolver {
    /// Derives the rest of the witness based on the initial low level variables
    pub fn solve<'a>(
        initial_witness: &mut BTreeMap<Witness, FieldElement>,
        gate: &'a Arithmetic,
    ) -> Option<&'a Arithmetic> {
        // Evaluate multiplication term
        let mul_result = ArithmeticSolver::solve_mul_term(&gate, &initial_witness);
        // Evaluate the fan-in terms
        let gate_status = ArithmeticSolver::solve_fan_in_term(&gate, &initial_witness);

        match (mul_result, gate_status) {
            (MulTerm::TooManyUnknowns, _) => Some(gate),
            (_, GateStatus::GateUnsolvable) => Some(gate),
            (MulTerm::OneUnknown(_, _), GateStatus::GateSolvable(_, _)) => Some(gate),
            (MulTerm::OneUnknown(partial_prod, unknown_var), GateStatus::GateSatisfied(sum)) => {
                // We have one unknown in the mul term and the fan-in terms are solved.
                // Hence the equation is solvable, since there is a single unknown
                // The equation is: partial_prod * unknown_var + sum + qC = 0

                let total_sum = sum + gate.q_c;
                let assignment = -(total_sum / partial_prod);
                // Add this into the witness assignments
                initial_witness.insert(unknown_var, assignment);
                None
            }
            (MulTerm::Solved(_), GateStatus::GateSatisfied(_)) => {
                // All the variables in the MulTerm are solved and the Fan-in is also solved
                // There is nothing to solve
                None
            }
            (
                MulTerm::Solved(total_prod),
                GateStatus::GateSolvable(partial_sum, (coeff, unknown_var)),
            ) => {
                // The variables in the MulTerm are solved nad there is one unknown in the Fan-in
                // Hence the equation is solvable, since we have one unknown
                // The equation is total_prod + partial_sum + coeff * unknown_var + q_C = 0

                let total_sum = total_prod + partial_sum + gate.q_c;
                let assignment = -(total_sum / coeff);
                // Add this into the witness assignments
                initial_witness.insert(unknown_var, assignment);
                None
            }
        }
    }

    /// Returns the evaluation of the multiplication term in the arithmetic gate
    /// If the witness values are not known, then the function returns a None
    /// XXX: Do we need to account for the case where 5xy + 6x = 0 ? We do not know y, but it can be solved given x . But I believe x can be solved with another gate
    /// XXX: What about making a mul gate = a constant 5xy + 7 = 0 ? This is the same as the above.
    fn solve_mul_term(
        arith_gate: &Arithmetic,
        witness_assignments: &BTreeMap<Witness, FieldElement>,
    ) -> MulTerm {
        // First note that the mul term can only contain one/zero term
        // We are assuming it has been optimised.
        match arith_gate.mul_terms.len() {
            0 => MulTerm::Solved(FieldElement::zero()),
            1 => {
                let q_m = &arith_gate.mul_terms[0].0;
                let w_l = &arith_gate.mul_terms[0].1;
                let w_r = &arith_gate.mul_terms[0].2;

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
            _ => panic!("Mul term in the arithmetic gate must contain either zero or one term"),
        }
    }

    /// Returns the summation of all of the variables, plus the unknown variable
    /// Returns None, if there is more than one unknown variable
    /// We cannot assign
    fn solve_fan_in_term(
        arith_gate: &Arithmetic,
        witness_assignments: &BTreeMap<Witness, FieldElement>,
    ) -> GateStatus {
        // This is assuming that the fan-in is more than 0

        // This is the variable that we want to assign the value to
        let mut unknown_variable = (FieldElement::zero(), Witness::default());
        let mut num_unknowns = 0;
        // This is the sum of all of the known variables
        let mut result = FieldElement::zero();

        for term in arith_gate.linear_combinations.iter() {
            let q_l = term.0;
            let w_l = &term.1;

            // Check if we have w_l
            let w_l_value = witness_assignments.get(w_l);

            match w_l_value {
                Some(a) => result += q_l * *a,
                None => {
                    unknown_variable = *term;
                    num_unknowns += 1;
                }
            };

            // If we have more than 1 unknown, then we cannot solve this equation
            if num_unknowns > 1 {
                return GateStatus::GateUnsolvable;
            }
        }

        if num_unknowns == 0 {
            return GateStatus::GateSatisfied(result);
        }

        GateStatus::GateSolvable(result, unknown_variable)
    }
}

#[test]
fn arithmetic_smoke_test() {
    let a = Witness(0);
    let b = Witness(1);
    let c = Witness(2);
    let d = Witness(3);

    // a = b + c + d;
    let gate_a = Arithmetic {
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
    let gate_b = Arithmetic {
        mul_terms: vec![],
        linear_combinations: vec![
            (FieldElement::one(), e),
            (-FieldElement::one(), a),
            (-FieldElement::one(), b),
        ],
        q_c: FieldElement::zero(),
    };

    let mut values: BTreeMap<Witness, FieldElement> = BTreeMap::new();
    values.insert(b, FieldElement::from(2));
    values.insert(c, FieldElement::from(1));
    values.insert(d, FieldElement::from(1));

    assert!(ArithmeticSolver::solve(&mut values, &gate_a).is_none());
    assert!(ArithmeticSolver::solve(&mut values, &gate_b).is_none());

    assert_eq!(values.get(&a).unwrap(), &FieldElement::from(4));
}
