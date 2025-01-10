//! This module analyzes the liveness of variables (non-constant values) throughout a function.
//! It uses the approach detailed in the section 4.2 of this paper https://inria.hal.science/inria-00558509v2/document

use crate::ssa::ir::{
    basic_block::{BasicBlock, BasicBlockId},
    cfg::ControlFlowGraph,
    dfg::DataFlowGraph,
    dom::DominatorTree,
    function::Function,
    instruction::{Instruction, InstructionId},
    post_order::PostOrder,
    value::{Value, ValueId},
};

use fxhash::{FxHashMap as HashMap, FxHashSet as HashSet};

use super::constant_allocation::ConstantAllocation;

/// A back edge is an edge from a node to one of its ancestors. It denotes a loop in the CFG.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct BackEdge {
    header: BasicBlockId,
    start: BasicBlockId,
}

fn find_back_edges(
    func: &Function,
    cfg: &ControlFlowGraph,
    post_order: &PostOrder,
) -> HashSet<BackEdge> {
    let mut tree = DominatorTree::with_cfg_and_post_order(cfg, post_order);
    let mut back_edges = HashSet::default();

    for block_id in func.reachable_blocks() {
        let block = &func.dfg[block_id];
        let successors = block.successors();
        for successor_id in successors {
            if tree.dominates(successor_id, block_id) {
                back_edges.insert(BackEdge { start: block_id, header: successor_id });
            }
        }
    }

    back_edges
}

/// Collects the underlying variables inside a value id. It might be more than one, for example in constant arrays that are constructed with multiple vars.
pub(crate) fn collect_variables_of_value(
    value_id: ValueId,
    dfg: &DataFlowGraph,
) -> Option<ValueId> {
    let value_id = dfg.resolve(value_id);
    let value = &dfg[value_id];

    match value {
        Value::Instruction { .. }
        | Value::Param { .. }
        | Value::NumericConstant { .. }
        | Value::Global(_) => Some(value_id),
        // Functions are not variables in a defunctionalized SSA. Only constant function values should appear.
        Value::ForeignFunction(_) | Value::Function(_) | Value::Intrinsic(..) => None,
    }
}

pub(crate) fn variables_used_in_instruction(
    instruction: &Instruction,
    dfg: &DataFlowGraph,
) -> Variables {
    let mut used = HashSet::default();

    instruction.for_each_value(|value_id| {
        let underlying_ids = collect_variables_of_value(value_id, dfg);
        used.extend(underlying_ids);
    });

    used
}

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
            used.extend(collect_variables_of_value(value_id, dfg));
        });
    }

    used
}

type Variables = HashSet<ValueId>;

fn compute_used_before_def(
    block: &BasicBlock,
    dfg: &DataFlowGraph,
    defined_in_block: &Variables,
) -> Variables {
    variables_used_in_block(block, dfg)
        .into_iter()
        .filter(|id| !defined_in_block.contains(id))
        .collect()
}

type LastUses = HashMap<InstructionId, Variables>;

/// A struct representing the liveness of variables throughout a function.
pub(crate) struct VariableLiveness {
    cfg: ControlFlowGraph,
    post_order: PostOrder,
    dominator_tree: DominatorTree,
    /// The variables that are alive before the block starts executing
    live_in: HashMap<BasicBlockId, Variables>,
    /// The variables that stop being alive after each specific instruction
    last_uses: HashMap<BasicBlockId, LastUses>,
    /// The list of block params the given block is defining. The order matters for the entry block, so it's a vec.
    param_definitions: HashMap<BasicBlockId, Vec<ValueId>>,
}

impl VariableLiveness {
    /// Computes the liveness of variables throughout a function.
    pub(crate) fn from_function(func: &Function, constants: &ConstantAllocation) -> Self {
        let cfg = ControlFlowGraph::with_function(func);
        let post_order = PostOrder::with_function(func);
        let dominator_tree = DominatorTree::with_cfg_and_post_order(&cfg, &post_order);

        let mut instance = Self {
            cfg,
            post_order,
            dominator_tree,
            live_in: HashMap::default(),
            last_uses: HashMap::default(),
            param_definitions: HashMap::default(),
        };

        instance.compute_block_param_definitions(func);

        instance.compute_live_in_of_blocks(func, constants);

        instance.compute_last_uses(func);

        instance
    }

    /// The set of values that are alive before the block starts executing
    pub(crate) fn get_live_in(&self, block_id: &BasicBlockId) -> &Variables {
        self.live_in.get(block_id).expect("Live ins should have been calculated")
    }

    /// The set of values that are alive after the block has finished executed
    pub(crate) fn get_live_out(&self, block_id: &BasicBlockId) -> Variables {
        let mut live_out = HashSet::default();
        for successor_id in self.cfg.successors(*block_id) {
            live_out.extend(self.get_live_in(&successor_id));
        }
        live_out
    }

    /// A map of instruction id to the set of values that die after the instruction has executed
    pub(crate) fn get_last_uses(&self, block_id: &BasicBlockId) -> &LastUses {
        self.last_uses.get(block_id).expect("Last uses should have been calculated")
    }

    /// Retrieves the list of block params the given block is defining.
    /// Block params are defined before the block that owns them (since they are used by the predecessor blocks). They must be defined in the immediate dominator.
    /// This is the last point where the block param can be allocated without it being allocated in different places in different branches.
    pub(crate) fn defined_block_params(&self, block_id: &BasicBlockId) -> Vec<ValueId> {
        self.param_definitions.get(block_id).cloned().unwrap_or_default()
    }

    fn compute_block_param_definitions(&mut self, func: &Function) {
        // Going in reverse post order to process the entry block first
        let mut reverse_post_order = Vec::new();
        reverse_post_order.extend_from_slice(self.post_order.as_slice());
        reverse_post_order.reverse();
        for block in reverse_post_order {
            let params = func.dfg[block].parameters();
            // If it has no dominator, it's the entry block
            let dominator_block =
                self.dominator_tree.immediate_dominator(block).unwrap_or(func.entry_block());
            let definitions_for_the_dominator =
                self.param_definitions.entry(dominator_block).or_default();
            definitions_for_the_dominator.extend(params.iter());
        }
    }

    fn compute_live_in_of_blocks(&mut self, func: &Function, constants: &ConstantAllocation) {
        let back_edges = find_back_edges(func, &self.cfg, &self.post_order);

        // First pass, propagate up the live_ins skipping back edges
        self.compute_live_in_recursive(func, func.entry_block(), &back_edges, constants);

        // Second pass, propagate header live_ins to the loop bodies
        for back_edge in back_edges {
            self.update_live_ins_within_loop(back_edge);
        }
    }

    fn compute_live_in_recursive(
        &mut self,
        func: &Function,
        block_id: BasicBlockId,
        back_edges: &HashSet<BackEdge>,
        constants: &ConstantAllocation,
    ) {
        let mut defined = self.compute_defined_variables(block_id, &func.dfg);

        defined.extend(constants.allocated_in_block(block_id));

        let block: &BasicBlock = &func.dfg[block_id];

        let used_before_def = compute_used_before_def(block, &func.dfg, &defined);

        let mut live_out = HashSet::default();

        for successor_id in block.successors() {
            if !back_edges.contains(&BackEdge { start: block_id, header: successor_id }) {
                if !self.live_in.contains_key(&successor_id) {
                    self.compute_live_in_recursive(func, successor_id, back_edges, constants);
                }
                live_out.extend(
                    self.live_in
                        .get(&successor_id)
                        .expect("Live ins for successor should have been calculated"),
                );
            }
        }

        // live_in[BlockId] = before_def[BlockId] union (live_out[BlockId] - killed[BlockId])
        let passthrough_vars = live_out.difference(&defined).cloned().collect();
        self.live_in.insert(block_id, used_before_def.union(&passthrough_vars).cloned().collect());
    }

    fn compute_defined_variables(&self, block_id: BasicBlockId, dfg: &DataFlowGraph) -> Variables {
        let block: &BasicBlock = &dfg[block_id];
        let mut defined_vars = HashSet::default();

        for parameter in self.defined_block_params(&block_id) {
            defined_vars.insert(dfg.resolve(parameter));
        }

        for instruction_id in block.instructions() {
            let result_values = dfg.instruction_results(*instruction_id);
            for result_value in result_values {
                defined_vars.insert(dfg.resolve(*result_value));
            }
        }

        defined_vars
    }

    fn update_live_ins_within_loop(&mut self, back_edge: BackEdge) {
        let header_live_ins = self
            .live_in
            .get(&back_edge.header)
            .expect("Live ins should have been calculated")
            .clone();

        let body = self.compute_loop_body(back_edge);
        for body_block_id in body {
            self.live_in
                .get_mut(&body_block_id)
                .expect("Live ins should have been calculated")
                .extend(&header_live_ins);
        }
    }

    fn compute_loop_body(&self, edge: BackEdge) -> HashSet<BasicBlockId> {
        let mut loop_blocks = HashSet::default();
        loop_blocks.insert(edge.header);
        loop_blocks.insert(edge.start);

        let mut stack = vec![edge.start];

        while let Some(block) = stack.pop() {
            for predecessor in self.cfg.predecessors(block) {
                if !loop_blocks.contains(&predecessor) {
                    loop_blocks.insert(predecessor);
                    stack.push(predecessor);
                }
            }
        }

        loop_blocks
    }

    fn compute_last_uses(&mut self, func: &Function) {
        for block_id in func.reachable_blocks() {
            let block = &func.dfg[block_id];
            let live_out = self.get_live_out(&block_id);

            let mut used_after: Variables = Default::default();
            let mut block_last_uses: LastUses = Default::default();

            // First, handle the terminator
            if let Some(terminator_instruction) = block.terminator() {
                terminator_instruction.for_each_value(|value_id| {
                    let underlying_vars = collect_variables_of_value(value_id, &func.dfg);
                    used_after.extend(underlying_vars);
                });
            }

            // Then, handle the instructions in reverse order to find the last use
            for instruction_id in block.instructions().iter().rev() {
                let instruction = &func.dfg[*instruction_id];
                let instruction_last_uses = variables_used_in_instruction(instruction, &func.dfg)
                    .into_iter()
                    .filter(|id| !used_after.contains(id) && !live_out.contains(id))
                    .collect();

                used_after.extend(&instruction_last_uses);
                block_last_uses.insert(*instruction_id, instruction_last_uses);
            }

            self.last_uses.insert(block_id, block_last_uses);
        }
    }
}

#[cfg(test)]
mod test {
    use fxhash::FxHashSet;
    use noirc_frontend::monomorphization::ast::InlineType;

    use crate::brillig::brillig_gen::constant_allocation::ConstantAllocation;
    use crate::brillig::brillig_gen::variable_liveness::VariableLiveness;
    use crate::ssa::function_builder::FunctionBuilder;
    use crate::ssa::ir::function::RuntimeType;
    use crate::ssa::ir::instruction::BinaryOp;
    use crate::ssa::ir::map::Id;
    use crate::ssa::ir::types::Type;

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
        //     b0(v0: Field, v1: Field):
        //       v3 = allocate
        //       store Field 0 at v3
        //       jmp b1(Field 0)
        //     b1(v4: Field):
        //       v5 = lt v4, v0
        //       jmpif v5 then: b2, else: b3
        //     b3():
        //       v17 = load v3
        //       return v17
        //     b2():
        //       v6 = mul v4, v4
        //       jmp b4(v0)
        //     b4(v7: Field):
        //       v8 = lt v7, v1
        //       jmpif v8 then: b5, else: b6
        //     b6():
        //       v16 = add v4, Field 1
        //       jmp b1(v16)
        //     b5():
        //       v10 = eq v7, Field 27
        //       v11 = not v10
        //       jmpif v11 then: b7, else: b8
        //     b7():
        //       v12 = load v3
        //       v13 = add v12, v6
        //       store v13 at v3
        //       jmp b8()
        //     b8():
        //       v15 = add v7, Field 1
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

        let v0 = builder.add_parameter(Type::field());
        let v1 = builder.add_parameter(Type::field());

        let v3 = builder.insert_allocate(Type::field());

        let zero = builder.field_constant(0u128);
        builder.insert_store(v3, zero);

        builder.terminate_with_jmp(b1, vec![zero]);

        builder.switch_to_block(b1);
        let v4 = builder.add_block_parameter(b1, Type::field());

        let v5 = builder.insert_binary(v4, BinaryOp::Lt, v0);

        builder.terminate_with_jmpif(v5, b2, b3);

        builder.switch_to_block(b2);

        let v6 = builder.insert_binary(v4, BinaryOp::Mul { unchecked: false }, v4);

        builder.terminate_with_jmp(b4, vec![v0]);

        builder.switch_to_block(b4);

        let v7 = builder.add_block_parameter(b4, Type::field());

        let v8 = builder.insert_binary(v7, BinaryOp::Lt, v1);

        builder.terminate_with_jmpif(v8, b5, b6);

        builder.switch_to_block(b5);

        let twenty_seven = builder.field_constant(27u128);
        let v10 = builder.insert_binary(v7, BinaryOp::Eq, twenty_seven);

        let v11 = builder.insert_not(v10);

        builder.terminate_with_jmpif(v11, b7, b8);

        builder.switch_to_block(b7);

        let v12 = builder.insert_load(v3, Type::field());

        let v13 = builder.insert_binary(v12, BinaryOp::Add { unchecked: false }, v6);

        builder.insert_store(v3, v13);

        builder.terminate_with_jmp(b8, vec![]);

        builder.switch_to_block(b8);

        let one = builder.field_constant(1u128);
        let v15 = builder.insert_binary(v7, BinaryOp::Add { unchecked: false }, one);

        builder.terminate_with_jmp(b4, vec![v15]);

        builder.switch_to_block(b6);

        let v16 = builder.insert_binary(v4, BinaryOp::Add { unchecked: false }, one);

        builder.terminate_with_jmp(b1, vec![v16]);

        builder.switch_to_block(b3);

        let v17 = builder.insert_load(v3, Type::field());

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
}
