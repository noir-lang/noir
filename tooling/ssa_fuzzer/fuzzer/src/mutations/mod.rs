mod commands_mutator;
mod instructions;
mod witness_mutator;

use crate::fuzz_lib::fuzz_target_lib::FuzzerData;
use rand::rngs::StdRng;

pub(crate) fn mutate(data: FuzzerData, rng: &mut StdRng) -> FuzzerData {
    FuzzerData {
        blocks: instructions::mutate_vec_instruction_block(data.blocks, rng),
        commands: commands_mutator::mutate_vec_fuzzer_command(data.commands, rng),
        initial_witness: witness_mutator::witness_mutate(data.initial_witness, rng),
        return_instruction_block_idx: data.return_instruction_block_idx,
    }
}
