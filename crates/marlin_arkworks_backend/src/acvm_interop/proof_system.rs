use std::collections::BTreeMap;

use acvm::{Language, ProofSystemCompiler};
use acvm::acir::{circuit::Circuit, native_types::Witness};
use acvm::FieldElement;

use super::Marlin;

impl ProofSystemCompiler for Marlin {
    fn prove_with_meta(
        &self,
        circuit: Circuit,
        witness_values: BTreeMap<Witness, FieldElement>,
    ) -> Vec<u8> {
        // XXX: modify arkworks serialiser to accept the BTreeMap
        let values: Vec<_> = witness_values.values().collect();
        arkworks_backend::prove(circuit, values)
    }

    fn verify_from_cs(
        &self,
        proof: &[u8],
        public_inputs: Vec<FieldElement>,
        circuit: Circuit,
    ) -> bool {
        arkworks_backend::verify(circuit, proof, public_inputs)
    }

    fn np_language(&self) -> Language {
        Language::R1CS
    }
}
