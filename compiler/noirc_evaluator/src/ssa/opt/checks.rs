//! Reusable debug assertion checks for SSA optimization passes.
//!
//! This module provides composable check functions that can be used in pre-checks
//! and post-checks for SSA passes. Each pass defines its own `{pass_name}_pre_check`
//! or `{pass_name}_post_check` function that calls the appropriate checks from this
//! module, making the requirements human-readable.
//!
//! These checks are pure - they do not contain any ACIR/Brillig filtering logic.
//! The caller is responsible for filtering by runtime type if needed.
//!
//! All functions in this module are only compiled with `#[cfg(debug_assertions)]`.

use crate::ssa::ir::{
    dfg::DataFlowGraph,
    function::Function,
    instruction::{Binary, BinaryOp, Instruction},
    types::Type,
};

/// Panics if any instruction in the function matches the given predicate.
fn assert_no_instruction_matching(
    function: &Function,
    predicate: impl Fn(&Instruction, &DataFlowGraph) -> bool,
    message: &str,
) {
    for block_id in function.reachable_blocks() {
        for instruction_id in function.dfg[block_id].instructions() {
            if predicate(&function.dfg[*instruction_id], &function.dfg) {
                panic!("{message}");
            }
        }
    }
}

/// Asserts that the function's CFG has been flattened to a single block.
pub(super) fn assert_cfg_is_flattened(function: &Function) {
    let blocks = function.reachable_blocks();
    assert_eq!(blocks.len(), 1, "CFG contains more than 1 block");
}

/// Asserts that the function contains no loops.
pub(super) fn assert_no_loops(function: &Function) {
    let loops = super::Loops::find_all(function, super::LoopOrder::OutsideIn);
    assert!(
        loops.yet_to_unroll.is_empty(),
        "Function {} still contains {} loop(s)",
        function.name(),
        loops.yet_to_unroll.len()
    );
}

/// Asserts that the function contains no checked signed binary operations (add, sub, mul).
///
/// These operations should have been expanded by the `expand_signed_checks` pass.
pub(super) fn assert_no_checked_signed_add_sub_mul(function: &Function) {
    assert_no_instruction_matching(
        function,
        |instruction, dfg| {
            if let Instruction::Binary(binary) = instruction {
                if dfg.type_of_value(binary.lhs).is_signed() {
                    return matches!(
                        binary.operator,
                        BinaryOp::Add { unchecked: false }
                            | BinaryOp::Sub { unchecked: false }
                            | BinaryOp::Mul { unchecked: false }
                    );
                }
            }
            false
        },
        "Checked signed binary operation found (add/sub/mul)",
    );
}

/// Asserts that the function contains no IfElse instructions.
pub(super) fn assert_no_if_else(function: &Function) {
    assert_no_instruction_matching(
        function,
        |instruction, _| matches!(instruction, Instruction::IfElse { .. }),
        "IfElse instruction found",
    );
}

/// Asserts that the function contains no Load or Store instructions.
pub(super) fn assert_no_load_store(function: &Function) {
    assert_no_instruction_matching(
        function,
        |instruction, _| {
            matches!(instruction, Instruction::Load { .. } | Instruction::Store { .. })
        },
        "Load or Store instruction found",
    );
}

/// Asserts that the function contains no bit shift instructions (Shl, Shr).
pub(super) fn assert_no_bit_shifts(function: &Function) {
    assert_no_instruction_matching(
        function,
        |instruction, _| {
            matches!(
                instruction,
                Instruction::Binary(Binary { operator: BinaryOp::Shl | BinaryOp::Shr, .. })
            )
        },
        "Bitshift instruction found",
    );
}

/// Asserts that the function contains no ConstrainNotEqual instructions.
pub(super) fn assert_no_constrain_not_equal(function: &Function) {
    assert_no_instruction_matching(
        function,
        |instruction, _| matches!(instruction, Instruction::ConstrainNotEqual(_, _, _)),
        "ConstrainNotEqual instruction found",
    );
}

/// Asserts that the function contains no signed less-than comparisons.
pub(super) fn assert_no_signed_lt(function: &Function) {
    assert_no_instruction_matching(
        function,
        |instruction, dfg| is_signed_binary_op(instruction, dfg, BinaryOp::Lt),
        "Signed less-than comparison found",
    );
}

/// Asserts that the function contains no signed division operations.
pub(super) fn assert_no_signed_div(function: &Function) {
    assert_no_instruction_matching(
        function,
        |instruction, dfg| is_signed_binary_op(instruction, dfg, BinaryOp::Div),
        "Signed division found",
    );
}

/// Asserts that the function contains no signed modulo operations.
pub(super) fn assert_no_signed_mod(function: &Function) {
    assert_no_instruction_matching(
        function,
        |instruction, dfg| is_signed_binary_op(instruction, dfg, BinaryOp::Mod),
        "Signed modulo found",
    );
}

/// Helper to check if an instruction is a binary operation with a signed lhs and the given operator.
fn is_signed_binary_op(instruction: &Instruction, dfg: &DataFlowGraph, op: BinaryOp) -> bool {
    if let Instruction::Binary(binary) = instruction {
        dfg.type_of_value(binary.lhs).is_signed() && binary.operator == op
    } else {
        false
    }
}

/// Asserts that IfElse instructions only operate on non-numeric types (arrays/vectors).
///
/// Numeric values should have been handled during flattening.
pub(super) fn assert_if_else_not_on_numeric(function: &Function) {
    assert_no_instruction_matching(
        function,
        |instruction, dfg| {
            if let Instruction::IfElse { then_value, .. } = instruction {
                matches!(dfg.type_of_value(*then_value), Type::Numeric(_))
            } else {
                false
            }
        },
        "IfElse on numeric values should have been handled during flattening",
    );
}

/// Asserts that the function contains no mutable ArraySet instructions.
pub(super) fn assert_no_mutable_array_set(function: &Function) {
    assert_no_instruction_matching(
        function,
        |instruction, _| matches!(instruction, Instruction::ArraySet { mutable: true, .. }),
        "Mutable array set instruction found",
    );
}
