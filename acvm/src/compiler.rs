use std::collections::BTreeMap;

use acir::{
    circuit::{Circuit, Gate},
    native_types::{Arithmetic, Witness},
    optimiser::CSatOptimiser,
};

use crate::ProofSystemCompiler;

pub fn default() -> impl ProofSystemCompiler {
    super::backends::csat_3_plonk_aztec::Plonk
}

pub struct OptimiserCircuit {
    pub circuit: acir::circuit::Circuit,
    pub intermediate_variables: BTreeMap<Witness, Arithmetic>,
}

pub fn compile<T: ProofSystemCompiler>(
    acir: Circuit,
    num_witnesses: usize,
    backend: T,
) -> OptimiserCircuit {
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
    let num_witness = num_witnesses + 1;
    let optimised_arith_gates: Vec<_> = acir
        .0
        .into_iter()
        .map(|gate| match gate {
            Gate::Arithmetic(arith) => {
                let arith = optimiser.optimise(arith, &mut intermediate_variables, num_witness);
                Gate::Arithmetic(arith)
            }
            other_gates => other_gates,
        })
        .collect();

    OptimiserCircuit {
        circuit: acir::circuit::Circuit(optimised_arith_gates),
        intermediate_variables,
    }
}
