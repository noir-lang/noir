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
    fn new_box() -> Box<dyn InstructionBlockMutator>;
}

/// Return new random instruction block
#[derive(Default)]
struct RandomMutation;
impl InstructionBlockMutator for RandomMutation {
    fn mutate(&self, rng: &mut StdRng, _value: InstructionBlock) -> InstructionBlock {
        let mut bytes = [0u8; 25];
        rng.fill(&mut bytes);
        Unstructured::new(&bytes).arbitrary().unwrap()
    }
}

/// Remove randomly chosen instruction from the block
#[derive(Default)]
struct InstructionBlockDeletionMutation;
impl InstructionBlockMutator for InstructionBlockDeletionMutation {
    fn mutate(&self, rng: &mut StdRng, value: InstructionBlock) -> InstructionBlock {
        let mut blocks = value.instructions;
        if !blocks.is_empty() {
            let block_idx = rng.gen_range(0..blocks.len());
            blocks.remove(block_idx);
        }
        InstructionBlock { instructions: blocks }
    }
}

/// Insert randomly generated instruction into the block
#[derive(Default)]
struct InstructionBlockInsertionMutation;
impl InstructionBlockMutator for InstructionBlockInsertionMutation {
    fn mutate(&self, rng: &mut StdRng, value: InstructionBlock) -> InstructionBlock {
        let mut bytes = [0u8; 25];
        rng.fill(&mut bytes);
        let mut unstructured = Unstructured::new(&bytes);
        let instruction = unstructured.arbitrary().unwrap();
        let mut blocks = value.instructions;
        blocks.insert(
            if blocks.is_empty() { 0 } else { rng.gen_range(0..blocks.len()) },
            instruction,
        );
        InstructionBlock { instructions: blocks }
    }
}

/// Mutate randomly chosen instruction in the block
#[derive(Default)]
struct InstructionBlockInstructionMutation;
impl InstructionBlockMutator for InstructionBlockInstructionMutation {
    fn mutate(&self, rng: &mut StdRng, value: InstructionBlock) -> InstructionBlock {
        let mut instructions = value.instructions;
        if !instructions.is_empty() {
            let instruction_idx = rng.gen_range(0..instructions.len());
            instructions[instruction_idx] = instruction_mutator(instructions[instruction_idx], rng);
        }
        InstructionBlock { instructions }
    }
}

impl<T> InstructionBlockMutatorFactory for T
where
    T: InstructionBlockMutator + Default + 'static,
{
    fn new_box() -> Box<dyn InstructionBlockMutator> {
        Box::new(T::default())
    }
}

fn mutation_factory(rng: &mut StdRng) -> Box<dyn InstructionBlockMutator> {
    match BASIC_INSTRUCTION_BLOCK_MUTATION_CONFIGURATION.select(rng) {
        InstructionBlockMutationOptions::Random => RandomMutation::new_box(),
        InstructionBlockMutationOptions::InstructionDeletion => {
            InstructionBlockDeletionMutation::new_box()
        }
        InstructionBlockMutationOptions::InstructionInsertion => {
            InstructionBlockInsertionMutation::new_box()
        }
        InstructionBlockMutationOptions::InstructionMutation => {
            InstructionBlockInstructionMutation::new_box()
        }
    }
}

pub(crate) fn instruction_block_mutator(
    instruction_block: InstructionBlock,
    rng: &mut StdRng,
) -> InstructionBlock {
    let mutator = mutation_factory(rng);
    mutator.mutate(rng, instruction_block)
}
