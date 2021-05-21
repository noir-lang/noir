use crate::concrete_cfg::{from_fe, CurveAcir, CurveAcirArithGate, Fr};
use acir::{circuit::Circuit, native_types::Arithmetic};

/// Converts an ACIR into an ACIR struct that
/// the arkworks backend can consume
pub fn serialise(acir: Circuit, values: Vec<Fr>) -> CurveAcir {
    (acir, values).into()
}

impl From<(Circuit, Vec<Fr>)> for CurveAcir {
    fn from(circ_val: (Circuit, Vec<Fr>)) -> CurveAcir {
        // Currently non-arithmetic gates are not supported
        // so we extract all of the arithmetic gates only
        let circ = circ_val.0;
        let arith_gates: Vec<_> = circ
            .gates
            .into_iter()
            .filter(|gate| gate.is_arithmetic())
            .map(|gate| CurveAcirArithGate::from(gate.arithmetic()))
            .collect();

        let values = circ_val.1;

        let num_vars = (circ.current_witness_index + 1) as usize;
        CurveAcir {
            gates: arith_gates,
            values,
            num_variables: num_vars,
            public_inputs: circ.public_inputs,
        }
    }
}

impl From<Arithmetic> for CurveAcirArithGate {
    fn from(arith_gate: Arithmetic) -> CurveAcirArithGate {
        let converted_mul_terms: Vec<_> = arith_gate
            .mul_terms
            .into_iter()
            .map(|(coeff, l_var, r_var)| (from_fe(coeff), l_var, r_var))
            .collect();

        let converted_linear_combinations: Vec<_> = arith_gate
            .linear_combinations
            .into_iter()
            .map(|(coeff, var)| (from_fe(coeff), var))
            .collect();

        CurveAcirArithGate {
            mul_terms: converted_mul_terms,
            add_terms: converted_linear_combinations,
            constant_term: from_fe(arith_gate.q_c),
        }
    }
}
