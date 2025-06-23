//! This file contains mechanisms for deterministically mutating a given vector of [InstructionBlock](crate::fuzz_lib::instruction::InstructionBlock) values
//! Types of mutations applied:
//! 1. Random (randomly select a new instruction block)
//! 2. Instruction block deletion
//! 3. Instruction block insertion
//! 4. Instruction mutation

mod argument_mutator;
mod instruction_block_mutator;
mod instruction_mutator;
use super::configuration::{
    BASIC_VECTOR_OF_INSTRUCTION_BLOCKS_MUTATION_CONFIGURATION,
    VectorOfInstructionBlocksMutationOptions,
};
use crate::fuzz_lib::instruction::InstructionBlock;
use instruction_block_mutator::instruction_block_mutator;
use libfuzzer_sys::arbitrary::Unstructured;
use rand::{Rng, rngs::StdRng};

trait MutateVecInstructionBlock {
    fn mutate(rng: &mut StdRng, value: &mut Vec<InstructionBlock>);
}

/// Return new random vector of instruction blocks
struct RandomMutation;
impl MutateVecInstructionBlock for RandomMutation {
    fn mutate(rng: &mut StdRng, _value: &mut Vec<InstructionBlock>) {
        let mut bytes = [0u8; 128];
        rng.fill(&mut bytes);
        Unstructured::new(&bytes).arbitrary().unwrap()
    }
}

/// Return vector of instruction blocks with one randomly chosen block removed
struct MutateInstructionBlockDeletionMutation;
impl MutateVecInstructionBlock for MutateInstructionBlockDeletionMutation {
    fn mutate(rng: &mut StdRng, value: &mut Vec<InstructionBlock>) {
        let mut blocks = value;
        if !blocks.is_empty() {
            let block_idx = rng.gen_range(0..blocks.len());
            blocks.remove(block_idx);
        }
    }
}

/// Return vector of instruction blocks with one randomly generated block inserted
struct MutateInstructionBlockInsertionMutation;
impl MutateVecInstructionBlock for MutateInstructionBlockInsertionMutation {
    fn mutate(rng: &mut StdRng, value: &mut Vec<InstructionBlock>) {
        let mut blocks = value;
        let block_idx = if blocks.is_empty() { 0 } else { rng.gen_range(0..blocks.len()) };
        let mut bytes = [0u8; 25];
        rng.fill(&mut bytes);
        let mut unstructured = Unstructured::new(&bytes);
        let instruction_block = unstructured.arbitrary().unwrap();
        blocks.insert(block_idx, instruction_block);
    }
}

/// Return vector of instruction blocks with one randomly chosen block mutated
struct MutateInstructionBlockInstructionMutation;
impl MutateVecInstructionBlock for MutateInstructionBlockInstructionMutation {
    fn mutate(rng: &mut StdRng, value: &mut Vec<InstructionBlock>) {
        let mut blocks = value;
        if !blocks.is_empty() {
            let block_idx = rng.gen_range(0..blocks.len());
            blocks[block_idx] = instruction_block_mutator(blocks[block_idx].clone(), rng);
        }
    }
}

pub(crate) fn mutate_vec_instruction_block(
    vec_instruction_block: &mut Vec<InstructionBlock>,
    rng: &mut StdRng,
) {
    match BASIC_VECTOR_OF_INSTRUCTION_BLOCKS_MUTATION_CONFIGURATION.select(rng) {
        VectorOfInstructionBlocksMutationOptions::Random => {
            RandomMutation::mutate(rng, vec_instruction_block)
        }
        VectorOfInstructionBlocksMutationOptions::InstructionBlockDeletion => {
            MutateInstructionBlockDeletionMutation::mutate(rng, vec_instruction_block)
        }
        VectorOfInstructionBlocksMutationOptions::InstructionBlockInsertion => {
            MutateInstructionBlockInsertionMutation::mutate(rng, vec_instruction_block)
        }
        VectorOfInstructionBlocksMutationOptions::InstructionBlockMutation => {
            MutateInstructionBlockInstructionMutation::mutate(rng, vec_instruction_block)
        }
    }
}
