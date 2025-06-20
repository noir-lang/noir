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
    fn mutate(&self, rng: &mut StdRng, value: InstructionBlock) -> InstructionBlock;
}

trait InstructionBlockMutatorFactory {
    fn new() -> Box<dyn InstructionBlockMutator>;
}

/// Return new random instruction block
struct RandomMutation;
impl InstructionBlockMutator for RandomMutation {
    fn mutate(&self, rng: &mut StdRng, _value: InstructionBlock) -> InstructionBlock {
        let mut bytes = [0u8; 25];
        rng.fill(&mut bytes);
        Unstructured::new(&bytes).arbitrary().unwrap()
    }
}
impl InstructionBlockMutatorFactory for RandomMutation {
    fn new() -> Box<dyn InstructionBlockMutator> {
        Box::new(RandomMutation)
    }
}

/// Remove randomly chosen instruction from the block
struct InstructionBlockDeletionMutation;
impl InstructionBlockMutator for InstructionBlockDeletionMutation {
    fn mutate(&self, rng: &mut StdRng, value: InstructionBlock) -> InstructionBlock {
        let mut blocks = value.instructions;
        if blocks.len() > 0 {
            let block_idx = rng.gen_range(0..blocks.len());
            blocks.remove(block_idx);
        }
        InstructionBlock { instructions: blocks }
    }
}
impl InstructionBlockMutatorFactory for InstructionBlockDeletionMutation {
    fn new() -> Box<dyn InstructionBlockMutator> {
        Box::new(InstructionBlockDeletionMutation)
    }
}

/// Insert randomly generated instruction into the block
struct InstructionBlockInsertionMutation;
impl InstructionBlockMutator for InstructionBlockInsertionMutation {
    fn mutate(&self, rng: &mut StdRng, value: InstructionBlock) -> InstructionBlock {
        let mut bytes = [0u8; 25];
        rng.fill(&mut bytes);
        let mut unstructured = Unstructured::new(&bytes);
        let instruction = unstructured.arbitrary().unwrap();
        let mut blocks = value.instructions;
        blocks.insert(
            if blocks.len() == 0 { 0 } else { rng.gen_range(0..blocks.len()) },
            instruction,
        );
        InstructionBlock { instructions: blocks }
    }
}
impl InstructionBlockMutatorFactory for InstructionBlockInsertionMutation {
    fn new() -> Box<dyn InstructionBlockMutator> {
        Box::new(InstructionBlockInsertionMutation)
    }
}

/// Mutate randomly chosen instruction in the block
struct InstructionBlockInstructionMutation;
impl InstructionBlockMutator for InstructionBlockInstructionMutation {
    fn mutate(&self, rng: &mut StdRng, value: InstructionBlock) -> InstructionBlock {
        let mut instructions = value.instructions;
        if instructions.len() > 0 {
            let instruction_idx = rng.gen_range(0..instructions.len());
            instructions[instruction_idx] = instruction_mutator(instructions[instruction_idx], rng);
        }
        InstructionBlock { instructions }
    }
}
impl InstructionBlockMutatorFactory for InstructionBlockInstructionMutation {
    fn new() -> Box<dyn InstructionBlockMutator> {
        Box::new(InstructionBlockInstructionMutation)
    }
}

fn mutation_factory(rng: &mut StdRng) -> Box<dyn InstructionBlockMutator> {
    let mutator = match BASIC_INSTRUCTION_BLOCK_MUTATION_CONFIGURATION.select(rng) {
        InstructionBlockMutationOptions::Random => RandomMutation::new(),
        InstructionBlockMutationOptions::InstructionDeletion => {
            InstructionBlockDeletionMutation::new()
        }
        InstructionBlockMutationOptions::InstructionInsertion => {
            InstructionBlockInsertionMutation::new()
        }
        InstructionBlockMutationOptions::InstructionMutation => {
            InstructionBlockInstructionMutation::new()
        }
    };
    mutator
}

pub(crate) fn instruction_block_mutator(
    instruction_block: InstructionBlock,
    rng: &mut StdRng,
) -> InstructionBlock {
    let mutator = mutation_factory(rng);
    mutator.mutate(rng, instruction_block)
}
