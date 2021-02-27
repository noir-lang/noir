use std::collections::BTreeMap;

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
    let num_witness = acir.num_witnesses + 1;
    let mut optimised_arith_gates: Vec<_> = acir
        .gates
        .into_iter()
        .map(|gate| match gate {
            Gate::Arithmetic(arith) => {
                let arith = optimiser.optimise(arith, &mut intermediate_variables, num_witness);
                Gate::Arithmetic(arith)
            }
            other_gates => other_gates,
        })
        .collect();

    let num_witnesses = acir.num_witnesses + intermediate_variables.len() as u32;
    for (_, gate) in intermediate_variables {
        optimised_arith_gates.push(Gate::Arithmetic(gate));
    }
    Circuit {
        num_witnesses,
        gates: optimised_arith_gates,
        public_inputs: acir.public_inputs, // The optimiser does not add public inputs
    }
}
