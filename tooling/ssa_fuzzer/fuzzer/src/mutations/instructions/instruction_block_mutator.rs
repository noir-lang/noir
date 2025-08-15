//! This file contains mechanisms for deterministically mutating a given [InstructionBlock](crate::fuzz_lib::instruction::InstructionBlock) value

use crate::fuzz_lib::instruction::InstructionBlock;
use crate::mutations::{
    basic_types::vec::mutate_vec, configuration::BASIC_VEC_MUTATION_CONFIGURATION,
    instructions::instruction_mutator::instruction_mutator,
};
use rand::rngs::StdRng;

pub(crate) fn instruction_block_mutator(
    instruction_block: &mut InstructionBlock,
    rng: &mut StdRng,
) {
    mutate_vec(
        &mut instruction_block.instructions,
        rng,
        instruction_mutator,
        BASIC_VEC_MUTATION_CONFIGURATION,
    );
}
