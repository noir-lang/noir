use crate::fuzz_lib::function_context::FunctionData;
use crate::mutations::configuration::{
    BASIC_FUNCTION_MUTATION_CONFIGURATION, FunctionMutationOptions, MAX_NUMBER_OF_MUTATIONS,
};
use crate::mutations::functions::commands_mutator;
use rand::{Rng, rngs::StdRng};

pub(crate) fn mutate_function(data: &mut FunctionData, rng: &mut StdRng) {
    let number_of_mutations = rng.gen_range(1..MAX_NUMBER_OF_MUTATIONS);
    for _ in 0..number_of_mutations {
        match BASIC_FUNCTION_MUTATION_CONFIGURATION.select(rng) {
            FunctionMutationOptions::ReturnBlockIdx => {
                // TODO(sn): implement
            }
            FunctionMutationOptions::FunctionFuzzerCommands => {
                commands_mutator::mutate_vec_fuzzer_command(&mut data.commands, rng);
            }
            FunctionMutationOptions::ReturnType => {
                // TODO(sn): implement
            }
        }
    }
}
