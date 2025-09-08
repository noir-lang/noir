//! This test is used to test the basic unsigned operations of the fuzzer.
//! It is used to ensure that the fuzzer is able to handle the basic unsigned operations correctly.
//! 1) Add
//! 2) Sub
//! 3) Mul
//! 4) Div
//! 5) Mod
//! 6) Not
//! 7) Shl
//! 8) Shr
//! 9) And
//! 10) Or
//! 11) Xor

use crate::function_context::FunctionData;
use crate::fuzz_target_lib::fuzz_target;
use crate::fuzzer::FuzzerData;
use crate::initial_witness::{WitnessValue, WitnessValueNumeric};
use crate::instruction::{Instruction, InstructionBlock, NumericArgument};
use crate::options::FuzzerOptions;
use crate::tests::common::{default_input_types, default_runtimes};
use acvm::FieldElement;
use noir_ssa_fuzzer::typed_value::{NumericType, Type};

/// Creates default witness values for testing
/// Returns [U64(0), U64(1), U64(2), U64(3), U64(4)]
fn default_unsigned_witness() -> Vec<WitnessValue> {
    vec![
        WitnessValue::Numeric(WitnessValueNumeric::U64(0)),
        WitnessValue::Numeric(WitnessValueNumeric::U64(1)),
        WitnessValue::Numeric(WitnessValueNumeric::U64(2)),
        WitnessValue::Numeric(WitnessValueNumeric::U64(3)),
        WitnessValue::Numeric(WitnessValueNumeric::U64(4)),
    ]
}

enum UnsignedOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Not,
    Shl,
    Shr,
    And,
    Or,
    Xor,
}

/// Returns `4_u64 op 2_u64` if binary, `op 4_u64` if unary
fn test_op_u64(op: UnsignedOp) -> FieldElement {
    let arg_2_u64 = NumericArgument { index: 2, numeric_type: NumericType::U64 };
    let arg_4_u64 = NumericArgument { index: 4, numeric_type: NumericType::U64 };
    let instruction = match op {
        UnsignedOp::Add => Instruction::AddChecked { lhs: arg_4_u64, rhs: arg_2_u64 },
        UnsignedOp::Sub => Instruction::SubChecked { lhs: arg_4_u64, rhs: arg_2_u64 },
        UnsignedOp::Mul => Instruction::MulChecked { lhs: arg_4_u64, rhs: arg_2_u64 },
        UnsignedOp::Div => Instruction::Div { lhs: arg_4_u64, rhs: arg_2_u64 },
        UnsignedOp::Mod => Instruction::Mod { lhs: arg_4_u64, rhs: arg_2_u64 },
        UnsignedOp::Not => Instruction::Not { lhs: arg_4_u64 },
        UnsignedOp::Shl => Instruction::Shl { lhs: arg_4_u64, rhs: arg_2_u64 },
        UnsignedOp::Shr => Instruction::Shr { lhs: arg_4_u64, rhs: arg_2_u64 },
        UnsignedOp::And => Instruction::And { lhs: arg_4_u64, rhs: arg_2_u64 },
        UnsignedOp::Or => Instruction::Or { lhs: arg_4_u64, rhs: arg_2_u64 },
        UnsignedOp::Xor => Instruction::Xor { lhs: arg_4_u64, rhs: arg_2_u64 },
    };
    let instruction_block = InstructionBlock { instructions: vec![instruction] };

    let data = FuzzerData {
        instruction_blocks: vec![instruction_block],
        functions: vec![FunctionData {
            commands: vec![],
            input_types: default_input_types(),
            return_instruction_block_idx: 0,
            return_type: Type::Numeric(NumericType::U64),
        }],
        initial_witness: default_unsigned_witness(),
    };
    fuzz_target(data, default_runtimes(), FuzzerOptions::default()).get_return_witnesses()[0]
}

#[test]
fn test_add() {
    let result = test_op_u64(UnsignedOp::Add);
    // 4 + 2 = 6
    assert_eq!(result, FieldElement::from(6_u64));
}

#[test]
fn test_sub() {
    let result = test_op_u64(UnsignedOp::Sub);
    // 4 - 2 = 2
    assert_eq!(result, FieldElement::from(2_u64));
}

#[test]
fn test_mul() {
    let result = test_op_u64(UnsignedOp::Mul);
    // 2 * 4 = 8
    assert_eq!(result, FieldElement::from(8_u64));
}

#[test]
fn test_div() {
    let result = test_op_u64(UnsignedOp::Div);
    // 4 / 2 = 2
    assert_eq!(result, FieldElement::from(2_u64));
}

#[test]
fn test_mod() {
    let result = test_op_u64(UnsignedOp::Mod);
    // 4 % 2 = 0
    assert_eq!(result, FieldElement::from(0_u64));
}

#[test]
fn test_not() {
    let result = test_op_u64(UnsignedOp::Not);
    assert_eq!(result, FieldElement::from(!4_u64));
}

#[test]
fn test_shl() {
    let result = test_op_u64(UnsignedOp::Shl);
    // 4_u64 << 2_u64 = 16_u64
    assert_eq!(result, FieldElement::from(16_u64));
}

#[test]
fn test_shr() {
    let result = test_op_u64(UnsignedOp::Shr);
    // 4_u64 >> 2_u64 = 1_u64
    assert_eq!(result, FieldElement::from(1_u64));
}

#[test]
fn test_and() {
    let result = test_op_u64(UnsignedOp::And);
    // 4_u64 & 2_u64 = 0_u64
    assert_eq!(result, FieldElement::from(0_u64));
}

#[test]
fn test_or() {
    let result = test_op_u64(UnsignedOp::Or);
    // 4_u64 | 2_u64 = 6_u64
    assert_eq!(result, FieldElement::from(6_u64));
}

#[test]
fn test_xor() {
    let result = test_op_u64(UnsignedOp::Xor);
    // 4_u64 ^ 2_u64 = 6_u64
    assert_eq!(result, FieldElement::from(6_u64));
}
