use crate::base_context::FuzzerCommand;
use crate::fuzzer::Fuzzer;
use crate::instruction::InstructionBlock;
use crate::options::FuzzerOptions;
use acvm::FieldElement;
use acvm::acir::native_types::{Witness, WitnessMap};
use libfuzzer_sys::arbitrary;
use libfuzzer_sys::arbitrary::Arbitrary;
use noir_ssa_fuzzer::config;
use noir_ssa_fuzzer::typed_value::ValueType;
use noirc_driver::CompileOptions;
/// Field modulus has 254 bits, and FieldElement::from supports u128, so we use two unsigneds to represent a field element
/// field = low + high * 2^128
#[derive(Debug, Clone, Hash, Arbitrary)]
pub(crate) struct FieldRepresentation {
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
pub(crate) enum WitnessValue {
    Field(FieldRepresentation),
    U64(u64),
    Boolean(bool),
}

/// Represents the data for the fuzzer
/// `methods` - sequence of instructions to be added to the program
/// `initial_witness` - initial witness values for the program as `FieldRepresentation`
#[derive(Arbitrary, Debug)]
pub(crate) struct FuzzerData {
    blocks: Vec<InstructionBlock>,
    commands: Vec<FuzzerCommand>,
    initial_witness: [WitnessValue; (config::NUMBER_OF_VARIABLES_INITIAL - 2) as usize],
    return_instruction_block_idx: usize,
}

pub(crate) fn fuzz_target(data: FuzzerData, options: FuzzerOptions) {
    let mut witness_map = WitnessMap::new();
    let mut values = vec![];
    let mut types = vec![];

    if data.blocks.is_empty() {
        return;
    }
    for instr_block in &data.blocks {
        if instr_block.instructions.is_empty() {
            return;
        }
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
    witness_map.insert(Witness(5_u32), FieldElement::from(1_u32));
    values.push(FieldElement::from(1_u32));
    types.push(ValueType::Boolean);
    witness_map.insert(Witness(6_u32), FieldElement::from(0_u32));
    values.push(FieldElement::from(0_u32));
    types.push(ValueType::Boolean);

    let initial_witness = witness_map;
    log::debug!("blocks: {:?}", data.blocks);
    log::debug!("initial_witness: {:?}", initial_witness);
    log::debug!("initial_witness_in_data: {:?}", data.initial_witness);
    log::debug!("commands: {:?}", data.commands);

    let mut fuzzer = Fuzzer::new(types, values, data.blocks, options);
    for command in data.commands {
        fuzzer.process_fuzzer_command(command);
    }
    fuzzer.run(initial_witness, data.return_instruction_block_idx);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzz_target() {
        let data = FuzzerData::arbitrary(&mut arbitrary::Unstructured::new(&[1, 2, 3])).unwrap();
        fuzz_target(data, FuzzerOptions::default());
    }

    #[test]
    fn test_jmp_if() {}
}
