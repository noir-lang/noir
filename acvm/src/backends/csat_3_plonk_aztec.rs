use std::collections::BTreeMap;

use crate::{Backend, Language, ProofSystemCompiler, SmartContract};
use acir::{circuit::Circuit, native_types::Witness};
use aztec_backend::barretenberg_rs::composer::{Assignments, StandardComposer};
use noir_field::FieldElement;

pub struct Plonk;

impl Backend for Plonk {}

impl ProofSystemCompiler for Plonk {
    fn prove_with_meta(
        &self,
        circuit: Circuit,
        witness_values: BTreeMap<Witness, FieldElement>,
        num_witnesses: usize,
        public_inputs: Vec<u32>,
    ) -> Vec<u8> {
        let constraint_system =
            aztec_backend::serialise_circuit(&circuit, num_witnesses, public_inputs);

        let mut composer = StandardComposer::new(constraint_system.size());

        // Add witnesses in the correct order
        // Note: The witnesses are sorted via their witness index, since we implement Ord on Witness and use a BTreeMap
        let mut sorted_witness = Assignments::new();
        for (_, value) in witness_values.iter() {
            sorted_witness.push(*value);
        }

        composer.create_proof(&constraint_system, sorted_witness)
    }

    fn verify_from_cs(
        &self,
        proof: &[u8],
        circuit: Circuit,
        num_witnesses: usize,
        public_inputs: Vec<u32>,
    ) -> bool {
        let constraint_system =
            aztec_backend::serialise_circuit(&circuit, num_witnesses, public_inputs);

        let mut composer = StandardComposer::new(constraint_system.size());

        //XXX: Currently barretenberg appends the public inputs to the proof
        let public_inputs = None;
        composer.verify(&constraint_system, &proof, public_inputs)
    }

    fn np_language(&self) -> Language {
        Language::PLONKCSat { width: 3 }
    }
}

impl SmartContract for Plonk {
    fn eth_contract_from_cs(
        &self,
        circuit: Circuit,
        num_witnesses: usize,
        public_inputs: Vec<u32>,
    ) -> String {
        let constraint_system =
            aztec_backend::serialise_circuit(&circuit, num_witnesses, public_inputs);

        let mut composer = StandardComposer::new(constraint_system.size());

        composer.smart_contract(&constraint_system)
    }
}
