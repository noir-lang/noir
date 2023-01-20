use indexmap::IndexMap;

use crate::{simplify::CircuitSimplifier, CustomGate, Language};
use acir::{
    circuit::{
        gate::{AndGate, GadgetCall, XorGate},
        Circuit, Gate,
    },
    native_types::{Expression, Witness},
    optimiser::{CSatOptimiser, GeneralOptimiser},
};

pub fn compile(acir: Circuit, np_language: Language, simplifier: &CircuitSimplifier) -> Circuit {
    // Instantiate the optimiser.
    // Currently the optimiser and reducer are one in the same
    // for CSAT

    // Fallback pass
    let fallback = fallback(&acir, &np_language, simplifier);

    let optimiser = match &np_language {
        crate::Language::R1CS => return optimise_r1cs(fallback),
        crate::Language::PLONKCSat { width } => CSatOptimiser::new(*width),
    };

    // Optimise the arithmetic gates by reducing them into the correct width and
    // creating intermediate variables when necessary
    let mut optimised_gates = Vec::new();

    let mut next_witness_index = fallback.current_witness_index + 1;
    for gate in fallback.gates {
        match gate {
            Gate::Arithmetic(arith_expr) => {
                let mut intermediate_variables: IndexMap<Witness, Expression> = IndexMap::new();

                let arith_expr =
                    optimiser.optimise(arith_expr, &mut intermediate_variables, next_witness_index);

                // Update next_witness counter
                next_witness_index += intermediate_variables.len() as u32;
                let mut new_gates = Vec::new();
                for (_, mut g) in intermediate_variables {
                    g.sort();
                    new_gates.push(g);
                }
                new_gates.push(arith_expr);
                new_gates.sort();
                for gate in new_gates {
                    optimised_gates.push(Gate::Arithmetic(gate));
                }
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

//Acir pass which replace unsupported gates using arithmetic fallback
pub fn fallback(acir: &Circuit, np_language: &Language, simplifier: &CircuitSimplifier) -> Circuit {
    let mut fallback_gates = Vec::new();
    let mut witness_idx = acir.current_witness_index + 1;
    for w in &simplifier.defined {
        fallback_gates.push(simplifier.define(w));
    }
    for (i, g) in acir.gates.iter().enumerate() {
        if !simplifier.solved_gates.contains(&i) {
            if !np_language.supports_gate(g) {
                let gates = gate_fallback(g, &mut witness_idx);
                fallback_gates.extend(gates);
            } else {
                fallback_gates.push(g.clone());
            }
        }
    }

    Circuit {
        current_witness_index: witness_idx,
        gates: fallback_gates,
        public_inputs: acir.public_inputs.clone(),
    }
}

fn gate_fallback(gate: &Gate, witness_idx: &mut u32) -> Vec<Gate> {
    let mut gadget_gates = Vec::new();
    match gate {
        Gate::Range(a, bit_size) => {
            *witness_idx = acir::fallback::range(
                Expression::from(a),
                *bit_size,
                *witness_idx,
                &mut gadget_gates,
            );
        }
        Gate::And(AndGate { a, b, result, num_bits }) => {
            *witness_idx = acir::fallback::and(
                Expression::from(a),
                Expression::from(b),
                *result,
                *num_bits,
                *witness_idx,
                &mut gadget_gates,
            );
        }
        Gate::Xor(XorGate { a, b, result, num_bits }) => {
            *witness_idx = acir::fallback::xor(
                Expression::from(a),
                Expression::from(b),
                *result,
                *num_bits,
                *witness_idx,
                &mut gadget_gates,
            );
        }
        Gate::GadgetCall(GadgetCall { name, .. }) => {
            unreachable!("Missing alternative for opcode {}", name)
        }
        _ => todo!("no fallback for gate {:?}", gate),
    }

    gadget_gates
}
