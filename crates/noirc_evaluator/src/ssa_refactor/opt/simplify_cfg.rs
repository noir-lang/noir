//! This file contains the simplify cfg pass of the SSA IR.
//!
//! This is a rather simple pass that is expected to be cheap to perform. It:
//! 1. Removes blocks with no predecessors
//! 2. Inlines a block into its sole predecessor if that predecessor only has one successor.
//! 3. Removes any block arguments for blocks with only a single predecessor.
//! 4. Removes any blocks which have no instructions other than a single terminating jmp.
//!
//! Currently, only 2 and 3 are implemented.
use std::collections::HashSet;

use crate::ssa_refactor::{
    ir::{
        basic_block::BasicBlockId, cfg::ControlFlowGraph, function::Function,
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
    ///
    /// Currently, only 2 and 3 are implemented.
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
    block_id: BasicBlockId,
    predecessor_id: BasicBlockId,
) -> bool {
    let mut successors = cfg.successors(predecessor_id);
    if successors.len() == 1 && successors.next() == Some(block_id) {
        drop(successors);

        // First remove all the instructions and terminator from the block we're removing
        let block = &mut function.dfg[block_id];
        let mut instructions = std::mem::take(block.instructions_mut());
        let terminator = block.take_terminator();

        // Then append each to the predecessor
        let predecessor = &mut function.dfg[predecessor_id];
        predecessor.instructions_mut().append(&mut instructions);

        predecessor.set_terminator(terminator);
        cfg.recompute_block(function, block_id);
        cfg.recompute_block(function, predecessor_id);
        true
    } else {
        false
    }
}

#[cfg(test)]
mod test {
    use crate::ssa_refactor::{
        ir::{instruction::TerminatorInstruction, map::Id, types::Type},
        ssa_builder::FunctionBuilder,
    };

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
        println!("{}", main);
        assert_eq!(main.reachable_blocks().len(), 1);

        match main.dfg[main.entry_block()].terminator() {
            Some(TerminatorInstruction::Return { return_values }) => {
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
}
