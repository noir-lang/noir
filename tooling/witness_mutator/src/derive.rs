//! Solving one witness out of the circuit's own constraints.
//!
//! The values that break a hint are usually not near the honest value and not near a round number:
//! they are whatever the constraints force once some other value moves. So instead of guessing both
//! halves of a hint's output, the mutator pins one output and asks an `AssertZero` opcode for the
//! other, exactly as the solver would.

use acir::{
    AcirField, FieldElement,
    circuit::{Circuit, Opcode},
    native_types::{Expression, Witness},
};
use std::collections::BTreeMap;

/// Values of every witness except the one being solved for.
pub type Known = BTreeMap<Witness, FieldElement>;

/// Solve `target` from the first `AssertZero` that determines it linearly.
///
/// Returns `None` when no such opcode exists: `target` may appear only in quadratic position,
/// alongside other unknowns, or with a zero coefficient once the other values are substituted.
///
/// The substituted values are the honest ones for witnesses the override does not touch. That is a
/// guess for anything downstream of the overridden call, which is why a derived value is only a
/// candidate: it is accepted or rejected by re-solving the whole circuit afterwards.
pub fn derive_from_constraints(
    circuit: &Circuit<FieldElement>,
    known: &Known,
    target: Witness,
) -> Option<FieldElement> {
    circuit.opcodes.iter().find_map(|opcode| match opcode {
        Opcode::AssertZero(expression) => solve_linear(expression, known, target),
        _ => None,
    })
}

/// Solve `expression == 0` for `target`, treating every other witness as known.
fn solve_linear(
    expression: &Expression<FieldElement>,
    known: &Known,
    target: Witness,
) -> Option<FieldElement> {
    let mut coefficient = FieldElement::zero();
    let mut constant = expression.q_c;

    for (factor, lhs, rhs) in &expression.mul_terms {
        match (*lhs == target, *rhs == target) {
            // `target^2` is not linear in `target`.
            (true, true) => return None,
            (true, false) => coefficient += *factor * *known.get(rhs)?,
            (false, true) => coefficient += *factor * *known.get(lhs)?,
            (false, false) => constant += *factor * *known.get(lhs)? * *known.get(rhs)?,
        }
    }

    for (factor, witness) in &expression.linear_combinations {
        if *witness == target {
            coefficient += *factor;
        } else {
            constant += *factor * *known.get(witness)?;
        }
    }

    if coefficient.is_zero() {
        return None;
    }
    Some(-constant / coefficient)
}

/// The coefficients `target` is multiplied by in the constraints, as field elements.
///
/// These drive the wraparound candidates: a witness multiplied by `c` can usually be shifted by
/// about `p / c` while another witness in the same constraint absorbs the difference, which is the
/// shape of an alias that satisfies the constraint in the field but not over the integers.
pub fn linear_coefficients(circuit: &Circuit<FieldElement>, target: Witness) -> Vec<FieldElement> {
    let mut coefficients = Vec::new();
    for opcode in &circuit.opcodes {
        let Opcode::AssertZero(expression) = opcode else {
            continue;
        };
        for (factor, witness) in &expression.linear_combinations {
            if *witness == target && !factor.is_zero() {
                coefficients.push(*factor);
            }
        }
        for (factor, lhs, rhs) in &expression.mul_terms {
            if (*lhs == target || *rhs == target) && !factor.is_zero() {
                coefficients.push(*factor);
            }
        }
    }
    coefficients.sort_by_key(|value| value.to_be_bytes());
    coefficients.dedup();
    coefficients
}

#[cfg(test)]
mod tests {
    use super::*;
    use acir::circuit::{Circuit, PublicInputs};
    use std::collections::BTreeSet;

    fn circuit_of(opcodes: Vec<Opcode<FieldElement>>) -> Circuit<FieldElement> {
        Circuit {
            function_name: "test".to_string(),
            opcodes,
            private_parameters: BTreeSet::new(),
            public_parameters: PublicInputs::default(),
            return_values: PublicInputs::default(),
            assert_messages: Vec::new(),
        }
    }

    /// `w2 = w0 - 4*w1`, the shape euclidean division recomposition takes.
    fn recomposition() -> Expression<FieldElement> {
        Expression {
            mul_terms: Vec::new(),
            linear_combinations: vec![
                (FieldElement::one(), Witness(0)),
                (-FieldElement::from(4u128), Witness(1)),
                (-FieldElement::one(), Witness(2)),
            ],
            q_c: FieldElement::zero(),
        }
    }

    #[test]
    fn solves_quotient_from_pinned_remainder() {
        let circuit = circuit_of(vec![Opcode::AssertZero(recomposition())]);
        // w0 = 30, remainder pinned to 2 => quotient must be 7.
        let known = BTreeMap::from([
            (Witness(0), FieldElement::from(30u128)),
            (Witness(2), FieldElement::from(2u128)),
        ]);

        let derived = derive_from_constraints(&circuit, &known, Witness(1));
        assert_eq!(derived, Some(FieldElement::from(7u128)));
    }

    #[test]
    fn refuses_when_two_witnesses_are_unknown() {
        let circuit = circuit_of(vec![Opcode::AssertZero(recomposition())]);
        let known = BTreeMap::from([(Witness(0), FieldElement::from(30u128))]);

        assert_eq!(derive_from_constraints(&circuit, &known, Witness(1)), None);
    }

    #[test]
    fn multiplier_of_a_witness_is_reported() {
        let circuit = circuit_of(vec![Opcode::AssertZero(recomposition())]);

        assert_eq!(linear_coefficients(&circuit, Witness(1)), vec![-FieldElement::from(4u128)]);
    }
}
