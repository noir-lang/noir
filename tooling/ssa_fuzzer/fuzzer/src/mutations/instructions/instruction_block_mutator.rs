//! This file contains mechanisms for deterministically mutating a given [InstructionBlock](crate::fuzz_lib::instruction::InstructionBlock) value

use crate::fuzz_lib::instruction::{Argument, Instruction, InstructionBlock, NumericArgument};
use crate::mutations::configuration::SIZE_OF_LARGE_ARBITRARY_BUFFER;
use crate::mutations::{
    basic_types::{
        bool::generate_random_bool, numeric_type::generate_random_numeric_type,
        point::generate_random_point, scalar::generate_random_scalar, vec::mutate_vec,
    },
    configuration::{
        BASIC_GENERATE_INSTRUCTION_CONFIGURATION, BASIC_GENERATE_NUMERIC_TYPE_CONFIGURATION,
        BASIC_VEC_MUTATION_CONFIGURATION, GENERATE_BOOL_CONFIGURATION_MOST_FALSE,
        GENERATE_BOOL_CONFIGURATION_MOST_TRUE, GenerateInstruction, SIZE_OF_SMALL_ARBITRARY_BUFFER,
    },
    instructions::instruction_mutator::instruction_mutator,
};
use libfuzzer_sys::arbitrary::Unstructured;
use rand::{Rng, RngCore, rngs::StdRng};

fn generate_random_numeric_argument(rng: &mut StdRng) -> NumericArgument {
    let mut buf = [0u8; SIZE_OF_SMALL_ARBITRARY_BUFFER];
    rng.fill(&mut buf);
    let mut unstructured = Unstructured::new(&buf);
    unstructured.arbitrary().unwrap()
}

fn generate_random_argument(rng: &mut StdRng) -> Argument {
    let mut buf = [0u8; SIZE_OF_SMALL_ARBITRARY_BUFFER];
    rng.fill(&mut buf);
    let mut unstructured = Unstructured::new(&buf);
    unstructured.arbitrary().unwrap()
}

fn generate_random_instruction(rng: &mut StdRng) -> Instruction {
    match BASIC_GENERATE_INSTRUCTION_CONFIGURATION.select(rng) {
        GenerateInstruction::AddChecked => Instruction::AddChecked {
            lhs: generate_random_numeric_argument(rng),
            rhs: generate_random_numeric_argument(rng),
        },
        GenerateInstruction::SubChecked => Instruction::SubChecked {
            lhs: generate_random_numeric_argument(rng),
            rhs: generate_random_numeric_argument(rng),
        },
        GenerateInstruction::MulChecked => Instruction::MulChecked {
            lhs: generate_random_numeric_argument(rng),
            rhs: generate_random_numeric_argument(rng),
        },
        GenerateInstruction::Div => Instruction::Div {
            lhs: generate_random_numeric_argument(rng),
            rhs: generate_random_numeric_argument(rng),
        },
        GenerateInstruction::Eq => Instruction::Eq {
            lhs: generate_random_numeric_argument(rng),
            rhs: generate_random_numeric_argument(rng),
        },
        GenerateInstruction::Mod => Instruction::Mod {
            lhs: generate_random_numeric_argument(rng),
            rhs: generate_random_numeric_argument(rng),
        },
        GenerateInstruction::Not => Instruction::Not { lhs: generate_random_numeric_argument(rng) },
        GenerateInstruction::Shl => Instruction::Shl {
            lhs: generate_random_numeric_argument(rng),
            rhs: generate_random_numeric_argument(rng),
        },
        GenerateInstruction::Shr => Instruction::Shr {
            lhs: generate_random_numeric_argument(rng),
            rhs: generate_random_numeric_argument(rng),
        },
        GenerateInstruction::Cast => Instruction::Cast {
            lhs: generate_random_numeric_argument(rng),
            type_: generate_random_numeric_type(rng, BASIC_GENERATE_NUMERIC_TYPE_CONFIGURATION),
        },
        GenerateInstruction::And => Instruction::And {
            lhs: generate_random_numeric_argument(rng),
            rhs: generate_random_numeric_argument(rng),
        },
        GenerateInstruction::Or => Instruction::Or {
            lhs: generate_random_numeric_argument(rng),
            rhs: generate_random_numeric_argument(rng),
        },
        GenerateInstruction::Xor => Instruction::Xor {
            lhs: generate_random_numeric_argument(rng),
            rhs: generate_random_numeric_argument(rng),
        },
        GenerateInstruction::Lt => Instruction::Lt {
            lhs: generate_random_numeric_argument(rng),
            rhs: generate_random_numeric_argument(rng),
        },
        GenerateInstruction::AddSubConstrain => Instruction::AddSubConstrain {
            lhs: rng.random_range(usize::MIN..usize::MAX),
            rhs: rng.random_range(usize::MIN..usize::MAX),
        },
        GenerateInstruction::MulDivConstrain => Instruction::MulDivConstrain {
            lhs: rng.random_range(usize::MIN..usize::MAX),
            rhs: rng.random_range(usize::MIN..usize::MAX),
        },
        GenerateInstruction::AddToMemory => {
            Instruction::AddToMemory { lhs: generate_random_argument(rng) }
        }
        GenerateInstruction::LoadFromMemory => {
            Instruction::LoadFromMemory { memory_addr: generate_random_argument(rng) }
        }
        GenerateInstruction::SetToMemory => Instruction::SetToMemory {
            memory_addr_index: rng.random_range(usize::MIN..usize::MAX),
            value: generate_random_argument(rng),
        },
        GenerateInstruction::CreateArray => Instruction::CreateArray {
            elements_indices: vec![rng.random_range(usize::MIN..usize::MAX); 10],
            element_type: generate_random_argument(rng).value_type,
        },
        GenerateInstruction::ArrayGet => Instruction::ArrayGet {
            array_index: rng.random_range(usize::MIN..usize::MAX),
            index: generate_random_numeric_argument(rng),
            safe_index: generate_random_bool(rng, GENERATE_BOOL_CONFIGURATION_MOST_TRUE),
        },
        GenerateInstruction::ArraySet => Instruction::ArraySet {
            array_index: rng.random_range(usize::MIN..usize::MAX),
            index: generate_random_numeric_argument(rng),
            value_index: rng.random_range(usize::MIN..usize::MAX),
            safe_index: generate_random_bool(rng, GENERATE_BOOL_CONFIGURATION_MOST_TRUE),
        },
        GenerateInstruction::ArrayGetWithConstantIndex => Instruction::ArrayGetWithConstantIndex {
            array_index: rng.random_range(usize::MIN..usize::MAX),
            index: rng.random_range(usize::MIN..usize::MAX),
            safe_index: generate_random_bool(rng, GENERATE_BOOL_CONFIGURATION_MOST_TRUE),
        },
        GenerateInstruction::ArraySetWithConstantIndex => Instruction::ArraySetWithConstantIndex {
            array_index: rng.random_range(usize::MIN..usize::MAX),
            index: rng.random_range(usize::MIN..usize::MAX),
            value_index: rng.random_range(usize::MIN..usize::MAX),
            safe_index: generate_random_bool(rng, GENERATE_BOOL_CONFIGURATION_MOST_TRUE),
        },
        GenerateInstruction::FieldToBytesToField => {
            Instruction::FieldToBytesToField { field_idx: rng.random_range(usize::MIN..usize::MAX) }
        }
        GenerateInstruction::Blake2sHash => Instruction::Blake2sHash {
            field_idx: rng.random_range(usize::MIN..usize::MAX),
            limbs_count: rng.random_range(u8::MIN..u8::MAX),
        },
        GenerateInstruction::Blake3Hash => Instruction::Blake3Hash {
            field_idx: rng.random_range(usize::MIN..usize::MAX),
            limbs_count: rng.random_range(u8::MIN..u8::MAX),
        },
        GenerateInstruction::Keccakf1600Hash => Instruction::Keccakf1600Hash {
            u64_indices: [rng.random_range(usize::MIN..usize::MAX); 25],
            load_elements_of_array: generate_random_bool(
                rng,
                GENERATE_BOOL_CONFIGURATION_MOST_FALSE,
            ),
        },
        GenerateInstruction::Aes128Encrypt => Instruction::Aes128Encrypt {
            input_idx: rng.random_range(usize::MIN..usize::MAX),
            input_limbs_count: rng.random_range(u8::MIN..u8::MAX),
            key_idx: rng.random_range(usize::MIN..usize::MAX),
            iv_idx: rng.random_range(usize::MIN..usize::MAX),
        },
        GenerateInstruction::Sha256Compression => Instruction::Sha256Compression {
            input_indices: [rng.random_range(usize::MIN..usize::MAX); 16],
            state_indices: [rng.random_range(usize::MIN..usize::MAX); 8],
            load_elements_of_array: generate_random_bool(
                rng,
                GENERATE_BOOL_CONFIGURATION_MOST_FALSE,
            ),
        },
        GenerateInstruction::MultiScalarMul => Instruction::MultiScalarMul {
            points_and_scalars: vec![
                (generate_random_point(rng), generate_random_scalar(rng)),
                (generate_random_point(rng), generate_random_scalar(rng)),
            ],
            predicate: true,
        },
        GenerateInstruction::PointAdd => Instruction::PointAdd {
            p1: generate_random_point(rng),
            p2: generate_random_point(rng),
            predicate: true,
        },

        GenerateInstruction::EcdsaSecp256r1 => {
            let mut msg = [0; SIZE_OF_LARGE_ARBITRARY_BUFFER];
            rng.fill_bytes(&mut msg);
            Instruction::EcdsaSecp256r1 {
                msg: msg.to_vec(),
                corrupt_hash: generate_random_bool(rng, GENERATE_BOOL_CONFIGURATION_MOST_FALSE),
                corrupt_pubkey_x: generate_random_bool(rng, GENERATE_BOOL_CONFIGURATION_MOST_FALSE),
                corrupt_pubkey_y: generate_random_bool(rng, GENERATE_BOOL_CONFIGURATION_MOST_FALSE),
                corrupt_signature: generate_random_bool(
                    rng,
                    GENERATE_BOOL_CONFIGURATION_MOST_FALSE,
                ),
                predicate: true,
            }
        }
        GenerateInstruction::EcdsaSecp256k1 => {
            let mut msg = [0; SIZE_OF_LARGE_ARBITRARY_BUFFER];
            rng.fill_bytes(&mut msg);
            Instruction::EcdsaSecp256k1 {
                msg: msg.to_vec(),
                corrupt_hash: generate_random_bool(rng, GENERATE_BOOL_CONFIGURATION_MOST_FALSE),
                corrupt_pubkey_x: generate_random_bool(rng, GENERATE_BOOL_CONFIGURATION_MOST_FALSE),
                corrupt_pubkey_y: generate_random_bool(rng, GENERATE_BOOL_CONFIGURATION_MOST_FALSE),
                corrupt_signature: generate_random_bool(
                    rng,
                    GENERATE_BOOL_CONFIGURATION_MOST_FALSE,
                ),
                predicate: true,
            }
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
