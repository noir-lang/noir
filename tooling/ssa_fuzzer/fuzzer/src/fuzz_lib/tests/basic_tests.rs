//! This file contains tests for basic operations.
//! 1) field addition
//! 2) jmpif
//! 3) mutable variable
//! 4) Test that from_le_radix(to_le_radix(field)) == field
//! 5) Test that function can return array
use crate::function_context::{FunctionData, FuzzerFunctionCommand};
use crate::fuzz_target_lib::fuzz_target;
use crate::fuzzer::FuzzerData;
use crate::instruction::{Argument, Instruction, InstructionBlock, NumericArgument};
use crate::options::FuzzerOptions;
use crate::tests::common::{default_input_types, default_runtimes, default_witness};
use acvm::FieldElement;
use noir_ssa_fuzzer::typed_value::{NumericType, Type};
use std::sync::Arc;

/// Test basic field addition: field_0 + field_1 = 1
#[test]
fn test_field_addition_zero_plus_one() {
    let _ = env_logger::try_init();

    // Create arguments referencing the first two fields from default_witness
    let arg_0_field = NumericArgument { index: 0, numeric_type: NumericType::Field }; // Field(0)
    let arg_1_field = NumericArgument { index: 1, numeric_type: NumericType::Field }; // Field(1)

    // Create an instruction block that adds field_0 + field_1
    let add_block = InstructionBlock {
        instructions: vec![Instruction::AddChecked { lhs: arg_0_field, rhs: arg_1_field }],
    };

    // Create function that executes the addition and returns the result
    let main_function = FunctionData {
        input_types: default_input_types(),
        commands: vec![],                // No additional commands needed
        return_instruction_block_idx: 0, // Return the result of the add block
        return_type: Type::Numeric(NumericType::Field),
    };

    // Create the fuzzer data with our test setup
    let fuzzer_data = FuzzerData {
        instruction_blocks: vec![add_block],
        functions: vec![main_function],
        initial_witness: default_witness(), // [Field(0), Field(1), Field(2), Field(3), Field(4)]
    };

    // Execute the fuzzer
    let result = fuzz_target(fuzzer_data, default_runtimes(), FuzzerOptions::default());

    // Verify the result
    match result.get_return_witnesses().is_empty() {
        true => {
            panic!("Program failed to execute");
        }
        false => {
            assert_eq!(result.get_return_witnesses()[0], FieldElement::from(1_u32));
            println!(
                "✓ Test passed: field_0 + field_1 = {} + {} = {}",
                0,
                1,
                result.get_return_witnesses()[0]
            );
        }
    }
}

///                b0
///if(LAST_BOOL) ↙   ↘ else
///             b1    b2
///              ↘   ↙
///                b3
/// suppose that b1 is failing block, b2 is succeeding block
/// jmpif uses last boolean value defined in the block as condition
/// for first program last boolean value is false
/// for second program sets last boolean value to true
/// we expect that first program succeeds, second program fails
#[test]
fn test_jmp_if() {
    let arg_0_field = NumericArgument { index: 0, numeric_type: NumericType::Field };
    let arg_1_field = NumericArgument { index: 1, numeric_type: NumericType::Field };
    let failing_block = InstructionBlock {
        instructions: vec![Instruction::Div { lhs: arg_1_field, rhs: arg_0_field }], // Field(1) / Field(0)
    };
    let succeeding_block = InstructionBlock {
        instructions: vec![Instruction::AddChecked { lhs: arg_0_field, rhs: arg_1_field }],
    };
    let commands =
        vec![FuzzerFunctionCommand::InsertJmpIfBlock { block_then_idx: 0, block_else_idx: 1 }];
    let data = FuzzerData {
        instruction_blocks: vec![failing_block.clone(), succeeding_block.clone()],
        functions: vec![FunctionData {
            input_types: default_input_types(),
            commands,
            return_instruction_block_idx: 1, // ends with non-failing block
            return_type: Type::Numeric(NumericType::Field),
        }],
        initial_witness: default_witness(),
    };
    let result = fuzz_target(data, default_runtimes(), FuzzerOptions::default());
    // we expect that this program executed successfully
    match result.get_return_witnesses().is_empty() {
        true => {
            panic!("Program failed to execute");
        }
        false => {
            assert_eq!(result.get_return_witnesses()[0], FieldElement::from(1_u32));
        }
    }

    let arg_0_boolean = NumericArgument { index: 0, numeric_type: NumericType::Boolean };
    let arg_1_boolean = NumericArgument { index: 1, numeric_type: NumericType::Boolean };
    let adding_bool_block = InstructionBlock {
        instructions: vec![Instruction::Or { lhs: arg_0_boolean, rhs: arg_1_boolean }],
    };
    let commands = vec![
        FuzzerFunctionCommand::InsertSimpleInstructionBlock { instruction_block_idx: 2 }, // add boolean
        FuzzerFunctionCommand::InsertJmpIfBlock { block_then_idx: 0, block_else_idx: 1 }, // jmpif
    ];
    let data = FuzzerData {
        instruction_blocks: vec![failing_block, succeeding_block, adding_bool_block],
        functions: vec![FunctionData {
            input_types: default_input_types(),
            commands,
            return_instruction_block_idx: 1, // ends with non-failing block
            return_type: Type::Numeric(NumericType::Field),
        }],
        initial_witness: default_witness(),
    };
    let result = fuzz_target(data, default_runtimes(), FuzzerOptions::default());
    // we expect that this program failed to execute
    assert!(result.is_program_compiled());
    assert!(result.get_return_witnesses().is_empty());
}

/// fn main f0 {
///   b0(v0: Field, v1: Field, v2: Field, v3: Field, v4: Field, v5: u1, v6: u1):
///     v7 = allocate -> &mut Field
///     store v0 at v7
///     v8 = add v0, v2
///     store v8 at v7
///     v9 = load v7 -> Field
///     jmp b1()
///   b1():
///     v10 = add v9, v2
///     return v10
/// }
/// v0 = 0, v2 = 2, so we expect that v10 = 4
#[test]
fn test_mutable_variable() {
    let _ = env_logger::try_init();
    let arg_0_field_numeric = NumericArgument { index: 0, numeric_type: NumericType::Field };
    let arg_2_field = NumericArgument { index: 2, numeric_type: NumericType::Field };
    let arg_5_field = Argument { index: 5, value_type: Type::Numeric(NumericType::Field) };
    let arg_6_field = NumericArgument { index: 6, numeric_type: NumericType::Field };

    let arg_0_field = Argument { index: 0, value_type: Type::Numeric(NumericType::Field) };
    let add_to_memory_block =
        InstructionBlock { instructions: vec![Instruction::AddToMemory { lhs: arg_0_field }] };

    let typed_memory_0 = Argument { index: 0, value_type: Type::Numeric(NumericType::Field) };
    let load_block = InstructionBlock {
        instructions: vec![Instruction::LoadFromMemory { memory_addr: typed_memory_0.clone() }],
    };
    let set_block = InstructionBlock {
        instructions: vec![Instruction::SetToMemory { memory_addr_index: 0, value: arg_5_field }],
    };
    let add_block = InstructionBlock {
        instructions: vec![Instruction::AddChecked { lhs: arg_0_field_numeric, rhs: arg_2_field }],
    };
    let add_block_2 = InstructionBlock {
        instructions: vec![Instruction::AddChecked { lhs: arg_6_field, rhs: arg_2_field }],
    };

    let commands = vec![
        FuzzerFunctionCommand::InsertSimpleInstructionBlock { instruction_block_idx: 0 }, // add v0 to memory
        FuzzerFunctionCommand::InsertSimpleInstructionBlock { instruction_block_idx: 3 }, // add v8 = v0 + v2 (6th field)
        FuzzerFunctionCommand::InsertSimpleInstructionBlock { instruction_block_idx: 2 }, // set v6 to memory
        FuzzerFunctionCommand::InsertSimpleInstructionBlock { instruction_block_idx: 1 }, // load from memory (v6)
    ];
    let data = FuzzerData {
        instruction_blocks: vec![
            add_to_memory_block,
            load_block,
            set_block,
            add_block,
            add_block_2,
        ],
        functions: vec![FunctionData {
            input_types: default_input_types(),
            commands,
            return_instruction_block_idx: 4, // last block adds v2 to loaded value, returns the result
            return_type: Type::Numeric(NumericType::Field),
        }],
        initial_witness: default_witness(),
    };
    let result = fuzz_target(data, default_runtimes(), FuzzerOptions::default());
    match result.get_return_witnesses().is_empty() {
        true => {
            panic!("Program failed to execute");
        }
        false => {
            assert_eq!(result.get_return_witnesses()[0], FieldElement::from(4_u32));
        }
    }
}

/// from_le_radix(to_le_radix(field)) == field
#[test]
fn smoke_test_field_to_bytes_to_field() {
    let _ = env_logger::try_init();
    let field_to_bytes_to_field_block =
        InstructionBlock { instructions: vec![Instruction::FieldToBytesToField { field_idx: 1 }] };
    let instructions_blocks = vec![field_to_bytes_to_field_block];
    let commands =
        vec![FuzzerFunctionCommand::InsertSimpleInstructionBlock { instruction_block_idx: 0 }];
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
            assert_eq!(result.get_return_witnesses()[0], FieldElement::from(1_u32));
        }
    }
}

/// fn main(a: Field, b: Field, c: Field) -> pub [Field; 3] {
//      [a, b, c]
/// }
#[test]
fn test_function_can_return_array() {
    let _ = env_logger::try_init();
    let add_array_block = InstructionBlock {
        instructions: vec![Instruction::CreateArray {
            elements_indices: vec![0, 1, 2],
            element_type: Type::Numeric(NumericType::Field),
        }],
    };
    let commands_for_main =
        vec![FuzzerFunctionCommand::InsertSimpleInstructionBlock { instruction_block_idx: 0 }];
    let main_func = FunctionData {
        input_types: default_input_types(),
        commands: commands_for_main,
        return_instruction_block_idx: 1,
        return_type: Type::Array(Arc::new(vec![Type::Numeric(NumericType::Field)]), 3),
    };
    let data = FuzzerData {
        instruction_blocks: vec![add_array_block],
        functions: vec![main_func],
        initial_witness: default_witness(),
    };
    let result = fuzz_target(data, default_runtimes(), FuzzerOptions::default());
    match result.get_return_witnesses().is_empty() {
        true => {
            panic!("Program failed to execute");
        }
        false => {
            assert!(
                result
                    .get_return_witnesses()
                    .iter()
                    .enumerate()
                    .all(|(i, v)| v == &FieldElement::from(i as u32))
            );
        }
    }
}
