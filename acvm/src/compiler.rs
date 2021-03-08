use std::{collections::BTreeMap, thread::current};

use acir::{
    circuit::{Circuit, Gate},
    native_types::{Arithmetic, Witness},
    optimiser::CSatOptimiser,
};

use crate::BackendPointer;

pub fn compile(acir: Circuit, backend: BackendPointer) -> Circuit {
    let backend = backend.backend();
    //
    // Instantiate the optimiser.
    // Currently the optimiser and reducer are one in the same
    let optimiser = match backend.np_language() {
        crate::Language::R1CS => todo!(),
        crate::Language::PLONKCSat { width } => CSatOptimiser::new(width),
    };

    let mut intermediate_variables: BTreeMap<Witness, Arithmetic> = BTreeMap::new();

    // Optimise the arithmetic gates by reducing them into the correct width and
    // creating intermediate variables when necessary
    let next_witness_index = acir.current_witness_index + 1;
    let mut optimised_arith_gates: Vec<_> = acir
        .gates
        .into_iter()
        .map(|gate| match gate {
            Gate::Arithmetic(arith) => {
                let arith =
                    optimiser.optimise(arith, &mut intermediate_variables, next_witness_index);
                Gate::Arithmetic(arith)
            }
            other_gates => other_gates,
        })
        .collect();

    let current_witness_index = acir.current_witness_index + intermediate_variables.len() as u32;
    for (_, gate) in intermediate_variables {
        optimised_arith_gates.push(Gate::Arithmetic(gate));
    }
    Circuit {
        current_witness_index,
        gates: optimised_arith_gates,
        public_inputs: acir.public_inputs, // The optimiser does not add public inputs
    }
}
