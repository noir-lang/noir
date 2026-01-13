//! This file contains tests for hash operations.
//! 1) blake2s
//! 2) blake3
//! 3) aes128_encrypt
//! 4) keccakf1600
//! 5) sha256_compression
use crate::function_context::{FunctionData, FuzzerFunctionCommand};
use crate::fuzz_target_lib::fuzz_target;
use crate::fuzzer::FuzzerData;
use crate::instruction::{Instruction, InstructionBlock, NumericArgument};
use crate::options::FuzzerOptions;
use crate::tests::common::{default_input_types, default_runtimes, default_witness};
use acvm::FieldElement;
use noir_ssa_fuzzer::typed_value::NumericType;
use noir_ssa_fuzzer::typed_value::Type;

/// blake2s(to_le_radix(0, 256, 32)) == blake2s computed with noir
///
/// fn main(x: u8) -> pub Field {
///     let x = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
///     let hash = std::hash::blake2s(x);
///     Field::from_le_bytes::<32>(hash)
/// }
/// [nargo_tests] Circuit output: Field(-9211429028062209127175291049466917975585300944217240748738694765619842249938)
#[test]
fn smoke_test_blake2s_hash() {
    let _ = env_logger::try_init();
    let blake2s_hash_block = InstructionBlock {
        instructions: vec![Instruction::Blake2sHash { field_idx: 0, limbs_count: 32 }],
    };
    let instructions_blocks = vec![blake2s_hash_block];
    let commands = vec![];
    let main_func = FunctionData {
        input_types: default_input_types(),
        commands,
        return_instruction_block_idx: 0,
        return_type: Type::Numeric(NumericType::Field),
    };
    let fuzzer_data = FuzzerData {
        instruction_blocks: instructions_blocks,
        functions: vec![main_func],
        initial_witness: default_witness(),
    };
    let result = fuzz_target(fuzzer_data, default_runtimes(), FuzzerOptions::default());
    match result.get_return_witnesses().is_empty() {
        true => {
            panic!("Program failed to execute");
        }
        false => {
            assert_eq!(
                result.get_return_witnesses()[0],
                FieldElement::try_from_str(
                    "-9211429028062209127175291049466917975585300944217240748738694765619842249938"
                )
                .unwrap()
            );
        }
    }
}

/// blake3(to_le_radix(0, 256, 32)) == blake3 computed with noir
///
/// fn main(x: u8) -> pub Field {
///     let x = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
///     let hash = std::hash::blake3(x);
///     Field::from_le_bytes::<32>(hash)
/// }
/// [nargo_tests] Circuit output: Field(11496696481601359239189947342432058980836600577383371976100559912527609453094)
#[test]
fn smoke_test_blake3_hash() {
    let _ = env_logger::try_init();
    let blake3_hash_block = InstructionBlock {
        instructions: vec![Instruction::Blake3Hash { field_idx: 0, limbs_count: 32 }],
    };
    let instructions_blocks = vec![blake3_hash_block];
    let commands = vec![];
    let main_func = FunctionData {
        input_types: default_input_types(),
        commands,
        return_instruction_block_idx: 0,
        return_type: Type::Numeric(NumericType::Field),
    };
    let fuzzer_data = FuzzerData {
        instruction_blocks: instructions_blocks,
        functions: vec![main_func],
        initial_witness: default_witness(),
    };
    let result = fuzz_target(fuzzer_data, default_runtimes(), FuzzerOptions::default());
    match result.get_return_witnesses().is_empty() {
        true => {
            panic!("Program failed to execute");
        }
        false => {
            assert_eq!(
                result.get_return_witnesses()[0],
                FieldElement::try_from_str(
                    "11496696481601359239189947342432058980836600577383371976100559912527609453094"
                )
                .unwrap()
            );
        }
    }
}

/// fn main() -> pub Field {
///     let input: [u8; 16] = b.to_le_radix(256);
///     let iv: [u8; 16] = b.to_le_radix(256);
///     let key: [u8; 16] = b.to_le_radix(256);
///     Field::from_le_bytes(std::aes128::aes128_encrypt(input, iv, key))
/// }
///
/// [nargo_tests] Circuit output: Field(7228449286344697221705732525592563926191809635549234005020486075743434697058)
#[test]
fn smoke_test_aes128_encrypt() {
    let _ = env_logger::try_init();
    let aes128_encrypt_block = InstructionBlock {
        instructions: vec![Instruction::Aes128Encrypt {
            input_idx: 0,
            input_limbs_count: 16,
            key_idx: 0,
            iv_idx: 0,
        }],
    };
    let instructions_blocks = vec![aes128_encrypt_block];
    let commands = vec![];
    let main_func = FunctionData {
        input_types: default_input_types(),
        commands,
        return_instruction_block_idx: 0,
        return_type: Type::Numeric(NumericType::Field),
    };
    let fuzzer_data = FuzzerData {
        instruction_blocks: instructions_blocks,
        functions: vec![main_func],
        initial_witness: default_witness(),
    };
    let result = fuzz_target(fuzzer_data, default_runtimes(), FuzzerOptions::default());
    match result.get_return_witnesses().is_empty() {
        true => {
            panic!("Program failed to execute");
        }
        false => {
            assert_eq!(
                result.get_return_witnesses()[0],
                FieldElement::try_from_str(
                    "7228449286344697221705732525592563926191809635549234005020486075743434697058"
                )
                .unwrap()
            );
        }
    }
}

/// fn main(a: Field, b: Field) -> pub u64 {
///     let input: [u64; 25] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
///     std::hash::keccakf1600(input)[24]
/// }
///
/// [nargo_tests] Circuit output: Field(16929593379567477321)
#[test]
fn smoke_test_keccakf1600() {
    let _ = env_logger::try_init();
    // default witness are fields
    // so take the first one and cast it to u64
    let arg_0_field = NumericArgument { index: 0, numeric_type: NumericType::Field };
    let cast_block = InstructionBlock {
        instructions: vec![Instruction::Cast { lhs: arg_0_field, type_: NumericType::U64 }],
    };
    // taking the first defined u64 variable which is v0 as u64
    let keccakf1600_block = InstructionBlock {
        instructions: vec![Instruction::Keccakf1600Hash {
            u64_indices: [0; 25],
            load_elements_of_array: true, // load all elements of the array into defined variables
        }],
    };
    let instructions_blocks = vec![cast_block, keccakf1600_block];
    let commands =
        vec![FuzzerFunctionCommand::InsertSimpleInstructionBlock { instruction_block_idx: 0 }];
    // this function will take the last defined u64, which is equal to the last element of the keccakf1600 permuted array
    let main_func = FunctionData {
        input_types: default_input_types(),
        commands,
        return_instruction_block_idx: 1,
        return_type: Type::Numeric(NumericType::U64),
    };
    let fuzzer_data = FuzzerData {
        instruction_blocks: instructions_blocks,
        functions: vec![main_func],
        initial_witness: default_witness(),
    };
    let result = fuzz_target(fuzzer_data, default_runtimes(), FuzzerOptions::default());
    match result.get_return_witnesses().is_empty() {
        true => {
            panic!("Program failed to execute");
        }
        false => {
            assert_eq!(
                result.get_return_witnesses()[0],
                FieldElement::from(16929593379567477321_u64)
            );
        }
    }
}

/// fn main(a: Field, b: Field) -> pub u32 {
///     let input: [u32; 16] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
///     let state: [u32; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
///     std::hash::sha256_compression(input, state)[7]
/// }
///
/// [nargo_tests] Circuit output: Field(3205228454)
#[test]
fn smoke_test_sha256_compression() {
    let _ = env_logger::try_init();
    let arg_0_field = NumericArgument { index: 0, numeric_type: NumericType::Field };
    let cast_block = InstructionBlock {
        instructions: vec![Instruction::Cast { lhs: arg_0_field, type_: NumericType::U32 }],
    };
    let sha256_compression_block = InstructionBlock {
        instructions: vec![Instruction::Sha256Compression {
            input_indices: [0; 16],
            state_indices: [0; 8],
            load_elements_of_array: true,
        }],
    };
    let instructions_blocks = vec![cast_block, sha256_compression_block];
    let commands =
        vec![FuzzerFunctionCommand::InsertSimpleInstructionBlock { instruction_block_idx: 0 }];
    let main_func = FunctionData {
        input_types: default_input_types(),
        commands,
        return_instruction_block_idx: 1,
        return_type: Type::Numeric(NumericType::U32),
    };
    let fuzzer_data = FuzzerData {
        instruction_blocks: instructions_blocks,
        functions: vec![main_func],
        initial_witness: default_witness(),
    };
    let result = fuzz_target(fuzzer_data, default_runtimes(), FuzzerOptions::default());
    match result.get_return_witnesses().is_empty() {
        true => {
            panic!("Program failed to execute");
        }
        false => {
            assert_eq!(result.get_return_witnesses()[0], FieldElement::from(3205228454_u32));
        }
    }
}
