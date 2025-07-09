//! This file contains mechanisms for deterministically mutating a given vector of [InstructionBlock](crate::fuzz_lib::instruction::InstructionBlock) values
//! Types of mutations applied:
//! 1. Random (randomly select a new instruction block)
//! 2. Instruction block deletion
//! 3. Instruction block insertion
//! 4. Instruction mutation

mod argument_mutator;
mod instruction_block_mutator;
mod instruction_mutator;
mod type_mutations;

use super::configuration::{
    BASIC_VECTOR_OF_INSTRUCTION_BLOCKS_MUTATION_CONFIGURATION, SIZE_OF_LARGE_ARBITRARY_BUFFER,
    SIZE_OF_SMALL_ARBITRARY_BUFFER, VectorOfInstructionBlocksMutationOptions,
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
    fn mutate(rng: &mut StdRng, value: &mut Vec<InstructionBlock>) {
        let mut bytes = [0u8; SIZE_OF_LARGE_ARBITRARY_BUFFER];
        rng.fill(&mut bytes);
        *value = Unstructured::new(&bytes).arbitrary().unwrap();
    }
}

/// Return vector of instruction blocks with one randomly chosen block removed
struct InstructionBlockDeletionMutation;
impl MutateVecInstructionBlock for InstructionBlockDeletionMutation {
    fn mutate(rng: &mut StdRng, value: &mut Vec<InstructionBlock>) {
        let blocks = value;
        if !blocks.is_empty() {
            let block_idx = rng.gen_range(0..blocks.len());
            blocks.remove(block_idx);
        }
    }
}

/// Return vector of instruction blocks with one randomly generated block inserted
struct InstructionBlockInsertionMutation;
impl MutateVecInstructionBlock for InstructionBlockInsertionMutation {
    fn mutate(rng: &mut StdRng, value: &mut Vec<InstructionBlock>) {
        let blocks = value;
        let block_idx = if blocks.is_empty() { 0 } else { rng.gen_range(0..blocks.len()) };
        let mut bytes = [0u8; SIZE_OF_SMALL_ARBITRARY_BUFFER];
        rng.fill(&mut bytes);
        let mut unstructured = Unstructured::new(&bytes);
        let instruction_block = unstructured.arbitrary().unwrap();
        blocks.insert(block_idx, instruction_block);
    }
}

/// Return vector of instruction blocks with one randomly chosen block mutated
struct InstructionBlockInstructionMutation;
impl MutateVecInstructionBlock for InstructionBlockInstructionMutation {
    fn mutate(rng: &mut StdRng, value: &mut Vec<InstructionBlock>) {
        let blocks = value;
        if !blocks.is_empty() {
            let block_idx = rng.gen_range(0..blocks.len());
            instruction_block_mutator(&mut blocks[block_idx], rng);
        }
    }
}

struct InstructionBlockInstructionSwapMutation;
impl MutateVecInstructionBlock for InstructionBlockInstructionSwapMutation {
    fn mutate(rng: &mut StdRng, value: &mut Vec<InstructionBlock>) {
        let blocks = value;
        if !blocks.is_empty() {
            let block_idx_1 = rng.gen_range(0..blocks.len());
            let block_idx_2 = rng.gen_range(0..blocks.len());
            blocks.swap(block_idx_1, block_idx_2);
        }
    }
}

pub(crate) fn mutate(vec_instruction_block: &mut Vec<InstructionBlock>, rng: &mut StdRng) {
    match BASIC_VECTOR_OF_INSTRUCTION_BLOCKS_MUTATION_CONFIGURATION.select(rng) {
        VectorOfInstructionBlocksMutationOptions::Random => {
            RandomMutation::mutate(rng, vec_instruction_block)
        }
        VectorOfInstructionBlocksMutationOptions::InstructionBlockDeletion => {
            InstructionBlockDeletionMutation::mutate(rng, vec_instruction_block)
        }
        VectorOfInstructionBlocksMutationOptions::InstructionBlockInsertion => {
            InstructionBlockInsertionMutation::mutate(rng, vec_instruction_block)
        }
        VectorOfInstructionBlocksMutationOptions::InstructionBlockMutation => {
            InstructionBlockInstructionMutation::mutate(rng, vec_instruction_block)
        }
        VectorOfInstructionBlocksMutationOptions::InstructionBlockSwap => {
            InstructionBlockInstructionSwapMutation::mutate(rng, vec_instruction_block)
        }
    }
}
