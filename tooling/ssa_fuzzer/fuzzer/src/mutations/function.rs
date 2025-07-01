use crate::fuzz_lib::function_context::FunctionData;
use crate::mutations::configuration::{
    BASIC_FUNCTION_MUTATION_CONFIGURATION, MAX_NUMBER_OF_MUTATIONS, MutationOptions,
};
use rand::{Rng, rngs::StdRng};

pub(crate) fn mutate_function(data: &mut FunctionData, rng: &mut StdRng) {
    let number_of_mutations = rng.gen_range(1..MAX_NUMBER_OF_MUTATIONS);
    for _ in 0..number_of_mutations {
        match BASIC_FUNCTION_MUTATION_CONFIGURATION.select(rng) {
            MutationOptions::InstructionBlocks => {
                crate::mutations::instructions::mutate_vec_instruction_block(&mut data.blocks, rng);
            }
            MutationOptions::FuzzerCommands => {
                crate::mutations::commands_mutator::mutate_vec_fuzzer_command(
                    &mut data.commands,
                    rng,
                );
            }
            MutationOptions::Witnesses => {
                let index = rng.gen_range(0..data.initial_witness.len());
                crate::mutations::witness_mutator::witness_mutate(
                    &mut data.initial_witness[index],
                    rng,
                );
            }
        }
    }
}
