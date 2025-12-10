//! This file contains tests for advanced references.
//! 1) Test that other function mutates reference
use crate::function_context::{FunctionData, FuzzerFunctionCommand};
use crate::fuzz_target_lib::fuzz_target;
use crate::fuzzer::FuzzerData;
use crate::instruction::{Argument, Instruction, InstructionBlock, NumericArgument};
use crate::options::FuzzerOptions;
use crate::tests::common::{default_input_types, default_runtimes, default_witness};
use acvm::FieldElement;
use noir_ssa_fuzzer::typed_value::{NumericType, Type};
use std::sync::Arc;

/// fn main(a: Field) -> pub Field {
///   let mut t = a;
///   func(&mut t);
///   t
/// }
///
/// fn func(a: &mut Field) {
///   *a += 1;
/// }
/// "a" = 0
/// [nargo_tests] Circuit output: 0x01
#[test]
fn test_other_function_mutates_reference() {
    let _ = env_logger::try_init();
    let arg_0_numeric = NumericArgument { index: 0, numeric_type: NumericType::Field };
    let arg_1_numeric = NumericArgument { index: 1, numeric_type: NumericType::Field };

    let arg_0 = Argument { index: 0, value_type: Type::Numeric(NumericType::Field) };
    let arg_2 = Argument { index: 2, value_type: Type::Numeric(NumericType::Field) };
    let add_to_memory_block =
        InstructionBlock { instructions: vec![Instruction::AddToMemory { lhs: arg_0 }] };
    let typed_memory_0 = Argument { index: 0, value_type: Type::Numeric(NumericType::Field) };
    let load_block = InstructionBlock {
        instructions: vec![Instruction::LoadFromMemory { memory_addr: typed_memory_0 }],
    };
    let set_block = InstructionBlock {
        instructions: vec![Instruction::SetToMemory { memory_addr_index: 0, value: arg_2 }],
    };
    let add_block = InstructionBlock {
        instructions: vec![Instruction::AddChecked { lhs: arg_0_numeric, rhs: arg_1_numeric }],
    };
    let instruction_blocks = vec![add_to_memory_block, load_block, set_block, add_block];
    let main_commands = vec![
        FuzzerFunctionCommand::InsertSimpleInstructionBlock { instruction_block_idx: 0 }, // add v0 to memory
        FuzzerFunctionCommand::InsertFunctionCall { function_idx: 0, args: [0, 1, 2, 3, 4, 5, 6] }, // call func(&mut v0, v1) other args ignored
    ];
    let func_commands = vec![
        FuzzerFunctionCommand::InsertSimpleInstructionBlock { instruction_block_idx: 1 }, // v3 := load v0 from memory
        FuzzerFunctionCommand::InsertSimpleInstructionBlock { instruction_block_idx: 3 }, // v4 := add v3 v1
        FuzzerFunctionCommand::InsertSimpleInstructionBlock { instruction_block_idx: 2 }, // set v4 to memory v0
    ];

    let main_function = FunctionData {
        input_types: default_input_types(),
        commands: main_commands,
        return_instruction_block_idx: 1, // load v0 from memory
        return_type: Type::Numeric(NumericType::Field),
    };
    let func_function = FunctionData {
        input_types: vec![
            Type::Reference(Arc::new(Type::Numeric(NumericType::Field))),
            Type::Numeric(NumericType::Field),
        ],
        commands: func_commands,
        return_instruction_block_idx: 1, // dummy
        return_type: Type::Numeric(NumericType::Field),
    };

    let data = FuzzerData {
        functions: vec![main_function, func_function],
        initial_witness: default_witness(),
        instruction_blocks,
    };

    let result = fuzz_target(data, default_runtimes(), FuzzerOptions::default());
    match result.get_return_witnesses().is_empty() {
        true => {
            panic!("Program failed to execute");
        }
        false => {
            assert_eq!(result.get_return_witnesses()[0], FieldElement::from(1_u32));
        }
    }
}

/// let t = &mut(&mut a);
/// *t = &mut b;
/// return **t;
/// This test is compiled into
/// fn main f0 {
///   b0(v0: Field, v1: Field, v2: Field, v3: Field, v4: Field, v5: u1, v6: u1):
///     v7 = allocate -> &mut Field
///     store v0 at v7
///     v8 = allocate -> &mut Field
///     store v1 at v8
///     v9 = allocate -> &mut &mut Field
///     store v7 at v9
///     store v8 at v9
///     v10 = load v9 -> &mut Field
///     v11 = load v10 -> Field
///     jmp b1()
///   b1():
///     store v8 at v9 // dummy
///     return v11
///   }
#[test]
fn test_reference_to_reference() {
    let arg_0 = Argument { index: 0, value_type: Type::Numeric(NumericType::Field) };
    let arg_1 = Argument { index: 1, value_type: Type::Numeric(NumericType::Field) };
    let typed_memory_0 = Argument {
        index: 0,
        value_type: Type::Reference(Arc::new(Type::Numeric(NumericType::Field))),
    };
    let typed_memory_1 = Argument {
        index: 1,
        value_type: Type::Reference(Arc::new(Type::Numeric(NumericType::Field))),
    };
    let add_to_memory_block = InstructionBlock {
        instructions: vec![
            Instruction::AddToMemory { lhs: arg_0 },          // &mut a
            Instruction::AddToMemory { lhs: arg_1 },          // &mut b
            Instruction::AddToMemory { lhs: typed_memory_0 }, // &mut (&mut a)
        ],
    };
    let set_to_memory_block = InstructionBlock {
        instructions: vec![Instruction::SetToMemory {
            memory_addr_index: 0,
            value: typed_memory_1,
        }],
    }; // sets typed_memory_0 to memory (&mut &mut b)

    let reference_arg_0 = Argument {
        index: 0,
        value_type: Type::Reference(Arc::new(Type::Numeric(NumericType::Field))),
    };
    let typed_memory_2 = Argument { index: 2, value_type: Type::Numeric(NumericType::Field) };
    let load_block = InstructionBlock {
        instructions: vec![
            Instruction::LoadFromMemory { memory_addr: reference_arg_0 }, // t = *(&mut &mut b)
            Instruction::LoadFromMemory { memory_addr: typed_memory_2 }, // r = *t == **(&mut &mut b)
        ],
    }; // loads reference_arg_0 from memory (&mut &mut b) and loads typed_memory_2 from memory (**(&mut &mut b))

    let instruction_blocks = vec![add_to_memory_block, set_to_memory_block, load_block];
    let main_commands = vec![
        FuzzerFunctionCommand::InsertSimpleInstructionBlock { instruction_block_idx: 0 },
        FuzzerFunctionCommand::InsertSimpleInstructionBlock { instruction_block_idx: 1 },
        FuzzerFunctionCommand::InsertSimpleInstructionBlock { instruction_block_idx: 2 },
    ];
    let main_function = FunctionData {
        input_types: default_input_types(),
        commands: main_commands,
        return_instruction_block_idx: 1, // load v0 from memory
        return_type: Type::Numeric(NumericType::Field),
    };
    let data = FuzzerData {
        functions: vec![main_function],
        initial_witness: default_witness(),
        instruction_blocks,
    };
    let result = fuzz_target(data, default_runtimes(), FuzzerOptions::default());
    match result.get_return_witnesses().is_empty() {
        true => {
            panic!("Program failed to execute");
        }
        false => {
            assert_eq!(result.get_return_witnesses()[0], FieldElement::from(1_u32));
        }
    }
}
