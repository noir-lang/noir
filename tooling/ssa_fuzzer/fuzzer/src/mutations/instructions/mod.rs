//! This file contains mechanisms for deterministically mutating a given vector of [InstructionBlock](crate::fuzz_lib::instruction::InstructionBlock) values

mod argument_mutator;
mod instruction_block_mutator;
mod instruction_mutator;

use super::basic_types::vec::mutate_vec;
use super::configuration::BASIC_VEC_MUTATION_CONFIGURATION;
use crate::fuzz_lib::instruction::InstructionBlock;
use instruction_block_mutator::instruction_block_mutator;
use rand::rngs::StdRng;

pub(crate) fn mutate(vec_instruction_block: &mut Vec<InstructionBlock>, rng: &mut StdRng) {
    mutate_vec(
        vec_instruction_block,
        rng,
        instruction_block_mutator,
        BASIC_VEC_MUTATION_CONFIGURATION,
    );
}
