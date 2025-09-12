//! This file contains tests for function calls.
//! 1) Simple function call
//! 2) Several functions with several calls
//! 3) Call in if else
//! 4) Test that the fuzzer doesn't insert too many instructions with function calls
use crate::function_context::{FunctionData, FuzzerFunctionCommand};
use crate::fuzz_target_lib::fuzz_target;
use crate::fuzzer::FuzzerData;
use crate::instruction::{Argument, Instruction, InstructionBlock, NumericArgument};
use crate::options::FuzzerOptions;
use crate::tests::common::{default_input_types, default_runtimes, default_witness};
use acvm::AcirField;
use acvm::FieldElement;
use noir_ssa_fuzzer::typed_value::{NumericType, Type};

/// brillig(inline) fn main f0 {
///    b0(v0: Field, v1: Field, v2: Field, v3: Field, v4: Field, v5: u1, v6: u1):
///      v8 = call f1(v0, v1, v2, v3, v4, v5, v6) -> Field
///      return v8
///  }
///  brillig(inline_always) fn f1 f1 {
///    b0(v0: Field, v1: Field, v2: Field, v3: Field, v4: Field, v5: u1, v6: u1):
///      v7 = add v2, v2
///      return v7
///  }
///  
#[test]
fn simple_function_call() {
    let _ = env_logger::try_init();
    let dummy_var = NumericArgument { index: 2, numeric_type: NumericType::I64 };
    let arg_2_field = NumericArgument { index: 2, numeric_type: NumericType::Field };
    let add_block = InstructionBlock {
        instructions: vec![Instruction::AddChecked { lhs: arg_2_field, rhs: arg_2_field }],
    };
    let dummy_block = InstructionBlock {
        instructions: vec![Instruction::AddChecked { lhs: dummy_var, rhs: dummy_var }],
    };
    let args: [usize; 7] = [0, 1, 2, 3, 4, 0, 1];
    let commands1 = vec![FuzzerFunctionCommand::InsertFunctionCall { function_idx: 0, args }];
    let fuzzer_data = FuzzerData {
        instruction_blocks: vec![dummy_block, add_block],
        functions: vec![
            FunctionData {
                input_types: default_input_types(),
                commands: commands1,
                return_instruction_block_idx: 0,
                return_type: Type::Numeric(NumericType::Field),
            },
            FunctionData {
                input_types: default_input_types(),
                commands: vec![],
                return_instruction_block_idx: 1,
                return_type: Type::Numeric(NumericType::Field),
            },
        ],
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

/// main() { f1() + f2() } -> 8 + 4 => 12
/// f1() { let var = f2(); var + var } -> 4 + 4 => 8
/// f2(v2) { v2 + v2 } -> 2 + 2 => 4
///
///   brillig(inline) fn main f0 {
///     b0(v0: Field, v1: Field, v2: Field, v3: Field, v4: Field, v5: u1, v6: u1):
///       v8 = call f1(v0, v1, v2, v3, v4, v5, v6) -> Field
///       v10 = call f2(v0, v1, v2, v3, v4, v5, v6) -> Field
///       v11 = add v8, v10
///       return v11
///   }
///   brillig(inline_always) fn f1 f1 {
///     b0(v0: Field, v1: Field, v2: Field, v3: Field, v4: Field, v5: u1, v6: u1):
///       v8 = call f2(v0, v1, v2, v3, v4, v5, v6) -> Field
///       v9 = add v8, v8
///       return v9
///   }
///   brillig(inline_always) fn f2 f2 {
///     b0(v0: Field, v1: Field, v2: Field, v3: Field, v4: Field, v5: u1, v6: u1):
///       v7 = add v2, v2
///       return v7
///   }
#[test]
fn several_functions_several_calls() {
    let dummy_var = NumericArgument { index: 2, numeric_type: NumericType::I64 };
    let arg_2_field = NumericArgument { index: 2, numeric_type: NumericType::Field };
    let arg_5_field = NumericArgument { index: 5, numeric_type: NumericType::Field };
    let arg_6_field = NumericArgument { index: 6, numeric_type: NumericType::Field };
    let dummy_block = InstructionBlock {
        instructions: vec![Instruction::AddChecked { lhs: dummy_var, rhs: dummy_var }],
    };
    let add_block_f2 = InstructionBlock {
        instructions: vec![Instruction::AddChecked { lhs: arg_2_field, rhs: arg_2_field }],
    };
    let add_block_f1 = InstructionBlock {
        instructions: vec![Instruction::AddChecked { lhs: arg_5_field, rhs: arg_5_field }], // f2() + f2()
    };
    let add_block_main = InstructionBlock {
        instructions: vec![Instruction::AddChecked { lhs: arg_5_field, rhs: arg_6_field }],
    };

    let args: [usize; 7] = [0, 1, 2, 3, 4, 0, 1];
    let commands_for_main = vec![
        FuzzerFunctionCommand::InsertFunctionCall { function_idx: 0, args },
        FuzzerFunctionCommand::InsertFunctionCall { function_idx: 1, args },
        FuzzerFunctionCommand::InsertSimpleInstructionBlock { instruction_block_idx: 0 },
    ];
    let main_func = FunctionData {
        input_types: default_input_types(),
        commands: commands_for_main,
        return_instruction_block_idx: 3,
        return_type: Type::Numeric(NumericType::Field),
    };
    // for f1 the only defined function is f2
    let commands_for_f1 = vec![FuzzerFunctionCommand::InsertFunctionCall { function_idx: 0, args }];
    let f1_func = FunctionData {
        input_types: default_input_types(),
        commands: commands_for_f1,
        return_instruction_block_idx: 1,
        return_type: Type::Numeric(NumericType::Field),
    };

    let f2_func = FunctionData {
        input_types: default_input_types(),
        commands: vec![],
        return_instruction_block_idx: 2,
        return_type: Type::Numeric(NumericType::Field),
    };
    let fuzzer_data = FuzzerData {
        instruction_blocks: vec![add_block_main, add_block_f1, add_block_f2, dummy_block],
        functions: vec![main_func, f1_func, f2_func],
        initial_witness: default_witness(),
    };
    let result = fuzz_target(fuzzer_data, default_runtimes(), FuzzerOptions::default());
    match result.get_return_witnesses().is_empty() {
        true => {
            panic!("Program failed to execute");
        }
        false => {
            assert_eq!(result.get_return_witnesses()[0], FieldElement::from(12_u32));
        }
    }
}

///   brillig(inline) fn main f0 {
///     b0(v0: Field, v1: Field, v2: Field, v3: Field, v4: Field, v5: u1, v6: u1):
///       v7 = allocate -> &mut Field
///       store v0 at v7
///       jmpif v6 then: b1, else: b2
///     b1():
///       v11 = call f1(v0, v1, v2, v3, v4, v5, v6) -> Field
///       store v11 at v7
///       jmp b3()
///     b2():
///       v9 = call f2(v0, v1, v2, v3, v4, v5, v6) -> Field
///       store v9 at v7
///       jmp b3()
///     b3():
///       v12 = load v7 -> Field
///       return v12
///   }
///   brillig(inline_always) fn f1 f1 {
///     b0(v0: Field, v1: Field, v2: Field, v3: Field, v4: Field, v5: u1, v6: u1):
///       v7 = add v3, v3
///       return v7
///   }
///   brillig(inline_always) fn f2 f2 {
///     b0(v0: Field, v1: Field, v2: Field, v3: Field, v4: Field, v5: u1, v6: u1):
///       v7 = add v2, v2
///       return v7
///   }
#[test]
fn call_in_if_else() {
    let _ = env_logger::try_init();
    let dummy_var = NumericArgument { index: 2, numeric_type: NumericType::I64 };
    let arg_2_field = NumericArgument { index: 2, numeric_type: NumericType::Field };
    let arg_3_field = NumericArgument { index: 3, numeric_type: NumericType::Field };
    let arg_5_field = Argument { index: 5, value_type: Type::Numeric(NumericType::Field) };

    let dummy_block = InstructionBlock {
        instructions: vec![Instruction::AddChecked { lhs: dummy_var, rhs: dummy_var }],
    };
    let add_to_memory_block = InstructionBlock {
        instructions: vec![Instruction::AddToMemory { lhs: arg_5_field.clone() }],
    };
    let typed_memory_0 = Argument { index: 0, value_type: Type::Numeric(NumericType::Field) };
    let set_to_memory_block = InstructionBlock {
        instructions: vec![Instruction::SetToMemory { memory_addr_index: 0, value: arg_5_field }],
    };
    let load_block = InstructionBlock {
        instructions: vec![Instruction::LoadFromMemory { memory_addr: typed_memory_0 }],
    };
    let add_block_f2 = InstructionBlock {
        instructions: vec![Instruction::AddChecked { lhs: arg_2_field, rhs: arg_2_field }], // v2 + v2
    };
    let add_block_f1 = InstructionBlock {
        instructions: vec![Instruction::AddChecked { lhs: arg_3_field, rhs: arg_3_field }], // v3 + v3
    };
    let f1_func = FunctionData {
        input_types: default_input_types(),
        commands: vec![],
        return_instruction_block_idx: 4,
        return_type: Type::Numeric(NumericType::Field),
    };
    let f2_func = FunctionData {
        input_types: default_input_types(),
        commands: vec![],
        return_instruction_block_idx: 5,
        return_type: Type::Numeric(NumericType::Field),
    };

    let args: [usize; 7] = [0, 1, 2, 3, 4, 0, 1];
    let commands_for_main = vec![
        FuzzerFunctionCommand::InsertSimpleInstructionBlock { instruction_block_idx: 1 }, // make memory address for fields
        FuzzerFunctionCommand::InsertJmpIfBlock { block_then_idx: 0, block_else_idx: 0 }, // insert dummy blocks to then and else branches (doesn't insert any opcode)
        FuzzerFunctionCommand::InsertFunctionCall { function_idx: 0, args }, // then call f1()
        FuzzerFunctionCommand::InsertSimpleInstructionBlock { instruction_block_idx: 2 }, // store the result of f1 to memory
        FuzzerFunctionCommand::SwitchToNextBlock,
        FuzzerFunctionCommand::InsertFunctionCall { function_idx: 1, args }, // else call f2()
        FuzzerFunctionCommand::InsertSimpleInstructionBlock { instruction_block_idx: 2 }, // store the result of f1 to memory
    ];
    let mut blocks = vec![
        dummy_block,
        add_to_memory_block,
        set_to_memory_block,
        load_block,
        add_block_f1,
        add_block_f2,
    ];
    let main_func = FunctionData {
        input_types: default_input_types(),
        commands: commands_for_main.clone(),
        return_instruction_block_idx: 3,
        return_type: Type::Numeric(NumericType::Field),
    };
    let result = fuzz_target(
        FuzzerData {
            instruction_blocks: blocks.clone(),
            functions: vec![main_func, f1_func.clone(), f2_func.clone()],
            initial_witness: default_witness(),
        },
        default_runtimes(),
        FuzzerOptions::default(),
    );
    match result.get_return_witnesses().is_empty() {
        true => {
            panic!("Program failed to execute");
        }
        false => {
            assert_eq!(result.get_return_witnesses()[0], FieldElement::from(4_u32));
        }
    }

    let arg_0_boolean = NumericArgument { index: 0, numeric_type: NumericType::Boolean };
    let arg_1_boolean = NumericArgument { index: 1, numeric_type: NumericType::Boolean };
    let add_boolean_block = InstructionBlock {
        instructions: vec![Instruction::Or { lhs: arg_0_boolean, rhs: arg_1_boolean }],
    };

    // use the same commands, but add boolean block at the beginning
    let mut commands = commands_for_main.clone();
    commands.insert(
        0,
        FuzzerFunctionCommand::InsertSimpleInstructionBlock { instruction_block_idx: 4 },
    ); // add true boolean
    log::debug!("commands: {commands:?}");
    blocks.push(add_boolean_block);
    log::debug!("blocks: {blocks:?}");
    let main_func = FunctionData {
        input_types: default_input_types(),
        commands,
        return_instruction_block_idx: 3,
        return_type: Type::Numeric(NumericType::Field),
    };
    let fuzzer_data = FuzzerData {
        instruction_blocks: blocks.clone(),
        functions: vec![main_func, f1_func, f2_func],
        initial_witness: default_witness(),
    };
    let result = fuzz_target(fuzzer_data, default_runtimes(), FuzzerOptions::default());
    match result.get_return_witnesses().is_empty() {
        true => {
            panic!("Program failed to execute");
        }
        false => {
            assert_eq!(result.get_return_witnesses()[0], FieldElement::from(6_u32));
        }
    }
}

/// Test that the fuzzer doesn't insert too many instructions with function calls
///
/// The most cursed test
/// Creates main and 2 functions
/// First function has cycle for 200 iterations for i in 1..200 {i *= i}
/// Second function has cycle for 10 iterations for i in 1..10  {i *= i}
/// Main function tries to insert second and first functions
/// If the second function is too big for the chosen configuration,
/// the fuzzer should insert only the second function and the result should be the output of the second function
/// Otherwise, the result should be the output of the first function
#[test]
fn test_does_not_insert_too_many_instructions_with_function_calls() {
    let dummy_arg = NumericArgument { index: 0, numeric_type: NumericType::I64 };
    let dummy_block = InstructionBlock {
        instructions: vec![Instruction::AddChecked { lhs: dummy_arg, rhs: dummy_arg }],
    };
    let arg_2_field_numeric = NumericArgument { index: 2, numeric_type: NumericType::Field };
    let arg_5_field_numeric = NumericArgument { index: 5, numeric_type: NumericType::Field };
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
            Instruction::MulChecked { lhs: arg_5_field_numeric, rhs: arg_2_field_numeric }, // v14 = mul v13, v2 (v14 -- 6th defined field)
            Instruction::SetToMemory { memory_addr_index: 0, value: arg_6_field }, // store v14 at v8
        ],
    };
    let commands_for_main = vec![
        FuzzerFunctionCommand::InsertFunctionCall {
            function_idx: 1, // second function has cycle for 10 iterations
            args: [0, 1, 2, 3, 4, 0, 1],
        }, // call function, should be skipped
        FuzzerFunctionCommand::InsertFunctionCall {
            function_idx: 0, // first function has cycle for 200 iterations
            args: [0, 1, 2, 3, 4, 0, 1],
        }, // call function, should be skipped
    ];
    let commands_for_function1 = vec![
        FuzzerFunctionCommand::InsertSimpleInstructionBlock { instruction_block_idx: 0 }, // add v2 to memory
        FuzzerFunctionCommand::InsertCycle { block_body_idx: 2, start_iter: 1, end_iter: 200 }, // for i in 1..200 do load_mul_set_block
    ];
    let commands_for_function2 = vec![
        FuzzerFunctionCommand::InsertSimpleInstructionBlock { instruction_block_idx: 0 }, // add v2 to memory
        FuzzerFunctionCommand::InsertCycle { block_body_idx: 2, start_iter: 1, end_iter: 10 }, // for i in 1..10 do load_mul_set_block
    ];
    let main_func = FunctionData {
        input_types: default_input_types(),
        commands: commands_for_main,
        return_instruction_block_idx: 3, // dummy block
        return_type: Type::Numeric(NumericType::Field),
    };
    let function_func = FunctionData {
        input_types: default_input_types(),
        commands: commands_for_function1,
        return_instruction_block_idx: 1, // v12 = load v8 -> Field; return v12
        return_type: Type::Numeric(NumericType::Field),
    };
    let function_func2 = FunctionData {
        input_types: default_input_types(),
        commands: commands_for_function2,
        return_instruction_block_idx: 1, // v12 = load v8 -> Field; return v12
        return_type: Type::Numeric(NumericType::Field),
    };
    let data = FuzzerData {
        instruction_blocks: vec![add_to_memory_block, load_block, load_mul_set_block, dummy_block],
        functions: vec![main_func, function_func, function_func2],
        initial_witness: default_witness(),
    };
    // with max 100 instructions only second function should be executed
    let options = FuzzerOptions { max_instructions_num: 100, ..FuzzerOptions::default() };
    let result = fuzz_target(data.clone(), default_runtimes(), options);
    match result.get_return_witnesses().is_empty() {
        true => {
            panic!("Program failed to execute");
        }
        false => {
            assert_eq!(result.get_return_witnesses()[0], FieldElement::from(1024_u32));
        }
    }
    // with max 1000 instructions both functions should be executed
    // and the result should be the output of the first function
    let options = FuzzerOptions { max_instructions_num: 1000, ..FuzzerOptions::default() };
    let result = fuzz_target(data.clone(), default_runtimes(), options);
    match result.get_return_witnesses().is_empty() {
        true => {
            panic!("Program failed to execute");
        }
        false => {
            assert_eq!(
                result.get_return_witnesses()[0],
                FieldElement::from(2_u32).pow(&FieldElement::from(200_u32)) // 2^200
            );
        }
    }
}
