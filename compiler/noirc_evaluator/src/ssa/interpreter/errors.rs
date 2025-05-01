use crate::ssa::ir::{basic_block::BasicBlockId, instruction::BinaryOp, value::ValueId};
use thiserror::Error;

pub(super) const MAX_SIGNED_BIT_SIZE: u32 = 64;
pub(super) const MAX_UNSIGNED_BIT_SIZE: u32 = 128;

#[derive(Debug, Error)]
pub(crate) enum InterpreterError {
    #[error(
        "Argument count {arguments} to block {block} does not match the expected parameter count {parameters}"
    )]
    BlockArgumentCountMismatch { block: BasicBlockId, arguments: usize, parameters: usize },
    #[error("Block {block} is missing the terminator instruction")]
    BlockMissingTerminator { block: BasicBlockId },
    #[error("constrain {lhs_id} == {rhs_id} failed:\n    {lhs} != {rhs}")]
    ConstrainEqFailed { lhs: String, lhs_id: ValueId, rhs: String, rhs_id: ValueId },
    #[error("constrain {lhs_id} != {rhs_id} failed:\n    {lhs} == {rhs}")]
    ConstrainNeFailed { lhs: String, lhs_id: ValueId, rhs: String, rhs_id: ValueId },
    #[error(
        "Range check of {value_id} = {value} failed.\n  Max bits allowed by range check = {max_bits}\n  Actual bit count = {actual_bits}"
    )]
    RangeCheckFailed { value: String, value_id: ValueId, actual_bits: u32, max_bits: u32 },
    #[error(
        "Range check of {value_id} = {value} failed.\n  Max bits allowed by range check = {max_bits}\n  Actual bit count = {actual_bits}\n  {message}"
    )]
    RangeCheckFailedWithMessage {
        value: String,
        value_id: ValueId,
        actual_bits: u32,
        max_bits: u32,
        message: String,
    },
    #[error("Call to unknown foreign function {name}")]
    UnknownForeignFunctionCall { name: String },
    #[error("Cannot call non-function value {value_id} = {value}")]
    CalledNonFunction { value: String, value_id: ValueId },
    // Note that we don't need to display the value_id because displaying a reference
    // value shows the original value id anyway
    #[error("Reference value `{value}` passed from a constrained to an unconstrained function")]
    ReferenceValueCrossedUnconstrainedBoundary { value: String },
    #[error("Reference value `{value}` loaded before it was first stored to")]
    UninitializedReferenceValueLoaded { value: String },
    #[error(
        "Mismatched types in binary operator: `{operator} {lhs_id}, {rhs_id}`  ({operator} {lhs}, {rhs})"
    )]
    MismatchedTypesInBinaryOperator {
        lhs_id: ValueId,
        lhs: String,
        operator: BinaryOp,
        rhs_id: ValueId,
        rhs: String,
    },
    #[error("Division by zero: `div {lhs_id}, {rhs_id}`  ({lhs} / {rhs})")]
    DivisionByZero { lhs_id: ValueId, lhs: String, rhs_id: ValueId, rhs: String },
    #[error("Unsupported operator `{operator}` for type `{typ}`")]
    UnsupportedOperatorForType { operator: &'static str, typ: &'static str },
    #[error(
        "Invalid bit size of `{bit_size}` given to truncate, maximum size allowed for unsigned values is {MAX_UNSIGNED_BIT_SIZE}"
    )]
    InvalidUnsignedTruncateBitSize { bit_size: u32 },
    #[error(
        "Invalid bit size of `{bit_size}` given to truncate, maximum size allowed for signed values is {MAX_SIGNED_BIT_SIZE}"
    )]
    InvalidSignedTruncateBitSize { bit_size: u32 },
    #[error("Rhs of `{operator}` should be a u32 but found `{rhs_id} = {rhs}`")]
    RhsOfBitShiftShouldBeU32 { operator: &'static str, rhs_id: ValueId, rhs: String },
    #[error(
        "Expected {expected_type} value in {instruction} but instead found `{value_id} = {value}`"
    )]
    TypeError {
        value_id: ValueId,
        value: String,
        expected_type: &'static str,
        instruction: &'static str,
    },
    #[error("Underflow in dec_rc when decrementing reference count of `{value_id} = {value}`")]
    DecRcUnderflow { value_id: ValueId, value: String },
    #[error(
        "Erroneously incremented reference count of value `{value_id} = {value}` from 0 back to 1"
    )]
    IncRcRevive { value_id: ValueId, value: String },
    #[error("An overflow occurred while evaluating {instruction}")]
    Overflow { instruction: String },
    #[error(
        "Function {function} ({function_name}) returned {actual} argument(s) but it was expected to return {expected}"
    )]
    FunctionReturnedIncorrectArgCount {
        function: ValueId,
        function_name: String,
        expected: usize,
        actual: usize,
    },
    #[error(
        "if-else instruction with then condition `{then_condition_id}` and else condition `{else_condition_id}` has both branches as true. This should be impossible except for malformed SSA code"
    )]
    DoubleTrueIfElse { then_condition_id: ValueId, else_condition_id: ValueId },
    #[error(
        "`truncate {value_id} to 0 bits, max_bit_size: {max_bit_size}` has invalid bit size 0. This should only be possible in malformed SSA."
    )]
    TruncateToZeroBits { value_id: ValueId, max_bit_size: u32 },
    #[error(
        "`range_check {value_id} to 0 bits` has invalid bit size 0. This should only be possible in malformed SSA."
    )]
    RangeCheckToZeroBits { value_id: ValueId },
}
