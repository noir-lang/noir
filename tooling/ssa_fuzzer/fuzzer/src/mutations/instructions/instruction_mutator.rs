//! This file contains mechanisms for deterministically mutating a given [Instruction](crate::fuzz_lib::instruction::Instruction) value
//! Types of mutations applied:
//! 1. Random (randomly select a new instruction)
//! 2. Argument mutation

use crate::fuzz_lib::instruction::Instruction;
use crate::mutations::{
    basic_types::{
        bool::mutate_bool, usize::mutate_usize, value_type::mutate_value_type, vec::mutate_vec,
    },
    configuration::{
        Aes128EncryptMutationOptions, ArrayGetMutationOptions, ArraySetMutationOptions,
        BASIC_AES_128_ENCRYPT_MUTATION_CONFIGURATION, BASIC_ARRAY_GET_MUTATION_CONFIGURATION,
        BASIC_ARRAY_SET_MUTATION_CONFIGURATION, BASIC_BLAKE_HASH_MUTATION_CONFIGURATION,
        BASIC_BOOL_MUTATION_CONFIGURATION, BASIC_CREATE_ARRAY_MUTATION_CONFIGURATION,
        BASIC_INSTRUCTION_ARGUMENT_MUTATION_CONFIGURATION,
        BASIC_INSTRUCTION_MUTATION_CONFIGURATION, BASIC_SAFE_INDEX_MUTATION_CONFIGURATION,
        BASIC_SHA256_COMPRESSION_MUTATION_CONFIGURATION, BASIC_USIZE_MUTATION_CONFIGURATION,
        BASIC_VALUE_TYPE_MUTATION_CONFIGURATION, BASIC_VEC_MUTATION_CONFIGURATION,
        BlakeHashMutationOptions, CreateArrayMutationOptions, InstructionArgumentMutationOptions,
        InstructionMutationOptions, SIZE_OF_SMALL_ARBITRARY_BUFFER,
        Sha256CompressionMutationOptions,
    },
    instructions::argument_mutator::argument_mutator,
};
use libfuzzer_sys::arbitrary::Unstructured;
use rand::{Rng, rngs::StdRng};

/// Return new random instruction
struct RandomMutation;
impl RandomMutation {
    fn mutate(rng: &mut StdRng, value: &mut Instruction) {
        let mut bytes = [0u8; SIZE_OF_SMALL_ARBITRARY_BUFFER];
        rng.fill(&mut bytes);
        *value = Unstructured::new(&bytes).arbitrary().unwrap();
    }
}

/// Mutate arguments of the instruction
struct InstructionArgumentsMutation;
impl InstructionArgumentsMutation {
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
                        mutate_value_type(type_, rng, BASIC_VALUE_TYPE_MUTATION_CONFIGURATION);
                    }
                }
            }
            Instruction::SetToMemory { memory_addr_index, value } => {
                match BASIC_INSTRUCTION_ARGUMENT_MUTATION_CONFIGURATION.select(rng) {
                    InstructionArgumentMutationOptions::Left => {
                        argument_mutator(value, rng);
                    }
                    InstructionArgumentMutationOptions::Right => {
                        mutate_usize(memory_addr_index, rng, BASIC_USIZE_MUTATION_CONFIGURATION);
                    }
                }
            }

            Instruction::AddSubConstrain { lhs, rhs }
            | Instruction::MulDivConstrain { lhs, rhs } => {
                match BASIC_INSTRUCTION_ARGUMENT_MUTATION_CONFIGURATION.select(rng) {
                    InstructionArgumentMutationOptions::Left => {
                        mutate_usize(lhs, rng, BASIC_USIZE_MUTATION_CONFIGURATION);
                    }
                    InstructionArgumentMutationOptions::Right => {
                        mutate_usize(rhs, rng, BASIC_USIZE_MUTATION_CONFIGURATION);
                    }
                }
            }

            // Arrays
            Instruction::ArrayGet { array_index, index, safe_index } => {
                match BASIC_ARRAY_GET_MUTATION_CONFIGURATION.select(rng) {
                    ArrayGetMutationOptions::ArrayIndex => {
                        mutate_usize(array_index, rng, BASIC_USIZE_MUTATION_CONFIGURATION);
                    }
                    ArrayGetMutationOptions::Index => {
                        argument_mutator(index, rng);
                    }
                    ArrayGetMutationOptions::SafeIndex => {
                        mutate_bool(safe_index, rng, BASIC_SAFE_INDEX_MUTATION_CONFIGURATION);
                    }
                }
            }
            Instruction::ArraySet { array_index, index, value_index, safe_index } => {
                match BASIC_ARRAY_SET_MUTATION_CONFIGURATION.select(rng) {
                    ArraySetMutationOptions::ArrayIndex => {
                        mutate_usize(array_index, rng, BASIC_USIZE_MUTATION_CONFIGURATION);
                    }
                    ArraySetMutationOptions::Index => {
                        argument_mutator(index, rng);
                    }
                    ArraySetMutationOptions::ValueIndex => {
                        mutate_usize(value_index, rng, BASIC_USIZE_MUTATION_CONFIGURATION);
                    }
                    ArraySetMutationOptions::SafeIndex => {
                        mutate_bool(safe_index, rng, BASIC_SAFE_INDEX_MUTATION_CONFIGURATION);
                    }
                }
            }
            Instruction::CreateArray { elements_indices, element_type, is_references } => {
                match BASIC_CREATE_ARRAY_MUTATION_CONFIGURATION.select(rng) {
                    CreateArrayMutationOptions::ElementsIndices => {
                        mutate_vec(
                            elements_indices,
                            rng,
                            |index, rng| {
                                mutate_usize(index, rng, BASIC_USIZE_MUTATION_CONFIGURATION);
                            },
                            |rng| rng.gen_range(usize::MIN..usize::MAX),
                            BASIC_VEC_MUTATION_CONFIGURATION,
                        );
                    }
                    CreateArrayMutationOptions::ElementType => {
                        mutate_value_type(
                            element_type,
                            rng,
                            BASIC_VALUE_TYPE_MUTATION_CONFIGURATION,
                        );
                    }
                    CreateArrayMutationOptions::IsReferences => {
                        mutate_bool(is_references, rng, BASIC_BOOL_MUTATION_CONFIGURATION);
                    }
                }
            }
            Instruction::ArrayGetWithConstantIndex { array_index, index, safe_index } => {
                match BASIC_ARRAY_GET_MUTATION_CONFIGURATION.select(rng) {
                    ArrayGetMutationOptions::ArrayIndex => {
                        mutate_usize(array_index, rng, BASIC_USIZE_MUTATION_CONFIGURATION);
                    }
                    ArrayGetMutationOptions::Index => {
                        mutate_usize(index, rng, BASIC_USIZE_MUTATION_CONFIGURATION);
                    }
                    ArrayGetMutationOptions::SafeIndex => {
                        mutate_bool(safe_index, rng, BASIC_SAFE_INDEX_MUTATION_CONFIGURATION);
                    }
                }
            }
            Instruction::ArraySetWithConstantIndex {
                array_index,
                index,
                value_index,
                safe_index,
            } => match BASIC_ARRAY_SET_MUTATION_CONFIGURATION.select(rng) {
                ArraySetMutationOptions::ArrayIndex => {
                    mutate_usize(array_index, rng, BASIC_USIZE_MUTATION_CONFIGURATION);
                }
                ArraySetMutationOptions::Index => {
                    mutate_usize(index, rng, BASIC_USIZE_MUTATION_CONFIGURATION);
                }
                ArraySetMutationOptions::ValueIndex => {
                    mutate_usize(value_index, rng, BASIC_USIZE_MUTATION_CONFIGURATION);
                }
                ArraySetMutationOptions::SafeIndex => {
                    mutate_bool(safe_index, rng, BASIC_SAFE_INDEX_MUTATION_CONFIGURATION);
                }
            },
            Instruction::FieldToBytesToField { field_idx } => {
                mutate_usize(field_idx, rng, BASIC_USIZE_MUTATION_CONFIGURATION);
            }
            Instruction::Blake2sHash { field_idx, limbs_count } => {
                match BASIC_BLAKE_HASH_MUTATION_CONFIGURATION.select(rng) {
                    BlakeHashMutationOptions::FieldIdx => {
                        mutate_usize(field_idx, rng, BASIC_USIZE_MUTATION_CONFIGURATION);
                    }
                    BlakeHashMutationOptions::LimbsCount => {
                        *limbs_count = rng.gen_range(u8::MIN..u8::MAX);
                    }
                }
            }
            Instruction::Blake3Hash { field_idx, limbs_count } => {
                match BASIC_BLAKE_HASH_MUTATION_CONFIGURATION.select(rng) {
                    BlakeHashMutationOptions::FieldIdx => {
                        mutate_usize(field_idx, rng, BASIC_USIZE_MUTATION_CONFIGURATION);
                    }
                    BlakeHashMutationOptions::LimbsCount => {
                        *limbs_count = rng.gen_range(u8::MIN..u8::MAX);
                    }
                }
            }
            Instruction::Keccakf1600Hash { u64_indices, load_elements_of_array } => {
                let idx = rng.gen_range(0..u64_indices.len());
                mutate_usize(&mut u64_indices[idx], rng, BASIC_USIZE_MUTATION_CONFIGURATION);
                mutate_bool(load_elements_of_array, rng, BASIC_BOOL_MUTATION_CONFIGURATION);
            }
            Instruction::Sha256Compression {
                input_indices,
                state_indices,
                load_elements_of_array,
            } => match BASIC_SHA256_COMPRESSION_MUTATION_CONFIGURATION.select(rng) {
                Sha256CompressionMutationOptions::InputIndices => {
                    let idx = rng.gen_range(0..input_indices.len());
                    mutate_usize(&mut input_indices[idx], rng, BASIC_USIZE_MUTATION_CONFIGURATION);
                }
                Sha256CompressionMutationOptions::StateIndices => {
                    let idx = rng.gen_range(0..state_indices.len());
                    mutate_usize(&mut state_indices[idx], rng, BASIC_USIZE_MUTATION_CONFIGURATION);
                }
                Sha256CompressionMutationOptions::LoadElementsOfArray => {
                    mutate_bool(load_elements_of_array, rng, BASIC_BOOL_MUTATION_CONFIGURATION);
                }
            },
            Instruction::Aes128Encrypt { input_idx, input_limbs_count, key_idx, iv_idx } => {
                match BASIC_AES_128_ENCRYPT_MUTATION_CONFIGURATION.select(rng) {
                    Aes128EncryptMutationOptions::InputIdx => {
                        mutate_usize(input_idx, rng, BASIC_USIZE_MUTATION_CONFIGURATION);
                    }
                    Aes128EncryptMutationOptions::InputLimbsCount => {
                        *input_limbs_count = rng.gen_range(u8::MIN..u8::MAX);
                    }
                    Aes128EncryptMutationOptions::KeyIdx => {
                        mutate_usize(key_idx, rng, BASIC_USIZE_MUTATION_CONFIGURATION);
                    }
                    Aes128EncryptMutationOptions::IvIdx => {
                        mutate_usize(iv_idx, rng, BASIC_USIZE_MUTATION_CONFIGURATION);
                    }
                }
            }
            Instruction::PointAdd { .. } | Instruction::MultiScalarMul { .. } => {
                unimplemented!()
            }
        }
    }
}

pub(crate) fn instruction_mutator(instruction: &mut Instruction, rng: &mut StdRng) {
    match BASIC_INSTRUCTION_MUTATION_CONFIGURATION.select(rng) {
        InstructionMutationOptions::Random => RandomMutation::mutate(rng, instruction),
        InstructionMutationOptions::ArgumentMutation => {
            InstructionArgumentsMutation::mutate(rng, instruction);
        }
    }
}
