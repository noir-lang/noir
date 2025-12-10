//! This file contains mechanisms for deterministically mutating a given vector of [InstructionBlock](crate::fuzz_lib::instruction::InstructionBlock) values

mod argument_mutator;
mod instruction_block_mutator;
mod instruction_mutator;

use crate::fuzz_lib::instruction::InstructionBlock;
use crate::mutations::{
    basic_types::vec::mutate_vec,
    configuration::{BASIC_VEC_MUTATION_CONFIGURATION, SIZE_OF_SMALL_ARBITRARY_BUFFER},
    instructions::instruction_block_mutator::instruction_block_mutator,
};
use libfuzzer_sys::arbitrary::Unstructured;
use rand::{Rng, rngs::StdRng};

fn generate_random_instruction_block(rng: &mut StdRng) -> InstructionBlock {
    let mut buf = [0u8; SIZE_OF_SMALL_ARBITRARY_BUFFER];
    rng.fill(&mut buf);
    let mut unstructured = Unstructured::new(&buf);
    unstructured.arbitrary().unwrap()
}

pub(crate) fn mutate(vec_instruction_block: &mut Vec<InstructionBlock>, rng: &mut StdRng) {
    mutate_vec(
        vec_instruction_block,
        rng,
        instruction_block_mutator,
        generate_random_instruction_block,
        BASIC_VEC_MUTATION_CONFIGURATION,
    );
}
