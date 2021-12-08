use indexmap::IndexMap;

use crate::Language;
use acir::{
    circuit::{Circuit, Gate},
    native_types::{Arithmetic, Witness},
    optimiser::{CSatOptimiser, GeneralOptimiser},
};

pub fn compile(acir: Circuit, np_language: Language) -> Circuit {
    // Instantiate the optimiser.
    // Currently the optimiser and reducer are one in the same
    // for CSAT
    let optimiser = match np_language {
        crate::Language::R1CS => return optimise_r1cs(acir),
        crate::Language::PLONKCSat { width } => CSatOptimiser::new(width),
    };

    // Optimise the arithmetic gates by reducing them into the correct width and
    // creating intermediate variables when necessary
    let mut optimised_gates = Vec::new();

    let mut next_witness_index = acir.current_witness_index + 1;
    for gate in acir.gates {
        match gate {
            Gate::Arithmetic(arith) => {
                let mut intermediate_variables: IndexMap<Witness, Arithmetic> = IndexMap::new();

                let arith =
                    optimiser.optimise(arith, &mut intermediate_variables, next_witness_index);

                // Update next_witness counter
                next_witness_index += intermediate_variables.len() as u32;

                for (_, gate) in intermediate_variables.into_iter() {
                    optimised_gates.push(Gate::Arithmetic(gate));
                }
                optimised_gates.push(Gate::Arithmetic(arith));
            }
            other_gate => optimised_gates.push(other_gate),
        }
    }

    let current_witness_index = next_witness_index - 1;

    Circuit {
        current_witness_index,
        gates: optimised_gates,
        public_inputs: acir.public_inputs, // The optimiser does not add public inputs
    }
}

// R1CS optimisations uses the general optimiser.
// Once R1CS specific optimisations are found, then we can
// refactor this function
fn optimise_r1cs(acir: Circuit) -> Circuit {
    let optimised_arith_gates: Vec<_> = acir
        .gates
        .into_iter()
        .map(|gate| match gate {
            Gate::Arithmetic(arith) => Gate::Arithmetic(GeneralOptimiser::optimise(arith)),
            other_gates => other_gates,
        })
        .collect();

    Circuit {
        // The general optimiser may remove enough gates that a witness is no longer used
        // however, we cannot decrement the number of witnesses, as that
        // would require a linear scan over all gates in order to decrement all witness indices
        // above the witness which was removed
        current_witness_index: acir.current_witness_index,
        gates: optimised_arith_gates,
        public_inputs: acir.public_inputs,
    }
}
