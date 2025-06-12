mod argument_mutator;
mod instruction_block_mutator;
mod instruction_mutator;
use crate::fuzz_lib::instruction::InstructionBlock;
use instruction_block_mutator::instruction_block_mutator;
use libfuzzer_sys::arbitrary::Unstructured;
use rand::{Rng, rngs::StdRng};

trait MutateVecInstructionBlock {
    fn mutate(&self, rng: &mut StdRng, value: Vec<InstructionBlock>) -> Vec<InstructionBlock>;
}

trait MutateVecInstructionBlockFactory {
    fn new() -> Box<dyn MutateVecInstructionBlock>;
}

struct RandomMutation;
impl MutateVecInstructionBlock for RandomMutation {
    fn mutate(&self, rng: &mut StdRng, value: Vec<InstructionBlock>) -> Vec<InstructionBlock> {
        let mut bytes = [0u8; 128];
        rng.fill(&mut bytes);
        Unstructured::new(&bytes).arbitrary().unwrap()
    }
}
impl MutateVecInstructionBlockFactory for RandomMutation {
    fn new() -> Box<dyn MutateVecInstructionBlock> {
        Box::new(RandomMutation)
    }
}

struct MutateInstructionBlockDeletionMutation;
impl MutateVecInstructionBlock for MutateInstructionBlockDeletionMutation {
    fn mutate(&self, rng: &mut StdRng, value: Vec<InstructionBlock>) -> Vec<InstructionBlock> {
        let mut blocks = value;
        let block_idx = if blocks.len() == 0 { 0 } else { rng.gen_range(0..blocks.len()) };
        blocks.remove(block_idx);
        blocks
    }
}
impl MutateVecInstructionBlockFactory for MutateInstructionBlockDeletionMutation {
    fn new() -> Box<dyn MutateVecInstructionBlock> {
        Box::new(MutateInstructionBlockDeletionMutation)
    }
}

struct MutateInstructionBlockInsertionMutation;
impl MutateVecInstructionBlock for MutateInstructionBlockInsertionMutation {
    fn mutate(&self, rng: &mut StdRng, value: Vec<InstructionBlock>) -> Vec<InstructionBlock> {
        let mut blocks = value;
        let block_idx = if blocks.len() == 0 { 0 } else { rng.gen_range(0..blocks.len()) };
        let mut bytes = [0u8; 25];
        rng.fill(&mut bytes);
        let mut unstructured = Unstructured::new(&bytes);
        let instruction_block = unstructured.arbitrary().unwrap();
        blocks.insert(block_idx, instruction_block);
        blocks
    }
}
impl MutateVecInstructionBlockFactory for MutateInstructionBlockInsertionMutation {
    fn new() -> Box<dyn MutateVecInstructionBlock> {
        Box::new(MutateInstructionBlockInsertionMutation)
    }
}

struct MutateInstructionBlockInstructionMutation;
impl MutateVecInstructionBlock for MutateInstructionBlockInstructionMutation {
    fn mutate(&self, rng: &mut StdRng, value: Vec<InstructionBlock>) -> Vec<InstructionBlock> {
        let mut blocks = value;
        let block_idx = if blocks.len() == 0 { 0 } else { rng.gen_range(0..blocks.len()) };
        blocks[block_idx] = instruction_block_mutator(blocks[block_idx].clone(), rng);
        blocks
    }
}
impl MutateVecInstructionBlockFactory for MutateInstructionBlockInstructionMutation {
    fn new() -> Box<dyn MutateVecInstructionBlock> {
        Box::new(MutateInstructionBlockInstructionMutation)
    }
}

struct MutateInstructionBlockDefaultMutation;
impl MutateVecInstructionBlock for MutateInstructionBlockDefaultMutation {
    fn mutate(&self, rng: &mut StdRng, value: Vec<InstructionBlock>) -> Vec<InstructionBlock> {
        value
    }
}
impl MutateVecInstructionBlockFactory for MutateInstructionBlockDefaultMutation {
    fn new() -> Box<dyn MutateVecInstructionBlock> {
        Box::new(MutateInstructionBlockDefaultMutation)
    }
}

fn mutation_factory(rng: &mut StdRng) -> Box<dyn MutateVecInstructionBlock> {
    let mutator = if rng.gen_bool(0.5) {
        RandomMutation::new()
    } else if rng.gen_bool(0.3) {
        MutateInstructionBlockDeletionMutation::new()
    } else if rng.gen_bool(0.2) {
        MutateInstructionBlockInsertionMutation::new()
    } else if rng.gen_bool(0.2) {
        MutateInstructionBlockInstructionMutation::new()
    } else {
        MutateInstructionBlockDefaultMutation::new()
    };
    mutator
}

pub(crate) fn mutate_vec_instruction_block(
    vec_instruction_block: Vec<InstructionBlock>,
    rng: &mut StdRng,
) -> Vec<InstructionBlock> {
    let mutator = mutation_factory(rng);
    mutator.mutate(rng, vec_instruction_block)
}
