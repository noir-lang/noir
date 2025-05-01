#![no_main]

use acvm::FieldElement;
use acvm::acir::native_types::{Witness, WitnessMap};
use libfuzzer_sys::arbitrary;
use libfuzzer_sys::arbitrary::Arbitrary;
use noir_ssa_fuzzer::config;
use noir_ssa_fuzzer::typed_value::ValueType;
mod base_context;
use crate::base_context::{FuzzerCommand, InstructionBlock};
mod fuzzer;
use crate::fuzzer::Fuzzer;

mod block_context;

impl Fuzzer {
    fn process_fuzzer_command(&mut self, command: FuzzerCommand) {
        self.context_non_constant.process_fuzzer_command(command.clone());
        self.context_constant.process_fuzzer_command(command);
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

#[derive(Debug, Clone, Hash, Arbitrary)]
enum WitnessValue {
    Field(FieldRepresentation),
    U64(u64),
    Boolean(bool),
}

/// Represents the data for the fuzzer
/// `methods` - sequence of instructions to be added to the program
/// `initial_witness` - initial witness values for the program as `FieldRepresentation`
#[derive(Arbitrary, Debug, Clone, Hash)]
struct FuzzerData {
    blocks: Vec<InstructionBlock>,
    commands: Vec<FuzzerCommand>,
    initial_witness: [WitnessValue; (config::NUMBER_OF_VARIABLES_INITIAL - 1) as usize],
    return_instruction_block_idx: usize,
}

// main fuzz loop
libfuzzer_sys::fuzz_target!(|data: FuzzerData| {
    // init logger and initialize witness map
    let _ = env_logger::try_init();
    let mut witness_map = WitnessMap::new();
    let mut values = vec![];
    let mut types = vec![];
    if data.blocks.is_empty() {
        return;
    }
    for (i, witness_value) in data.initial_witness.iter().enumerate() {
        let (value, type_) = match witness_value {
            WitnessValue::Field(field) => (FieldElement::from(field), ValueType::Field),
            WitnessValue::U64(u64) => (FieldElement::from(*u64), ValueType::U64),
            WitnessValue::Boolean(bool) => (FieldElement::from(*bool as u64), ValueType::Boolean),
        };
        witness_map.insert(Witness(i as u32), value);
        values.push(value);
        types.push(type_);
    }
    witness_map.insert(Witness(6_u32), FieldElement::from(1_u32));
    values.push(FieldElement::from(1_u32));
    types.push(ValueType::Boolean);

    let initial_witness = witness_map;
    log::debug!("blocks: {:?}", data.blocks);
    log::debug!("initial_witness: {:?}", initial_witness);
    log::debug!("initial_witness_in_data: {:?}", data.initial_witness);
    log::debug!("commands: {:?}", data.commands);

    let mut fuzzer = Fuzzer::new(types, values, data.blocks);
    for command in data.commands {
        fuzzer.process_fuzzer_command(command);
    }
    fuzzer.run(initial_witness, false, data.return_instruction_block_idx);
});
