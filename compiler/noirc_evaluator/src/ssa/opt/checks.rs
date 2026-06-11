//! Reusable debug assertion checks for SSA optimization passes.
//!
//! This module provides asserting predicates and a single-pass iteration
//! helper ([`for_each_instruction`]) so that multiple instruction-level checks
//! can be evaluated per-instruction in one traversal.
//!
//! Each pass defines its own `{pass_name}_pre_check` or `{pass_name}_post_check`
//! function that calls [`for_each_instruction`] with a callback that invokes
//! whichever asserting predicates are relevant.
//!
//! These checks are pure - they do not contain any ACIR/Brillig filtering logic.
//! The caller is responsible for filtering by runtime type if needed.
//!
//! All functions in this module are only compiled with `#[cfg(debug_assertions)]`.

use crate::ssa::ir::{
    basic_block::BasicBlockId,
    dfg::DataFlowGraph,
    function::Function,
    instruction::{Binary, BinaryOp, Instruction, TerminatorInstruction},
    types::Type,
    value::ValueId,
};

// ---------------------------------------------------------------------------
// Single-pass instruction iterator
// ---------------------------------------------------------------------------

/// Calls `check` on every instruction in every reachable block of `function`.
///
/// Use this to evaluate multiple asserting predicates per instruction in a
/// single pass:
///
/// ```ignore
/// checks::for_each_instruction(function, |instruction, dfg| {
///     checks::assert_not_if_else(instruction);
///     checks::assert_not_load_or_store(instruction);
/// });
/// ```
pub(super) fn for_each_instruction(
    function: &Function,
    check: impl Fn(&Instruction, &DataFlowGraph),
) {
    for block_id in function.reachable_blocks() {
        for instruction_id in function.dfg[block_id].instructions() {
            check(&function.dfg[*instruction_id], &function.dfg);
        }
    }
}

// ---------------------------------------------------------------------------
// Structural (non-instruction) checks
// ---------------------------------------------------------------------------

/// Asserts that the function's CFG has been flattened to a single block.
pub(super) fn assert_cfg_is_flattened(function: &Function) {
    let blocks = function.reachable_blocks();
    assert_eq!(blocks.len(), 1, "CFG contains more than 1 block");
}

/// Asserts that no reachable block has a `JmpIf` with a constant condition.
///
/// A constant-condition `JmpIf` should have been folded into an unconditional
/// `Jmp` by `simplify_cfg`. If one remains it indicates a missing simplification
/// pass in the pipeline.
pub(super) fn assert_no_constant_jmpif(function: &Function) {
    for block in function.reachable_blocks() {
        if let Some(TerminatorInstruction::JmpIf { condition, .. }) =
            function.dfg[block].terminator()
        {
            assert!(
                function.dfg.get_numeric_constant(*condition).is_none(),
                "Block {block} in function {} has a JmpIf with a constant condition. \
                 Run simplify_cfg to fold constant-condition branches.",
                function.name(),
            );
        }
    }
}

/// Asserts that every `Jmp` and `JmpIf` terminator in the function passes exactly as many
/// arguments to each destination as that destination block has parameters, with matching
/// types.
///
/// Passes which add or remove block parameters (e.g. mem2reg, remove_redundant_params)
/// must keep every predecessor's terminator arguments consistent with the destination's
/// parameter list; a mismatch leaves the SSA malformed in a way that only surfaces in
/// later passes or codegen.
pub(super) fn assert_terminator_args_match_params(function: &Function) {
    let check_args = |block: BasicBlockId, destination: BasicBlockId, arguments: &[ValueId]| {
        let params = function.dfg[destination].parameters();
        assert_eq!(
            arguments.len(),
            params.len(),
            "Block {block} in function {} jumps to {destination} with {} arguments, but it has {} parameters",
            function.name(),
            arguments.len(),
            params.len(),
        );
        for (argument, param) in arguments.iter().zip(params) {
            let argument_type = function.dfg.type_of_value(*argument);
            let param_type = function.dfg.type_of_value(*param);
            assert!(
                argument_type.canonical_eq(&param_type),
                "Block {block} in function {} jumps to {destination} passing {argument}, whose type does not match parameter {param}\n  left: {argument_type}\n right: {param_type}",
                function.name(),
            );
        }
    };

    for block in function.reachable_blocks() {
        match function.dfg[block].terminator() {
            Some(TerminatorInstruction::Jmp { destination, arguments, .. }) => {
                check_args(block, *destination, arguments);
            }
            Some(TerminatorInstruction::JmpIf {
                then_destination,
                then_arguments,
                else_destination,
                else_arguments,
                ..
            }) => {
                check_args(block, *then_destination, then_arguments);
                check_args(block, *else_destination, else_arguments);
            }
            _ => (),
        }
    }
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

// ---------------------------------------------------------------------------
// Asserting instruction predicates
// ---------------------------------------------------------------------------

/// Panics if the instruction is a checked signed add, sub, or mul.
pub(super) fn assert_not_checked_signed_add_sub_mul(
    instruction: &Instruction,
    dfg: &DataFlowGraph,
) {
    if let Instruction::Binary(binary) = instruction
        && dfg.type_of_value(binary.lhs).is_signed()
    {
        assert!(
            !matches!(
                binary.operator,
                BinaryOp::Add { unchecked: false }
                    | BinaryOp::Sub { unchecked: false }
                    | BinaryOp::Mul { unchecked: false }
            ),
            "Checked signed binary operation found (add/sub/mul)"
        );
    }
}

/// Panics if the instruction is an IfElse.
pub(super) fn assert_not_if_else(instruction: &Instruction) {
    assert!(!matches!(instruction, Instruction::IfElse { .. }), "IfElse instruction found");
}

/// Panics if the instruction is a Load or Store.
pub(super) fn assert_not_load_or_store(instruction: &Instruction) {
    assert!(
        !matches!(instruction, Instruction::Load { .. } | Instruction::Store { .. }),
        "Load or Store instruction found"
    );
}

/// Panics if the instruction is a bit shift (Shl or Shr).
pub(super) fn assert_not_bit_shift(instruction: &Instruction) {
    assert!(
        !matches!(
            instruction,
            Instruction::Binary(Binary { operator: BinaryOp::Shl | BinaryOp::Shr, .. })
        ),
        "Bitshift instruction found"
    );
}

/// Panics if the instruction is a ConstrainNotEqual.
pub(super) fn assert_not_constrain_not_equal(instruction: &Instruction) {
    assert!(
        !matches!(instruction, Instruction::ConstrainNotEqual(_, _, _)),
        "ConstrainNotEqual instruction found"
    );
}

/// Panics if the instruction is a signed less-than comparison.
pub(super) fn assert_not_signed_lt(instruction: &Instruction, dfg: &DataFlowGraph) {
    assert!(
        !is_signed_binary_op(instruction, dfg, BinaryOp::Lt),
        "Signed less-than comparison found"
    );
}

/// Panics if the instruction is a signed division.
pub(super) fn assert_not_signed_div(instruction: &Instruction, dfg: &DataFlowGraph) {
    assert!(!is_signed_binary_op(instruction, dfg, BinaryOp::Div), "Signed division found");
}

/// Panics if the instruction is a signed modulo.
pub(super) fn assert_not_signed_mod(instruction: &Instruction, dfg: &DataFlowGraph) {
    assert!(!is_signed_binary_op(instruction, dfg, BinaryOp::Mod), "Signed modulo found");
}

/// Panics if the instruction is an IfElse operating on a numeric type.
pub(super) fn assert_not_if_else_on_numeric(instruction: &Instruction, dfg: &DataFlowGraph) {
    if let Instruction::IfElse { then_value, .. } = instruction {
        assert!(
            !matches!(*dfg.type_of_value(*then_value), Type::Numeric(_)),
            "IfElse on numeric values should have been handled during flattening"
        );
    }
}

/// Panics if the instruction is a mutable ArraySet.
pub(super) fn assert_not_mutable_array_set(instruction: &Instruction) {
    assert!(
        !matches!(instruction, Instruction::ArraySet { mutable: true, .. }),
        "Mutable array set instruction found"
    );
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns true if the instruction is a binary op on signed operands with the given operator.
fn is_signed_binary_op(instruction: &Instruction, dfg: &DataFlowGraph, op: BinaryOp) -> bool {
    if let Instruction::Binary(binary) = instruction {
        dfg.type_of_value(binary.lhs).is_signed() && binary.operator == op
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use acvm::{AcirField, FieldElement};

    use crate::ssa::{
        function_builder::FunctionBuilder,
        ir::{
            map::Id,
            types::{NumericType, Type},
        },
    };

    #[test]
    #[should_panic(expected = "with 0 arguments, but it has 1 parameters")]
    fn detects_terminator_argument_arity_mismatch() {
        let main_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);
        let destination = builder.insert_block();
        builder.add_block_parameter(destination, Type::field());
        builder.terminate_with_jmp(destination, Vec::new());
        builder.switch_to_block(destination);
        builder.terminate_with_return(Vec::new());

        let ssa = builder.finish();
        super::assert_terminator_args_match_params(ssa.main());
    }

    #[test]
    #[should_panic(expected = "whose type does not match parameter")]
    fn detects_terminator_argument_type_mismatch() {
        let main_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);
        let destination = builder.insert_block();
        builder.add_block_parameter(destination, Type::field());
        let argument = builder.numeric_constant(FieldElement::one(), NumericType::bool());
        builder.terminate_with_jmp(destination, vec![argument]);
        builder.switch_to_block(destination);
        builder.terminate_with_return(Vec::new());

        let ssa = builder.finish();
        super::assert_terminator_args_match_params(ssa.main());
    }
}
