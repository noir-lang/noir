mod configuration;
mod functions;
mod instructions;
mod witness_mutator;

use crate::fuzz_lib::fuzzer::FuzzerData;
use crate::mutations::configuration::{
    BASIC_FUZZER_DATA_MUTATION_CONFIGURATION, FuzzerDataMutationOptions, MAX_NUMBER_OF_MUTATIONS,
};
use rand::{Rng, rngs::StdRng};

pub(crate) fn mutate(data: &mut FuzzerData, rng: &mut StdRng) {
    let number_of_mutations = rng.gen_range(1..MAX_NUMBER_OF_MUTATIONS);
    for _ in 0..number_of_mutations {
        match BASIC_FUZZER_DATA_MUTATION_CONFIGURATION.select(rng) {
            FuzzerDataMutationOptions::Functions => {
                functions::mutate(&mut data.functions, rng);
            }
            FuzzerDataMutationOptions::InstructionBlocks => {
                instructions::mutate(&mut data.instruction_blocks, rng);
            }
            FuzzerDataMutationOptions::Witnesses => {
                let idx = rng.gen_range(0..data.initial_witness.len());
                witness_mutator::mutate(&mut data.initial_witness[idx], rng);
            }
        }
    }
}
