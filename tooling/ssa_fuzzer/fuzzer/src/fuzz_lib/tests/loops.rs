//! This file contains tests for loops.
//! 1) Simple loop
//! 2) Nested loop
//! 3) Test that Jmp command breaks the loop
//! 4) Test that JmpIf command works in the loop
use crate::function_context::{FunctionData, FuzzerFunctionCommand};
use crate::fuzz_target_lib::fuzz_target;
use crate::fuzzer::FuzzerData;
use crate::instruction::{Argument, Instruction, InstructionBlock, NumericArgument};
use crate::options::FuzzerOptions;
use crate::tests::common::{default_input_types, default_runtimes, default_witness};
use acvm::FieldElement;
use noir_ssa_fuzzer::typed_value::{NumericType, Type};

/// fn main(x: Field) -> pub Field {
///   let mut y = x;
///   for i in 1..10 {
///     y *= x;
///   }
///   y
/// }
/// x = 2, so we expect that y = 2 * 2 ^ 9 = 2 ^ 10
#[test]
fn test_simple_loop() {
    let arg_2_field_numeric = NumericArgument { index: 2, numeric_type: NumericType::Field };
    let arg_5_field = NumericArgument { index: 5, numeric_type: NumericType::Field };
    let arg_6_field = Argument { index: 6, value_type: Type::Numeric(NumericType::Field) };

    let arg_2_field = Argument { index: 2, value_type: Type::Numeric(NumericType::Field) };

    // v8 = allocate -> &mut Field (memory address)
    // store v2 at v8
    let add_to_memory_block =
        InstructionBlock { instructions: vec![Instruction::AddToMemory { lhs: arg_2_field }] };
    let typed_memory_0 = Argument { index: 0, value_type: Type::Numeric(NumericType::Field) };
    // load v8 -> Field (loads from first defined memory address, which is v8)
    let load_block = InstructionBlock {
        instructions: vec![Instruction::LoadFromMemory { memory_addr: typed_memory_0.clone() }],
    };
    let load_mul_set_block = InstructionBlock {
        instructions: vec![
            Instruction::LoadFromMemory { memory_addr: typed_memory_0 }, // v13 = load v8 -> Field (loaded value is 5th defined field)
            Instruction::MulChecked { lhs: arg_5_field, rhs: arg_2_field_numeric }, // v14 = mul v13, v2 (v14 -- 6th defined field)
            Instruction::SetToMemory { memory_addr_index: 0, value: arg_6_field }, // store v14 at v8
        ],
    };
    let commands = vec![
        FuzzerFunctionCommand::InsertSimpleInstructionBlock { instruction_block_idx: 0 }, // add v2 to memory
        FuzzerFunctionCommand::InsertCycle { block_body_idx: 2, start_iter: 1, end_iter: 10 }, // for i in 1..10 do load_mul_set_block
    ];
    let data = FuzzerData {
        instruction_blocks: vec![add_to_memory_block, load_block, load_mul_set_block],
        functions: vec![FunctionData {
            input_types: default_input_types(),
            commands,
            return_instruction_block_idx: 1, // v12 = load v8 -> Field; return v12
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
            assert_eq!(result.get_return_witnesses()[0], FieldElement::from(1024_u32));
        }
    }
}

/// fn main(x: Field) -> pub Field{
///   let mut y = x;
///   for i in 0..4 {
///     y *= x;
///     for j in 1..4 {
///       y *= x;
///     }
///   }
///   y
/// }
/// x = 2; so we expect y = x * x ^ (4 * 4) = 2 * 2 ^ 16 = 2 ^ 17
#[test]
fn test_nested_loop() {
    let arg_2_field_numeric = NumericArgument { index: 2, numeric_type: NumericType::Field };
    let arg_5_field = NumericArgument { index: 5, numeric_type: NumericType::Field };
    let arg_6_field = Argument { index: 6, value_type: Type::Numeric(NumericType::Field) };
    let arg_7_field = NumericArgument { index: 7, numeric_type: NumericType::Field };
    let arg_8_field = Argument { index: 8, value_type: Type::Numeric(NumericType::Field) };

    let arg_2_field = Argument { index: 2, value_type: Type::Numeric(NumericType::Field) };

    // v9 = allocate -> &mut Field
    // store v2 at v9
    let add_to_memory_block =
        InstructionBlock { instructions: vec![Instruction::AddToMemory { lhs: arg_2_field }] };
    let typed_memory_0 = Argument { index: 0, value_type: Type::Numeric(NumericType::Field) };
    // load v9 -> Field (loads from first defined memory address, which is v9)
    let load_block = InstructionBlock {
        instructions: vec![Instruction::LoadFromMemory { memory_addr: typed_memory_0.clone() }],
    };
    let load_mul_set_block = InstructionBlock {
        instructions: vec![
            Instruction::LoadFromMemory { memory_addr: typed_memory_0.clone() }, // v14 = load v9 -> Field (loaded value is 5th defined field)
            Instruction::MulChecked { lhs: arg_5_field, rhs: arg_2_field_numeric }, // v15 = mul v14, v2 (v15 -- 6th defined field)
            Instruction::SetToMemory { memory_addr_index: 0, value: arg_6_field }, // store v15 at v9
        ],
    };
    let load_mul_set_block2 = InstructionBlock {
        instructions: vec![
            Instruction::LoadFromMemory { memory_addr: typed_memory_0 }, // v18 = load v9 -> Field (loaded value is 7th defined field)
            Instruction::MulChecked { lhs: arg_7_field, rhs: arg_2_field_numeric }, // v19 = mul v18, v2 (v19 -- 8th defined field)
            Instruction::SetToMemory { memory_addr_index: 0, value: arg_8_field }, // store v19 at v9
        ],
    };
    let commands = vec![
        FuzzerFunctionCommand::InsertSimpleInstructionBlock { instruction_block_idx: 0 }, // add v2 to memory
        FuzzerFunctionCommand::InsertCycle { block_body_idx: 2, start_iter: 0, end_iter: 4 }, // for i in 0..4 do load_mul_set_block
        FuzzerFunctionCommand::InsertCycle { block_body_idx: 3, start_iter: 1, end_iter: 4 }, // for j in 1..4 do load_mul_set_block2 (added to previous loop)
    ];
    let data = FuzzerData {
        instruction_blocks: vec![
            add_to_memory_block,
            load_block,
            load_mul_set_block,
            load_mul_set_block2,
        ],
        functions: vec![FunctionData {
            input_types: default_input_types(),
            commands,
            return_instruction_block_idx: 1, // v13 = load v9 -> Field; return v13
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
            assert_eq!(result.get_return_witnesses()[0], FieldElement::from(131072_u32));
        }
    }
}

/// Tests if fuzzer Jmp command breaks the loop
///
/// fn main(x: Field) -> pub Field{
///   let mut y = x;
///   for i in 0..10 {
///     y *= x;
///   }
///   y *= x;
///   y
/// }
/// x = 2; so we expect that y = 2 ^ 12
///
/// if Jmp command does not break the loop, we will receive the following program:
/// fn main(x: Field) -> pub Field{
///   let mut y = x;
///   for i in 0..10 {
///     y *= x;
///     y *= x;
///   }
///   y
/// }
#[test]
fn test_loop_broken_with_jmp() {
    let arg_2_field_numeric = NumericArgument { index: 2, numeric_type: NumericType::Field };
    let arg_5_field = NumericArgument { index: 5, numeric_type: NumericType::Field };
    let arg_6_field = Argument { index: 6, value_type: Type::Numeric(NumericType::Field) };

    let arg_2_field = Argument { index: 2, value_type: Type::Numeric(NumericType::Field) };

    // v8 = allocate -> &mut Field (memory address)
    // store v2 at v8
    let add_to_memory_block =
        InstructionBlock { instructions: vec![Instruction::AddToMemory { lhs: arg_2_field }] };
    let typed_memory_0 = Argument { index: 0, value_type: Type::Numeric(NumericType::Field) };

    // v14 = load v8 -> Field
    let load_block = InstructionBlock {
        instructions: vec![Instruction::LoadFromMemory { memory_addr: typed_memory_0.clone() }],
    };

    // end block does not inherit variables defined in cycle body, so we can use this instruction block twice
    // for loop body block and for the end block
    let load_mul_set_block = InstructionBlock {
        instructions: vec![
            Instruction::LoadFromMemory { memory_addr: typed_memory_0 }, // v14 = load v8 -> Field
            Instruction::MulChecked { lhs: arg_5_field, rhs: arg_2_field_numeric }, // v15 = mul v14, v2 (v15 -- 6th defined field)
            Instruction::SetToMemory { memory_addr_index: 0, value: arg_6_field }, // store v15 at v8
        ],
    };

    let commands = vec![
        FuzzerFunctionCommand::InsertSimpleInstructionBlock { instruction_block_idx: 0 }, // add v2 to memory
        FuzzerFunctionCommand::InsertCycle { block_body_idx: 2, start_iter: 0, end_iter: 10 }, // for i in 0..10 do load_mul_set_block
        FuzzerFunctionCommand::InsertJmpBlock { block_idx: 1 }, // if we in cycle body, jmp doesn't insert new block, just jumps to the end of the cycle
        FuzzerFunctionCommand::InsertSimpleInstructionBlock { instruction_block_idx: 2 }, // load_mul_set_block
    ];
    let data = FuzzerData {
        instruction_blocks: vec![add_to_memory_block, load_block, load_mul_set_block],
        functions: vec![FunctionData {
            input_types: default_input_types(),
            commands,
            return_instruction_block_idx: 1,
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
            assert_eq!(result.get_return_witnesses()[0], FieldElement::from(4096_u32));
        }
    }
}

/// fn main(x: Field, cond: bool) -> pub Field{
///   let mut y = x;
///   for i in 0..10 {
///     if cond {
///       y *= x;
///     } else {
///       y += x;
///     }
///   }
///   y
/// }
/// x = 2; if cond = 1: y = 2 * 2 ^ 10, if cond = 0: y = 2 + 10 * 2 = 22
#[test]
fn test_jmp_if_in_cycle() {
    let arg_2_field_numeric = NumericArgument { index: 2, numeric_type: NumericType::Field };
    let arg_6_field = NumericArgument { index: 6, numeric_type: NumericType::Field };
    let arg_7_field = Argument { index: 7, value_type: Type::Numeric(NumericType::Field) };

    let arg_2_field = Argument { index: 2, value_type: Type::Numeric(NumericType::Field) };
    // v9 = allocate -> &mut Field
    // store v2 at v9
    let add_to_memory_block =
        InstructionBlock { instructions: vec![Instruction::AddToMemory { lhs: arg_2_field }] };
    let typed_memory_0 = Argument { index: 0, value_type: Type::Numeric(NumericType::Field) };

    // load v9 -> Field
    let load_block = InstructionBlock {
        instructions: vec![Instruction::LoadFromMemory { memory_addr: typed_memory_0.clone() }],
    };

    // load_*_block will be used for then and else blocks
    // then and else blocks does not share variables, so indices of arguments are the same (loaded variables from then block cannot be used in else block)
    let load_mul_set_block = InstructionBlock {
        instructions: vec![
            Instruction::LoadFromMemory { memory_addr: typed_memory_0.clone() }, // v15 = load v9 -> Field (loaded value is 5th defined field in then block)
            Instruction::MulChecked { lhs: arg_6_field, rhs: arg_2_field_numeric }, // v16 = mul v15, v2 (v16 -- 6th defined field in then block)
            Instruction::SetToMemory { memory_addr_index: 0, value: arg_7_field.clone() }, // store v16 at v9
        ],
    };
    let load_add_set_block = InstructionBlock {
        instructions: vec![
            Instruction::LoadFromMemory { memory_addr: typed_memory_0 }, // v17 = load v9 -> Field (loaded value is 5th defined field in else block)
            Instruction::AddChecked { lhs: arg_6_field, rhs: arg_2_field_numeric }, // v18 = add v17, v2 (v18 -- 6th defined field in else block)
            Instruction::SetToMemory { memory_addr_index: 0, value: arg_7_field }, // store v18 at v9
        ],
    };
    let commands = vec![
        FuzzerFunctionCommand::InsertSimpleInstructionBlock { instruction_block_idx: 0 }, // add v2 to memory
        FuzzerFunctionCommand::InsertCycle { block_body_idx: 1, start_iter: 0, end_iter: 10 }, // for i in 0..10 do ...
        FuzzerFunctionCommand::InsertJmpIfBlock { block_then_idx: 2, block_else_idx: 3 }, // if(LAST_BOOL) { load_mul_set_block } else { load_add_set_block }
    ];
    let data = FuzzerData {
        instruction_blocks: vec![
            add_to_memory_block.clone(),
            load_block.clone(),
            load_mul_set_block.clone(),
            load_add_set_block.clone(),
        ],
        functions: vec![FunctionData {
            input_types: default_input_types(),
            commands: commands.clone(),
            return_instruction_block_idx: 1,
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
            assert_eq!(result.get_return_witnesses()[0], FieldElement::from(22_u32));
        }
    }

    let arg_0_boolean = NumericArgument { index: 0, numeric_type: NumericType::Boolean };
    let arg_1_boolean = NumericArgument { index: 1, numeric_type: NumericType::Boolean };
    let add_boolean_block = InstructionBlock {
        instructions: vec![Instruction::Or { lhs: arg_0_boolean, rhs: arg_1_boolean }], // jmpif uses last defined boolean variable
                                                                                        // [initialize_witness_map] func inserts two boolean variables itself, first is true, last is false
                                                                                        // so by inserting new boolean = first | last, we will get last variable = true
    };

    // use the same commands, but add boolean block at the beginning
    let mut commands = commands;
    commands.insert(
        0,
        FuzzerFunctionCommand::InsertSimpleInstructionBlock { instruction_block_idx: 4 },
    ); // add true boolean
    let data = FuzzerData {
        instruction_blocks: vec![
            add_to_memory_block,
            load_block,
            load_mul_set_block,
            load_add_set_block,
            add_boolean_block,
        ],
        functions: vec![FunctionData {
            input_types: default_input_types(),
            commands,
            return_instruction_block_idx: 1,
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
            assert_eq!(result.get_return_witnesses()[0], FieldElement::from(2048_u32));
        }
    }
}
