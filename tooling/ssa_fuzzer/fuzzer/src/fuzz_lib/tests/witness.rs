//! This file contains tests for the witness
//! 1) Test if array can be used as a initial witness
//! 2) Test if array of arrays can be used as a initial witness

use crate::function_context::FunctionData;
use crate::fuzz_target_lib::fuzz_target;
use crate::fuzzer::FuzzerData;
use crate::initial_witness::{FieldRepresentation, WitnessValue, WitnessValueNumeric};
use crate::instruction::{Instruction, InstructionBlock, NumericArgument};
use crate::options::FuzzerOptions;
use crate::tests::common::default_runtimes;
use acvm::FieldElement;
use noir_ssa_fuzzer::typed_value::{NumericType, Type};
use std::sync::Arc;

/// fn main(arr: [Field; 2], index: u32) -> pub Field {
///   arr[index];
/// }
///
/// Compiled into:
/// Note: v2 and v3 auto generated
/// fn main f0 {
///   b0(v0: [Field; 2], v1: u32, v2: u1, v3: u1):
///   v4 = array_get v0, index v1 -> Field
///   return v4
/// }
#[test]
fn test_array_as_initial_witness() {
    let _ = env_logger::try_init();
    let instruction_get = Instruction::ArrayGet {
        array_index: 0,
        index: NumericArgument { index: 0, numeric_type: NumericType::U32 },
        safe_index: false,
    };
    let array_get_block = InstructionBlock { instructions: vec![instruction_get] };
    let main_function = FunctionData {
        commands: vec![],
        input_types: vec![
            Type::Array(Arc::new(vec![Type::Numeric(NumericType::Field)]), 2),
            Type::Numeric(NumericType::U32),
        ],
        return_instruction_block_idx: 0, // array get
        return_type: Type::Numeric(NumericType::Field),
    };
    let array_witness = WitnessValue::Array(vec![
        WitnessValue::Numeric(WitnessValueNumeric::Field(FieldRepresentation { high: 0, low: 1 })),
        WitnessValue::Numeric(WitnessValueNumeric::Field(FieldRepresentation { high: 0, low: 2 })),
    ]);
    let index_witness_0 = WitnessValue::Numeric(WitnessValueNumeric::U32(0));
    let index_witness_1 = WitnessValue::Numeric(WitnessValueNumeric::U32(1));
    let data = FuzzerData {
        instruction_blocks: vec![array_get_block.clone()],
        initial_witness: vec![array_witness.clone(), index_witness_0],
        functions: vec![main_function.clone()],
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

    let data = FuzzerData {
        instruction_blocks: vec![array_get_block],
        initial_witness: vec![array_witness, index_witness_1],
        functions: vec![main_function],
    };
    let result = fuzz_target(data, default_runtimes(), FuzzerOptions::default());
    match result.get_return_witnesses().is_empty() {
        true => {
            panic!("Program failed to execute");
        }
        false => {
            assert_eq!(result.get_return_witnesses()[0], FieldElement::from(2_u32));
        }
    }
}

/// fn main(arr: [[Field; 2]; 2], index: u32) -> pub [Field; 2] {
///   arr[index];
/// }
#[test]
fn test_array_of_arrays_as_initial_witness() {
    let _ = env_logger::try_init();
    let instruction_get = Instruction::ArrayGet {
        array_index: 0,
        index: NumericArgument { index: 0, numeric_type: NumericType::U32 },
        safe_index: false,
    };
    let array_get_block = InstructionBlock { instructions: vec![instruction_get] };
    let main_function = FunctionData {
        commands: vec![],
        input_types: vec![
            Type::Array(
                Arc::new(vec![Type::Array(Arc::new(vec![Type::Numeric(NumericType::Field)]), 2)]),
                2,
            ),
            Type::Numeric(NumericType::U32),
        ],
        return_instruction_block_idx: 0, // array get
        return_type: Type::Array(Arc::new(vec![Type::Numeric(NumericType::Field)]), 2),
    };
    let array_1 = WitnessValue::Array(vec![
        WitnessValue::Numeric(WitnessValueNumeric::Field(FieldRepresentation { high: 0, low: 1 })),
        WitnessValue::Numeric(WitnessValueNumeric::Field(FieldRepresentation { high: 0, low: 2 })),
    ]);
    let array_2 = WitnessValue::Array(vec![
        WitnessValue::Numeric(WitnessValueNumeric::Field(FieldRepresentation { high: 0, low: 3 })),
        WitnessValue::Numeric(WitnessValueNumeric::Field(FieldRepresentation { high: 0, low: 4 })),
    ]);
    let array_witness = WitnessValue::Array(vec![array_1, array_2]);
    let index_witness_0 = WitnessValue::Numeric(WitnessValueNumeric::U32(0));
    let index_witness_1 = WitnessValue::Numeric(WitnessValueNumeric::U32(1));
    let data = FuzzerData {
        instruction_blocks: vec![array_get_block.clone()],
        initial_witness: vec![array_witness.clone(), index_witness_0],
        functions: vec![main_function.clone()],
    };
    let result = fuzz_target(data, default_runtimes(), FuzzerOptions::default());
    match result.get_return_witnesses().is_empty() {
        true => {
            panic!("Program failed to execute");
        }
        false => {
            assert_eq!(result.get_return_witnesses()[0], FieldElement::from(1_u32));
            assert_eq!(result.get_return_witnesses()[1], FieldElement::from(2_u32));
        }
    }

    let data = FuzzerData {
        instruction_blocks: vec![array_get_block],
        initial_witness: vec![array_witness, index_witness_1],
        functions: vec![main_function],
    };
    let result = fuzz_target(data, default_runtimes(), FuzzerOptions::default());
    match result.get_return_witnesses().is_empty() {
        true => {
            panic!("Program failed to execute");
        }
        false => {
            assert_eq!(result.get_return_witnesses()[0], FieldElement::from(3_u32));
            assert_eq!(result.get_return_witnesses()[1], FieldElement::from(4_u32));
        }
    }
}
