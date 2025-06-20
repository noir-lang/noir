mod commands_mutator;
mod configuration;
mod instructions;
mod witness_mutator;

use crate::fuzz_lib::fuzz_target_lib::FuzzerData;
use crate::mutations::configuration::{
    BASIC_MUTATION_CONFIGURATION, MAX_NUMBER_OF_MUTATIONS, MutationOptions,
};
use rand::{Rng, rngs::StdRng};

pub(crate) fn mutate(data: FuzzerData, rng: &mut StdRng) -> FuzzerData {
    let (mut blocks, mut commands, mut initial_witness) =
        (data.blocks, data.commands, data.initial_witness);
    let number_of_mutations = rng.gen_range(0..MAX_NUMBER_OF_MUTATIONS);
    for _ in 0..number_of_mutations {
        match BASIC_MUTATION_CONFIGURATION.select(rng) {
            MutationOptions::MutateInstructionBlocks => {
                blocks = instructions::mutate_vec_instruction_block(blocks, rng);
            }
            MutationOptions::MutateFuzzerCommands => {
                commands = commands_mutator::mutate_vec_fuzzer_command(commands, rng);
            }
            MutationOptions::MutateWitnesses => {
                let index = rng.gen_range(0..initial_witness.len());
                initial_witness[index] =
                    witness_mutator::witness_mutate(&initial_witness[index], rng);
            }
        }
    }
    FuzzerData {
        blocks,
        commands,
        initial_witness,
        return_instruction_block_idx: data.return_instruction_block_idx,
    }
}
