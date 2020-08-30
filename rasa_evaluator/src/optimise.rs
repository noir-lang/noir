use super::{Arithmetic, FieldElement, Witness};
use std::collections::BTreeMap;
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
        num_witness : usize,
    ) -> Arithmetic {
        // Remove all terms where the co-efficient is zero
        let gate = self.remove_zero_coefficients(gate);
        // Collect like terms in the fan-in and fan-out
        // XXX: Perhaps this optimisation can be done on the fly and we only have one vector

        let gate = self.simplify_fan(gate);

        // Collect like terms in the mul terms array
        let gate = self.simplify_mul_terms(gate);

        // Here we create intermediate variables and constrain them to be equal to any subset of the polynomial that can be represented as a full gate
        let gate = self.full_gate_scan_optimisation(gate, &mut intermediate_variables, num_witness);

        // The last optimisation to do is to create intermediate variables in order to flatten the fan-in and the amount of mul terms
        // If a gate has more than one mul term . we may need an intermediate variable for each one. Since not every variable will need to link to
        // the mul term, we could possibly do it that way.
        // We wil call this a partial gate scan optimisation which will result in the gates being able to fit into the correct width
        let gate = self.partial_gate_scan_optimisation(gate, &mut intermediate_variables, num_witness);

        self.sort(gate)
    }

    /// Sorts gate in a deterministic order
    /// XXX: We can probably make this more efficient by sorting on each phase. We only care if it is deterministic
    fn sort(&self,
        mut gate: Arithmetic) -> Arithmetic {
            gate.mul_terms.sort();
            gate.simplified_fan.sort();

            gate
        }

    // Remove all terms with zero as a coefficient
    fn remove_zero_coefficients(&self, mut gate: Arithmetic) -> Arithmetic {
        // Check the mul terms
        gate.mul_terms = gate
            .mul_terms
            .into_iter()
            .filter(|(scale, _, _)| !scale.is_zero())
            .collect();

        // Check the fan-in terms
        gate.fan_in = gate
            .fan_in
            .into_iter()
            .filter(|(scale, _)| !scale.is_zero())
            .collect();

        // Check the fan-out terms
        gate.fan_out = gate
            .fan_out
            .into_iter()
            .filter(|(scale, _)| !scale.is_zero())
            .collect();

        gate
    }

    // Adds all terms in the fan-in/out
    // We use a BTreeMap to do this instead of iterating over each element in time O(n^2)
    // XXX: This is really inefficient, we could probably use a variety of methods, such as if the fan-out was small then just modify the fan-in in place
    // or use BTreeMaps from the start
    fn simplify_fan(&self, mut gate: Arithmetic) -> Arithmetic {
        gate.simplify_fan();
        gate
    }

    // Simplifies all mul terms with the same bi-variate variables
    fn simplify_mul_terms(&self, mut gate: Arithmetic) -> Arithmetic {
        let mut hash_map: BTreeMap<(Witness, Witness), FieldElement> = BTreeMap::new();
            
        // Canonicalise the ordering of the multiplication, lets just order by variable name
        for (scale, wL, wR) in gate.mul_terms.clone().into_iter() {
            let mut pair = vec![wL, wR];
            // Sort using rust sort algorithm
            pair.sort();

            *hash_map
                .entry((pair[0].clone(), pair[1].clone()))
                .or_insert(FieldElement::zero()) += scale;
        }

        gate.mul_terms = hash_map
            .into_iter()
            .map(|((wL, wR), scale)| (scale, wL, wR))
            .collect();

        gate
    }

    // This optimisation will search for combinations of terms which can be represented in a single arithmetic gate
    // Case1 : qM * wL * wR + qL * wL + qR * wR + qO * wO + qC
    // This polynomial does not require any further optimisations, it can be safely represented in one gate
    // ie a polynomial with 1 mul(bi-variate) term and 3 (univariate) terms where 2 of those terms match the bivariate term
    // wL and wR, we can represent it in one gate
    // GENERALISED for WIDTH: instead of the number 3, we use `WIDTH`
    //
    //
    // Case2: qM * wL * wR + qL * wL + qR * wR + qO * wO + qC + qM2 * wL2 * wR2 + qL * wL2 + qR * wR2 + qO * wO2 + qC2
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
        num_witness : usize,
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

        while gate.mul_terms.len() > 0 {
            let pair = gate.mul_terms[0].clone();

            // Check if this pair is present in the simplified fan-in
            // We are assuming that the fan-in/fan-out has been simplified.
            // Note this function is not public, and can only be called within the optimise method, so this guarantee will always hold
            let index_wl = gate
                .simplified_fan
                .iter()
                .position(|(scale, witness)| *witness == pair.1);
            let index_wr = gate
                .simplified_fan
                .iter()
                .position(|(scale, witness)| *witness == pair.2);

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
                    let left_wire_term = gate.simplified_fan[x].clone();
                    let right_wire_term = gate.simplified_fan[y].clone();

                    // Lets create an intermediate gate to store this full gate
                    //
                    let mut intermediate_gate = Arithmetic::default();
                    intermediate_gate.mul_terms.push(pair);

                    // Add the left and right wires
                    intermediate_gate.simplified_fan.push(left_wire_term);
                    intermediate_gate.simplified_fan.push(right_wire_term);
                    // Remove the left and right wires so we do not re-add them
                    gate.simplified_fan.remove(x);
                    gate.simplified_fan.remove(y);

                    // Now we have used up 2 spaces in our arithmetic gate. The width now dictates, how many more we can add
                    let remaining_space = self.width - 2 - 1; // We minus 1 because we need an extra space to contrain the intermediate variable
                                                              // Keep adding terms until we have no more left, or we reach the width
                    for i in 0..remaining_space {
                        match gate.simplified_fan.pop() {
                            Some(wire_term) => {
                                // Add this element into the new gate
                                intermediate_gate.simplified_fan.push(wire_term);
                            }
                            None => {
                                // Nomore elements left in the old gate, we could stop the whole function
                                // We could alternative let it keep going, as it will never reach this branch again since there are nomore elements left
                                // XXX: Future optimisation
                                // nomoreleft = true
                            }
                        }
                    }
                    // Constraint this intermediate_gate to be equal to the temp variable by adding it into the BTreeMap
                    // We need a unique name for our intermediate variable
                    // XXX: Another optimisation, which could be applied in another algorithm
                    // If two gates have a large fan-in/out and they share a few common terms, then we should create intermediate variables for them
                    // Do some sort of subset matching algorithm for this on the terms of the polynomial
                    let inter_var_name = format!(
                        "{}{}",
                        "optim_inter_full_gate_",
                        intermediate_variables.len(),
                    );
                    let inter_var = Witness(inter_var_name, intermediate_variables.len() + num_witness);

                    // Constrain the gate to the intermediate variable
                    intermediate_gate
                        .simplified_fan
                        .push((-FieldElement::one(), inter_var.clone()));
                    // Add intermediate gate to the map
                    intermediate_variables.insert(inter_var.clone(), intermediate_gate);

                    // Add intermediate variable to the new gate instead of the full gate
                    new_gate
                        .simplified_fan
                        .push((FieldElement::one(), inter_var));
                }
            };
            // Remove this term as we are finished processing it
            gate.mul_terms.remove(0);
        }

        // Add the rest of the elements back into the new_gate
        new_gate.mul_terms.extend(gate.mul_terms.clone());
        new_gate.simplified_fan.extend(gate.simplified_fan.clone());
        new_gate.q_C = gate.q_C;

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
    // the gate does not need the multipler term to match with any of the fan-in terms
    // t2 = t + qL1 * wL3
    // The gate now looks like: t2 + qR1 * wR4+ qR2 * wR5 + qO1 * wO5 + qC
    // t3 = t2 + qR1 * wR4
    // The gate now looks like: t3 + qR2 * wR5 + qO1 * wO5 + qC
    // This took the same amount of gates, but which one is better when the width increases? Compute this and mayeb do both optimisations
    // naming : partial_gate_mul_first_opt and partial_gate_fan_first_opt
    // Also remember that since we did full gate scan, there is no way we can have a non-zero mul term along with the wL and wR terms being non-zero
    //
    // Cases, a lot of mul terms, a lot of fan-in terms, 50/50
    fn partial_gate_scan_optimisation(
        &self,
        mut gate: Arithmetic,
        intermediate_variables: &mut BTreeMap<Witness, Arithmetic>,
        num_witness: usize,
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
            let inter_var_name = format!(
                "{}{}",
                "optim_inter_squash_mul_",
                intermediate_variables.len(),
            );
            let inter_var = Witness(inter_var_name,intermediate_variables.len() + num_witness);
            let mut intermediate_gate = Arithmetic::default();
   
            // Push mul term into the gate
            intermediate_gate.mul_terms.push(mul_term);
            // Constrain it to be equal to the intermediate variable
            intermediate_gate
                .simplified_fan
                .push((-FieldElement::one(), inter_var.clone()));

            // Add intermediate gate and variable to map
            intermediate_variables.insert(inter_var.clone(), intermediate_gate);
    
            // Add intermediate variable as a part of the fan-in for the original gate
            gate.simplified_fan.push((FieldElement::one(),inter_var ));
        }

        // Remove all of the mul terms as we have intermediate variables to represent them now
        gate.mul_terms.clear();

        // We now only have a polynomial with only fan-in/fan-out terms ie terms of the form Ax + By + Cd + ...
        // Lets create intermediate variables if all of them cannot fit into the width
        //
        // If the polynomial fits perfectly within the given width, we are finished
        if gate.simplified_fan.len() <= self.width {
            return gate;
        }

        while gate.simplified_fan.len() > self.width {
            // Collect as many terms upto the given width-1 and constrain them to an intermediate variable
            let mut intermediate_gate = Arithmetic::default();

            for _ in 0..(self.width - 1) {
                match gate.simplified_fan.pop() {
                    Some(term) => {
                        intermediate_gate.simplified_fan.push(term);
                    }
                    None => {
                        break; // We can also do nothing here
                    }
                };
            }
            // Constrain the intermediate gate to be equal to the intermediate variable
            let inter_var_name = format!(
                "{}{}",
                "optim_inter_squash_fan_",
                intermediate_variables.len(),
            );
            let inter_var = Witness(inter_var_name,intermediate_variables.len() + num_witness); 

            intermediate_gate
                .simplified_fan
                .push((-FieldElement::one(), inter_var.clone()));

            // Add intermediate gate and variable to map
            intermediate_variables.insert(inter_var, intermediate_gate);
        }

        gate
    }
}
