//! This module implements a fuzzer for testing and comparing ACIR and Brillig SSA implementations.
//! It generates random sequences of arithmetic and logical operations and ensures both implementations
//! produce identical results.
//!
//! Main fuzz steps:
//!    1. Generate random witness
//!    2. Generate random sequence of instructions
//!    3. Insert instructions into ACIR and Brillig builders
//!    4. Get programs, and compile them
//!    5. Run and compare
//!
//! A bug is detected in two cases:
//!    - If programs return different results
//!    - If one program fails to compile but the other executes successfully

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
    for i in 0..config::NUMBER_OF_VARIABLES_INITIAL {
        let witness = Witness(i);
        // difference from uint.rs, we use try_from_str here
        let value = FieldElement::try_from(data.initial_witness[i as usize]);
        match value {
            Ok(value) => {
                witness_map.insert(witness, value);
                values.push(value);
            }
            Err(_) => {
                return;
            }
        }
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
