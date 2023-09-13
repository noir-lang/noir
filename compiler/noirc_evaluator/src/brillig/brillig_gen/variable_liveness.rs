use std::collections::HashSet;

use im::HashMap;

use crate::ssa::ir::{
    basic_block::{BasicBlock, BasicBlockId},
    cfg::ControlFlowGraph,
    dfg::DataFlowGraph,
    dom::DominatorTree,
    function::Function,
    post_order::PostOrder,
    value::{Value, ValueId},
};

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
    let mut back_edges = HashSet::new();

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

pub(crate) fn compute_defined_variables(
    block: &BasicBlock,
    dfg: &DataFlowGraph,
) -> HashSet<ValueId> {
    let mut defined_vars = HashSet::new();

    for parameter in block.parameters() {
        defined_vars.insert(dfg.resolve(*parameter));
    }

    for instruction_id in block.instructions() {
        let result_values = dfg.instruction_results(*instruction_id);
        for result_value in result_values {
            defined_vars.insert(dfg.resolve(*result_value));
        }
    }

    defined_vars
}

fn compute_before_def(
    block: &BasicBlock,
    dfg: &DataFlowGraph,
    defined_in_block: &HashSet<ValueId>,
) -> HashSet<ValueId> {
    let mut before_def = HashSet::new();

    fn process_value(
        value_id: ValueId,
        dfg: &DataFlowGraph,
        before_def: &mut HashSet<ValueId>,
        defined_in_this_block: &HashSet<ValueId>,
    ) {
        let value_id = dfg.resolve(value_id);
        let value = &dfg[value_id];
        match value {
            Value::Instruction { .. } | Value::Param { .. } => {
                if !defined_in_this_block.contains(&value_id) {
                    before_def.insert(value_id);
                }
            }
            Value::Array { array, .. } => {
                array.iter().for_each(|item_id| {
                    process_value(*item_id, dfg, before_def, defined_in_this_block);
                });
            }
            _ => {}
        }
    }

    for instruction_id in block.instructions() {
        let instruction = &dfg[*instruction_id];
        instruction.for_each_value(|value_id| {
            process_value(value_id, dfg, &mut before_def, defined_in_block);
        });
    }

    if let Some(terminator) = block.terminator() {
        terminator.for_each_value(|value_id| {
            process_value(value_id, dfg, &mut before_def, defined_in_block);
        });
    }

    before_def
}

pub(crate) struct VariableLiveness {
    cfg: ControlFlowGraph,
    post_order: PostOrder,
    live_in: HashMap<BasicBlockId, HashSet<ValueId>>,
}

impl VariableLiveness {
    pub(crate) fn from_function(func: &Function) -> Self {
        let cfg = ControlFlowGraph::with_function(func);
        let post_order = PostOrder::with_function(func);

        let mut instance = Self { cfg, post_order, live_in: HashMap::new() };

        instance.compute_live_in_of_blocks(func);

        instance
    }

    pub(crate) fn get_live_in(&self, block_id: &BasicBlockId) -> &HashSet<ValueId> {
        self.live_in.get(block_id).expect("Live ins should have been calculated")
    }

    pub(crate) fn get_live_out(&self, block_id: &BasicBlockId) -> HashSet<ValueId> {
        let mut live_out = HashSet::new();
        for successor_id in self.cfg.successors(*block_id) {
            live_out.extend(self.get_live_in(&successor_id));
        }
        live_out
    }

    fn compute_live_in_recursive(
        &mut self,
        func: &Function,
        block_id: BasicBlockId,
        back_edges: &HashSet<BackEdge>,
    ) {
        let block = &func.dfg[block_id];

        let defined = compute_defined_variables(block, &func.dfg);
        let before_def = compute_before_def(block, &func.dfg, &defined);

        let mut live_out = HashSet::new();

        for successor_id in block.successors() {
            if !back_edges.contains(&BackEdge { start: block_id, header: successor_id }) {
                if !self.live_in.contains_key(&successor_id) {
                    self.compute_live_in_recursive(func, successor_id, back_edges);
                }
                live_out.extend(
                    self.live_in
                        .get(&successor_id)
                        .expect("Live ins for successor should have been calculated"),
                );
            }
        }

        // live_in[BlockId] = before_def[BlockId] union (live_out[BlockId] - killed[BlockId])
        let difference = live_out.difference(&defined).cloned().collect();
        self.live_in.insert(block_id, before_def.union(&difference).cloned().collect());
    }

    fn compute_loop_body(&self, edge: BackEdge) -> HashSet<BasicBlockId> {
        let mut loop_blocks = HashSet::new();
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

    fn compute_live_in_of_blocks(&mut self, func: &Function) {
        let back_edges = find_back_edges(func, &self.cfg, &self.post_order);

        // First pass, propagate up the live_ins skipping back edges
        self.compute_live_in_recursive(func, func.entry_block(), &back_edges);

        // Second pass, propagate header live_ins to the loop bodies
        for back_edge in back_edges {
            self.update_live_ins_within_loop(back_edge);
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

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
        let mut builder = FunctionBuilder::new("main".into(), main_id, RuntimeType::Brillig);

        let b1 = builder.insert_block();
        let b2 = builder.insert_block();
        let b3 = builder.insert_block();

        let v0 = builder.add_parameter(Type::field());
        let v1 = builder.add_parameter(Type::field());

        let v3 = builder.insert_allocate();

        let zero = builder.numeric_constant(0u128, Type::field());
        builder.insert_store(v3, zero);

        let v4 = builder.insert_binary(v0, BinaryOp::Eq, zero);

        builder.terminate_with_jmpif(v4, b1, b2);

        builder.switch_to_block(b2);

        let twenty_seven = builder.numeric_constant(27u128, Type::field());
        let v7 = builder.insert_binary(v0, BinaryOp::Add, twenty_seven);
        builder.insert_store(v3, v7);

        builder.terminate_with_jmp(b3, vec![]);

        builder.switch_to_block(b1);

        let v6 = builder.insert_binary(v1, BinaryOp::Add, twenty_seven);
        builder.insert_store(v3, v6);

        builder.terminate_with_jmp(b3, vec![]);

        builder.switch_to_block(b3);

        let v8 = builder.insert_load(v3, Type::field());

        builder.terminate_with_return(vec![v8]);

        let ssa = builder.finish();
        let func = ssa.main();
        let liveness = VariableLiveness::from_function(func);

        assert!(liveness.get_live_in(&func.entry_block()).is_empty());
        assert_eq!(liveness.get_live_in(&b2), &HashSet::from([v3, v0]));
        assert_eq!(liveness.get_live_in(&b1), &HashSet::from([v3, v1]));
        assert_eq!(liveness.get_live_in(&b3), &HashSet::from([v3]));
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
        let mut builder = FunctionBuilder::new("main".into(), main_id, RuntimeType::Brillig);

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

        let v3 = builder.insert_allocate();

        let zero = builder.numeric_constant(0u128, Type::field());
        builder.insert_store(v3, zero);

        builder.terminate_with_jmp(b1, vec![zero]);

        builder.switch_to_block(b1);
        let v4 = builder.add_block_parameter(b1, Type::field());

        let v5 = builder.insert_binary(v4, BinaryOp::Lt, v0);

        builder.terminate_with_jmpif(v5, b2, b3);

        builder.switch_to_block(b2);

        let v6 = builder.insert_binary(v4, BinaryOp::Mul, v4);

        builder.terminate_with_jmp(b4, vec![v0]);

        builder.switch_to_block(b4);

        let v7 = builder.add_block_parameter(b4, Type::field());

        let v8 = builder.insert_binary(v7, BinaryOp::Lt, v1);

        builder.terminate_with_jmpif(v8, b5, b6);

        builder.switch_to_block(b5);

        let twenty_seven = builder.numeric_constant(27u128, Type::field());
        let v10 = builder.insert_binary(v7, BinaryOp::Eq, twenty_seven);

        let v11 = builder.insert_not(v10);

        builder.terminate_with_jmpif(v11, b7, b8);

        builder.switch_to_block(b7);

        let v12 = builder.insert_load(v3, Type::field());

        let v13 = builder.insert_binary(v12, BinaryOp::Add, v6);

        builder.insert_store(v3, v13);

        builder.terminate_with_jmp(b8, vec![]);

        builder.switch_to_block(b8);

        let one = builder.numeric_constant(1u128, Type::field());
        let v15 = builder.insert_binary(v7, BinaryOp::Add, one);

        builder.terminate_with_jmp(b4, vec![v15]);

        builder.switch_to_block(b6);

        let v16 = builder.insert_binary(v4, BinaryOp::Add, one);

        builder.terminate_with_jmp(b1, vec![v16]);

        builder.switch_to_block(b3);

        let v17 = builder.insert_load(v3, Type::field());

        builder.terminate_with_return(vec![v17]);

        let ssa = builder.finish();
        let func = ssa.main();

        let liveness = VariableLiveness::from_function(func);

        assert!(liveness.get_live_in(&func.entry_block()).is_empty());
        assert_eq!(liveness.get_live_in(&b1), &HashSet::from([v0, v1, v3]));
        assert_eq!(liveness.get_live_in(&b3), &HashSet::from([v3]));
        assert_eq!(liveness.get_live_in(&b2), &HashSet::from([v0, v1, v3, v4]));
        assert_eq!(liveness.get_live_in(&b4), &HashSet::from([v0, v1, v3, v4, v6]));
        assert_eq!(liveness.get_live_in(&b6), &HashSet::from([v0, v1, v3, v4]));
        assert_eq!(liveness.get_live_in(&b5), &HashSet::from([v0, v1, v3, v4, v6, v7]));
        assert_eq!(liveness.get_live_in(&b7), &HashSet::from([v0, v1, v3, v4, v6, v7]));
        assert_eq!(liveness.get_live_in(&b8), &HashSet::from([v0, v1, v3, v4, v6, v7]));
    }
}
