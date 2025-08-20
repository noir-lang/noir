use crate::function_context::FunctionData;
use crate::fuzz_target_lib::fuzz_target;
use crate::fuzzer::FuzzerData;
use crate::instruction::{Instruction, InstructionBlock};
use crate::options::FuzzerOptions;
use crate::tests::common::default_witness;
use acvm::AcirField;
use acvm::FieldElement;
use noir_ssa_fuzzer::typed_value::ValueType;

#[test]
fn test_valid_ecdsa_signature_secp256r1() {
    let _ = env_logger::try_init();
    let msg = b"zamay";
    let instruction = Instruction::EcdsaSecp256r1 {
        msg: msg.to_vec(),
        corrupt_hash: false,
        corrupt_pubkey_x: false,
        corrupt_pubkey_y: false,
        corrupt_signature: false,
    };
    let block = InstructionBlock { instructions: vec![instruction] };
    let commands = vec![];
    let function =
        FunctionData { commands, return_instruction_block_idx: 0, return_type: ValueType::Boolean };
    let data = FuzzerData {
        instruction_blocks: vec![block],
        functions: vec![function],
        initial_witness: default_witness(),
    };
    let result = fuzz_target(data, FuzzerOptions::default()).unwrap();
    assert_eq!(result.get_return_value(), FieldElement::one());
}

#[test]
fn test_valid_ecdsa_signature_secp256k1() {
    let _ = env_logger::try_init();
    let msg = b"zamay";
    let instruction = Instruction::EcdsaSecp256k1 {
        msg: msg.to_vec(),
        corrupt_hash: false,
        corrupt_pubkey_x: false,
        corrupt_pubkey_y: false,
        corrupt_signature: false,
    };
    let block = InstructionBlock { instructions: vec![instruction] };
    let commands = vec![];
    let function =
        FunctionData { commands, return_instruction_block_idx: 0, return_type: ValueType::Boolean };
    let data = FuzzerData {
        instruction_blocks: vec![block],
        functions: vec![function],
        initial_witness: default_witness(),
    };
    let result = fuzz_target(data, FuzzerOptions::default()).unwrap();
    assert_eq!(result.get_return_value(), FieldElement::one());
}

#[test]
fn test_corrupted_ecdsa_signature_secp256r1() {
    let _ = env_logger::try_init();
    let msg = b"zamay";
    let instruction = Instruction::EcdsaSecp256r1 {
        msg: msg.to_vec(),
        corrupt_hash: true,
        corrupt_pubkey_x: true,
        corrupt_pubkey_y: true,
        corrupt_signature: true,
    };
    let block = InstructionBlock { instructions: vec![instruction] };
    let commands = vec![];
    let function =
        FunctionData { commands, return_instruction_block_idx: 0, return_type: ValueType::Boolean };
    let data = FuzzerData {
        instruction_blocks: vec![block],
        functions: vec![function],
        initial_witness: default_witness(),
    };
    let result = fuzz_target(data, FuzzerOptions::default()).unwrap();
    assert_eq!(result.get_return_value(), FieldElement::zero());
}

#[test]
fn test_corrupted_ecdsa_signature_secp256k1() {
    let _ = env_logger::try_init();
    let msg = b"zamay";
    let instruction = Instruction::EcdsaSecp256k1 {
        msg: msg.to_vec(),
        corrupt_hash: true,
        corrupt_pubkey_x: true,
        corrupt_pubkey_y: true,
        corrupt_signature: true,
    };
    let block = InstructionBlock { instructions: vec![instruction] };
    let commands = vec![];
    let function =
        FunctionData { commands, return_instruction_block_idx: 0, return_type: ValueType::Boolean };
    let data = FuzzerData {
        instruction_blocks: vec![block],
        functions: vec![function],
        initial_witness: default_witness(),
    };
    let result = fuzz_target(data, FuzzerOptions::default()).unwrap();
    assert_eq!(result.get_return_value(), FieldElement::zero());
}
