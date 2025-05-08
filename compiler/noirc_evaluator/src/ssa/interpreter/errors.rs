use crate::ssa::ir::{
    basic_block::BasicBlockId,
    instruction::{BinaryOp, Intrinsic},
    value::ValueId,
};
use acvm::FieldElement;
use thiserror::Error;

pub(super) const MAX_SIGNED_BIT_SIZE: u32 = 64;
pub(super) const MAX_UNSIGNED_BIT_SIZE: u32 = 128;

#[derive(Debug, Error)]
pub(crate) enum InterpreterError {
    /// These errors are all the result from malformed input SSA
    #[error("{0}")]
    Internal(InternalError),
    #[error("constrain {lhs_id} == {rhs_id} failed:\n    {lhs} != {rhs}")]
    ConstrainEqFailed { lhs: String, lhs_id: ValueId, rhs: String, rhs_id: ValueId },
    #[error("constrain {lhs_id} != {rhs_id} failed:\n    {lhs} == {rhs}")]
    ConstrainNeFailed { lhs: String, lhs_id: ValueId, rhs: String, rhs_id: ValueId },
    #[error("static_assert `{condition}` failed: {message}")]
    StaticAssertFailed { condition: ValueId, message: String },
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
    /// This is not an internal error since the SSA is still valid. We're just not able to
    /// interpret it since we lack the context of what the external function is.
    #[error("Call to unknown foreign function {name}")]
    UnknownForeignFunctionCall { name: String },
    #[error("Division by zero: `div {lhs_id}, {rhs_id}`  ({lhs} / {rhs})")]
    DivisionByZero { lhs_id: ValueId, lhs: String, rhs_id: ValueId, rhs: String },
    #[error("Underflow in dec_rc when decrementing reference count of `{value_id} = {value}`")]
    DecRcUnderflow { value_id: ValueId, value: String },
    #[error(
        "Erroneously incremented reference count of value `{value_id} = {value}` from 0 back to 1"
    )]
    IncRcRevive { value_id: ValueId, value: String },
    #[error("An overflow occurred while evaluating {instruction}")]
    Overflow { instruction: String },
    #[error(
        "if-else instruction with then condition `{then_condition_id}` and else condition `{else_condition_id}` has both branches as true. This should be impossible except for malformed SSA code"
    )]
    DoubleTrueIfElse { then_condition_id: ValueId, else_condition_id: ValueId },
    #[error("Tried to pop from empty slice `{slice}` in `{instruction}`")]
    PoppedFromEmptySlice { slice: ValueId, instruction: &'static str },
    #[error("Unable to convert `{field_id} = {field}` to radix {radix}")]
    ToRadixFailed { field_id: ValueId, field: FieldElement, radix: u32 },
    #[error("Failed to solve blackbox function {name}: {reason}")]
    BlackBoxError { name: String, reason: String },
}

/// These errors can only result from interpreting malformed SSA
#[derive(Debug, Error)]
pub(crate) enum InternalError {
    #[error(
        "Argument count {arguments} to block {block} does not match the expected parameter count {parameters}"
    )]
    BlockArgumentCountMismatch { block: BasicBlockId, arguments: usize, parameters: usize },
    #[error(
        "Argument count {arguments} to `{intrinsic}` does not match the expected parameter count {parameters}"
    )]
    IntrinsicArgumentCountMismatch { intrinsic: Intrinsic, arguments: usize, parameters: usize },
    #[error("Block {block} is missing the terminator instruction")]
    BlockMissingTerminator { block: BasicBlockId },
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
    #[error("Rhs of `{operator}` should be a u8 but found `{rhs_id} = {rhs}`")]
    RhsOfBitShiftShouldBeU8 { operator: &'static str, rhs_id: ValueId, rhs: String },
    #[error(
        "Expected {expected_type} value in {instruction} but instead found `{value_id} = {value}`"
    )]
    TypeError {
        value_id: ValueId,
        value: String,
        expected_type: &'static str,
        instruction: &'static str,
    },
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
        "`truncate {value_id} to 0 bits, max_bit_size: {max_bit_size}` has invalid bit size 0. This should only be possible in malformed SSA."
    )]
    TruncateToZeroBits { value_id: ValueId, max_bit_size: u32 },
    #[error(
        "`range_check {value_id} to 0 bits` has invalid bit size 0. This should only be possible in malformed SSA."
    )]
    RangeCheckToZeroBits { value_id: ValueId },
    #[error("`field_less_than` can only be called in unconstrained contexts")]
    FieldLessThanCalledInConstrainedContext,
    #[error("Slice `{slice_id} = {slice}` contains struct/tuple elements of types `({})` and thus needs a minimum length of {} to pop 1 struct/tuple, but it is only of length {actual_length}", element_types.join(", "), element_types.len())]
    NotEnoughElementsToPopSliceOfStructs {
        slice_id: ValueId,
        slice: String,
        actual_length: usize,
        element_types: Vec<String>,
    },
    #[error("Unexpected instruction: `{reason}`")]
    UnexpectedInstruction { reason: &'static str },
    #[error("Expected array of {expected_size} elements, got {size}")]
    InvalidInputSize { expected_size: usize, size: usize },
}
