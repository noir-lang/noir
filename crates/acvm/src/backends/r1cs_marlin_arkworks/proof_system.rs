use std::collections::BTreeMap;

use crate::{Language, ProofSystemCompiler};
use acir::{circuit::Circuit, native_types::Witness};
use noir_field::Bn254Scalar;

use super::Marlin;

impl ProofSystemCompiler<Bn254Scalar> for Marlin {
    fn prove_with_meta(
        &self,
        circuit: Circuit<Bn254Scalar>,
        witness_values: BTreeMap<Witness, Bn254Scalar>,
    ) -> Vec<u8> {
        // XXX: modify arkworks serialiser to accept the BTreeMap
        let values: Vec<_> = witness_values.values().collect();
        arkworks_backend::prove(circuit, values)
    }

    fn verify_from_cs(
        &self,
        proof: &[u8],
        public_inputs: Vec<Bn254Scalar>,
        circuit: Circuit<Bn254Scalar>,
    ) -> bool {
        arkworks_backend::verify(circuit, proof, public_inputs)
    }

    fn np_language(&self) -> Language {
        Language::R1CS
    }
}
