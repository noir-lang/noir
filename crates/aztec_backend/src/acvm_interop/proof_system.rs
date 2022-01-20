use super::Plonk;
use crate::barretenberg_rs::composer::{Assignments, StandardComposer};
use acvm::acir::{circuit::Circuit, native_types::Witness};
use acvm::FieldElement;
use acvm::{Language, ProofSystemCompiler};
use std::collections::BTreeMap;

impl ProofSystemCompiler for Plonk {
    fn prove_with_meta(
        &self,
        circuit: Circuit,
        witness_values: BTreeMap<Witness, FieldElement>,
    ) -> Vec<u8> {
        let constraint_system = crate::serialise_circuit(&circuit);

        let mut composer = StandardComposer::new(constraint_system);

        // Add witnesses in the correct order
        // Note: The witnesses are sorted via their witness index, since we implement Ord on Witness and use a BTreeMap
        let mut sorted_witness = Assignments::new();
        for (_, value) in witness_values.iter() {
            sorted_witness.push(*value);
        }

        composer.create_proof(sorted_witness)
    }

    fn verify_from_cs(
        &self,
        proof: &[u8],
        public_inputs: Vec<FieldElement>,
        circuit: Circuit,
    ) -> bool {
        let constraint_system = crate::serialise_circuit(&circuit);

        let mut composer = StandardComposer::new(constraint_system);

        composer.verify(proof, Some(Assignments::from_vec(public_inputs)))
    }

    fn np_language(&self) -> Language {
        Language::PLONKCSat { width: 3 }
    }
}
