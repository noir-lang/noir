use crate::fuzz_lib::function_context::FunctionData;
use crate::mutations::basic_types::{usize::mutate_usize, value_type::mutate_value_type};
use crate::mutations::configuration::{
    BASIC_FUNCTION_MUTATION_CONFIGURATION, BASIC_USIZE_MUTATION_CONFIGURATION,
    BASIC_VALUE_TYPE_MUTATION_CONFIGURATION, FunctionMutationOptions,
};
use crate::mutations::functions::commands_mutator;
use rand::rngs::StdRng;

pub(crate) fn mutate_function(data: &mut FunctionData, rng: &mut StdRng) {
    match BASIC_FUNCTION_MUTATION_CONFIGURATION.select(rng) {
        FunctionMutationOptions::ReturnBlockIdx => {
            mutate_usize(
                &mut data.return_instruction_block_idx,
                rng,
                BASIC_USIZE_MUTATION_CONFIGURATION,
            );
        }
        FunctionMutationOptions::FunctionFuzzerCommands => {
            commands_mutator::mutate_vec_fuzzer_command(&mut data.commands, rng);
        }
        FunctionMutationOptions::ReturnType => {
            mutate_value_type(&mut data.return_type, rng, BASIC_VALUE_TYPE_MUTATION_CONFIGURATION);
        }
    }
}
