//! This file contains mechanisms for deterministically mutating a given [InstructionBlock](crate::fuzz_lib::instruction::InstructionBlock) value

use crate::fuzz_lib::instruction::{Argument, Instruction, InstructionBlock};
use crate::mutations::{
    basic_types::{
        bool::generate_random_bool, value_type::generate_random_value_type, vec::mutate_vec,
    },
    configuration::{
        BASIC_GENERATE_BOOL_CONFIGURATION, BASIC_GENERATE_INSTRUCTION_CONFIGURATION,
        BASIC_GENERATE_VALUE_TYPE_CONFIGURATION, BASIC_VEC_MUTATION_CONFIGURATION,
        GENERATE_BOOL_CONFIGURATION_MOST_FALSE, GENERATE_BOOL_CONFIGURATION_MOST_TRUE,
        GenerateInstruction, SIZE_OF_SMALL_ARBITRARY_BUFFER,
    },
    instructions::instruction_mutator::instruction_mutator,
};
use libfuzzer_sys::arbitrary::Unstructured;
use rand::{Rng, rngs::StdRng};

fn generate_random_argument(rng: &mut StdRng) -> Argument {
    let mut buf = [0u8; SIZE_OF_SMALL_ARBITRARY_BUFFER];
    rng.fill(&mut buf);
    let mut unstructured = Unstructured::new(&buf);
    unstructured.arbitrary().unwrap()
}

fn generate_random_instruction(rng: &mut StdRng) -> Instruction {
    match BASIC_GENERATE_INSTRUCTION_CONFIGURATION.select(rng) {
        GenerateInstruction::AddChecked => Instruction::AddChecked {
            lhs: generate_random_argument(rng),
            rhs: generate_random_argument(rng),
        },
        GenerateInstruction::SubChecked => Instruction::SubChecked {
            lhs: generate_random_argument(rng),
            rhs: generate_random_argument(rng),
        },
        GenerateInstruction::MulChecked => Instruction::MulChecked {
            lhs: generate_random_argument(rng),
            rhs: generate_random_argument(rng),
        },
        GenerateInstruction::Div => Instruction::Div {
            lhs: generate_random_argument(rng),
            rhs: generate_random_argument(rng),
        },
        GenerateInstruction::Eq => Instruction::Eq {
            lhs: generate_random_argument(rng),
            rhs: generate_random_argument(rng),
        },
        GenerateInstruction::Mod => Instruction::Mod {
            lhs: generate_random_argument(rng),
            rhs: generate_random_argument(rng),
        },
        GenerateInstruction::Not => Instruction::Not { lhs: generate_random_argument(rng) },
        GenerateInstruction::Shl => Instruction::Shl {
            lhs: generate_random_argument(rng),
            rhs: generate_random_argument(rng),
        },
        GenerateInstruction::Shr => Instruction::Shr {
            lhs: generate_random_argument(rng),
            rhs: generate_random_argument(rng),
        },
        GenerateInstruction::Cast => Instruction::Cast {
            lhs: generate_random_argument(rng),
            type_: generate_random_value_type(rng, BASIC_GENERATE_VALUE_TYPE_CONFIGURATION),
        },
        GenerateInstruction::And => Instruction::And {
            lhs: generate_random_argument(rng),
            rhs: generate_random_argument(rng),
        },
        GenerateInstruction::Or => Instruction::Or {
            lhs: generate_random_argument(rng),
            rhs: generate_random_argument(rng),
        },
        GenerateInstruction::Xor => Instruction::Xor {
            lhs: generate_random_argument(rng),
            rhs: generate_random_argument(rng),
        },
        GenerateInstruction::Lt => Instruction::Lt {
            lhs: generate_random_argument(rng),
            rhs: generate_random_argument(rng),
        },
        GenerateInstruction::AddSubConstrain => Instruction::AddSubConstrain {
            lhs: rng.gen_range(usize::MIN..usize::MAX),
            rhs: rng.gen_range(usize::MIN..usize::MAX),
        },
        GenerateInstruction::MulDivConstrain => Instruction::MulDivConstrain {
            lhs: rng.gen_range(usize::MIN..usize::MAX),
            rhs: rng.gen_range(usize::MIN..usize::MAX),
        },
        GenerateInstruction::AddToMemory => {
            Instruction::AddToMemory { lhs: generate_random_argument(rng) }
        }
        GenerateInstruction::LoadFromMemory => {
            Instruction::LoadFromMemory { memory_addr: generate_random_argument(rng) }
        }
        GenerateInstruction::SetToMemory => Instruction::SetToMemory {
            memory_addr_index: rng.gen_range(usize::MIN..usize::MAX),
            value: generate_random_argument(rng),
        },
        GenerateInstruction::CreateArray => Instruction::CreateArray {
            elements_indices: vec![rng.gen_range(usize::MIN..usize::MAX); 10],
            element_type: generate_random_value_type(rng, BASIC_GENERATE_VALUE_TYPE_CONFIGURATION),
            is_references: generate_random_bool(rng, BASIC_GENERATE_BOOL_CONFIGURATION),
        },
        GenerateInstruction::ArrayGet => Instruction::ArrayGet {
            array_index: rng.gen_range(usize::MIN..usize::MAX),
            index: generate_random_argument(rng),
            safe_index: generate_random_bool(rng, GENERATE_BOOL_CONFIGURATION_MOST_TRUE),
        },
        GenerateInstruction::ArraySet => Instruction::ArraySet {
            array_index: rng.gen_range(usize::MIN..usize::MAX),
            index: generate_random_argument(rng),
            value_index: rng.gen_range(usize::MIN..usize::MAX),
            safe_index: generate_random_bool(rng, GENERATE_BOOL_CONFIGURATION_MOST_TRUE),
        },
        GenerateInstruction::ArrayGetWithConstantIndex => Instruction::ArrayGetWithConstantIndex {
            array_index: rng.gen_range(usize::MIN..usize::MAX),
            index: rng.gen_range(usize::MIN..usize::MAX),
            safe_index: generate_random_bool(rng, GENERATE_BOOL_CONFIGURATION_MOST_TRUE),
        },
        GenerateInstruction::ArraySetWithConstantIndex => Instruction::ArraySetWithConstantIndex {
            array_index: rng.gen_range(usize::MIN..usize::MAX),
            index: rng.gen_range(usize::MIN..usize::MAX),
            value_index: rng.gen_range(usize::MIN..usize::MAX),
            safe_index: generate_random_bool(rng, GENERATE_BOOL_CONFIGURATION_MOST_TRUE),
        },
        GenerateInstruction::FieldToBytesToField => {
            Instruction::FieldToBytesToField { field_idx: rng.gen_range(usize::MIN..usize::MAX) }
        }
        GenerateInstruction::Blake2sHash => Instruction::Blake2sHash {
            field_idx: rng.gen_range(usize::MIN..usize::MAX),
            limbs_count: rng.gen_range(u8::MIN..u8::MAX),
        },
        GenerateInstruction::Blake3Hash => Instruction::Blake3Hash {
            field_idx: rng.gen_range(usize::MIN..usize::MAX),
            limbs_count: rng.gen_range(u8::MIN..u8::MAX),
        },
        GenerateInstruction::Keccakf1600Hash => Instruction::Keccakf1600Hash {
            u64_indices: [rng.gen_range(usize::MIN..usize::MAX); 25],
            load_elements_of_array: generate_random_bool(
                rng,
                GENERATE_BOOL_CONFIGURATION_MOST_FALSE,
            ),
        },
        GenerateInstruction::Aes128Encrypt => Instruction::Aes128Encrypt {
            input_idx: rng.gen_range(usize::MIN..usize::MAX),
            input_limbs_count: rng.gen_range(u8::MIN..u8::MAX),
            key_idx: rng.gen_range(usize::MIN..usize::MAX),
            iv_idx: rng.gen_range(usize::MIN..usize::MAX),
        },
        GenerateInstruction::Sha256Compression => Instruction::Sha256Compression {
            input_indices: [rng.gen_range(usize::MIN..usize::MAX); 16],
            state_indices: [rng.gen_range(usize::MIN..usize::MAX); 8],
            load_elements_of_array: generate_random_bool(
                rng,
                GENERATE_BOOL_CONFIGURATION_MOST_FALSE,
            ),
        },
        GenerateInstruction::MultiScalarMul { .. } | GenerateInstruction::PointAdd { .. } => {
            unimplemented!()
        }
    }
}

pub(crate) fn instruction_block_mutator(
    instruction_block: &mut InstructionBlock,
    rng: &mut StdRng,
) {
    mutate_vec(
        &mut instruction_block.instructions,
        rng,
        instruction_mutator,
        generate_random_instruction,
        BASIC_VEC_MUTATION_CONFIGURATION,
    );
}
