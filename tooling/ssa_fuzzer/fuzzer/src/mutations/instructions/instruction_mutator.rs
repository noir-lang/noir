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
    fn mutate(rng: &mut StdRng, value: &mut Instruction);
}

/// Return new random instruction
struct RandomMutation;
impl InstructionMutator for RandomMutation {
    fn mutate(rng: &mut StdRng, _value: &mut Instruction) {
        let mut bytes = [0u8; 17];
        rng.fill(&mut bytes);
        Unstructured::new(&bytes).arbitrary().unwrap()
    }
}

/// Mutate arguments of the instruction
struct InstructionArgumentsMutation;
impl InstructionMutator for InstructionArgumentsMutation {
    fn mutate(rng: &mut StdRng, value: &mut Instruction) {
        match value {
            // Binary operations with lhs and rhs
            Instruction::AddChecked { lhs, rhs }
            | Instruction::SubChecked { lhs, rhs }
            | Instruction::MulChecked { lhs, rhs }
            | Instruction::Div { lhs, rhs }
            | Instruction::Eq { lhs, rhs }
            | Instruction::Mod { lhs, rhs }
            | Instruction::Shl { lhs, rhs }
            | Instruction::Shr { lhs, rhs }
            | Instruction::And { lhs, rhs }
            | Instruction::Or { lhs, rhs }
            | Instruction::Xor { lhs, rhs }
            | Instruction::Lt { lhs, rhs } => {
                argument_mutator(lhs, rng);
                argument_mutator(rhs, rng);
            }

            // Unary operations with just lhs
            Instruction::Not { lhs } | Instruction::AddToMemory { lhs } => {
                argument_mutator(lhs, rng);
            }

            // Special cases
            Instruction::Cast { lhs, type_: _ } => {
                argument_mutator(lhs, rng);
                // Note: not mutating type_ for now
            }
            Instruction::LoadFromMemory { memory_addr } => {
                argument_mutator(memory_addr, rng);
            }
            Instruction::SetToMemory { memory_addr_index: _, value } => {
                argument_mutator(value, rng);
                // Note: not mutating memory_addr_index for now
            }

            // TODO: mutation of these instructions
            Instruction::AddSubConstrain { lhs: _, rhs: _ }
            | Instruction::MulDivConstrain { lhs: _, rhs: _ } => {
                // No mutation for now
            }
        }
    }
}

pub(crate) fn instruction_mutator(instruction: &mut Instruction, rng: &mut StdRng) {
    match BASIC_INSTRUCTION_MUTATION_CONFIGURATION.select(rng) {
        InstructionMutationOptions::Random => RandomMutation::mutate(rng, instruction),
        InstructionMutationOptions::ArgumentMutation => {
            InstructionArgumentsMutation::mutate(rng, instruction)
        }
    }
}
