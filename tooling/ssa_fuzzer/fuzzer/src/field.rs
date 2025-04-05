#![no_main]

use acvm::FieldElement;
use acvm::acir::native_types::{Witness, WitnessMap};
use libfuzzer_sys::arbitrary;
use libfuzzer_sys::arbitrary::Arbitrary;
use noir_ssa_fuzzer::config;
use noirc_evaluator::ssa::ir::types::Type;
mod base_context;
use crate::base_context::Instructions;
mod fuzzer;
use crate::fuzzer::Fuzzer;

impl Instructions {
    fn is_supported(&self) -> bool {
        !matches!(
            self,
            Instructions::Shl { .. }
                | Instructions::Shr { .. }
                | Instructions::And { .. }
                | Instructions::Or { .. }
                | Instructions::Xor { .. }
                | Instructions::Not { .. }
                | Instructions::Mod { .. }
        )
    }
}

impl Fuzzer {
    fn insert_instruction(&mut self, instruction: Instructions) {
        // Check if instruction is unsupported for field type
        if !instruction.is_supported() {
            return;
        }
        self.context_non_constant.insert_instruction(instruction.clone());
        self.context_constant.insert_instruction(instruction);
    }
}

/// Field modulus has 254 bits, and FieldElement::from supports u128, so we use two unsigneds to represent a field element
/// field = low + high * 2^128
#[derive(Debug, Clone, Hash, Arbitrary)]
struct FieldRepresentation {
    high: u128,
    low: u128,
}

impl From<&FieldRepresentation> for FieldElement {
    fn from(field: &FieldRepresentation) -> FieldElement {
        let lower = FieldElement::from(field.low);
        let upper = FieldElement::from(field.high);
        lower + upper * (FieldElement::from(u128::MAX) + FieldElement::from(1_u128))
    }
}

/// Represents the data for the fuzzer
/// `methods` - sequence of instructions to be added to the program
/// `initial_witness` - initial witness values for the program as `FieldRepresentation`
#[derive(Arbitrary, Debug, Clone, Hash)]
struct FuzzerData {
    methods: Vec<Instructions>,
    initial_witness: [FieldRepresentation; config::NUMBER_OF_VARIABLES_INITIAL as usize],
}

// main fuzz loop
libfuzzer_sys::fuzz_target!(|data: FuzzerData| {
    // init logger and initialize witness map
    let _ = env_logger::try_init();
    let type_ = Type::field();
    let mut witness_map = WitnessMap::new();
    let mut values = vec![];
    for (i, witness_value) in data.initial_witness.iter().map(FieldElement::from).enumerate() {
        witness_map.insert(Witness(i as u32), witness_value);
        values.push(witness_value);
    }
    let initial_witness = witness_map;
    log::debug!("instructions: {:?}", data.methods.clone());
    log::debug!("initial_witness: {:?}", initial_witness);

    let mut fuzzer = Fuzzer::new(type_.clone(), values);
    for method in data.methods.clone() {
        fuzzer.insert_instruction(method);
    }
    fuzzer.run(initial_witness);
});
