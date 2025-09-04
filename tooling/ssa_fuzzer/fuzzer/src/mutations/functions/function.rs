use crate::mutations::{
    basic_types::{ssa_fuzzer_type::mutate_ssa_fuzzer_type, usize::mutate_usize},
    configuration::{
        BASIC_FUNCTION_MUTATION_CONFIGURATION, BASIC_USIZE_MUTATION_CONFIGURATION,
        FunctionMutationOptions,
    },
    functions::{FunctionData, commands_mutator, input_types},
};
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
            mutate_ssa_fuzzer_type(&mut data.return_type, rng);
        }
        FunctionMutationOptions::InputTypes => {
            input_types::mutate_input_types(&mut data.input_types, rng);
        }
    }
}
