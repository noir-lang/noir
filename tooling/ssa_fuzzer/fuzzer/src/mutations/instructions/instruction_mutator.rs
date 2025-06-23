//! This file contains mechanisms for deterministically mutating a given [Instruction](crate::fuzz_lib::instruction::Instruction) value
//! Types of mutations applied:
//! 1. Random (randomly select a new instruction)
//! 2. Argument mutation

use crate::fuzz_lib::instruction::{Argument, Instruction};
use crate::mutations::configuration::{
    BASIC_INSTRUCTION_ARGUMENT_MUTATION_CONFIGURATION, BASIC_INSTRUCTION_MUTATION_CONFIGURATION,
    InstructionArgumentMutationOptions, InstructionMutationOptions,
};
use crate::mutations::instructions::argument_mutator::argument_mutator;
use crate::mutations::instructions::type_mutations::type_mutator;
use libfuzzer_sys::arbitrary::Unstructured;
use noir_ssa_fuzzer::typed_value::ValueType;
use rand::{Rng, rngs::StdRng};

trait InstructionMutator {
    fn mutate(rng: &mut StdRng, value: &mut Instruction);
}

/// Return new random instruction
struct RandomMutation;
impl InstructionMutator for RandomMutation {
    fn mutate(rng: &mut StdRng, value: &mut Instruction) {
        let mut bytes = [0u8; 17];
        rng.fill(&mut bytes);
        *value = Unstructured::new(&bytes).arbitrary().unwrap();
    }
}

/// Mutate arguments of the instruction
struct InstructionArgumentsMutation;
impl InstructionMutator for InstructionArgumentsMutation {
    fn mutate(rng: &mut StdRng, value: &mut Instruction) {
        match value {
            // Binary operations
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
                match BASIC_INSTRUCTION_ARGUMENT_MUTATION_CONFIGURATION.select(rng) {
                    InstructionArgumentMutationOptions::Left => {
                        argument_mutator(lhs, rng);
                    }
                    InstructionArgumentMutationOptions::Right => {
                        argument_mutator(rhs, rng);
                    }
                }
            }

            // Unary operations
            Instruction::Not { lhs }
            | Instruction::AddToMemory { lhs }
            | Instruction::LoadFromMemory { memory_addr: lhs } => {
                argument_mutator(lhs, rng);
            }

            // Special cases
            Instruction::Cast { lhs, type_ } => {
                match BASIC_INSTRUCTION_ARGUMENT_MUTATION_CONFIGURATION.select(rng) {
                    InstructionArgumentMutationOptions::Left => {
                        argument_mutator(lhs, rng);
                    }
                    InstructionArgumentMutationOptions::Right => {
                        type_mutator(type_, rng);
                    }
                }
            }
            Instruction::SetToMemory { memory_addr_index, value } => {
                match BASIC_INSTRUCTION_ARGUMENT_MUTATION_CONFIGURATION.select(rng) {
                    InstructionArgumentMutationOptions::Left => {
                        argument_mutator(value, rng);
                    }
                    InstructionArgumentMutationOptions::Right => {
                        let mut pseudo_argument =
                            Argument { index: *memory_addr_index, value_type: ValueType::U64 };
                        argument_mutator(&mut pseudo_argument, rng);
                        *memory_addr_index = pseudo_argument.index;
                    }
                }
            }

            Instruction::AddSubConstrain { lhs, rhs }
            | Instruction::MulDivConstrain { lhs, rhs } => {
                match BASIC_INSTRUCTION_ARGUMENT_MUTATION_CONFIGURATION.select(rng) {
                    InstructionArgumentMutationOptions::Left => {
                        let mut pseudo_argument =
                            Argument { index: *lhs, value_type: ValueType::U64 };
                        argument_mutator(&mut pseudo_argument, rng);
                        *lhs = pseudo_argument.index;
                    }
                    InstructionArgumentMutationOptions::Right => {
                        let mut pseudo_argument =
                            Argument { index: *rhs, value_type: ValueType::U64 };
                        argument_mutator(&mut pseudo_argument, rng);
                        *rhs = pseudo_argument.index;
                    }
                }
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
