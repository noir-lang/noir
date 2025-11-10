//! This module analyzes the liveness of variables (non-constant values) throughout a function.
//! It uses the approach detailed in the section 4.2 of this paper <https://inria.hal.science/inria-00558509v2/document>

use std::collections::BTreeSet;

use crate::ssa::{
    ir::{
        basic_block::{BasicBlock, BasicBlockId},
        cfg::ControlFlowGraph,
        dfg::DataFlowGraph,
        dom::DominatorTree,
        function::Function,
        instruction::{Instruction, InstructionId},
        post_order::PostOrder,
        value::{Value, ValueId},
    },
    opt::Loops,
};

use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use super::constant_allocation::ConstantAllocation;

/// A set of [ValueId]s referring to SSA variables (not functions).
type Variables = HashSet<ValueId>;
/// The set variables which are dead after a given instruction (in a given block).
type LastUses = HashMap<InstructionId, Variables>;
/// Maps a loop (identified by its header and back-edge) to the blocks in the loop body.
type LoopMap = HashMap<BackEdge, BTreeSet<BasicBlockId>>;

/// A back edge is an edge from a node to one of its ancestors. It denotes a loop in the CFG.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct BackEdge {
    /// The header of the loop.
    header: BasicBlockId,
    /// The back-edge of the loop.
    start: BasicBlockId,
}

/// Check if the [Value] behind the [ValueId] requires register allocation (like a function parameter),
/// rather than a global value like a user-defined function, intrinsic, or foreign function.
pub(super) fn is_variable(value_id: ValueId, dfg: &DataFlowGraph) -> bool {
    let value = &dfg[value_id];

    match value {
        Value::Instruction { .. }
        | Value::Param { .. }
        | Value::NumericConstant { .. }
        | Value::Global(_) => true,
        // Functions are not variables in a defunctionalized SSA. Only constant function values should appear.
        Value::ForeignFunction(_) | Value::Function(_) | Value::Intrinsic(..) => false,
    }
}

/// Collect all [ValueId]s used in an [Instruction] which refer to variables (not functions).
pub(super) fn variables_used_in_instruction(
    instruction: &Instruction,
    dfg: &DataFlowGraph,
) -> Variables {
    let mut used = HashSet::default();

    instruction.for_each_value(|value_id| {
        if is_variable(value_id, dfg) {
            used.insert(value_id);
        }
    });

    used
}

/// Collect all [ValueId]s used in an [BasicBlock] which refer to [Variables].
///
/// Includes all the variables in the parameters, instructions and the terminator.
fn variables_used_in_block(block: &BasicBlock, dfg: &DataFlowGraph) -> Variables {
    let mut used: Variables = block
        .instructions()
        .iter()
        .flat_map(|instruction_id| {
            let instruction = &dfg[*instruction_id];
            variables_used_in_instruction(instruction, dfg)
        })
        .collect();

    // We consider block parameters used, so they live up to the block that owns them.
    used.extend(block.parameters().iter());

    if let Some(terminator) = block.terminator() {
        terminator.for_each_value(|value_id| {
            if is_variable(value_id, dfg) {
                used.insert(value_id);
            }
        });
    }

    used
}

/// A struct representing the liveness of variables throughout a function.
#[derive(Default)]
pub(crate) struct VariableLiveness {
    cfg: ControlFlowGraph,
    /// The variables that are alive before the block starts executing.
    live_in: HashMap<BasicBlockId, Variables>,
    /// The variables that stop being alive after each specific instruction.
    last_uses: HashMap<BasicBlockId, LastUses>,
    /// The list of block params the given block is defining.
    /// The order matters for the entry block, so it's a vec.
    ///
    /// By _defining_  we mean the allocation of variables that appear in the parameter
    /// list of some other block which this one immediately dominates, with values to be
    /// assigned in the terminators of the predecessors of that block.
    param_definitions: HashMap<BasicBlockId, Vec<ValueId>>,
}

impl VariableLiveness {
    /// Computes the liveness of variables throughout a function.
    pub(crate) fn from_function(func: &Function, constants: &ConstantAllocation) -> Self {
        let loops = Loops::find_all(func);

        let back_edges: LoopMap = loops
            .yet_to_unroll
            .into_iter()
            .map(|_loop| {
                let back_edge = BackEdge { header: _loop.header, start: _loop.back_edge_start };
                let loop_body = _loop.blocks;
                (back_edge, loop_body)
            })
            .collect();

        Self {
            cfg: loops.cfg,
            live_in: HashMap::default(),
            last_uses: HashMap::default(),
            param_definitions: HashMap::default(),
        }
        .compute_block_param_definitions(func, &loops.dom)
        .compute_live_in_of_blocks(func, constants, back_edges)
        .compute_last_uses(func)
    }

    /// The set of values that are alive before the block starts executing.
    pub(crate) fn get_live_in(&self, block_id: &BasicBlockId) -> &Variables {
        self.live_in.get(block_id).expect("live_in should have been calculated")
    }

    /// The set of values that are alive after the block has finished executed.
    ///
    /// By definition this is the union of variables alive in the successors of the block.
    pub(crate) fn get_live_out(&self, block_id: &BasicBlockId) -> Variables {
        let mut live_out = HashSet::default();
        for successor_id in self.cfg.successors(*block_id) {
            live_out.extend(self.get_live_in(&successor_id).iter().copied());
        }
        live_out
    }

    /// For a given block, get the map of instruction ID to the set of values
    /// that die after the instruction has executed.
    pub(crate) fn get_last_uses(&self, block_id: &BasicBlockId) -> &LastUses {
        self.last_uses.get(block_id).expect("last_uses should have been calculated")
    }

    /// Retrieves the list of block parameters the given block is defining.
    ///
    /// Block parameters are defined before the block that owns them
    /// (since they are used by the predecessor blocks to pass values).
    /// They must be defined in the immediate dominator, which is the
    /// last point where the block parameter can be allocated
    /// without it being allocated in different places in different branches.
    pub(crate) fn defined_block_params(&self, block_id: &BasicBlockId) -> Vec<ValueId> {
        self.param_definitions.get(block_id).cloned().unwrap_or_default()
    }

    /// Compute [VariableLiveness::param_definitions].
    ///
    /// Append the parameters of each block to the parameter definition list of
    /// its immediate dominator.
    fn compute_block_param_definitions(mut self, func: &Function, dom: &DominatorTree) -> Self {
        assert!(self.param_definitions.is_empty(), "only define parameters once");

        // Going in reverse post order to process the entry block first.
        let reverse_post_order = PostOrder::with_function(func).into_vec_reverse();
        for block in reverse_post_order {
            let params = func.dfg[block].parameters();
            // If it has no dominator, it's the entry block
            let dominator_block = dom.immediate_dominator(block).unwrap_or(func.entry_block());
            let definitions_for_the_dominator =
                self.param_definitions.entry(dominator_block).or_default();
            definitions_for_the_dominator.extend(params.iter());
        }

        self
    }

    /// Compute [VariableLiveness::live_in].
    ///
    /// Collect the variables which are alive before each block.
    fn compute_live_in_of_blocks(
        mut self,
        func: &Function,
        constants: &ConstantAllocation,
        back_edges: LoopMap,
    ) -> Self {
        // First pass, propagate up the live_ins skipping back edges.
        self.compute_live_in(func, func.entry_block(), constants, &back_edges);

        // Second pass, propagate header live_ins to the loop bodies.
        for (back_edge, loop_body) in back_edges {
            self.update_live_ins_within_loop(back_edge, loop_body);
        }

        self
    }

    /// Starting with the entry block, traverse down all successors to compute their `live_in`,
    /// then propagate the information back up towards the ancestors as `live_out`.
    ///
    /// The variables live at the *beginning* of a block are the variables used by the block,
    /// plus the variables used by the successors of the block, minus the variables defined
    /// in the block (by definition not alive at the beginning).
    ///
    /// This is an iterative implementation to avoid stack overflows on complex programs.
    fn compute_live_in(
        &mut self,
        func: &Function,
        entry_block: BasicBlockId,
        constants: &ConstantAllocation,
        back_edges: &LoopMap,
    ) {
        // Each entry is (block_id, processing_state)
        // processing_state: false = need to process successors, true = ready to compute live_in
        let mut stack = vec![(entry_block, false)];
        let mut visited = HashSet::default();

        while let Some((block_id, processed)) = stack.pop() {
            if processed {
                // All successors have been processed, now compute live_in for this block
                let block = &func.dfg[block_id];
                let mut live_out = HashSet::default();

                // Collect the `live_in` of successors; their union is the `live_out` of the parent.
                for successor_id in block.successors() {
                    // Skip back edges: do not revisit the header of the loop
                    if back_edges.contains_key(&BackEdge { start: block_id, header: successor_id })
                    {
                        continue;
                    }
                    // Add the live_in of the successor to the union that forms the live_out of the parent.
                    live_out.extend(
                        self.live_in
                            .get(&successor_id)
                            .expect("live_in for successor should have been calculated")
                            .iter()
                            .copied(),
                    );
                }

                // Based on the paper mentioned in the module docs, the definition would be:
                // live_in[BlockId] = before_def[BlockId] union (live_out[BlockId] - killed[BlockId])

                // Variables used in this block, defined in this block or before.
                let used = variables_used_in_block(block, &func.dfg);

                // Variables defined in this block are not alive at the beginning.
                let defined = self.variables_defined_in_block(block_id, &func.dfg, constants);

                // Live at the beginning are the variables used, but not defined in this block, plus the ones
                // it passes through to its successors, which are used by them, but not defined here.
                // (Variables used by successors and defined in this block are part of `live_out`, but not `live_in`).
                let live_in =
                    used.union(&live_out).filter(|v| !defined.contains(v)).copied().collect();

                self.live_in.insert(block_id, live_in);
            } else {
                // First visit: check if we've already processed this block
                if !visited.insert(block_id) {
                    continue;
                }

                let block = &func.dfg[block_id];

                // Check if all successors (except back edges) have been processed
                let mut all_successors_processed = true;
                let mut unprocessed_successors = Vec::new();

                for successor_id in block.successors() {
                    // Skip back edges
                    if back_edges.contains_key(&BackEdge { start: block_id, header: successor_id })
                    {
                        continue;
                    }
                    // If successor hasn't been processed yet, we need to process it first
                    if !self.live_in.contains_key(&successor_id) {
                        all_successors_processed = false;
                        unprocessed_successors.push(successor_id);
                    }
                }

                // Push this block back with processed = true (for after successors)
                stack.push((block_id, true));
                if !all_successors_processed {
                    // Push unprocessed successors with processed = false
                    for successor_id in unprocessed_successors {
                        stack.push((successor_id, false));
                    }
                }
            }
        }
    }

    /// Collects all the variables defined in a block, which includes:
    /// * parameters of descendants this block immediately dominates
    /// * the results of the instructions in the block
    /// * constants which were allocated to this block
    fn variables_defined_in_block(
        &self,
        block_id: BasicBlockId,
        dfg: &DataFlowGraph,
        constants: &ConstantAllocation,
    ) -> Variables {
        let block = &dfg[block_id];
        let mut defined_vars = HashSet::default();

        defined_vars.extend(self.defined_block_params(&block_id));

        for instruction_id in block.instructions() {
            let result_values = dfg.instruction_results(*instruction_id);
            defined_vars.extend(result_values.iter().copied());
        }

        defined_vars.extend(constants.allocated_in_block(block_id));

        defined_vars
    }

    /// Once we know which variables are alive before the loop header,
    /// we can append those variables to all of the loop's blocks.
    /// Since we know that we have to come back
    /// to the beginning of the loop, none of those blocks are allowed
    /// anything but to keep these variables alive, so that the header
    /// can use them again.
    fn update_live_ins_within_loop(
        &mut self,
        back_edge: BackEdge,
        loop_body: BTreeSet<BasicBlockId>,
    ) {
        let header_live_in = self.get_live_in(&back_edge.header).clone();

        for body_block_id in loop_body {
            self.live_in
                .get_mut(&body_block_id)
                .expect("Live ins should have been calculated")
                .extend(header_live_in.iter().copied());
        }
    }

    /// Compute [VariableLiveness::last_uses].
    ///
    /// For each block, starting from the terminator than going backwards through the instructions,
    /// take note of the first (technically last) instruction the value is used in.
    fn compute_last_uses(mut self, func: &Function) -> Self {
        for block_id in func.reachable_blocks() {
            let block = &func.dfg[block_id];
            let live_out = self.get_live_out(&block_id);

            // Variables we have already visited, ie. they are used in "later" instructions or the terminator.
            let mut used_after: Variables = Default::default();
            // Variables becoming dead after each instruction.
            let mut block_last_uses: LastUses = Default::default();

            // First, handle the terminator; none of the instructions should cause these to go dead.
            if let Some(terminator_instruction) = block.terminator() {
                terminator_instruction.for_each_value(|value_id| {
                    if is_variable(value_id, &func.dfg) {
                        used_after.insert(value_id);
                    }
                });
            }

            // Then, handle the instructions in reverse order to find the last use.
            for instruction_id in block.instructions().iter().rev() {
                let instruction = &func.dfg[*instruction_id];
                // Collect the variables which will be dead after this instruction.
                let instruction_last_uses = variables_used_in_instruction(instruction, &func.dfg)
                    .into_iter()
                    .filter(|id| !used_after.contains(id) && !live_out.contains(id))
                    .collect();
                // Remember that we have already handled these.
                used_after.extend(&instruction_last_uses);
                // Remember that we can deallocate these after this instruction.
                block_last_uses.insert(*instruction_id, instruction_last_uses);
            }

            self.last_uses.insert(block_id, block_last_uses);
        }

        self
    }
}

#[cfg(test)]
mod test {
    use noirc_frontend::monomorphization::ast::InlineType;
    use rustc_hash::FxHashSet;

    use crate::assert_artifact_snapshot;
    use crate::brillig::brillig_gen::constant_allocation::ConstantAllocation;
    use crate::brillig::brillig_gen::tests::ssa_to_brillig_artifacts;
    use crate::brillig::brillig_gen::variable_liveness::VariableLiveness;
    use crate::ssa::function_builder::FunctionBuilder;
    use crate::ssa::ir::basic_block::BasicBlockId;
    use crate::ssa::ir::function::RuntimeType;
    use crate::ssa::ir::instruction::BinaryOp;
    use crate::ssa::ir::map::Id;
    use crate::ssa::ir::types::{NumericType, Type};
    use crate::ssa::ir::value::ValueId;
    use crate::ssa::ssa_gen::Ssa;

    #[test]
    fn simple_back_propagation() {
        // brillig fn main f0 {
        //     b0(v0: Field, v1: Field):
        //       v3 = allocate
        //       store Field 0 at v3
        //       v4 = eq v0, Field 0
        //       jmpif v4 then: b1, else: b2
        //     b2():
        //       v7 = add v0, Field 27
        //       store v7 at v3
        //       jmp b3()
        //     b1():
        //       v6 = add v1, Field 27
        //       store v6 at v3
        //       jmp b3()
        //     b3():
        //       v8 = load v3
        //       return v8
        //   }

        let main_id = Id::test_new(1);
        let mut builder = FunctionBuilder::new("main".into(), main_id);
        builder.set_runtime(RuntimeType::Brillig(InlineType::default()));

        let b1 = builder.insert_block();
        let b2 = builder.insert_block();
        let b3 = builder.insert_block();

        let v0 = builder.add_parameter(Type::field());
        let v1 = builder.add_parameter(Type::field());

        let v3 = builder.insert_allocate(Type::field());

        let zero = builder.field_constant(0u128);
        builder.insert_store(v3, zero);

        let v4 = builder.insert_binary(v0, BinaryOp::Eq, zero);

        builder.terminate_with_jmpif(v4, b1, b2);

        builder.switch_to_block(b2);

        let twenty_seven = builder.field_constant(27u128);
        let v7 = builder.insert_binary(v0, BinaryOp::Add { unchecked: false }, twenty_seven);
        builder.insert_store(v3, v7);

        builder.terminate_with_jmp(b3, vec![]);

        builder.switch_to_block(b1);

        let v6 = builder.insert_binary(v1, BinaryOp::Add { unchecked: false }, twenty_seven);
        builder.insert_store(v3, v6);

        builder.terminate_with_jmp(b3, vec![]);

        builder.switch_to_block(b3);

        let v8 = builder.insert_load(v3, Type::field());

        builder.terminate_with_return(vec![v8]);

        let ssa = builder.finish();
        let func = ssa.main();
        let constants = ConstantAllocation::from_function(func);
        let liveness = VariableLiveness::from_function(func, &constants);

        assert!(liveness.get_live_in(&func.entry_block()).is_empty());
        assert_eq!(
            liveness.get_live_in(&b2),
            &FxHashSet::from_iter([v3, v0, twenty_seven].into_iter())
        );
        assert_eq!(
            liveness.get_live_in(&b1),
            &FxHashSet::from_iter([v3, v1, twenty_seven].into_iter())
        );
        assert_eq!(liveness.get_live_in(&b3), &FxHashSet::from_iter([v3].into_iter()));

        let block_1 = &func.dfg[b1];
        let block_2 = &func.dfg[b2];
        let block_3 = &func.dfg[b3];
        assert_eq!(
            liveness.get_last_uses(&b1).get(&block_1.instructions()[0]),
            Some(&FxHashSet::from_iter([v1, twenty_seven].into_iter()))
        );
        assert_eq!(
            liveness.get_last_uses(&b2).get(&block_2.instructions()[0]),
            Some(&FxHashSet::from_iter([v0, twenty_seven].into_iter()))
        );
        assert_eq!(
            liveness.get_last_uses(&b3).get(&block_3.instructions()[0]),
            Some(&FxHashSet::from_iter([v3].into_iter()))
        );
    }

    #[test]
    fn propagation_with_nested_loops() {
        // brillig fn main f0 {
        //     b0(v0: u32, v1: u32):
        //       v3 = allocate
        //       store u32 0 at v3
        //       jmp b1(u32 0)
        //     b1(v4: u32):
        //       v5 = lt v4, v0
        //       jmpif v5 then: b2, else: b3
        //     b3():
        //       v17 = load v3
        //       return v17
        //     b2():
        //       v6 = mul v4, v4
        //       jmp b4(v0)
        //     b4(v7: u32):
        //       v8 = lt v7, v1
        //       jmpif v8 then: b5, else: b6
        //     b6():
        //       v16 = add v4, u32 1
        //       jmp b1(v16)
        //     b5():
        //       v10 = eq v7, u32 27
        //       v11 = not v10
        //       jmpif v11 then: b7, else: b8
        //     b7():
        //       v12 = load v3
        //       v13 = add v12, v6
        //       store v13 at v3
        //       jmp b8()
        //     b8():
        //       v15 = add v7, u32 1
        //       jmp b4(v15)
        //   }

        let main_id = Id::test_new(1);
        let mut builder = FunctionBuilder::new("main".into(), main_id);
        builder.set_runtime(RuntimeType::Brillig(InlineType::default()));

        let b1 = builder.insert_block();
        let b2 = builder.insert_block();
        let b3 = builder.insert_block();
        let b4 = builder.insert_block();
        let b5 = builder.insert_block();
        let b6 = builder.insert_block();
        let b7 = builder.insert_block();
        let b8 = builder.insert_block();

        let v0 = builder.add_parameter(Type::unsigned(32));
        let v1 = builder.add_parameter(Type::unsigned(32));

        let v3 = builder.insert_allocate(Type::unsigned(32));

        let zero = builder.numeric_constant(0u128, NumericType::Unsigned { bit_size: 32 });
        builder.insert_store(v3, zero);

        builder.terminate_with_jmp(b1, vec![zero]);

        builder.switch_to_block(b1);
        let v4 = builder.add_block_parameter(b1, Type::unsigned(32));

        let v5 = builder.insert_binary(v4, BinaryOp::Lt, v0);

        builder.terminate_with_jmpif(v5, b2, b3);

        builder.switch_to_block(b2);

        let v6 = builder.insert_binary(v4, BinaryOp::Mul { unchecked: false }, v4);

        builder.terminate_with_jmp(b4, vec![v0]);

        builder.switch_to_block(b4);

        let v7 = builder.add_block_parameter(b4, Type::unsigned(32));

        let v8 = builder.insert_binary(v7, BinaryOp::Lt, v1);

        builder.terminate_with_jmpif(v8, b5, b6);

        builder.switch_to_block(b5);

        let twenty_seven = builder.numeric_constant(27u128, NumericType::Unsigned { bit_size: 32 });
        let v10 = builder.insert_binary(v7, BinaryOp::Eq, twenty_seven);

        let v11 = builder.insert_not(v10);

        builder.terminate_with_jmpif(v11, b7, b8);

        builder.switch_to_block(b7);

        let v12 = builder.insert_load(v3, Type::unsigned(32));

        let v13 = builder.insert_binary(v12, BinaryOp::Add { unchecked: false }, v6);

        builder.insert_store(v3, v13);

        builder.terminate_with_jmp(b8, vec![]);

        builder.switch_to_block(b8);

        let one = builder.numeric_constant(1u128, NumericType::Unsigned { bit_size: 32 });
        let v15 = builder.insert_binary(v7, BinaryOp::Add { unchecked: false }, one);

        builder.terminate_with_jmp(b4, vec![v15]);

        builder.switch_to_block(b6);

        let v16 = builder.insert_binary(v4, BinaryOp::Add { unchecked: false }, one);

        builder.terminate_with_jmp(b1, vec![v16]);

        builder.switch_to_block(b3);

        let v17 = builder.insert_load(v3, Type::unsigned(32));

        builder.terminate_with_return(vec![v17]);

        let ssa = builder.finish();
        let func = ssa.main();

        let constants = ConstantAllocation::from_function(func);
        let liveness = VariableLiveness::from_function(func, &constants);

        assert!(liveness.get_live_in(&func.entry_block()).is_empty());
        assert_eq!(
            liveness.get_live_in(&b1),
            &FxHashSet::from_iter([v0, v1, v3, v4, twenty_seven, one].into_iter())
        );
        assert_eq!(liveness.get_live_in(&b3), &FxHashSet::from_iter([v3].into_iter()));
        assert_eq!(
            liveness.get_live_in(&b2),
            &FxHashSet::from_iter([v0, v1, v3, v4, twenty_seven, one].into_iter())
        );
        assert_eq!(
            liveness.get_live_in(&b4),
            &FxHashSet::from_iter([v0, v1, v3, v4, v6, v7, twenty_seven, one].into_iter())
        );
        assert_eq!(
            liveness.get_live_in(&b6),
            &FxHashSet::from_iter([v0, v1, v3, v4, twenty_seven, one].into_iter())
        );
        assert_eq!(
            liveness.get_live_in(&b5),
            &FxHashSet::from_iter([v0, v1, v3, v4, v6, v7, twenty_seven, one].into_iter())
        );
        assert_eq!(
            liveness.get_live_in(&b7),
            &FxHashSet::from_iter([v0, v1, v3, v4, v6, v7, twenty_seven, one].into_iter())
        );
        assert_eq!(
            liveness.get_live_in(&b8),
            &FxHashSet::from_iter([v0, v1, v3, v4, v6, v7, twenty_seven, one].into_iter())
        );

        let block_3 = &func.dfg[b3];
        assert_eq!(
            liveness.get_last_uses(&b3).get(&block_3.instructions()[0]),
            Some(&FxHashSet::from_iter([v3].into_iter()))
        );
    }

    #[test]
    fn block_params() {
        // brillig fn main f0 {
        //     b0(v0: u1):
        //       jmpif v0 then: b1, else: b2
        //     b1():
        //       jmp b3(Field 27, Field 29)
        //     b3(v1: Field, v2: Field):
        //       return v1
        //     b2():
        //       jmp b3(Field 28, Field 40)
        //   }

        let main_id = Id::test_new(1);
        let mut builder = FunctionBuilder::new("main".into(), main_id);
        builder.set_runtime(RuntimeType::Brillig(InlineType::default()));

        let v0 = builder.add_parameter(Type::bool());

        let b1 = builder.insert_block();
        let b2 = builder.insert_block();
        let b3 = builder.insert_block();

        builder.terminate_with_jmpif(v0, b1, b2);

        builder.switch_to_block(b1);
        let twenty_seven = builder.field_constant(27_u128);
        let twenty_nine = builder.field_constant(29_u128);
        builder.terminate_with_jmp(b3, vec![twenty_seven, twenty_nine]);

        builder.switch_to_block(b3);
        let v1 = builder.add_block_parameter(b3, Type::field());
        let v2 = builder.add_block_parameter(b3, Type::field());
        builder.terminate_with_return(vec![v1]);

        builder.switch_to_block(b2);
        let twenty_eight = builder.field_constant(28_u128);
        let forty = builder.field_constant(40_u128);
        builder.terminate_with_jmp(b3, vec![twenty_eight, forty]);

        let ssa = builder.finish();
        let func = ssa.main();

        let constants = ConstantAllocation::from_function(func);
        let liveness = VariableLiveness::from_function(func, &constants);

        // Entry point defines its own params and also b3's params.
        assert_eq!(liveness.defined_block_params(&func.entry_block()), vec![v0, v1, v2]);
        assert_eq!(liveness.defined_block_params(&b1), vec![]);
        assert_eq!(liveness.defined_block_params(&b2), vec![]);
        assert_eq!(liveness.defined_block_params(&b3), vec![]);

        assert_eq!(liveness.get_live_in(&b1), &FxHashSet::from_iter([v1, v2].into_iter()));
        assert_eq!(liveness.get_live_in(&b2), &FxHashSet::from_iter([v1, v2].into_iter()));
    }

    /// A block parameter should be considered used, even if it's not actually used in the SSA.
    #[test]
    fn unused_block_parameter() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u1):
            jmpif v1 then: b1, else: b2
          b1():
            v7 = add v0, u32 10
            jmp b3(v0, v7)
          b2():
            jmp b3(u32 1, u32 2)
          b3(v2: u32, v3: u32):
            return v3
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let func = ssa.main();
        let constants = ConstantAllocation::from_function(func);
        let liveness = VariableLiveness::from_function(func, &constants);

        let [b0, b1, b2, b3] = block_ids();
        let [_v0, _v1, v2, v3] = value_ids();

        // v2 and v3 are parameters of b3, but only v3 is used.
        for p in [v2, v3] {
            // Both should be allocated in b0, which is the immediate dominator of b3.
            assert!(liveness.param_definitions[&b0].contains(&p), "{p} should be allocated in b0");
            for b in [b1, b2, b3] {
                // Since they are defined in b0, they should be considered live all the way.
                assert!(liveness.live_in[&b].contains(&p), "{p} should be live in {b}");
            }
        }
    }

    fn value_ids<const N: usize>() -> [ValueId; N] {
        std::array::from_fn(|i| ValueId::new(i as u32))
    }

    fn block_ids<const N: usize>() -> [BasicBlockId; N] {
        std::array::from_fn(|i| BasicBlockId::new(i as u32))
    }

    #[test]
    fn test_entry_block_parameters() {
        // Entry block parameters are allocated in the entry block itself
        // but are not in the live-in set (no predecessor to pass them).
        // The entry block defines its own parameters via defined_block_params().
        //
        // SSA v0 (entry parameter) → Brillig sp[1]
        // SSA v1 (result of add) → Brillig sp[3] (line 2: result of add stored in sp[3])
        // Line 3: sp[1] = sp[3] moves v1 to return position
        let src = "
        brillig(inline) fn main f0 {
        b0(v0: Field):
            v1 = add v0, Field 10
            return v1
        }
        ";
        let brillig = ssa_to_brillig_artifacts(src);
        let main = &brillig.ssa_function_to_brillig[&Id::test_new(0)];

        assert_artifact_snapshot!(main, @r"
        fn main
        0: call 0
        1: sp[2] = const field 10
        2: sp[3] = field add sp[1], sp[2]
        3: sp[1] = sp[3]
        4: return
        ");
    }

    #[test]
    fn test_last_use_deallocation() {
        // When a variable reaches its last use, its register is deallocated
        // and can be reused for subsequent variables. This is tracked per-instruction
        // via get_last_uses().
        //
        // SSA v0 (entry parameter) → Brillig sp[1]
        // SSA v1 (v0 + 1) → Brillig sp[3] (line 2: result). v0 dies after this, sp[1] freed
        // SSA v2 (v1 + 2) → Brillig sp[2] (line 4: result, reuses sp[1]). v1 dies, sp[3] freed
        // SSA v3 (v2 + 3) → Brillig sp[3] (line 6: result, reuses freed sp[3]). v2 dies
        // Line 7: sp[1] = sp[3] moves v3 to return position
        // Register reuse: sp[1] freed after v0, reused for v2; sp[3] freed after v1, reused for v3
        let src = "
        brillig(inline) fn main f0 {
        b0(v0: Field):
            v1 = add v0, Field 1
            v2 = add v1, Field 2
            v3 = add v2, Field 3
            return v3
        }
        ";
        let brillig = ssa_to_brillig_artifacts(src);
        let main = &brillig.ssa_function_to_brillig[&Id::test_new(0)];

        assert_artifact_snapshot!(main, @r"
        fn main
        0: call 0
        1: sp[2] = const field 1
        2: sp[3] = field add sp[1], sp[2]
        3: sp[1] = const field 2
        4: sp[2] = field add sp[3], sp[1]
        5: sp[1] = const field 3
        6: sp[3] = field add sp[2], sp[1]
        7: sp[1] = sp[3]
        8: return
        ");
    }

    #[test]
    fn test_loop_liveness() {
        // Variables live in the loop header must remain alive throughout the loop body
        // because the back edge jumps back to the header.
        //
        // SSA v0 (loop bound) → Brillig sp[1]
        // SSA v1 (loop variable, b1's parameter) → Brillig sp[2] (allocated in b0)
        // Line 3: sp[2] = sp[3] initializes v1 to 0
        // Line 5 (b1 header): sp[3] = lt sp[2], sp[1] tests v1 < v0
        // Line 10 (b2 body): sp[3] = add sp[2], sp[4] computes v1 + 1
        // Line 14: sp[2] = sp[3] updates v1 for next iteration
        // Line 15: jump back to b1
        // v0 (sp[1]) stays alive throughout loop (used at line 5 each iteration)
        // Back edge at line 15 ensures b1's live-in includes both v0 and v1
        let src = "
        brillig(inline) fn main f0 {
        b0(v0: u32):
            jmp b1(u32 0)
        b1(v1: u32):
            v2 = lt v1, v0
            jmpif v2 then: b2, else: b3
        b2():
            v3 = add v1, u32 1
            jmp b1(v3)
        b3():
            return v1
        }
        ";
        let brillig = ssa_to_brillig_artifacts(src);
        let main = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
        assert_artifact_snapshot!(main, @r"
        fn main
         0: call 0
         1: sp[3] = const u32 0
         2: sp[4] = const u32 1
         3: sp[2] = sp[3]
         4: jump to 0
         5: sp[3] = u32 lt sp[2], sp[1]
         6: jump if sp[3] to 0
         7: jump to 0
         8: sp[1] = sp[2]
         9: return
        10: sp[3] = u32 add sp[2], sp[4]
        11: sp[5] = u32 lt_eq sp[2], sp[3]
        12: jump if sp[5] to 0
        13: call 0
        14: sp[2] = sp[3]
        15: jump to 0
        ");
    }

    #[test]
    fn test_block_parameters() {
        // Multiple predecessors pass different values (42 and v2) to the same parameter.
        // The parameter must be allocated in the immediate dominator (v1: allocated to sp[2] in b0),
        // and each predecessor generates a mov to that register.
        //
        // SSA v0 (condition) → Brillig sp[1]
        // SSA v1 (b3's parameter) → Brillig sp[2] (allocated in b0, dominator of b3)
        // SSA v2 (27 + 42) → Brillig sp[4] (line 7: result of add)
        // Field 42 constant → sp[3] (line 1)
        // Line 2: jmpif sp[1] branches to b1 or b2
        // Line 4 (b2 path): sp[2] = sp[3] moves constant 42 into v1's register
        // Line 8 (b1 path): sp[2] = sp[4] moves v2 into v1's register
        // Line 10 (b3): sp[1] = sp[2] prepares return
        let src = "
        brillig(inline) fn main f0 {
        b0(v0: u1):
            jmpif v0 then: b1, else: b2
        b1():
            v2 = add Field 27, Field 42
            jmp b3(v2)
        b2():
            jmp b3(Field 42)
        b3(v1: Field):
            return v1
        }
        ";
        let brillig = ssa_to_brillig_artifacts(src);
        let main = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
        assert_artifact_snapshot!(main, @r"
        fn main
         0: call 0
         1: sp[3] = const field 42
         2: jump if sp[1] to 0
         3: jump to 0
         4: sp[2] = sp[3]
         5: jump to 0
         6: sp[1] = const field 27
         7: sp[4] = field add sp[1], sp[3]
         8: sp[2] = sp[4]
         9: jump to 0
        10: sp[1] = sp[2]
        11: return
        ");
    }

    #[test]
    fn test_constants_liveness() {
        // Constant SSA values are also included in liveness analysis
        // Only hoisted global constants are filtered out.
        //
        // SSA v0 (entry parameter) → Brillig sp[1]
        // SSA Field 100 constant → Brillig sp[2] (line 1: allocated on-demand via constant_allocation)
        // SSA v1 (v0 + 100) → Brillig sp[3] (line 2: result)
        // After line 2: Field 100 reaches its LAST USE, sp[2] is deallocated
        // SSA Field 200 constant → Brillig sp[2] (line 3: REUSES sp[2])
        // SSA v2 (v0 * 200) → Brillig sp[4] (line 4: result)
        // SSA v3 (v1 + v2) → Brillig sp[1] (line 5: result, reuses sp[1] after v0 dies)
        //
        let src = "
        brillig(inline) fn main f0 {
        b0(v0: Field):
            v1 = add v0, Field 100
            v2 = mul v0, Field 200
            v3 = add v1, v2
            return v3
        }
        ";
        let brillig = ssa_to_brillig_artifacts(src);
        let main = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
        assert_artifact_snapshot!(main, @r"
        fn main
        0: call 0
        1: sp[2] = const field 100
        2: sp[3] = field add sp[1], sp[2]
        3: sp[2] = const field 200
        4: sp[4] = field mul sp[1], sp[2]
        5: sp[1] = field add sp[3], sp[4]
        6: return
        ");
    }

    #[test]
    fn test_terminator_arguments_stay_alive() {
        // Arguments to terminator instructions must remain alive until the terminator executes.
        // They cannot be deallocated in the last instruction of the block.
        //
        // SSA v0 (entry parameter) → Brillig sp[1]
        // SSA v1 (v0 + 1) → Brillig sp[4] (line 2: result)
        // SSA v2 (v1 + 2) → Brillig sp[3] (line 4: result)
        // SSA v3 (v2 * 3) → Brillig sp[4] (line 6: result, last instruction before terminator)
        // SSA v4 (b1's parameter) → Brillig sp[2] (allocated in b0)
        // Line 7: sp[2] = sp[4] copies v3 into v4's register (terminator argument)
        // Line 8: jump to b1
        // v3 cannot be marked as dead at line 6 because it's used in terminator at line 7
        // This ensures v3's register (sp[4]) stays valid throughout the copy instruction
        let src = "
        brillig(inline) fn main f0 {
        b0(v0: Field):
            v1 = add v0, Field 1
            v2 = add v1, Field 2
            v3 = mul v2, Field 3
            jmp b1(v3)
        b1(v4: Field):
            return v4
        }
        ";
        let brillig = ssa_to_brillig_artifacts(src);
        let main = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
        assert_artifact_snapshot!(main, @r"
        fn main
         0: call 0
         1: sp[3] = const field 1
         2: sp[4] = field add sp[1], sp[3]
         3: sp[1] = const field 2
         4: sp[3] = field add sp[4], sp[1]
         5: sp[1] = const field 3
         6: sp[4] = field mul sp[3], sp[1]
         7: sp[2] = sp[4]
         8: jump to 0
         9: sp[1] = sp[2]
        10: return
        ");
    }
}
