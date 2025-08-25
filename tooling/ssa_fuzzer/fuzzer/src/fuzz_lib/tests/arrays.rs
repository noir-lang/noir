//! This file contains tests for array operations.
//! 1) array_get and array_set
//! 2) Tests that mutating references in arrays works

use crate::function_context::{FunctionData, FuzzerFunctionCommand};
use crate::fuzz_target_lib::fuzz_target;
use crate::fuzzer::FuzzerData;
use crate::instruction::{Argument, Instruction, InstructionBlock};
use crate::options::FuzzerOptions;
use crate::tests::common::default_witness;
use acvm::FieldElement;
use noir_ssa_fuzzer::new_type::NumericType;

/// Test array get and set
/// fn main f0 {
///     b0(v0: Field, v1: Field, v2: Field, v3: Field, v4: Field, v5: u1, v6: u1):
///       v7 = make_array [v0, v1, v2, v3, v4] : [Field; 5]
///       v8 = truncate v0 to 32 bits, max_bit_size: 254
///       v9 = cast v8 as u32
///       v11 = mod v9, u32 5
///       v12 = array_set v7, index v11, value v4
///       v13 = truncate v0 to 32 bits, max_bit_size: 254
///       v14 = cast v13 as u32
///       v15 = mod v14, u32 5
///       v16 = array_get v12, index v15 -> Field
///       return v16
///   }
#[test]
fn array_get_and_set() {
    let arg_0_field = Argument { index: 0, value_type: NumericType::Field };
    // create array [v0, v1]
    let create_array_block = InstructionBlock {
        instructions: vec![Instruction::CreateArray {
            elements_indices: vec![0, 1, 2, 3, 4],
            element_type: NumericType::Field,
            is_references: false,
        }],
    };
    // create new array setting new_array[0] = v4
    let array_set_block = InstructionBlock {
        instructions: vec![Instruction::ArraySet {
            array_index: 0,
            index: arg_0_field,
            value_index: 4, // set v4
            safe_index: true,
        }],
    };
    // get new_array[0]
    let array_get_block = InstructionBlock {
        instructions: vec![Instruction::ArrayGet {
            array_index: 1,
            index: arg_0_field,
            safe_index: true,
        }],
    };
    let instructions_blocks = vec![create_array_block, array_set_block, array_get_block];
    let commands = vec![
        FuzzerFunctionCommand::InsertSimpleInstructionBlock { instruction_block_idx: 0 },
        FuzzerFunctionCommand::InsertSimpleInstructionBlock { instruction_block_idx: 1 },
    ];
    let main_func =
        FunctionData { commands, return_instruction_block_idx: 2, return_type: NumericType::Field };
    let fuzzer_data = FuzzerData {
        instruction_blocks: instructions_blocks,
        functions: vec![main_func],
        initial_witness: default_witness(),
    };
    let result = fuzz_target(fuzzer_data, FuzzerOptions::default());
    match result {
        Some(result) => assert_eq!(result.get_return_value(), FieldElement::from(4_u32)),
        None => panic!("Program failed to execute"),
    }
}

/// Test that references in arrays work
/// fn main f0 {
///     b0(v0: Field, v1: Field, v2: Field, v3: Field, v4: Field, v5: u1, v6: u1):
///       v7 = allocate -> &mut Field
///       store v0 at v7
///       v8 = allocate -> &mut Field
///       store v2 at v8
///       v9 = make_array [v7, v8, v7] : [&mut Field; 3]
///       store v1 at v7
///       v10 = make_array [v7, v7, v7] : [&mut Field; 3]  <----- set v9, index 1, value v7
///       jmp b1()
///     b1():
///       v11 = load v7 -> Field <---- its simplified from  v11 = v10[1]; v12 = load v11
///       return v11
///   }
/// assert that return value is v1
#[test]
fn test_reference_in_array() {
    let _ = env_logger::try_init();
    let arg_0_field = Argument { index: 0, value_type: NumericType::Field };
    let arg_1_field = Argument { index: 1, value_type: NumericType::Field };
    let arg_2_field = Argument { index: 2, value_type: NumericType::Field };

    let add_to_memory_block =
        InstructionBlock { instructions: vec![Instruction::AddToMemory { lhs: arg_0_field }] };
    let add_to_memory_block_2 =
        InstructionBlock { instructions: vec![Instruction::AddToMemory { lhs: arg_2_field }] };
    let set_to_memory_block = InstructionBlock {
        instructions: vec![Instruction::SetToMemory { memory_addr_index: 0, value: arg_1_field }],
    };
    let set_to_array_block = InstructionBlock {
        instructions: vec![Instruction::ArraySetWithConstantIndex {
            array_index: 0,
            index: 1,
            value_index: 0,
            safe_index: true,
        }],
    };
    let create_array_block = InstructionBlock {
        instructions: vec![Instruction::CreateArray {
            elements_indices: vec![0, 1, 2],
            element_type: NumericType::Field,
            is_references: true,
        }],
    };
    let get_from_array_block = InstructionBlock {
        instructions: vec![Instruction::ArrayGetWithConstantIndex {
            array_index: 1,
            index: 1,
            safe_index: true,
        }],
    };
    let typed_memory_2 = Argument { index: 2, value_type: NumericType::Field };
    let load_from_memory_block = InstructionBlock {
        instructions: vec![Instruction::LoadFromMemory { memory_addr: typed_memory_2 }],
    };
    let instructions_blocks = vec![
        add_to_memory_block,
        add_to_memory_block_2,
        create_array_block,
        set_to_memory_block,
        set_to_array_block,
        get_from_array_block,
        load_from_memory_block,
    ];
    let commands = vec![
        FuzzerFunctionCommand::InsertSimpleInstructionBlock { instruction_block_idx: 0 },
        FuzzerFunctionCommand::InsertSimpleInstructionBlock { instruction_block_idx: 1 },
        FuzzerFunctionCommand::InsertSimpleInstructionBlock { instruction_block_idx: 2 },
        FuzzerFunctionCommand::InsertSimpleInstructionBlock { instruction_block_idx: 3 },
        FuzzerFunctionCommand::InsertSimpleInstructionBlock { instruction_block_idx: 4 },
        FuzzerFunctionCommand::InsertSimpleInstructionBlock { instruction_block_idx: 5 },
    ];
    let main_func =
        FunctionData { commands, return_instruction_block_idx: 6, return_type: NumericType::Field };
    let fuzzer_data = FuzzerData {
        instruction_blocks: instructions_blocks,
        functions: vec![main_func],
        initial_witness: default_witness(),
    };
    let result = fuzz_target(fuzzer_data, FuzzerOptions::default());
    match result {
        Some(result) => assert_eq!(result.get_return_value(), FieldElement::from(1_u32)),
        None => panic!("Program failed to execute"),
    }
}
