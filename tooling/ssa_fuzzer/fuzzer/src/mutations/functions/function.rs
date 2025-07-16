use crate::fuzz_lib::function_context::FunctionData;
use crate::mutations::configuration::{
    BASIC_FUNCTION_MUTATION_CONFIGURATION, FunctionMutationOptions,
};
use crate::mutations::functions::commands_mutator;
use rand::rngs::StdRng;

pub(crate) fn mutate_function(data: &mut FunctionData, rng: &mut StdRng) {
    match BASIC_FUNCTION_MUTATION_CONFIGURATION.select(rng) {
        FunctionMutationOptions::ReturnBlockIdx => {
            unimplemented!()
        }
        FunctionMutationOptions::FunctionFuzzerCommands => {
            commands_mutator::mutate_vec_fuzzer_command(&mut data.commands, rng);
        }
        FunctionMutationOptions::ReturnType => {
            unimplemented!()
        }
    }
}
