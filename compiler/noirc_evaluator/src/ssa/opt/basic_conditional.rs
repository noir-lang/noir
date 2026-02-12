//! An SSA pass that operates on Brillig functions
//! This optimization pass identifies simple conditional control flow patterns in unconstrained code
//! and flattens them to reduce the number of basic blocks and improve performance.
//!
//! e.g: `if c {a} else {b}` would be flattened to `c*(a-b)+b`
//! A simple conditional pattern is defined as a conditional sub-graph of the form `jmpif c: A, else B`, where A and B are basic blocks which join
//! on the same successor. This exclude the graph from having any nested conditional or loop statements.
//! Performance improvement is based on a simple execution cost metric
//!
//! This pass does not have any pre/post conditions.

use std::collections::HashSet;

use iter_extended::vecmap;
use rustc_hash::FxHashMap as HashMap;

use crate::ssa::{
    Ssa,
    ir::{
        basic_block::BasicBlockId,
        cfg::ControlFlowGraph,
        dfg::DataFlowGraph,
        function::{Function, FunctionId},
        function_inserter::FunctionInserter,
        instruction::TerminatorInstruction,
        post_order::PostOrder,
        value::ValueId,
    },
    opt::flatten_cfg::WorkList,
};

use super::flatten_cfg::Context;
#[derive(Debug, Clone)]
struct BasicConditional {
    block_entry: BasicBlockId,
    block_then: Option<BasicBlockId>,
    block_else: Option<BasicBlockId>,
    block_exit: BasicBlockId,
}

impl Ssa {
    #[tracing::instrument(level = "trace", skip(self))]
    /// Apply the basic_conditional pass to all functions of the program.
    /// It first retrieve the `no_predicates` attribute of each function which will be used during the flattening.
    pub(crate) fn flatten_basic_conditionals(mut self) -> Ssa {
        // Retrieve the 'no_predicates' attribute of the functions in a map, to avoid problems with borrowing
        let mut no_predicates = HashMap::default();
        for function in self.functions.values() {
            no_predicates.insert(function.id(), function.is_no_predicates());
        }
        for function in self.functions.values_mut() {
            flatten_function(function, &no_predicates);
        }
        self
    }
}

/// Returns the blocks of the simple conditional sub-graph of the CFG whose input block is the entry.
/// Returns None if the input block is not the entry block of a simple conditional.
/// A simple conditional is an if-then(-else) statement where branches are 'small' basic blocks.
/// 'Small' basic blocks means that we expect their execution cost to be small.
///
/// In case the block is the entry of a 'simple conditional', the function returns a BasicConditional which
/// consist of the list of the conditional blocks:
///     block_entry
///      /       \
/// block_then   block_else
///     \       /
///     block_exit
/// block_then and block_else are optionals, in order to account for the case when there is no 'then' or no 'else' branch
/// Only structured CFG with this shape are considered:
/// - block_entry has exactly 2 successors
/// - block_then and block_else have exactly 1 successor, which is block_exit, or one of them is block_exit
/// - block_exit has exactly 2 predecessors (block_then and block_else)
///
/// Furthermore, cost of block_then + cost of block_else must be less than their average cost + jump overhead cost
fn is_conditional(
    block: BasicBlockId,
    cfg: &ControlFlowGraph,
    function: &Function,
) -> Option<BasicConditional> {
    // jump overhead is the cost for doing the conditional and jumping around the blocks
    // We use 10 as a rough estimate, the real cost is less.
    let jump_overhead = 10;

    // A conditional must end with a JmpIf
    let Some(TerminatorInstruction::JmpIf {
        condition: _,
        then_destination,
        else_destination,
        call_stack: _,
    }) = function.dfg[block].terminator()
    else {
        return None;
    };

    let mut then_successors = cfg.successors(*then_destination);
    let mut else_successors = cfg.successors(*else_destination);
    let then_successors_len = then_successors.len();
    let else_successors_len = else_successors.len();
    let next_then = then_successors.next();
    let next_else = else_successors.next();

    if next_then == Some(block) || next_else == Some(block) {
        // this is a loop, not a conditional
        return None;
    }

    let result = if then_successors_len == 1 && else_successors_len == 1 && next_then == next_else {
        // The branches join on one block so it is a non-nested conditional with a classical diamond shape:
        //    block
        //    /    \
        // then   else
        //    \    /
        //   next_then
        // We check that the cost of the flattened code is lower than the cost of the branches
        let cost_left = block_flatten_cost(*then_destination, &function.dfg)?;
        let cost_right = block_flatten_cost(*else_destination, &function.dfg)?;
        // For the flattening to be valuable, we compare the cost of the flattened code with the average cost of the 2 branches,
        // including an overhead to take into account the jumps between the blocks.
        // We use the average cost of the 2 branches, assuming that both branches are equally likely to be executed.
        let cost = cost_right.saturating_add(cost_left);
        if cost >= cost / 2 + jump_overhead {
            return None;
        }
        BasicConditional {
            block_entry: block,
            block_then: Some(*then_destination),
            block_else: Some(*else_destination),
            block_exit: next_then.unwrap(),
        }
    } else if then_successors_len == 1 && next_then == Some(*else_destination) {
        // Left branch joins the right branch, e.g if/then statement with no else:
        //    block
        //    /    \
        // then     \
        //     \    |
        //      -> else
        // This case may not happen (i.e not generated), but it is safer to handle it (e.g in case it happens due to some optimizations)
        let cost = block_flatten_cost(*then_destination, &function.dfg)?;
        if cost >= cost / 2 + jump_overhead {
            return None;
        }
        BasicConditional {
            block_entry: block,
            block_then: Some(*then_destination),
            block_else: None,
            block_exit: *else_destination,
        }
    } else if else_successors_len == 1 && next_else == Some(*then_destination) {
        // Right branch joins the left branch, e.g if/else statement with no then
        // This case may not happen (i.e not generated), but it is safer to handle it (e.g in case it happens due to some optimizations)
        //    block
        //    /    \
        //   |     else
        //   |      |
        //    \    /
        //     then
        let cost = block_flatten_cost(*else_destination, &function.dfg)?;
        if cost >= cost / 2 + jump_overhead {
            return None;
        }
        BasicConditional {
            block_entry: block,
            block_then: None,
            block_else: Some(*else_destination),
            block_exit: *else_destination,
        }
    } else {
        return None;
    };

    // A conditional exit would have exactly 2 predecessors
    (cfg.predecessors(result.block_exit).len() == 2).then_some(result)
}

/// Computes a cost estimate for flattening a basic block in a conditional.
///
/// Returns `None` if the block contains instructions that cannot be safely
/// flattened (side-effectful instructions like constraints, calls, memory ops,
/// div/mod, shifts). Otherwise returns the estimated Brillig opcode cost.
fn block_flatten_cost(block: BasicBlockId, dfg: &DataFlowGraph) -> Option<u32> {
    let mut cost: u32 = 0;
    for instruction_id in dfg[block].instructions() {
        let instruction = &dfg[*instruction_id];
        if !instruction.can_flatten_in_conditional(dfg) {
            return None;
        }
        cost = cost.saturating_add(instruction.cost(*instruction_id, dfg) as u32);
    }
    Some(cost)
}

/// Identifies all simple conditionals in the function and flattens them
fn flatten_function(function: &mut Function, no_predicates: &HashMap<FunctionId, bool>) {
    // This pass is dedicated to brillig functions
    if !function.runtime().is_brillig() {
        return;
    }
    let cfg = ControlFlowGraph::with_function(function);
    let mut stack = vec![function.entry_block()];
    let mut processed = HashSet::new();
    // List of all the simple conditionals that we will identify in the function
    let mut conditionals = Vec::new();

    // 1. Process all blocks of the cfg, starting from the root and following the successors
    while let Some(block) = stack.pop() {
        // Avoid cycles
        if processed.contains(&block) {
            continue;
        }
        processed.insert(block);

        // Identify the simple conditionals
        if let Some(conditional) = is_conditional(block, &cfg, function) {
            // no need to check the branches, process the join block directly
            stack.push(conditional.block_exit);
            conditionals.push(conditional);
        } else {
            stack.extend(cfg.successors(block));
        }
    }

    // 2. Flatten all simple conditionals
    // process basic conditionals in reverse order so that
    // a conditional does not impact the previous ones
    conditionals.reverse();
    flatten_multiple(&conditionals, function, no_predicates);
}

/// Flattens multiple basic conditionals within a function.
///
/// This function processes a collection of basic conditionals identified in the CFG and flattens them
/// to reduce control flow complexity. Each conditional is processed with its own context, and the
/// flattening results are then propagated throughout the entire function.
///
/// # Parameters
/// * `conditionals` - The list of basic conditionals to flatten, assumed in reverse order
/// * `function` - The function being optimized
/// * `no_predicates` - Map of function IDs to their no_predicates attribute for handling function calls
///
/// # Process
/// 1. Each conditional is flattened independently using a fresh context
/// 2. Value mappings from all conditionals are collected into a unified mapping
/// 3. The entire function is remapped only once in post-order to apply all value simplifications
fn flatten_multiple(
    conditionals: &Vec<BasicConditional>,
    function: &mut Function,
    no_predicates: &HashMap<FunctionId, bool>,
) {
    // 1. process each basic conditional, using a new context per conditional
    let post_order = PostOrder::with_function(function);

    let mut mapping = HashMap::default();
    for conditional in conditionals {
        let cfg = ControlFlowGraph::with_function(function);
        let cfg_root = function.entry_block();
        let mut branch_ends = HashMap::default();
        branch_ends.insert(conditional.block_entry, conditional.block_exit);
        let mut context = Context::new(function, cfg, branch_ends, cfg_root);
        context.flatten_single_conditional(conditional, no_predicates);
        // extract the mapping into 'mapping
        context.inserter.extract_mapping(&mut mapping);
    }
    // 2. re-map the full program for values that may been simplified.
    if !mapping.is_empty() {
        for block in post_order.as_slice() {
            Context::map_block_with_mapping(mapping.clone(), function, *block);
        }
    }
}

impl Context<'_> {
    /// Flattens a single basic conditional by inlining the 2 branches into the entry block.
    ///
    /// This method transforms a conditional control flow pattern (if-then-else) into straight-line code
    /// by merging the entry, then, else, and exit blocks. The conditional logic is converted into
    /// predicated operations using cast and multiplication operations to select between branch values.
    /// The method is adapted from `flatten_cfg`, tailored to do flatten only the input conditional.
    ///
    /// # Parameters
    /// * `conditional` - The basic conditional structure to flatten
    /// * `no_predicates` - Map of function IDs to their no_predicates attribute
    ///
    /// # Implementation Details
    /// - Sets up context state (target_block, no_predicate) to enable proper inlining
    /// - Inlines each block's instructions into the entry block
    /// - Handles terminators to manage control flow during inlining
    /// - Uses a WorkList to track which blocks need processing
    /// - Copies the exit block's terminator to the entry block after inlining
    /// - Restores original context state after completion
    fn flatten_single_conditional(
        &mut self,
        conditional: &BasicConditional,
        no_predicates: &HashMap<FunctionId, bool>,
    ) {
        // Manually inline 'then', 'else' and 'exit' into the entry block
        //0. initialize the context for flattening a 'single conditional'
        let old_target = self.target_block;
        let old_no_predicate = self.no_predicate;
        self.target_block = conditional.block_entry;
        self.no_predicate = true;
        //1. process 'then' branch
        self.inline_block(conditional.block_entry, no_predicates);
        let mut work_list = WorkList::new();
        let to_process = self.handle_terminator(conditional.block_entry, &work_list);
        work_list.extend(to_process);

        if let Some(then) = conditional.block_then {
            assert_eq!(work_list.pop(), conditional.block_then);
            self.inline_block(then, no_predicates);
            let to_process = self.handle_terminator(then, &work_list);
            work_list.extend(to_process);
        }

        //2. process 'else' branch, in case there is no 'then'
        let next = work_list.pop();
        if next == conditional.block_else {
            let next = next.unwrap();
            self.inline_block(next, no_predicates);
            let _ = self.handle_terminator(next, &work_list);
        } else {
            assert_eq!(next, Some(conditional.block_exit));
        }

        //3. process 'exit' block
        self.inline_block(conditional.block_exit, no_predicates);
        // Manually set the terminator of the entry block to the one of the exit block
        let terminator =
            self.inserter.function.dfg[conditional.block_exit].terminator().unwrap().clone();
        let new_terminator = match terminator {
            TerminatorInstruction::JmpIf {
                condition,
                then_destination,
                else_destination,
                call_stack,
            } => {
                let condition = self.inserter.resolve(condition);
                TerminatorInstruction::JmpIf {
                    condition,
                    then_destination,
                    else_destination,
                    call_stack,
                }
            }
            TerminatorInstruction::Jmp { destination, arguments, call_stack } => {
                let arguments = vecmap(arguments, |value| self.inserter.resolve(value));
                TerminatorInstruction::Jmp { destination, arguments, call_stack }
            }
            TerminatorInstruction::Return { return_values, call_stack } => {
                let return_values = vecmap(return_values, |value| self.inserter.resolve(value));
                TerminatorInstruction::Return { return_values, call_stack }
            }
            TerminatorInstruction::Unreachable { call_stack } => {
                TerminatorInstruction::Unreachable { call_stack }
            }
        };
        self.inserter.function.dfg.set_block_terminator(conditional.block_entry, new_terminator);
        self.inserter.map_data_bus_in_place();
        //4. restore the context, in case it is re-used.
        self.target_block = old_target;
        self.no_predicate = old_no_predicate;
    }

    /// Applies value mappings to all instructions and terminators in a block.
    ///
    /// This method rewrites a block by replacing old value IDs with their mapped equivalents
    /// according to the provided mapping. This is used to propagate value simplifications
    /// from conditional flattening throughout the rest of the function.
    ///
    /// # Parameters
    /// * `mapping` - HashMap mapping old ValueIds to their simplified/replaced ValueIds
    /// * `func` - The function containing the block to update
    /// * `block` - The BasicBlockId of the block to remap
    fn map_block_with_mapping(
        mapping: HashMap<ValueId, ValueId>,
        func: &mut Function,
        block: BasicBlockId,
    ) {
        // Map all instructions in the block
        let mut inserter = FunctionInserter::new(func);
        inserter.set_mapping(mapping);
        let instructions = inserter.function.dfg[block].instructions().to_vec();
        for instruction in instructions {
            inserter.map_instruction_in_place(instruction);
        }
        inserter.map_terminator_in_place(block);
    }
}

#[cfg(test)]
mod tests {
    use crate::{assert_ssa_snapshot, ssa::Ssa};

    #[test]
    fn basic_jmpif() {
        let src = "
              brillig(inline) fn foo f0 {
                b0(v0: u32):
                  v3 = eq v0, u32 0
                  jmpif v3 then: b2, else: b1
                b1():
                  jmp b3(u32 5)
                b2():
                  jmp b3(u32 3)
                b3(v1: u32):
                  return v1
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();
        assert_eq!(ssa.main().reachable_blocks().len(), 4);

        let ssa = ssa.flatten_basic_conditionals();
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn foo f0 {
          b0(v0: u32):
            v2 = eq v0, u32 0
            v3 = not v2
            v4 = cast v2 as u32
            v5 = cast v3 as u32
            v7 = unchecked_mul v4, u32 3
            v9 = unchecked_mul v5, u32 5
            v10 = unchecked_add v7, v9
            return v10
        }
        ");
    }

    #[test]
    fn array_jmpif() {
        let src = r#"
              brillig(inline) fn foo f0 {
                b0(v0: u32):
                  v3 = eq v0, u32 5
                  jmpif v3 then: b2, else: b1
                b1():
                  v10 = make_array b"foo"
                  jmp b3(v10)
                b2():
                  v7 = make_array b"bar"
                  jmp b3(v7)
                b3(v1: [u8; 3]):
                  return v1
            }
            "#;
        let ssa = Ssa::from_str(src).unwrap();
        assert_eq!(ssa.main().reachable_blocks().len(), 4);
        let ssa = ssa.flatten_basic_conditionals();
        // With target_cost, a 3-element MakeArray costs min(3*2, 20) = 6 < jump_overhead(10),
        // so the conditional gets flattened.
        assert_ssa_snapshot!(ssa, @r#"
        brillig(inline) fn foo f0 {
          b0(v0: u32):
            v2 = eq v0, u32 5
            v6 = make_array b"bar"
            v7 = not v2
            v10 = make_array b"foo"
            v11 = if v2 then v6 else (if v7) v10
            return v11
        }
        "#);
    }

    #[test]
    fn large_array_jmpif_not_flattened() {
        // Large MakeArrays (10+ elements) cost min(len*2, 20) = 20 each.
        // Combined cost of 40 >= 40/2 + 10 = 30, so the conditional is not worth flattening.
        let src = r#"
              brillig(inline) fn foo f0 {
                b0(v0: u32):
                  v3 = eq v0, u32 5
                  jmpif v3 then: b2, else: b1
                b1():
                  v10 = make_array b"0123456789a"
                  jmp b3(v10)
                b2():
                  v7 = make_array b"abcdefghijk"
                  jmp b3(v7)
                b3(v1: [u8; 11]):
                  return v1
            }
            "#;
        let ssa = Ssa::from_str(src).unwrap();
        assert_eq!(ssa.main().reachable_blocks().len(), 4);
        let ssa = ssa.flatten_basic_conditionals();
        // Not flattened — too expensive
        assert_eq!(ssa.main().reachable_blocks().len(), 4);
    }

    #[test]
    fn nested_jmpifs() {
        let src = "
            brillig(inline) fn foo f0 {
              b0(v0: u32):
                v5 = eq v0, u32 5
                v6 = not v5
                jmpif v5 then: b5, else: b1
              b1():
                v8 = lt v0, u32 3
                jmpif v8 then: b3, else: b2
              b2():
                v9 = truncate v0 to 2 bits, max_bit_size: 32
                jmp b4(v9)
              b3():
                v10 = truncate v0 to 1 bits, max_bit_size: 32
                jmp b4(v10)
              b4(v1: u32):
                jmp b9(v1)
              b5():
                v12 = lt u32 2, v0
                jmpif v12 then: b7, else: b6
              b6():
                v13 = truncate v0 to 3 bits, max_bit_size: 32
                jmp b8(v13)
              b7():
                v14 = and v0, u32 2
                jmp b8(v14)
              b8(v2: u32):
                jmp b9(v2)
              b9(v3: u32):
                return v3
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();
        assert_eq!(ssa.main().reachable_blocks().len(), 10);

        let ssa = ssa.flatten_basic_conditionals();
        assert_eq!(ssa.main().reachable_blocks().len(), 4);
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn foo f0 {
          b0(v0: u32):
            v3 = eq v0, u32 5
            v4 = not v3
            jmpif v3 then: b2, else: b1
          b1():
            v16 = lt v0, u32 3
            v17 = truncate v0 to 1 bits, max_bit_size: 32
            v18 = not v16
            v19 = truncate v0 to 2 bits, max_bit_size: 32
            v20 = cast v16 as u32
            v21 = cast v18 as u32
            v22 = unchecked_mul v20, v17
            v23 = unchecked_mul v21, v19
            v24 = unchecked_add v22, v23
            jmp b3(v24)
          b2():
            v6 = lt u32 2, v0
            v7 = and v0, u32 2
            v8 = not v6
            v9 = truncate v0 to 3 bits, max_bit_size: 32
            v10 = cast v6 as u32
            v11 = cast v8 as u32
            v12 = unchecked_mul v10, v7
            v13 = unchecked_mul v11, v9
            v14 = unchecked_add v12, v13
            jmp b3(v14)
          b3(v1: u32):
            return v1
        }
        ");
    }
}
