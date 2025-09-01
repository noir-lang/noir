use super::{
    fuzzer::{Fuzzer, FuzzerData, FuzzerOutput},
    initial_witness::{ensure_boolean_defined_in_all_functions, initialize_witness_map},
    options::FuzzerOptions,
};
use noir_ssa_fuzzer::typed_value::Type;

/// Creates ACIR and Brillig programs from the data, runs and compares them
pub(crate) fn fuzz_target(data: FuzzerData, options: FuzzerOptions) -> Option<FuzzerOutput> {
    if data.instruction_blocks.is_empty() {
        return None;
    }
    if data.functions.is_empty() {
        return None;
    }
    log::debug!("instruction_blocks: {:?}", data.instruction_blocks);
    log::debug!("initial_witness: {:?}", data.initial_witness);
    let (witness_map, values, types) = initialize_witness_map(&data.initial_witness);
    let mut data = data;
    data.functions[0].input_types = types;
    ensure_boolean_defined_in_all_functions(&mut data);

    if data.functions[0].return_type.is_reference() {
        // main cannot return a reference
        data.functions[0].return_type = Type::default();
    }

    let mut fuzzer = Fuzzer::new(data.instruction_blocks, values, options);
    for func in data.functions {
        log::debug!("commands: {:?}", func.commands);
        log::debug!("input_types: {:?}", func.input_types);
        fuzzer.process_function(func.clone(), func.input_types.clone());
    }
    fuzzer.finalize_and_run(witness_map)
}
