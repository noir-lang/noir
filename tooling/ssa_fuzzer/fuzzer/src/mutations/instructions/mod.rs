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
    fn mutate(&self, rng: &mut StdRng, value: Vec<InstructionBlock>) -> Vec<InstructionBlock>;
}

trait MutateVecInstructionBlockFactory {
    fn new_box() -> Box<dyn MutateVecInstructionBlock>;
}

/// Return new random vector of instruction blocks
#[derive(Default)]
struct RandomMutation;
impl MutateVecInstructionBlock for RandomMutation {
    fn mutate(&self, rng: &mut StdRng, _value: Vec<InstructionBlock>) -> Vec<InstructionBlock> {
        let mut bytes = [0u8; 128];
        rng.fill(&mut bytes);
        Unstructured::new(&bytes).arbitrary().unwrap()
    }
}

/// Return vector of instruction blocks with one randomly chosen block removed
#[derive(Default)]
struct MutateInstructionBlockDeletionMutation;
impl MutateVecInstructionBlock for MutateInstructionBlockDeletionMutation {
    fn mutate(&self, rng: &mut StdRng, value: Vec<InstructionBlock>) -> Vec<InstructionBlock> {
        let mut blocks = value;
        if !blocks.is_empty() {
            let block_idx = rng.gen_range(0..blocks.len());
            blocks.remove(block_idx);
        }
        blocks
    }
}

/// Return vector of instruction blocks with one randomly generated block inserted
#[derive(Default)]
struct MutateInstructionBlockInsertionMutation;
impl MutateVecInstructionBlock for MutateInstructionBlockInsertionMutation {
    fn mutate(&self, rng: &mut StdRng, value: Vec<InstructionBlock>) -> Vec<InstructionBlock> {
        let mut blocks = value;
        let block_idx = if blocks.is_empty() { 0 } else { rng.gen_range(0..blocks.len()) };
        let mut bytes = [0u8; 25];
        rng.fill(&mut bytes);
        let mut unstructured = Unstructured::new(&bytes);
        let instruction_block = unstructured.arbitrary().unwrap();
        blocks.insert(block_idx, instruction_block);
        blocks
    }
}

/// Return vector of instruction blocks with one randomly chosen block mutated
#[derive(Default)]
struct MutateInstructionBlockInstructionMutation;
impl MutateVecInstructionBlock for MutateInstructionBlockInstructionMutation {
    fn mutate(&self, rng: &mut StdRng, value: Vec<InstructionBlock>) -> Vec<InstructionBlock> {
        let mut blocks = value;
        if !blocks.is_empty() {
            let block_idx = rng.gen_range(0..blocks.len());
            blocks[block_idx] = instruction_block_mutator(blocks[block_idx].clone(), rng);
        }
        blocks
    }
}

impl<T> MutateVecInstructionBlockFactory for T
where
    T: MutateVecInstructionBlock + Default + 'static,
{
    fn new_box() -> Box<dyn MutateVecInstructionBlock> {
        Box::new(T::default())
    }
}

fn mutation_factory(rng: &mut StdRng) -> Box<dyn MutateVecInstructionBlock> {
    match BASIC_VECTOR_OF_INSTRUCTION_BLOCKS_MUTATION_CONFIGURATION.select(rng) {
        VectorOfInstructionBlocksMutationOptions::Random => RandomMutation::new_box(),
        VectorOfInstructionBlocksMutationOptions::InstructionBlockDeletion => {
            MutateInstructionBlockDeletionMutation::new_box()
        }
        VectorOfInstructionBlocksMutationOptions::InstructionBlockInsertion => {
            MutateInstructionBlockInsertionMutation::new_box()
        }
        VectorOfInstructionBlocksMutationOptions::InstructionBlockMutation => {
            MutateInstructionBlockInstructionMutation::new_box()
        }
    }
}

pub(crate) fn mutate_vec_instruction_block(
    vec_instruction_block: Vec<InstructionBlock>,
    rng: &mut StdRng,
) -> Vec<InstructionBlock> {
    let mutator = mutation_factory(rng);
    mutator.mutate(rng, vec_instruction_block)
}
