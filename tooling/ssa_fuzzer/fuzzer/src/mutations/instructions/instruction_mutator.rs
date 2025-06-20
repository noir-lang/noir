//! This file contains mechanisms for deterministically mutating a given [Instruction](crate::fuzz_lib::instruction::Instruction) value
//! Types of mutations applied:
//! 1. Random (randomly select a new instruction)
//! 2. Argument mutation

use crate::fuzz_lib::instruction::Instruction;
use crate::mutations::configuration::{
    BASIC_INSTRUCTION_MUTATION_CONFIGURATION, InstructionMutationOptions,
};
use crate::mutations::instructions::argument_mutator::argument_mutator;
use libfuzzer_sys::arbitrary::Unstructured;
use rand::{Rng, rngs::StdRng};

trait InstructionMutator {
    fn mutate(&self, rng: &mut StdRng, value: Instruction) -> Instruction;
}
trait InstructionMutatorFactory {
    fn new_box() -> Box<dyn InstructionMutator>;
}

/// Return new random instruction
struct RandomMutation;
impl InstructionMutator for RandomMutation {
    fn mutate(&self, rng: &mut StdRng, _value: Instruction) -> Instruction {
        let mut bytes = [0u8; 17];
        rng.fill(&mut bytes);
        Unstructured::new(&bytes).arbitrary().unwrap()
    }
}
impl InstructionMutatorFactory for RandomMutation {
    fn new_box() -> Box<dyn InstructionMutator> {
        Box::new(RandomMutation)
    }
}

/// Mutate arguments of the instruction
struct InstructionArgumentsMutation;
impl InstructionMutator for InstructionArgumentsMutation {
    fn mutate(&self, rng: &mut StdRng, value: Instruction) -> Instruction {
        match value {
            Instruction::AddChecked { lhs, rhs } => Instruction::AddChecked {
                lhs: argument_mutator(lhs, rng),
                rhs: argument_mutator(rhs, rng),
            },
            Instruction::SubChecked { lhs, rhs } => Instruction::SubChecked {
                lhs: argument_mutator(lhs, rng),
                rhs: argument_mutator(rhs, rng),
            },
            Instruction::MulChecked { lhs, rhs } => Instruction::MulChecked {
                lhs: argument_mutator(lhs, rng),
                rhs: argument_mutator(rhs, rng),
            },
            Instruction::Div { lhs, rhs } => Instruction::Div {
                lhs: argument_mutator(lhs, rng),
                rhs: argument_mutator(rhs, rng),
            },
            Instruction::Eq { lhs, rhs } => {
                Instruction::Eq { lhs: argument_mutator(lhs, rng), rhs: argument_mutator(rhs, rng) }
            }
            Instruction::Mod { lhs, rhs } => Instruction::Mod {
                lhs: argument_mutator(lhs, rng),
                rhs: argument_mutator(rhs, rng),
            },
            Instruction::Not { lhs } => Instruction::Not { lhs: argument_mutator(lhs, rng) },
            Instruction::Shl { lhs, rhs } => Instruction::Shl {
                lhs: argument_mutator(lhs, rng),
                rhs: argument_mutator(rhs, rng),
            },
            Instruction::Shr { lhs, rhs } => Instruction::Shr {
                lhs: argument_mutator(lhs, rng),
                rhs: argument_mutator(rhs, rng),
            },
            Instruction::Cast { lhs, type_ } => {
                Instruction::Cast { lhs: argument_mutator(lhs, rng), type_ }
            }
            Instruction::And { lhs, rhs } => Instruction::And {
                lhs: argument_mutator(lhs, rng),
                rhs: argument_mutator(rhs, rng),
            },
            Instruction::Or { lhs, rhs } => {
                Instruction::Or { lhs: argument_mutator(lhs, rng), rhs: argument_mutator(rhs, rng) }
            }
            Instruction::Xor { lhs, rhs } => Instruction::Xor {
                lhs: argument_mutator(lhs, rng),
                rhs: argument_mutator(rhs, rng),
            },
            Instruction::Lt { lhs, rhs } => {
                Instruction::Lt { lhs: argument_mutator(lhs, rng), rhs: argument_mutator(rhs, rng) }
            }
            // TODO: mutation of these instructions
            Instruction::AddSubConstrain { lhs, rhs } => Instruction::AddSubConstrain { lhs, rhs },
            Instruction::MulDivConstrain { lhs, rhs } => Instruction::MulDivConstrain { lhs, rhs },
            Instruction::AddToMemory { lhs } => {
                Instruction::AddToMemory { lhs: argument_mutator(lhs, rng) }
            }
            Instruction::LoadFromMemory { memory_addr } => {
                Instruction::LoadFromMemory { memory_addr: argument_mutator(memory_addr, rng) }
            }
            Instruction::SetToMemory { memory_addr_index, value } => {
                Instruction::SetToMemory { memory_addr_index, value: argument_mutator(value, rng) }
            }
        }
    }
}
impl InstructionMutatorFactory for InstructionArgumentsMutation {
    fn new_box() -> Box<dyn InstructionMutator> {
        Box::new(InstructionArgumentsMutation)
    }
}

fn mutation_factory(rng: &mut StdRng) -> Box<dyn InstructionMutator> {
    match BASIC_INSTRUCTION_MUTATION_CONFIGURATION.select(rng) {
        InstructionMutationOptions::Random => RandomMutation::new_box(),
        InstructionMutationOptions::ArgumentMutation => InstructionArgumentsMutation::new_box(),
    }
}

pub(crate) fn instruction_mutator(instruction: Instruction, rng: &mut StdRng) -> Instruction {
    let mutator = mutation_factory(rng);
    mutator.mutate(rng, instruction)
}
