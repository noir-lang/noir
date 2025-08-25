use super::{
    NUMBER_OF_PREDEFINED_VARIABLES, NUMBER_OF_VARIABLES_INITIAL,
    function_context::WitnessValue,
    fuzzer::{Fuzzer, FuzzerData, FuzzerOutput},
    options::FuzzerOptions,
};
use acvm::FieldElement;
use acvm::acir::native_types::{Witness, WitnessMap};
use noir_ssa_fuzzer::typed_value::ValueType;

fn initialize_witness_map(
    initial_witness: &[WitnessValue;
         (NUMBER_OF_VARIABLES_INITIAL - NUMBER_OF_PREDEFINED_VARIABLES) as usize],
) -> (WitnessMap<FieldElement>, Vec<FieldElement>, Vec<ValueType>) {
    let mut witness_map = WitnessMap::new();
    let mut values = vec![];
    let mut types = vec![];
    for (i, witness_value) in initial_witness.iter().enumerate() {
        let (value, type_) = match witness_value {
            WitnessValue::Field(field) => (FieldElement::from(field), ValueType::Field),
            WitnessValue::U64(u64) => (FieldElement::from(*u64), ValueType::U64),
            WitnessValue::Boolean(bool) => (FieldElement::from(*bool as u64), ValueType::Boolean),
            WitnessValue::I64(i64) => (FieldElement::from(*i64), ValueType::I64),
            WitnessValue::I32(i32) => (FieldElement::from(*i32 as u64), ValueType::I32),
        };
        witness_map.insert(Witness(i as u32), value);
        values.push(value);
        types.push(type_);
    }
    // insert true and false boolean values
    witness_map.insert(
        Witness(NUMBER_OF_VARIABLES_INITIAL - NUMBER_OF_PREDEFINED_VARIABLES),
        FieldElement::from(1_u32),
    );
    values.push(FieldElement::from(1_u32));
    types.push(ValueType::Boolean);
    witness_map.insert(
        Witness(NUMBER_OF_VARIABLES_INITIAL - NUMBER_OF_PREDEFINED_VARIABLES + 1),
        FieldElement::from(0_u32),
    );
    values.push(FieldElement::from(0_u32));
    types.push(ValueType::Boolean);
    (witness_map, values, types)
}

/// Creates ACIR and Brillig programs from the data, runs and compares them
pub(crate) fn fuzz_target(data: FuzzerData, options: FuzzerOptions) -> Option<FuzzerOutput> {
    // to triage
    if data.instruction_blocks.is_empty() {
        return None;
    }
    log::debug!("instruction_blocks: {:?}", data.instruction_blocks);
    let (witness_map, values, types) = initialize_witness_map(&data.initial_witness);

    let mut fuzzer = Fuzzer::new(data.instruction_blocks, values, options);
    for func in data.functions {
        log::debug!("initial_witness: {witness_map:?}");
        log::debug!("commands: {:?}", func.commands);
        fuzzer.process_function(func, types.clone());
    }
    fuzzer.finalize_and_run(witness_map)
}
