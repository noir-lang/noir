use super::NUMBER_OF_VARIABLES_INITIAL;
use super::base_context::FuzzerCommand;
use super::fuzzer::Fuzzer;
use super::instruction::InstructionBlock;
use super::options::FuzzerOptions;
use acvm::FieldElement;
use acvm::acir::native_types::{Witness, WitnessMap};
use libfuzzer_sys::arbitrary;
use libfuzzer_sys::arbitrary::Arbitrary;
use noir_ssa_fuzzer::typed_value::ValueType;

/// Field modulus has 254 bits, and FieldElement::from supports u128, so we use two unsigneds to represent a field element
/// field = low + high * 2^128
#[derive(Debug, Clone, Hash, Arbitrary)]
pub(crate) struct FieldRepresentation {
    high: u128,
    low: u128,
}

impl From<&FieldRepresentation> for FieldElement {
    fn from(field: &FieldRepresentation) -> FieldElement {
        let lower = FieldElement::from(field.low);
        let upper = FieldElement::from(field.high);
        lower + upper * (FieldElement::from(u128::MAX) + FieldElement::from(1_u128))
    }
}

#[derive(Debug, Clone, Hash, Arbitrary)]
pub(crate) enum WitnessValue {
    Field(FieldRepresentation),
    U64(u64),
    Boolean(bool),
    I64(u64),
}

/// Represents the data for the fuzzer
/// `methods` - sequence of instructions to be added to the program
/// `initial_witness` - initial witness values for the program as `FieldRepresentation`
#[derive(Arbitrary, Debug)]
pub(crate) struct FuzzerData {
    blocks: Vec<InstructionBlock>,
    commands: Vec<FuzzerCommand>,
    /// initial witness values for the program as `WitnessValue`
    /// last and last but one values are preserved for the boolean values (true, false)
    ///                                                            ↓ we subtract 2, because [initialize_witness_map] func inserts two boolean variables itself
    initial_witness: [WitnessValue; (NUMBER_OF_VARIABLES_INITIAL - 2) as usize],
    return_instruction_block_idx: usize,
}

fn initialize_witness_map(
    data: &FuzzerData,
) -> (WitnessMap<FieldElement>, Vec<FieldElement>, Vec<ValueType>) {
    let mut witness_map = WitnessMap::new();
    let mut values = vec![];
    let mut types = vec![];
    for (i, witness_value) in data.initial_witness.iter().enumerate() {
        let (value, type_) = match witness_value {
            WitnessValue::Field(field) => (FieldElement::from(field), ValueType::Field),
            WitnessValue::U64(u64) => (FieldElement::from(*u64), ValueType::U64),
            WitnessValue::Boolean(bool) => (FieldElement::from(*bool as u64), ValueType::Boolean),
            WitnessValue::I64(i64) => (FieldElement::from(*i64 as u64), ValueType::I64),
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
pub(crate) fn fuzz_target(data: FuzzerData, options: FuzzerOptions) -> Option<FieldElement> {
    // If there are no blocks, [crate::base_context::FuzzerContext::get_return_witness] will try to take witness with index NUMBER_OF_VARIABLES_INITIAL
    // But we only have our initial witness, and last index in resulting witness map is NUMBER_OF_VARIABLES_INITIAL - 1
    // So we just skip this case
    if data.blocks.is_empty() {
        return None;
    }
    for instr_block in &data.blocks {
        if instr_block.instructions.is_empty() {
            return None;
        }
    }

    let (witness_map, values, types) = initialize_witness_map(&data);

    // to triage
    log::debug!("initial_witness: {:?}", witness_map);
    log::debug!("commands: {:?}", data.commands);

    let mut fuzzer = Fuzzer::new(types, values, data.blocks, options);
    for command in data.commands {
        fuzzer.process_fuzzer_command(command);
    }
    fuzzer.finalize_and_run(witness_map, data.return_instruction_block_idx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instruction::{Argument, Instruction};

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
        let data = FuzzerData::arbitrary(&mut arbitrary::Unstructured::new(&[1, 2, 3])).unwrap();
        fuzz_target(data, FuzzerOptions::default());
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
            vec![FuzzerCommand::InsertJmpIfBlock { block_then_idx: 0, block_else_idx: 1 }];
        let data = FuzzerData {
            blocks: vec![failing_block.clone(), succeeding_block.clone()],
            commands,
            initial_witness: default_witness(),
            return_instruction_block_idx: 1, // ends with non-failing block
        };
        let result = fuzz_target(data, FuzzerOptions::default());
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
            FuzzerCommand::InsertSimpleInstructionBlock { instruction_block_idx: 2 }, // add boolean
            FuzzerCommand::InsertJmpIfBlock { block_then_idx: 0, block_else_idx: 1 }, // jmpif
        ];
        let data = FuzzerData {
            blocks: vec![failing_block, succeeding_block, adding_bool_block],
            commands,
            initial_witness: default_witness(),
            return_instruction_block_idx: 1, // ends with non-failing block
        };
        let result = fuzz_target(data, FuzzerOptions::default());
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
            FuzzerCommand::InsertSimpleInstructionBlock { instruction_block_idx: 0 }, // add v0 to memory
            FuzzerCommand::InsertSimpleInstructionBlock { instruction_block_idx: 3 }, // add v8 = v0 + v2 (6th field)
            FuzzerCommand::InsertSimpleInstructionBlock { instruction_block_idx: 2 }, // set v6 to memory
            FuzzerCommand::InsertSimpleInstructionBlock { instruction_block_idx: 1 }, // load from memory (v6)
        ];
        let data = FuzzerData {
            blocks: vec![add_to_memory_block, load_block, set_block, add_block, add_block_2],
            commands,
            initial_witness: default_witness(),
            return_instruction_block_idx: 4, // last block adds v2 to loaded value, returns the result
        };
        let result = fuzz_target(data, FuzzerOptions::default());
        match result {
            Some(result) => assert_eq!(result, FieldElement::from(4_u32)),
            None => panic!("Program failed to execute"),
        }
    }

    /// fn main(x: Field) -> pub Field {
    ///   let mut y = x;
    ///   for i in 0..10 {
    ///     y *= x;
    ///   }
    ///   y
    /// }
    #[test]
    fn test_simple_loop() {
        let arg_2_field = Argument { index: 2, value_type: ValueType::Field };
        let arg_5_field = Argument { index: 5, value_type: ValueType::Field };
        let arg_6_field = Argument { index: 6, value_type: ValueType::Field };
        let add_to_memory_block =
            InstructionBlock { instructions: vec![Instruction::AddToMemory { lhs: arg_2_field }] };
        let typed_memory_0 = Argument { index: 0, value_type: ValueType::Field };
        let load_block = InstructionBlock {
            instructions: vec![Instruction::LoadFromMemory { memory_addr: typed_memory_0 }],
        };
        let load_mul_set_block = InstructionBlock {
            instructions: vec![
                Instruction::LoadFromMemory { memory_addr: typed_memory_0 },
                Instruction::MulChecked { lhs: arg_5_field, rhs: arg_2_field },
                Instruction::SetToMemory { memory_addr_index: 0, value: arg_6_field },
            ],
        };
        let commands = vec![
            FuzzerCommand::InsertSimpleInstructionBlock { instruction_block_idx: 0 }, // add v2 to memory
            FuzzerCommand::InsertCycle {
                block_body_idx: 2,
                block_end_idx: 1,
                start_iter: 1,
                end_iter: 10,
            },
        ];
        let data = FuzzerData {
            blocks: vec![add_to_memory_block, load_block, load_mul_set_block],
            commands,
            initial_witness: default_witness(),
            return_instruction_block_idx: 1, // last block returns v6
        };
        let result = fuzz_target(data, FuzzerOptions::default());
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
        let add_to_memory_block =
            InstructionBlock { instructions: vec![Instruction::AddToMemory { lhs: arg_2_field }] };
        let typed_memory_0 = Argument { index: 0, value_type: ValueType::Field };
        let load_block = InstructionBlock {
            instructions: vec![Instruction::LoadFromMemory { memory_addr: typed_memory_0 }],
        };
        let load_mul_set_block = InstructionBlock {
            instructions: vec![
                Instruction::LoadFromMemory { memory_addr: typed_memory_0 },
                Instruction::MulChecked { lhs: arg_5_field, rhs: arg_2_field },
                Instruction::SetToMemory { memory_addr_index: 0, value: arg_6_field },
            ],
        };
        let load_mul_set_block2 = InstructionBlock {
            instructions: vec![
                Instruction::LoadFromMemory { memory_addr: typed_memory_0 },
                Instruction::MulChecked { lhs: arg_7_field, rhs: arg_2_field },
                Instruction::SetToMemory { memory_addr_index: 0, value: arg_8_field },
            ],
        };
        let commands = vec![
            FuzzerCommand::InsertSimpleInstructionBlock { instruction_block_idx: 0 }, // add v2 to memory
            FuzzerCommand::InsertCycle {
                block_body_idx: 2,
                block_end_idx: 1,
                start_iter: 0,
                end_iter: 4,
            },
            FuzzerCommand::InsertCycle {
                block_body_idx: 3,
                block_end_idx: 1,
                start_iter: 1,
                end_iter: 4,
            },
        ];
        let data = FuzzerData {
            blocks: vec![add_to_memory_block, load_block, load_mul_set_block, load_mul_set_block2],
            commands,
            initial_witness: default_witness(),
            return_instruction_block_idx: 1, // last block returns v6
        };
        let result = fuzz_target(data, FuzzerOptions::default());
        match result {
            Some(result) => assert_eq!(result, FieldElement::from(131072_u32)),
            None => panic!("Program failed to execute"),
        }
    }

    /// fn main(x: Field) -> pub Field{
    ///   let mut y = x;
    ///   for i in 0..10 {
    ///     y *= x;
    ///   }
    ///   y *= x;
    ///   y
    /// }
    /// x = 2; y = 2 ^ 12
    #[test]
    fn test_loop_broken_with_jmp() {
        let arg_2_field = Argument { index: 2, value_type: ValueType::Field };
        let arg_5_field = Argument { index: 5, value_type: ValueType::Field };
        let arg_6_field = Argument { index: 6, value_type: ValueType::Field };
        let add_to_memory_block =
            InstructionBlock { instructions: vec![Instruction::AddToMemory { lhs: arg_2_field }] };
        let typed_memory_0 = Argument { index: 0, value_type: ValueType::Field };
        let load_block = InstructionBlock {
            instructions: vec![Instruction::LoadFromMemory { memory_addr: typed_memory_0 }],
        };
        let load_mul_set_block = InstructionBlock {
            instructions: vec![
                Instruction::LoadFromMemory { memory_addr: typed_memory_0 },
                Instruction::MulChecked { lhs: arg_5_field, rhs: arg_2_field },
                Instruction::SetToMemory { memory_addr_index: 0, value: arg_6_field },
            ],
        };
        let load_mul_set_block2 = InstructionBlock {
            instructions: vec![
                Instruction::LoadFromMemory { memory_addr: typed_memory_0 },
                Instruction::MulChecked { lhs: arg_5_field, rhs: arg_2_field },
                Instruction::SetToMemory { memory_addr_index: 0, value: arg_6_field },
            ],
        };
        let commands = vec![
            FuzzerCommand::InsertSimpleInstructionBlock { instruction_block_idx: 0 }, // add v2 to memory
            FuzzerCommand::InsertCycle {
                block_body_idx: 2,
                block_end_idx: 1,
                start_iter: 0,
                end_iter: 10,
            },
            FuzzerCommand::InsertJmpBlock { block_idx: 1 },
            FuzzerCommand::InsertSimpleInstructionBlock { instruction_block_idx: 3 },
        ];
        let data = FuzzerData {
            blocks: vec![add_to_memory_block, load_block, load_mul_set_block, load_mul_set_block2],
            commands,
            initial_witness: default_witness(),
            return_instruction_block_idx: 1,
        };
        let result = fuzz_target(data, FuzzerOptions::default());
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
        let arg_6_field = Argument { index: 6, value_type: ValueType::Field };
        let arg_8_field = Argument { index: 8, value_type: ValueType::Field };
        let arg_9_field = Argument { index: 9, value_type: ValueType::Field };
        let add_to_memory_block =
            InstructionBlock { instructions: vec![Instruction::AddToMemory { lhs: arg_2_field }] };
        let typed_memory_0 = Argument { index: 0, value_type: ValueType::Field };
        let load_block = InstructionBlock {
            instructions: vec![Instruction::LoadFromMemory { memory_addr: typed_memory_0 }],
        };
        let load_mul_set_block = InstructionBlock {
            instructions: vec![
                Instruction::LoadFromMemory { memory_addr: typed_memory_0 },
                Instruction::MulChecked { lhs: arg_6_field, rhs: arg_2_field },
                Instruction::SetToMemory { memory_addr_index: 0, value: arg_6_field },
            ],
        };
        let load_add_set_block = InstructionBlock {
            instructions: vec![
                Instruction::LoadFromMemory { memory_addr: typed_memory_0 },
                Instruction::AddChecked { lhs: arg_8_field, rhs: arg_2_field },
                Instruction::SetToMemory { memory_addr_index: 0, value: arg_9_field },
            ],
        };
        let commands = vec![
            FuzzerCommand::InsertSimpleInstructionBlock { instruction_block_idx: 0 }, // add v2 to memory
            FuzzerCommand::InsertCycle {
                block_body_idx: 1,
                block_end_idx: 1,
                start_iter: 0,
                end_iter: 10,
            },
            FuzzerCommand::InsertJmpIfBlock { block_then_idx: 2, block_else_idx: 3 },
        ];
        let data = FuzzerData {
            blocks: vec![
                add_to_memory_block.clone(),
                load_block.clone(),
                load_mul_set_block.clone(),
                load_add_set_block.clone(),
            ],
            commands,
            initial_witness: default_witness(),
            return_instruction_block_idx: 1,
        };
        let result = fuzz_target(data, FuzzerOptions::default());
        match result {
            Some(result) => assert_eq!(result, FieldElement::from(22_u32)),
            None => panic!("Program failed to execute"),
        }

        // replaced args order
        let load_mul_set_block = InstructionBlock {
            instructions: vec![
                Instruction::LoadFromMemory { memory_addr: typed_memory_0 },
                Instruction::MulChecked { lhs: arg_8_field, rhs: arg_2_field },
                Instruction::SetToMemory { memory_addr_index: 0, value: arg_9_field },
            ],
        };
        let load_add_set_block = InstructionBlock {
            instructions: vec![
                Instruction::LoadFromMemory { memory_addr: typed_memory_0 },
                Instruction::AddChecked { lhs: arg_6_field, rhs: arg_2_field },
                Instruction::SetToMemory { memory_addr_index: 0, value: arg_6_field },
            ],
        };

        let commands = vec![
            FuzzerCommand::InsertSimpleInstructionBlock { instruction_block_idx: 0 }, // add v2 to memory
            FuzzerCommand::InsertCycle {
                block_body_idx: 1,
                block_end_idx: 1,
                start_iter: 0,
                end_iter: 10,
            },
            FuzzerCommand::InsertJmpIfBlock { block_then_idx: 3, block_else_idx: 2 }, // replaced blocks order
        ];
        let data = FuzzerData {
            blocks: vec![add_to_memory_block, load_block, load_mul_set_block, load_add_set_block],
            commands,
            initial_witness: default_witness(),
            return_instruction_block_idx: 1,
        };
        let result = fuzz_target(data, FuzzerOptions::default());
        match result {
            Some(result) => assert_eq!(result, FieldElement::from(2048_u32)),
            None => panic!("Program failed to execute"),
        }
    }
}
