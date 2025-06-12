use crate::fuzz_lib::instruction::InstructionBlock;
use crate::mutations::instructions::instruction_mutator::instruction_mutator;
use libfuzzer_sys::arbitrary::Unstructured;
use rand::{Rng, rngs::StdRng};

trait InstructionBlockMutator {
    fn mutate(&self, rng: &mut StdRng, value: InstructionBlock) -> InstructionBlock;
}

trait InstructionBlockMutatorFactory {
    fn new() -> Box<dyn InstructionBlockMutator>;
}

struct RandomMutation;
impl InstructionBlockMutator for RandomMutation {
    fn mutate(&self, rng: &mut StdRng, value: InstructionBlock) -> InstructionBlock {
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

struct InstructionBlockDeletionMutation;
impl InstructionBlockMutator for InstructionBlockDeletionMutation {
    fn mutate(&self, rng: &mut StdRng, value: InstructionBlock) -> InstructionBlock {
        let mut blocks = value.instructions;
        let block_idx = if blocks.len() == 0 { 0 } else { rng.gen_range(0..blocks.len()) };
        blocks.remove(block_idx);
        InstructionBlock { instructions: blocks }
    }
}
impl InstructionBlockMutatorFactory for InstructionBlockDeletionMutation {
    fn new() -> Box<dyn InstructionBlockMutator> {
        Box::new(InstructionBlockDeletionMutation)
    }
}

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

struct InstructionBlockInstructionMutation;
impl InstructionBlockMutator for InstructionBlockInstructionMutation {
    fn mutate(&self, rng: &mut StdRng, value: InstructionBlock) -> InstructionBlock {
        let mut instructions = value.instructions;
        let instruction_idx = rng.gen_range(0..instructions.len());
        instructions[instruction_idx] = instruction_mutator(instructions[instruction_idx], rng);
        InstructionBlock { instructions }
    }
}
impl InstructionBlockMutatorFactory for InstructionBlockInstructionMutation {
    fn new() -> Box<dyn InstructionBlockMutator> {
        Box::new(InstructionBlockInstructionMutation)
    }
}

struct InstructionBlockDefaultMutation;
impl InstructionBlockMutator for InstructionBlockDefaultMutation {
    fn mutate(&self, rng: &mut StdRng, value: InstructionBlock) -> InstructionBlock {
        value
    }
}
impl InstructionBlockMutatorFactory for InstructionBlockDefaultMutation {
    fn new() -> Box<dyn InstructionBlockMutator> {
        Box::new(InstructionBlockDefaultMutation)
    }
}

fn mutation_factory(rng: &mut StdRng) -> Box<dyn InstructionBlockMutator> {
    let mutator = if rng.gen_bool(0.7) {
        InstructionBlockInstructionMutation::new()
    } else if rng.gen_bool(0.1) {
        RandomMutation::new()
    } else if rng.gen_bool(0.2) {
        InstructionBlockInsertionMutation::new()
    } else if rng.gen_bool(0.2) {
        InstructionBlockInstructionMutation::new()
    } else {
        InstructionBlockDefaultMutation::new()
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
