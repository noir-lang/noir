//! Reusable debug assertion checks for SSA optimization passes.
//!
//! This module provides composable check functions that can be used in pre-checks
//! and post-checks for SSA passes. Each pass defines its own `{pass_name}_pre_check`
//! or `{pass_name}_post_check` function that calls the appropriate checks from this
//! module, making the requirements human-readable.
//!
//! All functions in this module are only compiled with `#[cfg(debug_assertions)]`.

use crate::ssa::ir::{
    function::Function,
    instruction::{Binary, BinaryOp, Instruction},
    types::Type,
};

/// Asserts that an ACIR function's CFG has been flattened to a single block.
///
/// This check is skipped for Brillig functions.
#[cfg(debug_assertions)]
pub(super) fn assert_cfg_is_flattened(function: &Function) {
    if !function.runtime().is_acir() {
        return;
    }
    let blocks = function.reachable_blocks();
    assert_eq!(blocks.len(), 1, "CFG contains more than 1 block");
}

/// Asserts that an ACIR function contains no loops.
///
/// This check is skipped for Brillig functions.
#[cfg(debug_assertions)]
pub(super) fn assert_no_loops(function: &Function) {
    if !function.runtime().is_acir() {
        return;
    }
    let loops = super::Loops::find_all(function, super::LoopOrder::OutsideIn);
    assert!(
        loops.yet_to_unroll.is_empty(),
        "ACIR function {} still contains {} loop(s)",
        function.name(),
        loops.yet_to_unroll.len()
    );
}

/// Asserts that a function contains no checked signed binary operations (add, sub, mul).
///
/// These operations should have been expanded by the `expand_signed_checks` pass.
#[cfg(debug_assertions)]
pub(super) fn assert_no_checked_signed_add_sub_mul(function: &Function) {
    for block_id in function.reachable_blocks() {
        for instruction_id in function.dfg[block_id].instructions() {
            if let Instruction::Binary(binary) = &function.dfg[*instruction_id] {
                if function.dfg.type_of_value(binary.lhs).is_signed() {
                    match binary.operator {
                        BinaryOp::Add { unchecked: false }
                        | BinaryOp::Sub { unchecked: false }
                        | BinaryOp::Mul { unchecked: false } => {
                            panic!("Checked signed binary operation found (add/sub/mul)")
                        }
                        _ => (),
                    }
                }
            }
        }
    }
}

/// Asserts that an ACIR function contains no IfElse instructions.
///
/// This check is skipped for Brillig functions.
#[cfg(debug_assertions)]
pub(super) fn assert_no_if_else(function: &Function) {
    if function.runtime().is_brillig() {
        return;
    }

    for block_id in function.reachable_blocks() {
        for instruction_id in function.dfg[block_id].instructions() {
            if matches!(function.dfg[*instruction_id], Instruction::IfElse { .. }) {
                panic!("IfElse instruction still remains in ACIR function");
            }
        }
    }
}

/// Asserts that an ACIR function contains no Load or Store instructions.
///
/// This check is skipped for Brillig functions.
#[cfg(debug_assertions)]
pub(super) fn assert_no_load_store(function: &Function) {
    if !function.runtime().is_acir() {
        return;
    }

    for block_id in function.reachable_blocks() {
        for (i, instruction_id) in function.dfg[block_id].instructions().iter().enumerate() {
            let instruction = &function.dfg[*instruction_id];
            if matches!(instruction, Instruction::Load { .. } | Instruction::Store { .. }) {
                panic!(
                    "Load or Store instruction found in ACIR function: {} {} / {block_id} / {i}: {:?}",
                    function.name(),
                    function.id(),
                    instruction
                );
            }
        }
    }
}

/// Asserts that an ACIR function contains no bit shift instructions (Shl, Shr).
///
/// This check is skipped for Brillig functions.
#[cfg(debug_assertions)]
pub(super) fn assert_no_bit_shifts(function: &Function) {
    if !function.runtime().is_acir() {
        return;
    }

    for block_id in function.reachable_blocks() {
        for instruction_id in function.dfg[block_id].instructions() {
            if matches!(
                function.dfg[*instruction_id],
                Instruction::Binary(Binary { operator: BinaryOp::Shl | BinaryOp::Shr, .. })
            ) {
                panic!("Bitshift instruction still remains in ACIR function");
            }
        }
    }
}

/// Asserts that a function contains no ConstrainNotEqual instructions.
#[cfg(debug_assertions)]
pub(super) fn assert_no_constrain_not_equal(function: &Function) {
    for block_id in function.reachable_blocks() {
        for instruction_id in function.dfg[block_id].instructions() {
            if matches!(function.dfg[*instruction_id], Instruction::ConstrainNotEqual(_, _, _)) {
                panic!("ConstrainNotEqual should not be present");
            }
        }
    }
}

/// Asserts that an ACIR function contains no signed less-than comparisons.
///
/// This check is skipped for Brillig functions.
#[cfg(debug_assertions)]
pub(super) fn assert_no_signed_lt(function: &Function) {
    if !function.runtime().is_acir() {
        return;
    }

    for block_id in function.reachable_blocks() {
        for instruction_id in function.dfg[block_id].instructions() {
            if let Instruction::Binary(binary) = &function.dfg[*instruction_id] {
                if function.dfg.type_of_value(binary.lhs).is_signed()
                    && binary.operator == BinaryOp::Lt
                {
                    panic!("Signed less-than comparison found in ACIR function");
                }
            }
        }
    }
}

/// Asserts that IfElse instructions only operate on non-numeric types (arrays/vectors).
///
/// Numeric values should have been handled during flattening.
#[cfg(debug_assertions)]
pub(super) fn assert_if_else_not_on_numeric(function: &Function) {
    for block_id in function.reachable_blocks() {
        for instruction_id in function.dfg[block_id].instructions() {
            if let Instruction::IfElse { then_value, .. } = &function.dfg[*instruction_id] {
                let typ = function.dfg.type_of_value(*then_value);
                assert!(
                    !matches!(typ, Type::Numeric(_)),
                    "IfElse on numeric values should have been handled during flattening"
                );
            }
        }
    }
}

/// Asserts that a Brillig function contains no mutable ArraySet instructions.
///
/// This check is skipped for ACIR functions.
#[cfg(debug_assertions)]
pub(super) fn assert_no_mutable_array_set_in_brillig(function: &Function) {
    if !function.runtime().is_brillig() {
        return;
    }

    for block_id in function.reachable_blocks() {
        for instruction_id in function.dfg[block_id].instructions() {
            if matches!(function.dfg[*instruction_id], Instruction::ArraySet { mutable: true, .. })
            {
                panic!("Mutable array set instruction in Brillig function");
            }
        }
    }
}
