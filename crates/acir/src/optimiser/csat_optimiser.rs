use crate::native_types::{Arithmetic, Witness};
use noir_field::FieldElement;
use std::collections::BTreeMap;

use super::general_optimiser::GeneralOpt;
// Optimiser struct with all of the related optimisations to the arithmetic gate

// Is this more of a Reducer than an optimiser?
// Should we give it all of the gates?
// Have a single optimiser that you instantiate with a width, then pass many gates through
pub struct Optimiser {
    width: usize,
}

impl Optimiser {
    // Configure the width for the optimiser
    pub fn new(width: usize) -> Optimiser {
        assert!(width > 2);

        Optimiser { width }
    }

    // Still missing dead witness optimisation.
    // To do this, we will need the whole set of arithmetic gates
    // I think it can also be done before the local optimisation seen here, as dead variables will come from the user
    pub fn optimise(
        &self,
        gate: Arithmetic,
        mut intermediate_variables: &mut BTreeMap<Witness, Arithmetic>,
        num_witness: u32,
    ) -> Arithmetic {
        let gate = GeneralOpt::optimise(gate);

        // Here we create intermediate variables and constrain them to be equal to any subset of the polynomial that can be represented as a full gate
        let gate = self.full_gate_scan_optimisation(gate, &mut intermediate_variables, num_witness);
        // The last optimisation to do is to create intermediate variables in order to flatten the fan-in and the amount of mul terms
        // If a gate has more than one mul term. We may need an intermediate variable for each one. Since not every variable will need to link to
        // the mul term, we could possibly do it that way.
        // We wil call this a partial gate scan optimisation which will result in the gates being able to fit into the correct width
        let gate =
            self.partial_gate_scan_optimisation(gate, &mut intermediate_variables, num_witness);

        self.sort(gate)
    }

    /// Sorts gate in a deterministic order
    /// XXX: We can probably make this more efficient by sorting on each phase. We only care if it is deterministic
    fn sort(&self, mut gate: Arithmetic) -> Arithmetic {
        gate.mul_terms.sort();
        gate.linear_combinations.sort();

        gate
    }

    // This optimisation will search for combinations of terms which can be represented in a single arithmetic gate
    // Case 1 : qM * wL * wR + qL * wL + qR * wR + qO * wO + qC
    // This polynomial does not require any further optimisations, it can be safely represented in one gate
    // ie a polynomial with 1 mul(bi-variate) term and 3 (univariate) terms where 2 of those terms match the bivariate term
    // wL and wR, we can represent it in one gate
    // GENERALISED for WIDTH: instead of the number 3, we use `WIDTH`
    //
    //
    // Case 2: qM * wL * wR + qL * wL + qR * wR + qO * wO + qC + qM2 * wL2 * wR2 + qL * wL2 + qR * wR2 + qO * wO2 + qC2
    // This polynomial cannot be represented using one arithmetic gate.
    //
    // This algorithm will first extract the first full gate(if possible):
    // t = qM * wL * wR + qL * wL + qR * wR + qO * wO + qC
    //
    // The polynomial now looks like so t + qM2 * wL2 * wR2 + qL * wL2 + qR * wR2 + qO * wO2 + qC2
    // This polynomial cannot be represented using one arithmetic gate.
    //
    // This algorithm will then extract the second full gate(if possible):
    // t2 = qM2 * wL2 * wR2 + qL * wL2 + qR * wR2 + qO * wO2 + qC2
    //
    // The polynomial now looks like so t + t2
    // We can no longer extract another full gate, hence the algorithm terminates. Creating two intermediate variables t and t2.
    // This stage of preprocessing does not guarantee that all polynomials can fit into a gate. It only guarantees that all full gates have been extracted from each polynomial
    fn full_gate_scan_optimisation(
        &self,
        mut gate: Arithmetic,
        intermediate_variables: &mut BTreeMap<Witness, Arithmetic>,
        num_witness: u32,
    ) -> Arithmetic {
        // We pass around this intermediate variable BTreeMap, so that we do not create intermediate variables that we have created before
        // One instance where this might happen is t1 = wL * wR and t2 = wR * wL

        // First check that this is not a simple gate which does not need optimisation
        //
        // If the gate only has one mul term, then this algorithm cannot optimise it any further
        // Either it can be represented in a single arithmetic equation or it's fan-in is too large and we need intermediate variables for those
        // large-fan-in optimisation is not this algorithms purpose.
        // If the gate has 0 mul terms, then it is an add gate and similarly it can either fit into a single arithmetic gate or it has a large fan-in
        if gate.mul_terms.len() <= 1 {
            return gate;
        }

        // We now know that this gate has multiple mul terms and can possibly be simplified into multiple full gates
        // We need to create a (wl, wr) BTreeMap and then check the simplified fan-in to verify if we have terms both with wl and wr
        // In general, we can then push more terms into the gate until we are at width-1 then the last variable will be the intermediate variable
        //

        // This will be our new gate which will be equal to `self` except we will have intermediate variables that will be constrained to any
        // subset of the terms that can be represented as full gates
        let mut new_gate = Arithmetic::default();

        while !gate.mul_terms.is_empty() {
            let pair = gate.mul_terms[0];

            // Check if this pair is present in the simplified fan-in
            // We are assuming that the fan-in/fan-out has been simplified.
            // Note this function is not public, and can only be called within the optimise method, so this guarantee will always hold
            let index_wl = gate
                .linear_combinations
                .iter()
                .position(|(_scale, witness)| *witness == pair.1);
            let index_wr = gate
                .linear_combinations
                .iter()
                .position(|(_scale, witness)| *witness == pair.2);

            match (index_wl, index_wr) {
                (None, _) => {
                    // This means that the polynomial does not contain both terms
                    // Just push the Qm term as it cannot form a full gate
                    new_gate.mul_terms.push(pair);
                }
                (_, None) => {
                    // This means that the polynomial does not contain both terms
                    // Just push the Qm term as it cannot form a full gate
                    new_gate.mul_terms.push(pair);
                }
                (Some(x), Some(y)) => {
                    // This means that we can form a full gate with this Qm term

                    // First fetch the left and right wires which match the mul term
                    let left_wire_term = gate.linear_combinations[x];
                    let right_wire_term = gate.linear_combinations[y];

                    // Lets create an intermediate gate to store this full gate
                    //
                    let mut intermediate_gate = Arithmetic::default();
                    intermediate_gate.mul_terms.push(pair);

                    // Add the left and right wires
                    intermediate_gate.linear_combinations.push(left_wire_term);
                    intermediate_gate.linear_combinations.push(right_wire_term);
                    // Remove the left and right wires so we do not re-add them
                    gate.linear_combinations.remove(x);
                    gate.linear_combinations.remove(y);

                    // Now we have used up 2 spaces in our arithmetic gate. The width now dictates, how many more we can add
                    let remaining_space = self.width - 2 - 1; // We minus 1 because we need an extra space to contain the intermediate variable
                                                              // Keep adding terms until we have no more left, or we reach the width
                    for _ in 0..remaining_space {
                        if let Some(wire_term) = gate.linear_combinations.pop() {
                            // Add this element into the new gate
                            intermediate_gate.linear_combinations.push(wire_term);
                        } else {
                            // Nomore elements left in the old gate, we could stop the whole function
                            // We could alternative let it keep going, as it will never reach this branch again since there are nomore elements left
                            // XXX: Future optimisation
                            // nomoreleft = true
                        }
                    }
                    // Constraint this intermediate_gate to be equal to the temp variable by adding it into the BTreeMap
                    // We need a unique name for our intermediate variable
                    // XXX: Another optimisation, which could be applied in another algorithm
                    // If two gates have a large fan-in/out and they share a few common terms, then we should create intermediate variables for them
                    // Do some sort of subset matching algorithm for this on the terms of the polynomial
                    let inter_var = Witness(intermediate_variables.len() as u32 + num_witness);

                    // Constrain the gate to the intermediate variable
                    intermediate_gate
                        .linear_combinations
                        .push((-FieldElement::one(), inter_var));
                    // Add intermediate gate to the map
                    intermediate_variables.insert(inter_var, intermediate_gate);

                    // Add intermediate variable to the new gate instead of the full gate
                    new_gate
                        .linear_combinations
                        .push((FieldElement::one(), inter_var));
                }
            };
            // Remove this term as we are finished processing it
            gate.mul_terms.remove(0);
        }

        // Add the rest of the elements back into the new_gate
        new_gate.mul_terms.extend(gate.mul_terms.clone());
        new_gate
            .linear_combinations
            .extend(gate.linear_combinations.clone());
        new_gate.q_c = gate.q_c;

        new_gate
    }

    // A partial gate scan optimisation aim to create intermediate variables in order to compress the polynomial
    // So that it fits within the given width
    // Note that this gate follows the full gate scan optimisation.
    // We define the partial width as equal to the full width - 2.
    // This is because two of our variables cannot be used as they are linked to the multiplication terms
    // Example: qM1 * wL1 * wR2 + qL1 * wL3 + qR1 * wR4+ qR2 * wR5 + qO1 * wO5 + qC
    // One thing to note is that the multiplication wires do not match any of the fan-in/out wires. This is guaranteed as we have
    // just completed the full gate optimisation algorithm.
    //
    //Actually we can optimise in two ways here: We can create an intermediate variable which is equal to the fan-in terms
    // t = qL1 * wL3 + qR1 * wR4 -> width = 3
    // This `t` value can only use width - 1 terms
    // The gate now looks like: qM1 * wL1 * wR2 + t + qR2 * wR5+ qO1 * wO5 + qC
    // But this is still not acceptable since wR5 is not wR2, so we need another intermediate variable
    // t2 = t + qR2 * wR5
    //
    // The gate now looks like: qM1 * wL1 * wR2 + t2 + qO1 * wO5 + qC
    // This is still not good, so we do it one more time:
    // t3 = t2 + qO1 * wO5
    // The gate now looks like: qM1 * wL1 * wR2 + t3 + qC
    //
    // Another strategy is to create a temporary variable for the multiplier term and then we can see it as a term in the fan-in
    //
    // Same Example: qM1 * wL1 * wR2 + qL1 * wL3 + qR1 * wR4+ qR2 * wR5 + qO1 * wO5 + qC
    // t = qM1 * wL1 * wR2
    // The gate now looks like: t + qL1 * wL3 + qR1 * wR4+ qR2 * wR5 + qO1 * wO5 + qC
    // Still assuming width3, we still need to use width-1 terms for the intermediate variables, however we can stop at an earlier stage because
    // the gate does not need the multiplier term to match with any of the fan-in terms
    // t2 = t + qL1 * wL3
    // The gate now looks like: t2 + qR1 * wR4+ qR2 * wR5 + qO1 * wO5 + qC
    // t3 = t2 + qR1 * wR4
    // The gate now looks like: t3 + qR2 * wR5 + qO1 * wO5 + qC
    // This took the same amount of gates, but which one is better when the width increases? Compute this and maybe do both optimisations
    // naming : partial_gate_mul_first_opt and partial_gate_fan_first_opt
    // Also remember that since we did full gate scan, there is no way we can have a non-zero mul term along with the wL and wR terms being non-zero
    //
    // Cases, a lot of mul terms, a lot of fan-in terms, 50/50
    fn partial_gate_scan_optimisation(
        &self,
        mut gate: Arithmetic,
        intermediate_variables: &mut BTreeMap<Witness, Arithmetic>,
        num_witness: u32,
    ) -> Arithmetic {
        // We will go for the easiest route, which is to convert all multiplications into additions using intermediate variables
        // Then use intermediate variables again to squash the fan-in, so that it can fit into the appropriate width

        // First check if this polynomial actually needs a partial gate optimisation
        // There is the chance that it fits perfectly within the arithmetic gate
        if gate.fits_in_one_identity(self.width) {
            return gate;
        }

        // 2. Create Intermediate variables for the multiplication gates
        for mul_term in gate.mul_terms.clone().into_iter() {
            // Create intermediate variable to squash the multiplication term
            let inter_var = Witness((intermediate_variables.len() as u32) + num_witness);
            let mut intermediate_gate = Arithmetic::default();

            // Push mul term into the gate
            intermediate_gate.mul_terms.push(mul_term);
            // Constrain it to be equal to the intermediate variable
            intermediate_gate
                .linear_combinations
                .push((-FieldElement::one(), inter_var));

            // Add intermediate gate and variable to map
            intermediate_variables.insert(inter_var, intermediate_gate);

            // Add intermediate variable as a part of the fan-in for the original gate
            gate.linear_combinations
                .push((FieldElement::one(), inter_var));
        }

        // Remove all of the mul terms as we have intermediate variables to represent them now
        gate.mul_terms.clear();

        // We now only have a polynomial with only fan-in/fan-out terms i.e. terms of the form Ax + By + Cd + ...
        // Lets create intermediate variables if all of them cannot fit into the width
        //
        // If the polynomial fits perfectly within the given width, we are finished
        if gate.linear_combinations.len() <= self.width {
            return gate;
        }

        // Stores the intermediate variables that are used to
        // reduce the fan in.
        let mut added = Vec::new();

        while gate.linear_combinations.len() > self.width {
            // Collect as many terms up to the given width-1 and constrain them to an intermediate variable
            let mut intermediate_gate = Arithmetic::default();

            for _ in 0..(self.width - 1) {
                match gate.linear_combinations.pop() {
                    Some(term) => {
                        intermediate_gate.linear_combinations.push(term);
                    }
                    None => {
                        break; // We can also do nothing here
                    }
                };
            }
            // Constrain the intermediate gate to be equal to the intermediate variable
            let inter_var = Witness((intermediate_variables.len() as u32) + num_witness);

            added.push((FieldElement::one(), inter_var));

            intermediate_gate
                .linear_combinations
                .push((-FieldElement::one(), inter_var));

            // Add intermediate gate and variable to map
            intermediate_variables.insert(inter_var, intermediate_gate);
        }

        // Add back the intermediate variables to
        // keep consistency with the original equation.
        gate.linear_combinations.extend(added);

        self.partial_gate_scan_optimisation(gate, intermediate_variables, num_witness)
    }
}

#[test]
fn simple_reduction_smoke_test() {
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

    let mut intermediate_variables: BTreeMap<Witness, Arithmetic> = BTreeMap::new();

    let num_witness = 4;

    let optimiser = Optimiser::new(3);
    let got_optimised_gate_a = optimiser.optimise(gate_a, &mut intermediate_variables, num_witness);

    // a = b + c + d => a - b - c - d = 0
    // For width3, the result becomes:
    // a - b + e = 0
    // - c - d  - e = 0
    //
    // a - b + e = 0
    let e = Witness(4);
    let expected_optimised_gate_a = Arithmetic {
        mul_terms: vec![],
        linear_combinations: vec![
            (FieldElement::one(), a),
            (FieldElement::one(), e),
            (-FieldElement::one(), b),
        ],
        q_c: FieldElement::zero(),
    };
    assert_eq!(expected_optimised_gate_a, got_optimised_gate_a);

    assert_eq!(intermediate_variables.len(), 1);

    let got_intermediate_gate = intermediate_variables.get(&e).unwrap();

    // - c - d  - e = 0
    let expected_intermediate_gate = Arithmetic {
        mul_terms: vec![],
        linear_combinations: vec![
            (-FieldElement::one(), d),
            (-FieldElement::one(), c),
            (-FieldElement::one(), e),
        ],
        q_c: FieldElement::zero(),
    };
    assert_eq!(&expected_intermediate_gate, got_intermediate_gate);
}
