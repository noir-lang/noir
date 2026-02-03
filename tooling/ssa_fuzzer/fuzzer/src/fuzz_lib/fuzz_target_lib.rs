use super::{
    fuzzer::{CompareResults, Fuzzer, FuzzerData, FuzzerOutput},
    initial_witness::{ensure_boolean_defined_in_all_functions, initialize_witness_map},
    options::FuzzerOptions,
};
use acvm::acir::native_types::WitnessStack;
use noir_ssa_fuzzer::typed_value::Type;
use noirc_evaluator::ssa::ir::function::RuntimeType;

fn type_contains_vector_or_reference(type_: &Type) -> bool {
    match type_ {
        Type::Vector(_) => true,
        Type::Reference(_) => true,
        Type::Array(arr, _) => arr.iter().any(type_contains_vector_or_reference),
        Type::Numeric(_) => false,
    }
}

/// Creates programs from the data and provided runtimes, runs and compares them
/// Returns [`FuzzerOutput`] of the first runtime provided
/// Panics if the runtimes disagree on the return values
pub(crate) fn fuzz_target(
    data: FuzzerData,
    runtimes: Vec<RuntimeType>,
    options: FuzzerOptions,
) -> FuzzerOutput {
    assert!(!runtimes.is_empty(), "No runtimes provided");
    log::debug!("instruction_blocks: {:?}", data.instruction_blocks);
    log::debug!("initial_witness: {:?}", data.initial_witness);
    let (witness_map, values, types) = initialize_witness_map(&data.initial_witness);
    let mut data = data;
    data.functions[0].input_types = types;
    ensure_boolean_defined_in_all_functions(&mut data);
    if data.instruction_blocks.is_empty() {
        return FuzzerOutput { witness_stack: WitnessStack::from(witness_map), program: None };
    }
    if data.functions.is_empty() {
        return FuzzerOutput { witness_stack: WitnessStack::from(witness_map), program: None };
    }

    if type_contains_vector_or_reference(&data.functions[0].return_type) {
        // main cannot return a reference
        data.functions[0].return_type = Type::default();
    }
    let mut fuzzer_outputs = Vec::new();
    for runtime in runtimes.clone() {
        let mut fuzzer =
            Fuzzer::new(runtime, data.instruction_blocks.clone(), values.clone(), options.clone());
        for func in data.functions.clone() {
            log::debug!("commands: {:?}", func.commands);
            log::debug!("input_types: {:?}", func.input_types);
            fuzzer.process_function(func.clone(), func.input_types.clone());
        }
        fuzzer_outputs.push(fuzzer.finalize_and_run(witness_map.clone()));
    }

    for i in 0..fuzzer_outputs.len() {
        for j in i + 1..fuzzer_outputs.len() {
            let result = fuzzer_outputs[i].compare_results(&fuzzer_outputs[j]);
            match result {
                CompareResults::Agree(_witness_stack) => {
                    // thats fine
                    log::debug!(
                        "Fuzzer runtimes {} and {} agree on the return values",
                        runtimes[i],
                        runtimes[j]
                    );
                }
                CompareResults::Disagree(witness_stack_1, witness_stack_2) => {
                    panic!(
                        "Fuzzer runtimes {} and {} disagree on the return values: {:?} and {:?}",
                        runtimes[i], runtimes[j], witness_stack_1, witness_stack_2
                    );
                }
                CompareResults::LeftCompilationFailed => {
                    panic!(
                        "Fuzzer runtime {} failed to compile, other returned {:?}",
                        runtimes[i],
                        fuzzer_outputs[j].get_return_witnesses()
                    );
                }
                CompareResults::RightCompilationFailed => {
                    panic!(
                        "Fuzzer runtime {} failed to compile, other returned {:?}",
                        runtimes[j],
                        fuzzer_outputs[i].get_return_witnesses()
                    );
                }
                CompareResults::LeftExecutionFailed => {
                    panic!(
                        "Fuzzer runtime {} failed to execute, other returned {:?}",
                        runtimes[i],
                        fuzzer_outputs[j].get_return_witnesses()
                    );
                }
                CompareResults::RightExecutionFailed => {
                    panic!(
                        "Fuzzer runtime {} failed to execute, other returned {:?}",
                        runtimes[j],
                        fuzzer_outputs[i].get_return_witnesses()
                    );
                }
                CompareResults::BothFailed => {
                    // thats fine
                    log::debug!("Fuzzer runtimes {} and {} both failed", runtimes[i], runtimes[j]);
                }
            }
        }
    }
    fuzzer_outputs[0].clone()
}
