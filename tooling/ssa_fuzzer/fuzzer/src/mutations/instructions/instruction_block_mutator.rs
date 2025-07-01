//! This file contains mechanisms for deterministically mutating a given [InstructionBlock](crate::fuzz_lib::instruction::InstructionBlock) value
//! Types of mutations applied:
//! 1. Random (randomly select a new instruction block)
//! 2. Instruction deletion
//! 3. Instruction insertion
//! 4. Instruction mutation

use crate::fuzz_lib::instruction::InstructionBlock;
use crate::mutations::configuration::{
    BASIC_INSTRUCTION_BLOCK_MUTATION_CONFIGURATION, InstructionBlockMutationOptions,
};
use crate::mutations::instructions::instruction_mutator::instruction_mutator;
use libfuzzer_sys::arbitrary::Unstructured;
use rand::{Rng, rngs::StdRng};

trait InstructionBlockMutator {
    fn mutate(rng: &mut StdRng, value: &mut InstructionBlock);
}

/// Return new random instruction block
struct RandomMutation;
impl InstructionBlockMutator for RandomMutation {
    fn mutate(rng: &mut StdRng, value: &mut InstructionBlock) {
        let mut bytes = [0u8; 25];
        rng.fill(&mut bytes);
        *value = Unstructured::new(&bytes).arbitrary().unwrap();
    }
}

/// Remove randomly chosen instruction from the block
struct InstructionBlockDeletionMutation;
impl InstructionBlockMutator for InstructionBlockDeletionMutation {
    fn mutate(rng: &mut StdRng, value: &mut InstructionBlock) {
        let blocks = &mut value.instructions;
        if !blocks.is_empty() {
            let block_idx = rng.gen_range(0..blocks.len());
            blocks.remove(block_idx);
        }
    }
}

/// Insert randomly generated instruction into the block
struct InstructionBlockInsertionMutation;
impl InstructionBlockMutator for InstructionBlockInsertionMutation {
    fn mutate(rng: &mut StdRng, value: &mut InstructionBlock) {
        let mut bytes = [0u8; 25];
        rng.fill(&mut bytes);
        let mut unstructured = Unstructured::new(&bytes);
        let instruction = unstructured.arbitrary().unwrap();
        let blocks = &mut value.instructions;
        blocks.insert(
            if blocks.is_empty() { 0 } else { rng.gen_range(0..blocks.len()) },
            instruction,
        );
    }
}

/// Mutate randomly chosen instruction in the block
struct InstructionBlockInstructionMutation;
impl InstructionBlockMutator for InstructionBlockInstructionMutation {
    fn mutate(rng: &mut StdRng, value: &mut InstructionBlock) {
        let instructions = &mut value.instructions;
        if !instructions.is_empty() {
            let instruction_idx = rng.gen_range(0..instructions.len());
            instruction_mutator(&mut instructions[instruction_idx], rng);
        }
    }
}

/// Swap randomly chosen instruction in the block
struct InstructionBlockInstructionSwapMutation;
impl InstructionBlockMutator for InstructionBlockInstructionSwapMutation {
    fn mutate(rng: &mut StdRng, value: &mut InstructionBlock) {
        let instructions = &mut value.instructions;
        if !instructions.is_empty() {
            let instruction_idx_1 = rng.gen_range(0..instructions.len());
            let instruction_idx_2 = rng.gen_range(0..instructions.len());
            instructions.swap(instruction_idx_1, instruction_idx_2);
        }
    }
}

pub(crate) fn instruction_block_mutator(
    instruction_block: &mut InstructionBlock,
    rng: &mut StdRng,
) {
    match BASIC_INSTRUCTION_BLOCK_MUTATION_CONFIGURATION.select(rng) {
        InstructionBlockMutationOptions::Random => RandomMutation::mutate(rng, instruction_block),
        InstructionBlockMutationOptions::InstructionDeletion => {
            InstructionBlockDeletionMutation::mutate(rng, instruction_block)
        }
        InstructionBlockMutationOptions::InstructionInsertion => {
            InstructionBlockInsertionMutation::mutate(rng, instruction_block)
        }
        InstructionBlockMutationOptions::InstructionMutation => {
            InstructionBlockInstructionMutation::mutate(rng, instruction_block)
        }
        InstructionBlockMutationOptions::InstructionSwap => {
            InstructionBlockInstructionSwapMutation::mutate(rng, instruction_block)
        }
    }
}
