//! This file contains the simplify cfg pass of the SSA IR.
//!
//! This is a rather simple pass that is expected to be cheap to perform. It:
//! 1. Removes blocks with no predecessors
//! 2. Inlines a block into its sole predecessor if that predecessor only has one successor.
//! 3. Removes any block arguments for blocks with only a single predecessor.
//! 4. Removes any blocks which have no instructions other than a single terminating jmp.
//! 5. Replaces any jmpifs with constant conditions with jmps. If this causes the block to have
//!    only 1 successor then (2) also will be applied.
//! 6. Replacing any jmpifs with a negated condition with a jmpif with a un-negated condition and reversed branches.
//! 7. Replaces any jmpif whose branches converge. Convergence is defined as two paths that ultimately jump to the same block through empty jump chains.
//!    If the branches converge, the jmpif is unnecessary and can be replaced with a simple jmp.
//!
//! Currently only 1 is unimplemented.
use std::collections::HashSet;

use acvm::acir::AcirField;

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        cfg::ControlFlowGraph,
        function::{Function, RuntimeType},
        instruction::{Instruction, TerminatorInstruction},
        value::{Value, ValueMapping},
    },
    ssa_gen::Ssa,
};

impl Ssa {
    /// See [`simplify_cfg`][self] module for more information
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn simplify_cfg(mut self) -> Self {
        for function in self.functions.values_mut() {
            function.simplify_function();
        }
        self
    }
}

impl Function {
    /// Simplify a function's cfg by going through each block to check for any simple blocks that can
    /// be inlined into their predecessor.
    pub(crate) fn simplify_function(&mut self) {
        let mut cfg = ControlFlowGraph::with_function(self);
        let mut values_to_replace = ValueMapping::default();
        let mut stack = vec![self.entry_block()];
        let mut visited = HashSet::new();

        while let Some(block) = stack.pop() {
            if cfg.predecessors(block).len() == 0 && block != self.entry_block() {
                // If the block has no predecessors, it's no longer reachable and can be ignored.
                cfg.invalidate_block_successors(block);
                continue;
            }

            if !values_to_replace.is_empty() {
                self.dfg.replace_values_in_block_instructions(block, &values_to_replace);
            }

            // First perform any simplifications on the current block, this ensures that we add the proper successors
            // to the stack.
            simplify_current_block(self, block, &mut cfg, &mut values_to_replace);

            if visited.insert(block) {
                stack.extend(self.dfg[block].successors().filter(|block| !visited.contains(block)));
            }

            let mut predecessors = cfg.predecessors(block);
            if predecessors.len() == 1 {
                let predecessor =
                    predecessors.next().expect("Already checked length of predecessors");
                drop(predecessors);

                try_inline_successor(self, &mut cfg, predecessor, &mut values_to_replace);
            } else {
                drop(predecessors);

                check_for_double_jmp(self, block, &mut cfg);
            }

            if !values_to_replace.is_empty() {
                self.dfg.replace_values_in_block_terminator(block, &values_to_replace);
            }
        }

        if !values_to_replace.is_empty() {
            // Values from previous blocks might need to be replaced
            for block in self.reachable_blocks() {
                self.dfg.replace_values_in_block(block, &values_to_replace);
            }
            self.dfg.data_bus.replace_values(&values_to_replace);
        }
    }
}

/// A helper function to simplify the current block based on information on its successors.
///
/// This function will recursively simplify the current block until no further simplifications can be made.
fn simplify_current_block(
    function: &mut Function,
    block: BasicBlockId,
    cfg: &mut ControlFlowGraph,
    values_to_replace: &mut ValueMapping,
) {
    // These functions return `true` if they successfully simplified the CFG for the current block.
    let mut simplified = true;

    while simplified {
        simplified = check_for_negated_jmpif_condition(function, block, cfg)
            | check_for_constant_jmpif(function, block, cfg)
            | check_for_converging_jmpif(function, block, cfg)
            | try_inline_successor(function, cfg, block, values_to_replace);
    }
}

/// Optimize a jmpif into a jmp if the condition is known
fn check_for_constant_jmpif(
    function: &mut Function,
    block: BasicBlockId,
    cfg: &mut ControlFlowGraph,
) -> bool {
    if let Some(TerminatorInstruction::JmpIf {
        condition,
        then_destination,
        else_destination,
        call_stack,
    }) = function.dfg[block].terminator()
    {
        if let Some(constant) = function.dfg.get_numeric_constant(*condition) {
            let (destination, unchosen_destination) = if constant.is_zero() {
                (*else_destination, *then_destination)
            } else {
                (*then_destination, *else_destination)
            };

            let arguments = Vec::new();
            let call_stack = *call_stack;
            let jmp = TerminatorInstruction::Jmp { destination, arguments, call_stack };
            function.dfg[block].set_terminator(jmp);
            cfg.recompute_block(function, block);

            // If `block` was the only predecessor to `unchosen_destination` then it's no long reachable through the CFG,
            // we can then invalidate it successors as it's an invalid predecessor.
            if cfg.predecessors(unchosen_destination).len() == 0 {
                cfg.invalidate_block_successors(unchosen_destination);
            }

            return true;
        }
    }
    false
}

/// Optimize a jmp to a block which immediately jmps elsewhere to just jmp to the second block.
fn check_for_double_jmp(function: &mut Function, block: BasicBlockId, cfg: &mut ControlFlowGraph) {
    if matches!(function.runtime(), RuntimeType::Acir(_)) {
        // We can't remove double jumps in ACIR functions as this interferes with the `flatten_cfg` pass.
        return;
    }

    // We only want to remove double jumps if the block has no instructions or parameters.
    if !function.dfg[block].instructions().is_empty()
        || !function.dfg[block].parameters().is_empty()
    {
        return;
    }

    // We expect the block to have a simple jmp terminator with no arguments.
    let Some(TerminatorInstruction::Jmp { destination: final_destination, arguments, .. }) =
        function.dfg[block].terminator()
    else {
        return;
    };

    if !arguments.is_empty() {
        return;
    }

    let final_destination = *final_destination;

    // At this point we know that `block` is a simple jmp block with no instructions or parameters.
    // We can then update all of its predecessors to jump directly to the final destination.
    let predecessors: Vec<_> = cfg.predecessors(block).collect();
    for predecessor_block in predecessors {
        let terminator_instruction = function.dfg[predecessor_block].take_terminator();
        let redirected_terminator_instruction = match terminator_instruction {
            TerminatorInstruction::JmpIf {
                condition,
                then_destination,
                else_destination,
                call_stack,
            } => {
                let then_destination =
                    if then_destination == block { final_destination } else { then_destination };
                let else_destination =
                    if else_destination == block { final_destination } else { else_destination };
                TerminatorInstruction::JmpIf {
                    condition,
                    then_destination,
                    else_destination,
                    call_stack,
                }
            }
            TerminatorInstruction::Jmp { destination, arguments, call_stack } => {
                assert_eq!(
                    destination, block,
                    "ICE: predecessor block doesn't jump to current block"
                );
                assert!(arguments.is_empty(), "ICE: predecessor jmp has arguments");
                TerminatorInstruction::Jmp { destination: final_destination, arguments, call_stack }
            }
            TerminatorInstruction::Return { .. } => {
                unreachable!("ICE: predecessor block should not have return terminator instruction")
            }
            TerminatorInstruction::Unreachable { .. } => {
                unreachable!(
                    "ICE: predecessor block should not have unreachable terminator instruction"
                )
            }
        };

        function.dfg[predecessor_block].set_terminator(redirected_terminator_instruction);
        cfg.recompute_block(function, predecessor_block);
    }
    cfg.recompute_block(function, block);
}

/// Optimize a jmpif on a negated condition by swapping the branches.
fn check_for_negated_jmpif_condition(
    function: &mut Function,
    block: BasicBlockId,
    cfg: &mut ControlFlowGraph,
) -> bool {
    if matches!(function.runtime(), RuntimeType::Acir(_)) {
        // Swapping the `then` and `else` branches of a `JmpIf` within an ACIR function
        // can result in the situation where the branches merge together again in the `then` block, e.g.
        //
        // acir(inline) fn main f0 {
        //   b0(v0: u1):
        //     jmpif v0 then: b2, else: b1
        //   b2():
        //     return
        //   b1():
        //     jmp b2()
        // }
        //
        // This breaks the `flatten_cfg` pass as it assumes that merges only happen in
        // the `else` block or a 3rd block.
        //
        // See: https://github.com/noir-lang/noir/pull/5891#issuecomment-2500219428
        return false;
    }

    if let Some(TerminatorInstruction::JmpIf {
        condition,
        then_destination,
        else_destination,
        call_stack,
    }) = function.dfg[block].terminator()
    {
        if let Value::Instruction { instruction, .. } = function.dfg[*condition] {
            if let Instruction::Not(negated_condition) = function.dfg[instruction] {
                let call_stack = *call_stack;
                let jmpif = TerminatorInstruction::JmpIf {
                    condition: negated_condition,
                    then_destination: *else_destination,
                    else_destination: *then_destination,
                    call_stack,
                };
                function.dfg[block].set_terminator(jmpif);
                cfg.recompute_block(function, block);
                return true;
            }
        }
    }
    false
}

/// Attempts to simplify a `jmpif` terminator if both branches converge.
///
/// We define convergence as when two branches of a `jmpif` ultimately lead to the same
/// destination block, after following chains of empty blocks. If they do, the conditional
/// jump is unnecessary and can be replaced with a simple `jmp`.
fn check_for_converging_jmpif(
    function: &mut Function,
    block: BasicBlockId,
    cfg: &mut ControlFlowGraph,
) -> bool {
    if matches!(function.runtime(), RuntimeType::Acir(_)) {
        // The `flatten_cfg` pass expects two blocks to join to the same block.
        // If we have a nested if the inner if statement could potentially be a converging jmpif.
        // This may change the final block we converge into.
        return false;
    }

    let Some(TerminatorInstruction::JmpIf {
        then_destination, else_destination, call_stack, ..
    }) = function.dfg[block].terminator()
    else {
        return false;
    };

    let then_final = resolve_jmp_chain(function, *then_destination);
    let else_final = resolve_jmp_chain(function, *else_destination);

    // If both branches end at the same target, we can replace the jmpif with a jmp
    if then_final == else_final {
        let jmp = TerminatorInstruction::Jmp {
            destination: then_final,
            // The blocks in a jmp chain are checked to have empty arguments by resolve_jmp_chain
            arguments: Vec::new(),
            call_stack: *call_stack,
        };
        function.dfg[block].set_terminator(jmp);
        cfg.recompute_block(function, block);
        true
    } else {
        false
    }
}

/// Follow a chain of empty blocks to find the real destination.
///
/// This function assumes only [unconditional jumps][TerminatorInstruction::Jmp] are allowed
/// in a block for chaining. It returns the final destination reached through empty blocks.
fn resolve_jmp_chain(function: &Function, mut current: BasicBlockId) -> BasicBlockId {
    // Need to maintain a visited set to prevent infinite cycles
    let mut visited = HashSet::new();

    while visited.insert(current) {
        let block = &function.dfg[current];
        // Exit early if block has instructions or parameters
        if !block.instructions().is_empty() || !block.parameters().is_empty() {
            return current;
        }

        match block.terminator() {
            Some(TerminatorInstruction::Jmp { destination, arguments, .. })
                if arguments.is_empty() =>
            {
                // Continue following the current chain
                current = *destination;
            }
            _ => return current,
        }
    }

    current
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
    values_to_replace: &mut ValueMapping,
) {
    let block = &mut function.dfg[block];

    if !block.parameters().is_empty() {
        let block_params = block.take_parameters();

        let jump_args = match function.dfg[predecessor].unwrap_terminator_mut() {
            TerminatorInstruction::Jmp { arguments, .. } => std::mem::take(arguments),
            TerminatorInstruction::JmpIf { .. } => unreachable!(
                "If jmpif instructions are modified to support block arguments in the future, this match will need to be updated"
            ),
            _ => unreachable!(
                "Predecessor was already validated to have only a single jmp destination"
            ),
        };

        assert_eq!(block_params.len(), jump_args.len());
        for (param, arg) in block_params.iter().zip(jump_args) {
            values_to_replace.insert(*param, arg);
        }
    }
}

/// Try to inline a block into its predecessor, returning true if successful.
///
/// This will only occur if the predecessor's only successor is the given block.
/// It is also expected that the given block's only predecessor is the given one.
fn try_inline_successor(
    function: &mut Function,
    cfg: &mut ControlFlowGraph,
    block: BasicBlockId,
    values_to_replace: &mut ValueMapping,
) -> bool {
    if let Some(TerminatorInstruction::Jmp { destination, .. }) = function.dfg[block].terminator() {
        let destination = *destination;
        let predecessors = cfg.predecessors(destination);
        if predecessors.len() == 1 {
            drop(predecessors);

            // If the block has only 1 predecessor, we can safely remove its block parameters
            remove_block_parameters(function, destination, block, values_to_replace);

            // Note: this function relies on `remove_block_parameters` being called first.
            // Otherwise the inlined block will refer to parameters that no longer exist.
            //
            // If successful, `block` will be empty and unreachable after this call, so any
            // optimizations performed after this point on the same block should check if
            // the inlining here was successful before continuing.
            try_inline_into_predecessor(function, cfg, destination, block)
        } else {
            false
        }
    } else {
        false
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
mod tests {
    use crate::{
        assert_ssa_snapshot,
        ssa::{Ssa, opt::assert_ssa_does_not_change},
    };

    #[test]
    fn inline_blocks() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            jmp b1(Field 7)
          b1(v0: Field):
            jmp b2(v0)
          b2(v1: Field):
            return v1
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.simplify_cfg();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            return Field 7
        }
        ");
    }

    #[test]
    fn remove_known_jmpif() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u1):
            jmpif u1 1 then: b1, else: b2
          b1():
            return Field 1
          b2():
            jmp b1()
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.simplify_cfg();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u1):
            return Field 1
        }
        ");
    }

    #[test]
    fn swap_negated_jmpif_branches_in_brillig() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u1):
            v1 = allocate -> &mut Field
            store Field 0 at v1
            v3 = not v0
            jmpif v3 then: b1, else: b2
          b1():
            store Field 2 at v1
            jmp b2()
          b2():
            v5 = load v1 -> Field
            v6 = eq v5, Field 2
            constrain v5 == Field 2
            return
        }";
        let ssa = Ssa::from_str(src).unwrap();

        assert_ssa_snapshot!(ssa.simplify_cfg(), @r"
        brillig(inline) fn main f0 {
          b0(v0: u1):
            v1 = allocate -> &mut Field
            store Field 0 at v1
            v3 = not v0
            jmpif v0 then: b2, else: b1
          b1():
            store Field 2 at v1
            jmp b2()
          b2():
            v5 = load v1 -> Field
            v6 = eq v5, Field 2
            constrain v5 == Field 2
            return
        }
        ");
    }

    #[test]
    fn does_not_swap_negated_jmpif_branches_in_acir() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u1):
            v1 = not v0
            jmpif v1 then: b1, else: b2
          b1():
            jmp b2()
          b2():
            return
        }";
        assert_ssa_does_not_change(src, Ssa::simplify_cfg);
    }

    #[test]
    fn remove_converging_jmpif() {
        let src = r"
        brillig(inline) predicate_pure fn main f0 {
          b0(v0: i16):
            v2 = lt i16 3, v0
            jmpif v2 then: b1, else: b2
          b1():
            jmp b3()
          b2():
            jmp b3()
          b3():
            v4 = lt i16 5, v0
            jmpif v4 then: b4, else: b5
          b4():
            jmp b6()
          b5():
            jmp b6()
          b6():
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.simplify_cfg();

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) predicate_pure fn main f0 {
          b0(v0: i16):
            v2 = lt i16 3, v0
            jmp b1()
          b1():
            v4 = lt i16 5, v0
            jmp b2()
          b2():
            return
        }
        ");
    }

    #[test]
    fn remove_deep_converging_jmpif() {
        // This test is the same as `remove_converging_jmpif` except there is an extra layer of indirection
        // as b1 and b2 jump to b3 and b4 respectively before ultimately jumping to b5.
        // b5 then also continues the jump chain. We expect the b1 and b2 jump chain to settle on b7.
        let src = r"
        brillig(inline) predicate_pure fn main f0 {
          b0(v0: i16):
            v1 = lt i16 1, v0
            jmpif v1 then: b1, else: b2
          b1():
            jmp b3()
          b2():
            jmp b4()
          b3():
            jmp b5()
          b4():
            jmp b5()
          b5():
            jmp b6()
          b6():
            jmp b7()
          b7():
            v2 = lt i16 2, v0
            jmpif v2 then: b8, else: b9
          b8():
            jmp b10()
          b9():
            jmp b10()
          b10():
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.simplify_cfg();

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) predicate_pure fn main f0 {
          b0(v0: i16):
            v2 = lt i16 1, v0
            jmp b1()
          b1():
            v4 = lt i16 2, v0
            jmp b2()
          b2():
            return
        }
        ");
    }

    #[test]
    fn do_not_remove_non_converging_jmpif() {
        let src = r"
        brillig(inline) predicate_pure fn main f0 {
          b0(v0: i16):
            v1 = lt i16 1, v0
            jmpif v1 then: b1, else: b2
          b1():
            jmp b3()
          b2():
            jmp b4()
          b3():
            jmp b5()
          b4():
            jmp b6()
          b5():
            jmp b7()
          b6():
            jmp b8()
          b7():
            jmp b1()
          b8():
            return u32 2
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.simplify_cfg();

        // We expect the jmpif in b0 to remain in place as the jump chains for b1 and b2
        // resolved to b7 and b8 respectively which are not the same block.
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) predicate_pure fn main f0 {
          b0(v0: i16):
            v2 = lt i16 1, v0
            jmpif v2 then: b1, else: b2
          b1():
            jmp b1()
          b2():
            return u32 2
        }
        ");
    }

    #[test]
    fn do_not_remove_non_converging_jmpif_acir() {
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v13: [(u1, u1, [u8; 1], [u8; 1]); 3]):
            v23 = array_get v13, index u32 8 -> u1
            jmpif v23 then: b1, else: b2
          b1():
            v45 = array_get v13, index u32 4 -> u1
            jmpif v45 then: b3, else: b4
          b2():
            v25 = array_get v13, index u32 4 -> u1
            jmp b5(v25)
          b3():
            v46 = array_get v13, index u32 5 -> u1
            jmpif v46 then: b6, else: b7
          b4():
            jmp b8()
          b5(v14: u1):
            return v14
          b6():
            v47 = array_get v13, index u32 8 -> u1
            jmpif v47 then: b11, else: b12
          b7():
            jmp b9()
          b8():
            jmp b5(u1 0)
          b9():
            jmp b8()
          b10():
            jmp b9()
          b11():
            jmp b13()
          b12():
            jmp b13()
          b13():
            jmpif v47 then: b14, else: b15
          b14():
            jmp b16()
          b15():
            jmp b16()
          b16():
            jmp b10()
        }
        "#;

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.simplify_cfg();

        // We expect all jmpifs to remain.
        // The remaining jmpifs cannot be simplified as the flattening pass expects
        // to be able to merge into a single block.
        // We could potentially merge converging jmpifs in an ACIR runtime as well if
        // this restriction was removed or the SSA input to this pass was validated to pass
        // branch analysis.
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: [(u1, u1, [u8; 1], [u8; 1]); 3]):
            v3 = array_get v0, index u32 8 -> u1
            jmpif v3 then: b1, else: b2
          b1():
            v6 = array_get v0, index u32 4 -> u1
            jmpif v6 then: b3, else: b4
          b2():
            v5 = array_get v0, index u32 4 -> u1
            jmp b5(v5)
          b3():
            v8 = array_get v0, index u32 5 -> u1
            jmpif v8 then: b6, else: b7
          b4():
            jmp b8()
          b5(v1: u1):
            return v1
          b6():
            v9 = array_get v0, index u32 8 -> u1
            jmpif v9 then: b10, else: b11
          b7():
            jmp b9()
          b8():
            jmp b5(u1 0)
          b9():
            jmp b8()
          b10():
            jmp b12()
          b11():
            jmp b12()
          b12():
            jmpif v9 then: b13, else: b14
          b13():
            jmp b15()
          b14():
            jmp b15()
          b15():
            jmp b9()
        }
        ");
    }

    #[test]
    fn do_not_remove_converging_jmpif_with_instructions() {
        let src = r"
        brillig(inline) predicate_pure fn main f0 {
          b0(v0: i16):
            v2 = lt i16 3, v0
            jmpif v2 then: b1, else: b2
          b1():
            v4 = unchecked_add i16 1, v0
            jmp b3()
          b2():
            jmp b3()
          b3():
            return
        }
        ";
        assert_ssa_does_not_change(src, Ssa::simplify_cfg);
    }

    #[test]
    fn cyclic_jump_chain_in_converging_jmpif() {
        // Check that we handle a cyclic jump chain when checking for a converging jmpif.
        // If we were missing the appropriate checks this code could trigger an infinite loop.
        let src = r#"
        brillig(inline) predicate_pure fn main f0 {
          b0(v0: i16):
            v1 = lt i16 1, v0
            jmpif v1 then: b1, else: b2
          b1():
            jmp b2()
          b2():
            jmp b1()
        }
        "#;

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.simplify_cfg();

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) predicate_pure fn main f0 {
          b0(v0: i16):
            v2 = lt i16 1, v0
            jmpif v2 then: b1, else: b1
          b1():
            jmp b1()
        }
        ");
    }

    #[test]
    fn completely_removes_noop_jmpif() {
        let src = r#"
        brillig(inline) fn main f0 {
          b0():
            jmpif u1 1 then: b1, else: b2
          b1():
            jmp b3()
          b2():
            jmp b3()
          b3():
            return
        }
        "#;

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.simplify_cfg();

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0():
            return
        }
        ");
    }

    #[test]
    fn handles_cascading_simplifications() {
        // Simplifying the CFG from a block can result the block being updated to a form which can be simplified further.
        // We want to ensure that we handle any followup simplifications correctly.
        //
        // In this case we have a jmpif which is simplified to a jmp, which then can be inlined into its predecessor.
        // The new terminator instruction of the block is then a jmpif which can be simplified to a jmp.
        let src = r#"
        brillig(inline) impure fn main f0 {
          b0():
            jmpif u1 1 then: b1, else: b2
          b1():
            jmp b3(u1 1)
          b2():
            jmp b3(u1 0)
          b3(v0: u1):
            jmpif v0 then: b4, else: b5
          b4():
            jmp b6()
          b5():
            jmp b6()
          b6():
            return
        }
        "#;

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.simplify_cfg();

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) impure fn main f0 {
          b0():
            jmp b1()
          b1():
            return
        }
        ");
    }

    #[test]
    fn removes_unreachable_block() {
        let src = r#"
        brillig(inline) impure fn main f0 {
          b0():
            jmp b1()
          b1():
            return
          b2():
            jmp b1()
        }
        "#;

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.simplify_cfg();

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) impure fn main f0 {
          b0():
            return
        }
        ");
    }

    #[test]
    fn double_jmp_empty_blocks() {
        let src = "
        brillig(inline) fn test f0 {
          b0():
            jmp b1()
          b1():
            jmp b2()
          b2():
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.simplify_cfg();

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn test f0 {
          b0():
            return
        }
        ");
    }

    #[test]
    fn double_jmp_with_args_blocks() {
        let src = "
        brillig(inline) fn test f0 {
          b0(v0: Field):
            jmp b1(v0, Field 2)
          b1(v1: Field, v2: Field):
            jmp b2(v1)
          b2(v3: Field):
            return v3
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.simplify_cfg();

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn test f0 {
          b0(v0: Field):
            return v0
        }
        ");
    }

    #[test]
    fn deep_jmp_empty_blocks() {
        let src = "
        brillig(inline) fn test f0 {
          b0():
            jmp b1()
          b1():
            jmp b2()
          b2():
            jmp b3()
          b3():
            jmp b4()
          b4():
            jmp b5()
          b5():
            jmp b6()
          b6():
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.simplify_cfg();

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn test f0 {
          b0():
            return
        }
        ");
    }
}
