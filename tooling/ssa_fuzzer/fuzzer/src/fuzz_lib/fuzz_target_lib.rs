use super::NUMBER_OF_VARIABLES_INITIAL;
use super::function_context::{FunctionData, WitnessValue};
use super::fuzzer::Fuzzer;
use super::options::FuzzerOptions;
use acvm::FieldElement;
use acvm::acir::native_types::{Witness, WitnessMap};
use noir_ssa_fuzzer::typed_value::ValueType;

fn initialize_witness_map(
    data: &FunctionData,
) -> (WitnessMap<FieldElement>, Vec<FieldElement>, Vec<ValueType>) {
    let mut witness_map = WitnessMap::new();
    let mut values = vec![];
    let mut types = vec![];
    for (i, witness_value) in data.initial_witness.iter().enumerate() {
        let (value, type_) = match witness_value {
            WitnessValue::Field(field) => (FieldElement::from(field), ValueType::Field),
            WitnessValue::U64(u64) => (FieldElement::from(*u64), ValueType::U64),
            WitnessValue::Boolean(bool) => (FieldElement::from(*bool as u64), ValueType::Boolean),
            WitnessValue::I64(i64) => (FieldElement::from(*i64), ValueType::I64),
            WitnessValue::I32(i32) => (FieldElement::from(*i32 as u64), ValueType::I32),
        };
        witness_map.insert(Witness(i as u32), value);
        values.push(value);
        types.push(type_);
    }
    // insert true and false boolean values
    witness_map.insert(Witness(NUMBER_OF_VARIABLES_INITIAL - 2), FieldElement::from(1_u32));
    values.push(FieldElement::from(1_u32));
    types.push(ValueType::Boolean);
    witness_map.insert(Witness(NUMBER_OF_VARIABLES_INITIAL - 1), FieldElement::from(0_u32));
    values.push(FieldElement::from(0_u32));
    types.push(ValueType::Boolean);
    (witness_map, values, types)
}

/// Creates ACIR and Brillig programs from the data, runs and compares them
pub(crate) fn fuzz_target(data: Vec<FunctionData>, options: FuzzerOptions) -> Option<FieldElement> {
    // If there are no blocks, [crate::base_context::FuzzerContext::get_return_witness] will try to take witness with index NUMBER_OF_VARIABLES_INITIAL
    // But we only have our initial witness, and last index in resulting witness map is NUMBER_OF_VARIABLES_INITIAL - 1
    // So we just skip this case
    // TODO
    /*if data.blocks.is_empty() {
        return None;
    }
    for instr_block in &data.blocks {
        if instr_block.instructions.is_empty() {
            return None;
        }
    }*/

    // to triage
    let (witness_map, _values, _types) = initialize_witness_map(&data[0]);

    let mut fuzzer = Fuzzer::new(options);
    for func in data {
        let (witness_map, values, types) = initialize_witness_map(&func);
        log::debug!("initial_witness: {:?}", witness_map);
        log::debug!("commands: {:?}", func.commands);
        fuzzer.process_function(func, types, values);
    }
    fuzzer.finalize_and_run(witness_map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::function_context::{FieldRepresentation, FuzzerFunctionCommand};
    use crate::instruction::{Argument, Instruction, InstructionBlock};
    use libfuzzer_sys::{arbitrary, arbitrary::Arbitrary};

    fn default_witness() -> [WitnessValue; (NUMBER_OF_VARIABLES_INITIAL - 2) as usize] {
        [
            WitnessValue::Field(FieldRepresentation { high: 0, low: 0 }),
            WitnessValue::Field(FieldRepresentation { high: 0, low: 1 }),
            WitnessValue::Field(FieldRepresentation { high: 0, low: 2 }),
            WitnessValue::Field(FieldRepresentation { high: 0, low: 3 }),
            WitnessValue::Field(FieldRepresentation { high: 0, low: 4 }),
        ]
    }

    #[test]
    fn test_fuzz_target() {
        let data = FunctionData::arbitrary(&mut arbitrary::Unstructured::new(&[1, 2, 3])).unwrap();
        fuzz_target(vec![data], FuzzerOptions::default());
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
        let arg_0_field = Argument { index: 0, value_type: ValueType::Field };
        let arg_1_field = Argument { index: 1, value_type: ValueType::Field };
        let failing_block = InstructionBlock {
            instructions: vec![Instruction::Div { lhs: arg_1_field, rhs: arg_0_field }], // Field(1) / Field(0)
        };
        let succeeding_block = InstructionBlock {
            instructions: vec![Instruction::AddChecked { lhs: arg_0_field, rhs: arg_1_field }],
        };
        let commands =
            vec![FuzzerFunctionCommand::InsertJmpIfBlock { block_then_idx: 0, block_else_idx: 1 }];
        let data = FunctionData {
            blocks: vec![failing_block.clone(), succeeding_block.clone()],
            commands,
            initial_witness: default_witness(),
            return_instruction_block_idx: 1, // ends with non-failing block
            return_type: ValueType::Field,
        };
        let result = fuzz_target(vec![data], FuzzerOptions::default());
        // we expect that this program executed successfully
        match result {
            Some(result) => assert_eq!(result, FieldElement::from(1_u32)),
            None => panic!("Program failed to execute"),
        }

        let arg_0_boolean = Argument { index: 0, value_type: ValueType::Boolean };
        let arg_1_boolean = Argument { index: 1, value_type: ValueType::Boolean };
        let adding_bool_block = InstructionBlock {
            instructions: vec![Instruction::Or { lhs: arg_0_boolean, rhs: arg_1_boolean }],
        };
        let commands = vec![
            FuzzerFunctionCommand::InsertSimpleInstructionBlock { instruction_block_idx: 2 }, // add boolean
            FuzzerFunctionCommand::InsertJmpIfBlock { block_then_idx: 0, block_else_idx: 1 }, // jmpif
        ];
        let data = FunctionData {
            blocks: vec![failing_block, succeeding_block, adding_bool_block],
            commands,
            initial_witness: default_witness(),
            return_instruction_block_idx: 1, // ends with non-failing block
            return_type: ValueType::Field,
        };
        let result = fuzz_target(vec![data], FuzzerOptions::default());
        // we expect that this program failed to execute
        if let Some(result) = result {
            panic!("Program executed successfully with result: {:?}", result);
        }
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
        let arg_0_field = Argument { index: 0, value_type: ValueType::Field };
        let arg_2_field = Argument { index: 2, value_type: ValueType::Field };
        let arg_5_field = Argument { index: 5, value_type: ValueType::Field };
        let arg_6_field = Argument { index: 6, value_type: ValueType::Field };
        let add_to_memory_block =
            InstructionBlock { instructions: vec![Instruction::AddToMemory { lhs: arg_0_field }] };

        let typed_memory_0 = Argument { index: 0, value_type: ValueType::Field };
        let load_block = InstructionBlock {
            instructions: vec![Instruction::LoadFromMemory { memory_addr: typed_memory_0 }],
        };
        let set_block = InstructionBlock {
            instructions: vec![Instruction::SetToMemory {
                memory_addr_index: 0,
                value: arg_5_field,
            }],
        };
        let add_block = InstructionBlock {
            instructions: vec![Instruction::AddChecked { lhs: arg_0_field, rhs: arg_2_field }],
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
        let data = FunctionData {
            blocks: vec![add_to_memory_block, load_block, set_block, add_block, add_block_2],
            commands,
            initial_witness: default_witness(),
            return_instruction_block_idx: 4, // last block adds v2 to loaded value, returns the result
            return_type: ValueType::Field,
        };
        let result = fuzz_target(vec![data], FuzzerOptions::default());
        match result {
            Some(result) => assert_eq!(result, FieldElement::from(4_u32)),
            None => panic!("Program failed to execute"),
        }
    }

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
        let arg_2_field = Argument { index: 2, value_type: ValueType::Field };
        let arg_5_field = Argument { index: 5, value_type: ValueType::Field };
        let arg_6_field = Argument { index: 6, value_type: ValueType::Field };

        // v8 = allocate -> &mut Field (memory address)
        // store v2 at v8
        let add_to_memory_block =
            InstructionBlock { instructions: vec![Instruction::AddToMemory { lhs: arg_2_field }] };
        let typed_memory_0 = Argument { index: 0, value_type: ValueType::Field };
        // load v8 -> Field (loads from first defined memory address, which is v8)
        let load_block = InstructionBlock {
            instructions: vec![Instruction::LoadFromMemory { memory_addr: typed_memory_0 }],
        };
        let load_mul_set_block = InstructionBlock {
            instructions: vec![
                Instruction::LoadFromMemory { memory_addr: typed_memory_0 }, // v13 = load v8 -> Field (loaded value is 5th defined field)
                Instruction::MulChecked { lhs: arg_5_field, rhs: arg_2_field }, // v14 = mul v13, v2 (v14 -- 6th defined field)
                Instruction::SetToMemory { memory_addr_index: 0, value: arg_6_field }, // store v14 at v8
            ],
        };
        let commands = vec![
            FuzzerFunctionCommand::InsertSimpleInstructionBlock { instruction_block_idx: 0 }, // add v2 to memory
            FuzzerFunctionCommand::InsertCycle { block_body_idx: 2, start_iter: 1, end_iter: 10 }, // for i in 1..10 do load_mul_set_block
        ];
        let data = FunctionData {
            blocks: vec![add_to_memory_block, load_block, load_mul_set_block],
            commands,
            initial_witness: default_witness(),
            return_instruction_block_idx: 1, // v12 = load v8 -> Field; return v12
            return_type: ValueType::Field,
        };
        let result = fuzz_target(vec![data], FuzzerOptions::default());
        match result {
            Some(result) => assert_eq!(result, FieldElement::from(1024_u32)),
            None => panic!("Program failed to execute"),
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
        let arg_2_field = Argument { index: 2, value_type: ValueType::Field };
        let arg_5_field = Argument { index: 5, value_type: ValueType::Field };
        let arg_6_field = Argument { index: 6, value_type: ValueType::Field };
        let arg_7_field = Argument { index: 7, value_type: ValueType::Field };
        let arg_8_field = Argument { index: 8, value_type: ValueType::Field };

        // v9 = allocate -> &mut Field
        // store v2 at v9
        let add_to_memory_block =
            InstructionBlock { instructions: vec![Instruction::AddToMemory { lhs: arg_2_field }] };
        let typed_memory_0 = Argument { index: 0, value_type: ValueType::Field };
        // load v9 -> Field (loads from first defined memory address, which is v9)
        let load_block = InstructionBlock {
            instructions: vec![Instruction::LoadFromMemory { memory_addr: typed_memory_0 }],
        };
        let load_mul_set_block = InstructionBlock {
            instructions: vec![
                Instruction::LoadFromMemory { memory_addr: typed_memory_0 }, // v14 = load v9 -> Field (loaded value is 5th defined field)
                Instruction::MulChecked { lhs: arg_5_field, rhs: arg_2_field }, // v15 = mul v14, v2 (v15 -- 6th defined field)
                Instruction::SetToMemory { memory_addr_index: 0, value: arg_6_field }, // store v15 at v9
            ],
        };
        let load_mul_set_block2 = InstructionBlock {
            instructions: vec![
                Instruction::LoadFromMemory { memory_addr: typed_memory_0 }, // v18 = load v9 -> Field (loaded value is 7th defined field)
                Instruction::MulChecked { lhs: arg_7_field, rhs: arg_2_field }, // v19 = mul v18, v2 (v19 -- 8th defined field)
                Instruction::SetToMemory { memory_addr_index: 0, value: arg_8_field }, // store v19 at v9
            ],
        };
        let commands = vec![
            FuzzerFunctionCommand::InsertSimpleInstructionBlock { instruction_block_idx: 0 }, // add v2 to memory
            FuzzerFunctionCommand::InsertCycle { block_body_idx: 2, start_iter: 0, end_iter: 4 }, // for i in 0..4 do load_mul_set_block
            FuzzerFunctionCommand::InsertCycle { block_body_idx: 3, start_iter: 1, end_iter: 4 }, // for j in 1..4 do load_mul_set_block2 (added to previous loop)
        ];
        let data = FunctionData {
            blocks: vec![add_to_memory_block, load_block, load_mul_set_block, load_mul_set_block2],
            commands,
            initial_witness: default_witness(),
            return_instruction_block_idx: 1, // v13 = load v9 -> Field; return v13
            return_type: ValueType::Field,
        };
        let result = fuzz_target(vec![data], FuzzerOptions::default());
        match result {
            Some(result) => assert_eq!(result, FieldElement::from(131072_u32)),
            None => panic!("Program failed to execute"),
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
        let arg_2_field = Argument { index: 2, value_type: ValueType::Field };
        let arg_5_field = Argument { index: 5, value_type: ValueType::Field };
        let arg_6_field = Argument { index: 6, value_type: ValueType::Field };

        // v8 = allocate -> &mut Field (memory address)
        // store v2 at v8
        let add_to_memory_block =
            InstructionBlock { instructions: vec![Instruction::AddToMemory { lhs: arg_2_field }] };
        let typed_memory_0 = Argument { index: 0, value_type: ValueType::Field };

        // v14 = load v8 -> Field
        let load_block = InstructionBlock {
            instructions: vec![Instruction::LoadFromMemory { memory_addr: typed_memory_0 }],
        };

        // end block does not inherit variables defined in cycle body, so we can use this instruction block twice
        // for loop body block and for the end block
        let load_mul_set_block = InstructionBlock {
            instructions: vec![
                Instruction::LoadFromMemory { memory_addr: typed_memory_0 }, // v14 = load v8 -> Field
                Instruction::MulChecked { lhs: arg_5_field, rhs: arg_2_field }, // v15 = mul v14, v2 (v15 -- 6th defined field)
                Instruction::SetToMemory { memory_addr_index: 0, value: arg_6_field }, // store v15 at v8
            ],
        };

        let commands = vec![
            FuzzerFunctionCommand::InsertSimpleInstructionBlock { instruction_block_idx: 0 }, // add v2 to memory
            FuzzerFunctionCommand::InsertCycle { block_body_idx: 2, start_iter: 0, end_iter: 10 }, // for i in 0..10 do load_mul_set_block
            FuzzerFunctionCommand::InsertJmpBlock { block_idx: 1 }, // if we in cycle body, jmp doesn't insert new block, just jumps to the end of the cycle
            FuzzerFunctionCommand::InsertSimpleInstructionBlock { instruction_block_idx: 2 }, // load_mul_set_block
        ];
        let data = FunctionData {
            blocks: vec![add_to_memory_block, load_block, load_mul_set_block],
            commands,
            initial_witness: default_witness(),
            return_instruction_block_idx: 1,
            return_type: ValueType::Field,
        };
        let result = fuzz_target(vec![data], FuzzerOptions::default());
        match result {
            Some(result) => assert_eq!(result, FieldElement::from(4096_u32)),
            None => panic!("Program failed to execute"),
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
        let arg_2_field = Argument { index: 2, value_type: ValueType::Field };
        let arg_5_field = Argument { index: 5, value_type: ValueType::Field };
        let arg_6_field = Argument { index: 6, value_type: ValueType::Field };
        // v9 = allocate -> &mut Field
        // store v2 at v9
        let add_to_memory_block =
            InstructionBlock { instructions: vec![Instruction::AddToMemory { lhs: arg_2_field }] };
        let typed_memory_0 = Argument { index: 0, value_type: ValueType::Field };

        // load v9 -> Field
        let load_block = InstructionBlock {
            instructions: vec![Instruction::LoadFromMemory { memory_addr: typed_memory_0 }],
        };

        // load_*_block will be used for then and else blocks
        // then and else blocks does not share variables, so indices of arguments are the same (loaded variables from then block cannot be used in else block)
        let load_mul_set_block = InstructionBlock {
            instructions: vec![
                Instruction::LoadFromMemory { memory_addr: typed_memory_0 }, // v15 = load v9 -> Field (loaded value is 5th defined field in then block)
                Instruction::MulChecked { lhs: arg_5_field, rhs: arg_2_field }, // v16 = mul v15, v2 (v16 -- 6th defined field in then block)
                Instruction::SetToMemory { memory_addr_index: 0, value: arg_6_field }, // store v16 at v9
            ],
        };
        let load_add_set_block = InstructionBlock {
            instructions: vec![
                Instruction::LoadFromMemory { memory_addr: typed_memory_0 }, // v17 = load v9 -> Field (loaded value is 5th defined field in else block)
                Instruction::AddChecked { lhs: arg_5_field, rhs: arg_2_field }, // v18 = add v17, v2 (v18 -- 6th defined field in else block)
                Instruction::SetToMemory { memory_addr_index: 0, value: arg_6_field }, // store v18 at v9
            ],
        };
        let commands = vec![
            FuzzerFunctionCommand::InsertSimpleInstructionBlock { instruction_block_idx: 0 }, // add v2 to memory
            FuzzerFunctionCommand::InsertCycle { block_body_idx: 1, start_iter: 0, end_iter: 10 }, // for i in 0..10 do ...
            FuzzerFunctionCommand::InsertJmpIfBlock { block_then_idx: 2, block_else_idx: 3 }, // if(LAST_BOOL) { load_mul_set_block } else { load_add_set_block }
        ];
        let data = FunctionData {
            blocks: vec![
                add_to_memory_block.clone(),
                load_block.clone(),
                load_mul_set_block.clone(),
                load_add_set_block.clone(),
            ],
            commands: commands.clone(),
            initial_witness: default_witness(),
            return_instruction_block_idx: 1,
            return_type: ValueType::Field,
        };
        let result = fuzz_target(vec![data], FuzzerOptions::default());
        match result {
            Some(result) => assert_eq!(result, FieldElement::from(22_u32)),
            None => panic!("Program failed to execute"),
        }

        let arg_0_boolean = Argument { index: 0, value_type: ValueType::Boolean };
        let arg_1_boolean = Argument { index: 1, value_type: ValueType::Boolean };
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
        let data = FunctionData {
            blocks: vec![
                add_to_memory_block,
                load_block,
                load_mul_set_block,
                load_add_set_block,
                add_boolean_block,
            ],
            commands,
            initial_witness: default_witness(),
            return_instruction_block_idx: 1,
            return_type: ValueType::Field,
        };
        let result = fuzz_target(vec![data], FuzzerOptions::default());
        match result {
            Some(result) => assert_eq!(result, FieldElement::from(2048_u32)),
            None => panic!("Program failed to execute"),
        }
    }
}
