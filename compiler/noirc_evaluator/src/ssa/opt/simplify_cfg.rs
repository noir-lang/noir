//! This file contains the simplify cfg pass of the SSA IR.
//!
//! This is a rather simple pass that is expected to be cheap to perform. It:
//! 1. Removes blocks with no predecessors
//! 2. Inlines a block into its sole predecessor if that predecessor only has one successor.
//! 3. Removes any block arguments for blocks with only a single predecessor.
//! 4. Removes any blocks which have no instructions other than a single terminating jmp.
//! 5. Replaces any jmpifs with constant conditions with jmps. If this causes the block to have
//!    only 1 successor then (2) also will be applied.
//!
//! Currently, 1 and 4 are unimplemented.
use std::collections::HashSet;

use acvm::acir::AcirField;

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId, cfg::ControlFlowGraph, dfg::CallStack, function::Function,
        instruction::TerminatorInstruction,
    },
    ssa_gen::Ssa,
};

impl Ssa {
    /// Simplify each function's control flow graph by:
    /// 1. Removing blocks with no predecessors
    /// 2. Inlining a block into its sole predecessor if that predecessor only has one successor.
    /// 3. Removing any block arguments for blocks with only a single predecessor.
    /// 4. Removing any blocks which have no instructions other than a single terminating jmp.
    /// 5. Replacing any jmpifs with constant conditions with jmps. If this causes the block to have
    ///    only 1 successor then (2) also will be applied.
    ///
    /// Currently, 1 and 4 are unimplemented.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn simplify_cfg(mut self) -> Self {
        for function in self.functions.values_mut() {
            simplify_function(function);
        }
        self
    }
}

/// Simplify a function's cfg by going through each block to check for any simple blocks that can
/// be inlined into their predecessor.
fn simplify_function(function: &mut Function) {
    let mut cfg = ControlFlowGraph::with_function(function);
    let mut stack = vec![function.entry_block()];
    let mut visited = HashSet::new();

    while let Some(block) = stack.pop() {
        if visited.insert(block) {
            stack.extend(function.dfg[block].successors().filter(|block| !visited.contains(block)));
        }

        // This call is before try_inline_into_predecessor so that if it succeeds in changing a
        // jmpif into a jmp, the block may then be inlined entirely into its predecessor in try_inline_into_predecessor.
        check_for_constant_jmpif(function, block, &mut cfg);

        let mut predecessors = cfg.predecessors(block);

        if predecessors.len() == 1 {
            let predecessor = predecessors.next().expect("Already checked length of predecessors");
            drop(predecessors);

            // If the block has only 1 predecessor, we can safely remove its block parameters
            remove_block_parameters(function, block, predecessor);

            // Note: this function relies on `remove_block_parameters` being called first.
            // Otherwise the inlined block will refer to parameters that no longer exist.
            //
            // If successful, `block` will be empty and unreachable after this call, so any
            // optimizations performed after this point on the same block should check if
            // the inlining here was successful before continuing.
            try_inline_into_predecessor(function, &mut cfg, block, predecessor);
        }
    }
}

/// Optimize a jmpif into a jmp if the condition is known
fn check_for_constant_jmpif(
    function: &mut Function,
    block: BasicBlockId,
    cfg: &mut ControlFlowGraph,
) {
    if let Some(TerminatorInstruction::JmpIf { condition, then_destination, else_destination }) =
        function.dfg[block].terminator()
    {
        if let Some(constant) = function.dfg.get_numeric_constant(*condition) {
            let destination =
                if constant.is_zero() { *else_destination } else { *then_destination };

            let arguments = Vec::new();
            let jmp =
                TerminatorInstruction::Jmp { destination, arguments, call_stack: CallStack::new() };
            function.dfg[block].set_terminator(jmp);
            cfg.recompute_block(function, block);
        }
    }
}

/// If the given block has block parameters, replace them with the jump arguments from the predecessor.
///
/// Currently, if this function is needed, `try_inline_into_predecessor` will also always apply,
/// although in the future it is possible for only this function to apply if jmpif instructions
/// with block arguments are ever added.
fn remove_block_parameters(
    function: &mut Function,
    block: BasicBlockId,
    predecessor: BasicBlockId,
) {
    let block = &mut function.dfg[block];

    if !block.parameters().is_empty() {
        let block_params = block.take_parameters();

        let jump_args = match function.dfg[predecessor].unwrap_terminator_mut() {
            TerminatorInstruction::Jmp { arguments, .. } => std::mem::take(arguments),
            TerminatorInstruction::JmpIf { .. } => unreachable!("If jmpif instructions are modified to support block arguments in the future, this match will need to be updated"),
            _ => unreachable!(
                "Predecessor was already validated to have only a single jmp destination"
            ),
        };

        assert_eq!(block_params.len(), jump_args.len());
        for (param, arg) in block_params.iter().zip(jump_args) {
            function.dfg.set_value_from_id(*param, arg);
        }
    }
}

/// Try to inline a block into its predecessor, returning true if successful.
///
/// This will only occur if the predecessor's only successor is the given block.
/// It is also expected that the given block's only predecessor is the given one.
fn try_inline_into_predecessor(
    function: &mut Function,
    cfg: &mut ControlFlowGraph,
    block: BasicBlockId,
    predecessor: BasicBlockId,
) -> bool {
    let mut successors = cfg.successors(predecessor);
    if successors.len() == 1 && successors.next() == Some(block) {
        drop(successors);
        function.dfg.inline_block(block, predecessor);

        cfg.recompute_block(function, block);
        cfg.recompute_block(function, predecessor);
        true
    } else {
        false
    }
}

#[cfg(test)]
mod test {
    use crate::ssa::{
        function_builder::FunctionBuilder,
        ir::{
            instruction::{BinaryOp, TerminatorInstruction},
            map::Id,
            types::Type,
        },
    };
    use acvm::acir::AcirField;

    #[test]
    fn inline_blocks() {
        // fn main {
        //   b0():
        //     jmp b1(Field 7)
        //   b1(v0: Field):
        //     jmp b2(v0)
        //   b2(v1: Field):
        //     return v1
        // }
        let main_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);

        let b1 = builder.insert_block();
        let b2 = builder.insert_block();

        let v0 = builder.add_block_parameter(b1, Type::field());
        let v1 = builder.add_block_parameter(b2, Type::field());

        let expected_return = 7u128;
        let seven = builder.field_constant(expected_return);
        builder.terminate_with_jmp(b1, vec![seven]);

        builder.switch_to_block(b1);
        builder.terminate_with_jmp(b2, vec![v0]);

        builder.switch_to_block(b2);
        builder.terminate_with_return(vec![v1]);

        let ssa = builder.finish();
        assert_eq!(ssa.main().reachable_blocks().len(), 3);

        // Expected output:
        // fn main {
        //   b0():
        //     return Field 7
        // }
        let ssa = ssa.simplify_cfg();
        let main = ssa.main();
        assert_eq!(main.reachable_blocks().len(), 1);

        match main.dfg[main.entry_block()].terminator() {
            Some(TerminatorInstruction::Return { return_values, .. }) => {
                assert_eq!(return_values.len(), 1);
                let return_value = main
                    .dfg
                    .get_numeric_constant(return_values[0])
                    .expect("Expected return value to be constant")
                    .to_u128();
                assert_eq!(return_value, expected_return);
            }
            other => panic!("Unexpected terminator {other:?}"),
        }
    }

    #[test]
    fn remove_known_jmpif() {
        // fn main {
        //   b0(v0: u1):
        //     v1 = eq v0, v0
        //     jmpif v1, then: b1, else: b2
        //   b1():
        //     return Field 1
        //   b2():
        //     return Field 2
        // }
        let main_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);
        let v0 = builder.add_parameter(Type::bool());

        let b1 = builder.insert_block();
        let b2 = builder.insert_block();

        let one = builder.field_constant(1u128);
        let two = builder.field_constant(2u128);

        let v1 = builder.insert_binary(v0, BinaryOp::Eq, v0);
        builder.terminate_with_jmpif(v1, b1, b2);

        builder.switch_to_block(b1);
        builder.terminate_with_return(vec![one]);

        builder.switch_to_block(b2);
        builder.terminate_with_return(vec![two]);

        let ssa = builder.finish();
        assert_eq!(ssa.main().reachable_blocks().len(), 3);

        // Expected output:
        // fn main {
        //   b0():
        //     return Field 1
        // }
        let ssa = ssa.simplify_cfg();
        let main = ssa.main();
        assert_eq!(main.reachable_blocks().len(), 1);

        match main.dfg[main.entry_block()].terminator() {
            Some(TerminatorInstruction::Return { return_values, .. }) => {
                assert_eq!(return_values.len(), 1);
                let return_value = main
                    .dfg
                    .get_numeric_constant(return_values[0])
                    .expect("Expected return value to be constant")
                    .to_u128();
                assert_eq!(return_value, 1u128);
            }
            other => panic!("Unexpected terminator {other:?}"),
        }
    }
}
