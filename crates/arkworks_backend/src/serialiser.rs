use crate::bridge::{Bn254Acir, Bn254AcirArithGate};
use acir::{circuit::Circuit, native_types::Arithmetic};
use ark_bn254::Fr;

/// Converts an ACIR into an ACIR struct that
/// the arkworks backend can consume
pub fn serialise(acir: Circuit, values: Vec<Fr>) -> Bn254Acir {
    (acir, values).into()
}

impl From<(Circuit, Vec<Fr>)> for Bn254Acir {
    fn from(circ_val: (Circuit, Vec<Fr>)) -> Bn254Acir {
        // Currently non-arithmetic gates are not supported
        // so we extract all of the arithmetic gates only
        let circ = circ_val.0;
        let arith_gates: Vec<_> = circ
            .gates
            .into_iter()
            .filter(|gate| gate.is_arithmetic())
            .map(|gate| Bn254AcirArithGate::from(gate.arithmetic()))
            .collect();

        let values = circ_val.1;

        let num_vars = (circ.current_witness_index + 1) as usize;
        Bn254Acir {
            gates: arith_gates,
            values,
            num_variables: num_vars,
            public_inputs: circ.public_inputs,
        }
    }
}

impl From<Arithmetic> for Bn254AcirArithGate {
    fn from(arith_gate: Arithmetic) -> Bn254AcirArithGate {
        let converted_mul_terms: Vec<_> = arith_gate
            .mul_terms
            .into_iter()
            .map(|(coeff, l_var, r_var)| (coeff.into_repr(), l_var, r_var))
            .collect();

        let converted_linear_combinations: Vec<_> = arith_gate
            .linear_combinations
            .into_iter()
            .map(|(coeff, var)| (coeff.into_repr(), var))
            .collect();

        Bn254AcirArithGate {
            mul_terms: converted_mul_terms,
            add_terms: converted_linear_combinations,
            constant_term: arith_gate.q_c.into_repr(),
        }
    }
}
