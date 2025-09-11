//! This file contains tests for array operations.
//! 1) array_get and array_set
//! 2) Tests that mutating references in arrays works

use crate::function_context::{FunctionData, FuzzerFunctionCommand};
use crate::fuzz_target_lib::fuzz_target;
use crate::fuzzer::FuzzerData;
use crate::instruction::{Argument, Instruction, InstructionBlock, NumericArgument};
use crate::options::FuzzerOptions;
use crate::tests::common::{default_input_types, default_runtimes, default_witness};
use acvm::FieldElement;
use noir_ssa_fuzzer::typed_value::{NumericType, Type};
use std::sync::Arc;

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
    let arg_0_field = NumericArgument { index: 0, numeric_type: NumericType::Field };
    // create array [v0, v1]
    let create_array_block = InstructionBlock {
        instructions: vec![Instruction::CreateArray {
            elements_indices: vec![0, 1, 2, 3, 4],
            element_type: Type::Numeric(NumericType::Field),
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
    let main_func = FunctionData {
        commands,
        input_types: default_input_types(),
        return_instruction_block_idx: 2,
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
            assert_eq!(result.get_return_witnesses()[0], FieldElement::from(4_u32));
        }
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
    let arg_0_field = Argument { index: 0, value_type: Type::Numeric(NumericType::Field) };
    let arg_1_field = Argument { index: 1, value_type: Type::Numeric(NumericType::Field) };
    let arg_2_field = Argument { index: 2, value_type: Type::Numeric(NumericType::Field) };

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
            element_type: Type::Reference(Arc::new(Type::Numeric(NumericType::Field))),
        }],
    };
    let get_from_array_block = InstructionBlock {
        instructions: vec![Instruction::ArrayGetWithConstantIndex {
            array_index: 1,
            index: 1,
            safe_index: true,
        }],
    };
    let typed_memory_2 = Argument { index: 2, value_type: Type::Numeric(NumericType::Field) };
    let load_from_memory_block = InstructionBlock {
        instructions: vec![Instruction::LoadFromMemory { memory_addr: typed_memory_2.clone() }],
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
    let main_func = FunctionData {
        commands,
        input_types: default_input_types(),
        return_instruction_block_idx: 6,
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
            assert_eq!(result.get_return_witnesses()[0], FieldElement::from(1_u32));
        }
    }
}

/// Previously the fuzzer generated arrays of incorrect sizes with find_values_with_type
/// See https://github.com/noir-lang/noir/issues/9678
///
/// fn main(v0: u1, v1: u1) -> pub u1 {
///   func([v0, v0], v1 as u32)
/// }
///
/// fn func(v0: [u1; 2], index: u32) -> u1 {
///   v0[index]
/// }
///
/// This test is compiled into the following SSA:
/// Note: the array (argument to f1) is auto generated by find_values_with_type
/// brillig(inline) fn main f0 {
///     b0(v0: Field, v1: Field, v2: Field, v3: Field, v4: Field, v5: u1, v6: u1):
///       v7 = make_array [v6, v6] : [u1; 2] // this array is auto generated by find_values_with_type
///       v8 = cast v6 as u32
///       v10 = call f1(v7, v8, v5) -> u1
///       jmp b1()
///     b1():
///       v11 = make_array [v6, v6] : [u1; 2]
///       return v10
///     }
/// brillig(inline) fn f1 f1 {
///     b0(v0: [u1; 2], v1: u32, v2: u1):
///       jmp b1()
///     b1():
///       v3 = truncate v1 to 1 bits, max_bit_size: 32
///       v4 = array_get v0, index v3 -> u1
///       return v4
///     }
#[test]
fn regression_fuzzer_generated_wrong_arrays() {
    let _ = env_logger::try_init();
    let arg_1_bool = NumericArgument { index: 1, numeric_type: NumericType::Boolean };
    let arg_0_u32 = NumericArgument { index: 0, numeric_type: NumericType::U32 };
    let cast_block = InstructionBlock {
        instructions: vec![Instruction::Cast { lhs: arg_1_bool, type_: NumericType::U32 }],
    };
    let get_from_array_block = InstructionBlock {
        instructions: vec![Instruction::ArrayGet {
            array_index: 0,
            index: arg_0_u32,
            safe_index: true,
        }],
    };
    let instructions_blocks = vec![cast_block, get_from_array_block];
    let main_commands = vec![
        FuzzerFunctionCommand::InsertSimpleInstructionBlock { instruction_block_idx: 0 }, // cast
        FuzzerFunctionCommand::InsertFunctionCall { function_idx: 0, args: [0, 1, 2, 3, 4, 5, 6] }, // call func([v0, v0], v1 as u32), random args
    ];
    let main_func = FunctionData {
        commands: main_commands,
        input_types: default_input_types(),
        return_instruction_block_idx: 0, // dummy
        return_type: Type::Numeric(NumericType::Boolean),
    };
    let func_func = FunctionData {
        commands: vec![],
        input_types: vec![
            Type::Array(Arc::new(vec![Type::Numeric(NumericType::Boolean)]), 2),
            Type::Numeric(NumericType::U32),
        ],
        return_instruction_block_idx: 1, // get from array
        return_type: Type::Numeric(NumericType::Boolean),
    };
    let fuzzer_data = FuzzerData {
        instruction_blocks: instructions_blocks,
        functions: vec![main_func, func_func],
        initial_witness: default_witness(),
    };
    let result = fuzz_target(fuzzer_data, default_runtimes(), FuzzerOptions::default());
    match result.get_return_witnesses().is_empty() {
        true => {
            panic!("Program failed to execute");
        }
        false => {
            assert_eq!(result.get_return_witnesses()[0], FieldElement::from(0_u32));
        }
    }
}

/// Test that creating array of arrays works
/// fn main f0 {
///     b0(v0: Field, v1: Field, v2: Field, v3: Field, v4: Field, v5: u1, v6: u1):
///       v7 = make_array [v0, v1, v2] : [Field; 3]
///       v8 = make_array [v7, v7] : [[Field; 3]; 2]
///       return v8
/// }
/// output should be 0,1,2,0,1,2
#[test]
fn test_create_array_of_arrays() {
    let _ = env_logger::try_init();

    let array_type = Type::Array(Arc::new(vec![Type::Numeric(NumericType::Field)]), 3);
    let array_of_arrays_type = Type::Array(Arc::new(vec![array_type.clone()]), 2);
    let create_arrays_block = InstructionBlock {
        instructions: vec![
            Instruction::CreateArray {
                elements_indices: vec![0, 1, 2],
                element_type: Type::Numeric(NumericType::Field),
            },
            Instruction::CreateArray { elements_indices: vec![0, 1, 2], element_type: array_type },
        ],
    };
    let main_commands =
        vec![FuzzerFunctionCommand::InsertSimpleInstructionBlock { instruction_block_idx: 0 }];
    let main_func = FunctionData {
        commands: main_commands,
        input_types: default_input_types(),
        return_instruction_block_idx: 1,
        return_type: array_of_arrays_type,
    };
    let fuzzer_data = FuzzerData {
        instruction_blocks: vec![create_arrays_block],
        functions: vec![main_func],
        initial_witness: default_witness(),
    };
    let expected_return_value =
        (0..3).map(|i| FieldElement::from(i as u32)).collect::<Vec<_>>().repeat(2);
    let result = fuzz_target(fuzzer_data, default_runtimes(), FuzzerOptions::default());
    match result.get_return_witnesses().is_empty() {
        true => {
            panic!("Program failed to execute");
        }
        false => {
            assert_eq!(result.get_return_witnesses(), expected_return_value);
        }
    }
}
