use crate::function_context::FunctionData;
use crate::fuzz_target_lib::fuzz_target;
use crate::fuzzer::FuzzerData;
use crate::instruction::{Instruction, InstructionBlock};
use crate::options::FuzzerOptions;
use crate::tests::common::{default_input_types, default_runtimes, default_witness};
use acvm::AcirField;
use acvm::FieldElement;
use noir_ssa_fuzzer::typed_value::{NumericType, Type};

#[test]
fn test_valid_ecdsa_signature_secp256r1() {
    let _ = env_logger::try_init();
    let msg = [122, 97, 109, 97, 121];
    let instruction = Instruction::EcdsaSecp256r1 {
        msg: msg.to_vec(),
        corrupt_hash: false,
        corrupt_pubkey_x: false,
        corrupt_pubkey_y: false,
        corrupt_signature: false,
        predicate: true,
    };
    let block = InstructionBlock { instructions: vec![instruction] };
    let commands = vec![];
    let function = FunctionData {
        commands,
        input_types: default_input_types(),
        return_instruction_block_idx: 0,
        return_type: Type::Numeric(NumericType::Boolean),
    };
    let data = FuzzerData {
        instruction_blocks: vec![block],
        functions: vec![function],
        initial_witness: default_witness(),
    };
    let result = fuzz_target(data, default_runtimes(), FuzzerOptions::default());
    assert_eq!(result.get_return_witnesses()[0], FieldElement::one());
}

#[test]
fn test_valid_ecdsa_signature_secp256k1() {
    let _ = env_logger::try_init();
    let msg = [122, 97, 109, 97, 121];
    let instruction = Instruction::EcdsaSecp256k1 {
        msg: msg.to_vec(),
        corrupt_hash: false,
        corrupt_pubkey_x: false,
        corrupt_pubkey_y: false,
        corrupt_signature: false,
        predicate: true,
    };
    let block = InstructionBlock { instructions: vec![instruction] };
    let commands = vec![];
    let function = FunctionData {
        commands,
        input_types: default_input_types(),
        return_instruction_block_idx: 0,
        return_type: Type::Numeric(NumericType::Boolean),
    };
    let data = FuzzerData {
        instruction_blocks: vec![block],
        functions: vec![function],
        initial_witness: default_witness(),
    };
    let result = fuzz_target(data, default_runtimes(), FuzzerOptions::default());
    assert_eq!(result.get_return_witnesses()[0], FieldElement::one());
}

#[test]
fn test_corrupted_ecdsa_signature_secp256r1() {
    let _ = env_logger::try_init();
    let msg = [122, 97, 109, 97, 121];
    let instruction = Instruction::EcdsaSecp256r1 {
        msg: msg.to_vec(),
        corrupt_hash: true,
        corrupt_pubkey_x: true,
        corrupt_pubkey_y: true,
        corrupt_signature: true,
        predicate: true,
    };
    let block = InstructionBlock { instructions: vec![instruction] };
    let commands = vec![];
    let function = FunctionData {
        commands,
        input_types: default_input_types(),
        return_instruction_block_idx: 0,
        return_type: Type::Numeric(NumericType::Boolean),
    };
    let data = FuzzerData {
        instruction_blocks: vec![block],
        functions: vec![function],
        initial_witness: default_witness(),
    };
    let result = fuzz_target(data, default_runtimes(), FuzzerOptions::default());
    match result.get_return_witnesses().is_empty() {
        true => {
            // thats okay
        }
        false => {
            panic!("Programs executed with the Result: {:?}", result.get_return_witnesses())
        }
    }
}

#[test]
fn test_corrupted_ecdsa_signature_secp256k1() {
    let _ = env_logger::try_init();
    let msg = [122, 97, 109, 97, 121];
    let instruction = Instruction::EcdsaSecp256k1 {
        msg: msg.to_vec(),
        corrupt_hash: true,
        corrupt_pubkey_x: true,
        corrupt_pubkey_y: true,
        corrupt_signature: true,
        predicate: true,
    };
    let block = InstructionBlock { instructions: vec![instruction] };
    let commands = vec![];
    let function = FunctionData {
        commands,
        input_types: default_input_types(),
        return_instruction_block_idx: 0,
        return_type: Type::Numeric(NumericType::Boolean),
    };
    let data = FuzzerData {
        instruction_blocks: vec![block],
        functions: vec![function],
        initial_witness: default_witness(),
    };
    let result = fuzz_target(data, default_runtimes(), FuzzerOptions::default());
    match result.get_return_witnesses().is_empty() {
        true => {
            // thats okay
        }
        false => {
            panic!("Programs executed with the Result: {:?}", result.get_return_witnesses())
        }
    }
}
