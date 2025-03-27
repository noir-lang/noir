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


/// Represents the data for the fuzzer
/// `methods` - sequence of instructions to be added to the program
#[derive(Arbitrary, Debug, Clone, Hash)]
struct FuzzerData {
    methods: Vec<Instructions>,
    initial_witness: [u64; config::NUMBER_OF_VARIABLES_INITIAL as usize],
}

impl Fuzzer {
    fn insert_instruction(&mut self, instruction: Instructions) {
        self.context_non_constant.insert_instruction(instruction.clone());
        self.context_constant.insert_instruction(instruction);
    }
}


// main fuzz loop
libfuzzer_sys::fuzz_target!(|data: FuzzerData| {
    // init logger and initialize witness map
    let _ = env_logger::try_init();
    let type_ = Type::unsigned(64);
    let mut witness_map = WitnessMap::new();
    let mut values = vec![];
    for (i, witness_value) in data.initial_witness.iter().map(|x| FieldElement::from(*x)).enumerate() {
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
