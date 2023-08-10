//! mem2reg implements a pass for promoting values stored in memory to values in registers where
//! possible. This is particularly important for converting our memory-based representation of
//! mutable variables into values that are easier to manipulate.
use std::collections::{BTreeMap, HashSet};

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        cfg::ControlFlowGraph,
        dfg::DataFlowGraph,
        dom::DominatorTree,
        function::Function,
        instruction::{Instruction, InstructionId, TerminatorInstruction},
        post_order::PostOrder,
        value::{Value, ValueId},
    },
    ssa_gen::Ssa,
};

use super::unrolling::{find_all_loops, Loops};

impl Ssa {
    /// Attempts to remove any load instructions that recover values that are already available in
    /// scope, and attempts to remove stores that are subsequently redundant.
    /// As long as they are not stores on memory used inside of loops
    pub(crate) fn mem2reg(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            let mut all_protected_allocations = HashSet::new();

            let mut context = PerFunctionContext::new(function);

            for block in function.reachable_blocks() {
                // Maps Load instruction id -> value to replace the result of the load with
                let mut loads_to_substitute_per_block = BTreeMap::new();

                // Maps Load result id -> value to replace the result of the load with
                let mut load_values_to_substitute = BTreeMap::new();

                let allocations_protected_by_block = context
                    .analyze_allocations_and_eliminate_known_loads(
                        &mut function.dfg,
                        &mut loads_to_substitute_per_block,
                        &mut load_values_to_substitute,
                        block,
                    );
                all_protected_allocations.extend(allocations_protected_by_block.into_iter());
            }

            // Now that we have a comprehensive list of used allocations across all the
            // function's blocks, it is safe to remove any stores that do not touch such
            // allocations.
            for block in function.reachable_blocks() {
                context.remove_unused_stores(&mut function.dfg, &all_protected_allocations, block);
            }
        }

        self
    }
}

struct PerFunctionContext {
    last_stores_with_block: BTreeMap<(AllocId, BasicBlockId), ValueId>,
    // Maps Load result id -> (value, block_id)
    // Used to replace the result of a load with the appropriate block
    load_values_to_substitute_per_func: BTreeMap<ValueId, (ValueId, BasicBlockId)>,
    store_ids: Vec<InstructionId>,
    cfg: ControlFlowGraph,
    post_order: PostOrder,
    loops: Loops,
}
impl PerFunctionContext {
    fn new(function: &Function) -> Self {
        PerFunctionContext {
            last_stores_with_block: BTreeMap::new(),
            load_values_to_substitute_per_func: BTreeMap::new(),
            store_ids: Vec::new(),
            cfg: ControlFlowGraph::with_function(function),
            post_order: PostOrder::with_function(function),
            loops: find_all_loops(function),
        }
    }
}

/// An AllocId is the ValueId returned from an allocate instruction. E.g. v0 in v0 = allocate.
/// This type alias is used to help signal where the only valid ValueIds are those that are from
/// an allocate instruction.
type AllocId = ValueId;

impl PerFunctionContext {
    // Attempts to remove load instructions for which the result is already known from previous
    // store instructions to the same address in the same block.
    fn analyze_allocations_and_eliminate_known_loads(
        &mut self,
        dfg: &mut DataFlowGraph,
        loads_to_substitute: &mut BTreeMap<InstructionId, ValueId>,
        load_values_to_substitute_per_block: &mut BTreeMap<ValueId, ValueId>,
        block_id: BasicBlockId,
    ) -> HashSet<AllocId> {
        let mut protected_allocations = HashSet::new();
        let block = &dfg[block_id];

        for instruction_id in block.instructions() {
            match &dfg[*instruction_id] {
                Instruction::Store { mut address, value } => {
                    address = self.fetch_load_value_to_substitute(block_id, address);

                    self.last_stores_with_block.insert((address, block_id), *value);
                    self.store_ids.push(*instruction_id);
                }
                Instruction::Load { mut address } => {
                    address = self.fetch_load_value_to_substitute(block_id, address);

                    let found_last_value = self.find_load_to_substitute(
                        block_id,
                        address,
                        dfg,
                        instruction_id,
                        loads_to_substitute,
                        load_values_to_substitute_per_block,
                    );
                    if !found_last_value {
                        protected_allocations.insert(address);
                    }
                }
                Instruction::Call { arguments, .. } => {
                    for arg in arguments {
                        if Self::value_is_from_allocation(*arg, dfg) {
                            protected_allocations.insert(*arg);
                        }
                    }
                }
                _ => {
                    // Nothing to do
                }
            }
        }

        // Identify any arrays that are returned from this function
        if let TerminatorInstruction::Return { return_values } = block.unwrap_terminator() {
            for value in return_values {
                if Self::value_is_from_allocation(*value, dfg) {
                    protected_allocations.insert(*value);
                }
            }
        }

        // Substitute load result values
        for (result_value, new_value) in load_values_to_substitute_per_block {
            let result_value = dfg.resolve(*result_value);
            dfg.set_value_from_id(result_value, *new_value);
        }

        // Delete load instructions
        // Even though we could let DIE handle this, doing it here makes the debug output easier
        // to read.
        dfg[block_id]
            .instructions_mut()
            .retain(|instruction| !loads_to_substitute.contains_key(instruction));

        protected_allocations
    }

    fn fetch_load_value_to_substitute(&self, block_id: BasicBlockId, address: ValueId) -> ValueId {
        let mut stack = vec![block_id];
        let mut visited = HashSet::new();

        let mut dom_tree = DominatorTree::with_cfg_and_post_order(&self.cfg, &self.post_order);
        while let Some(block) = stack.pop() {
            visited.insert(block);

            if let Some((value, load_block_id)) =
                self.load_values_to_substitute_per_func.get(&address)
            {
                if *load_block_id == block {
                    return *value;
                }
            }

            let predecessors = self.cfg.predecessors(block);
            for predecessor in predecessors {
                if dom_tree.is_reachable(predecessor)
                    && dom_tree.dominates(predecessor, block)
                    && !visited.contains(&predecessor)
                {
                    stack.push(predecessor);
                }
            }
        }
        address
    }

    fn find_load_to_substitute(
        &mut self,
        block_id: BasicBlockId,
        address: ValueId,
        dfg: &DataFlowGraph,
        instruction_id: &InstructionId,
        loads_to_substitute: &mut BTreeMap<InstructionId, ValueId>,
        load_values_to_substitute_per_block: &mut BTreeMap<ValueId, ValueId>,
    ) -> bool {
        let mut stack = vec![block_id];
        let mut visited = HashSet::new();

        let mut dom_tree = DominatorTree::with_cfg_and_post_order(&self.cfg, &self.post_order);
        while let Some(block) = stack.pop() {
            visited.insert(block);

            for l in self.loops.yet_to_unroll.iter() {
                // We do not want to substitute loads that take place within loops as this pass
                // can occur before loop unrolling
                if block == l.header {
                    return false;
                }
            }

            if let Some(last_value) = self.last_stores_with_block.get(&(address, block)) {
                let result_value = *dfg
                    .instruction_results(*instruction_id)
                    .first()
                    .expect("ICE: Load instructions should have single result");

                loads_to_substitute.insert(*instruction_id, *last_value);
                load_values_to_substitute_per_block.insert(result_value, *last_value);
                self.load_values_to_substitute_per_func.insert(result_value, (*last_value, block));
                return true;
            }

            let predecessors = self.cfg.predecessors(block);
            for predecessor in predecessors {
                // TODO: Do I need is_reachable here? We are looping over only the reachable blocks but does
                // that include a reachable block's predecessors?
                if dom_tree.is_reachable(predecessor)
                    && dom_tree.dominates(predecessor, block)
                    && !visited.contains(&predecessor)
                {
                    stack.push(predecessor);
                }
            }
        }
        false
    }

    /// Checks whether the given value id refers to an allocation.
    fn value_is_from_allocation(value: ValueId, dfg: &DataFlowGraph) -> bool {
        match &dfg[value] {
            Value::Instruction { instruction, .. } => {
                matches!(&dfg[*instruction], Instruction::Allocate)
            }
            _ => false,
        }
    }

    /// Removes all store instructions identified during analysis that aren't present in the
    /// provided `protected_allocations` `HashSet`.
    fn remove_unused_stores(
        &self,
        dfg: &mut DataFlowGraph,
        protected_allocations: &HashSet<AllocId>,
        block_id: BasicBlockId,
    ) {
        // Scan for unused stores
        let mut stores_to_remove = HashSet::new();

        for instruction_id in &self.store_ids {
            let address = match &dfg[*instruction_id] {
                Instruction::Store { address, .. } => *address,
                _ => unreachable!("store_ids should contain only store instructions"),
            };

            if !protected_allocations.contains(&address) {
                stores_to_remove.insert(*instruction_id);
            }
        }

        // Delete unused stores
        dfg[block_id]
            .instructions_mut()
            .retain(|instruction| !stores_to_remove.contains(instruction));
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use acvm::FieldElement;
    use im::vector;

    use crate::ssa::{
        ir::{
            basic_block::BasicBlockId,
            dfg::DataFlowGraph,
            function::RuntimeType,
            instruction::{BinaryOp, Instruction, Intrinsic, TerminatorInstruction},
            map::Id,
            types::Type,
        },
        ssa_builder::FunctionBuilder,
    };

    #[test]
    fn test_simple() {
        // fn func() {
        //   b0():
        //     v0 = allocate
        //     store [Field 1, Field 2] in v0
        //     v1 = load v0
        //     v2 = array_get v1, index 1
        //     return v2
        // }

        let func_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("func".into(), func_id, RuntimeType::Acir);
        let v0 = builder.insert_allocate();
        let one = builder.field_constant(FieldElement::one());
        let two = builder.field_constant(FieldElement::one());

        let element_type = Rc::new(vec![Type::field()]);
        let array_type = Type::Array(element_type, 2);
        let array = builder.array_constant(vector![one, two], array_type.clone());

        builder.insert_store(v0, array);
        let v1 = builder.insert_load(v0, array_type);
        let v2 = builder.insert_array_get(v1, one, Type::field());
        builder.terminate_with_return(vec![v2]);

        let ssa = builder.finish().mem2reg().fold_constants();

        println!("{ssa}");

        let func = ssa.main();
        let block_id = func.entry_block();

        assert_eq!(count_loads(block_id, &func.dfg), 0);
        assert_eq!(count_stores(block_id, &func.dfg), 0);

        let ret_val_id = match func.dfg[block_id].terminator().unwrap() {
            TerminatorInstruction::Return { return_values } => return_values.first().unwrap(),
            _ => unreachable!(),
        };
        assert_eq!(func.dfg[*ret_val_id], func.dfg[two]);
    }

    #[test]
    fn test_simple_with_call() {
        // fn func {
        //   b0():
        //     v0 = allocate
        //     store v0, Field 1
        //     v1 = load v0
        //     call f0(v0)
        //     return v1
        // }

        let func_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("func".into(), func_id, RuntimeType::Acir);
        let v0 = builder.insert_allocate();
        let one = builder.field_constant(FieldElement::one());
        builder.insert_store(v0, one);
        let v1 = builder.insert_load(v0, Type::field());
        let f0 = builder.import_intrinsic_id(Intrinsic::Println);
        builder.insert_call(f0, vec![v0], vec![]);
        builder.terminate_with_return(vec![v1]);

        let ssa = builder.finish().mem2reg();

        let func = ssa.main();
        let block_id = func.entry_block();

        assert_eq!(count_loads(block_id, &func.dfg), 0);
        assert_eq!(count_stores(block_id, &func.dfg), 1);

        let ret_val_id = match func.dfg[block_id].terminator().unwrap() {
            TerminatorInstruction::Return { return_values } => return_values.first().unwrap(),
            _ => unreachable!(),
        };
        assert_eq!(func.dfg[*ret_val_id], func.dfg[one]);
    }

    #[test]
    fn test_simple_with_return() {
        // fn func {
        //   b0():
        //     v0 = allocate
        //     store v0, Field 1
        //     return v0
        // }

        let func_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("func".into(), func_id, RuntimeType::Acir);
        let v0 = builder.insert_allocate();
        let const_one = builder.field_constant(FieldElement::one());
        builder.insert_store(v0, const_one);
        builder.terminate_with_return(vec![v0]);

        let ssa = builder.finish().mem2reg();

        let func = ssa.main();
        let block_id = func.entry_block();

        // Store affects outcome of returned array, and can't be removed
        assert_eq!(count_stores(block_id, &func.dfg), 1);

        let ret_val_id = match func.dfg[block_id].terminator().unwrap() {
            TerminatorInstruction::Return { return_values } => return_values.first().unwrap(),
            _ => unreachable!(),
        };
        assert_eq!(func.dfg[*ret_val_id], func.dfg[v0]);
    }

    fn count_stores(block: BasicBlockId, dfg: &DataFlowGraph) -> usize {
        dfg[block]
            .instructions()
            .iter()
            .filter(|instruction_id| matches!(dfg[**instruction_id], Instruction::Store { .. }))
            .count()
    }

    fn count_loads(block: BasicBlockId, dfg: &DataFlowGraph) -> usize {
        dfg[block]
            .instructions()
            .iter()
            .filter(|instruction_id| matches!(dfg[**instruction_id], Instruction::Load { .. }))
            .count()
    }

    // Test that loads across multiple blocks are removed
    #[test]
    fn multiple_blocks() {
        // fn main {
        //   b0():
        //     v0 = allocate
        //     store Field 5 in v0
        //     v1 = load v0
        //     jmp b1(v1):
        //   b1(v2: Field):
        //     v3 = load v0
        //     store Field 6 in v0
        //     v4 = load v0
        //     return v2, v3, v4
        // }
        let main_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id, RuntimeType::Acir);

        let v0 = builder.insert_allocate();

        let five = builder.field_constant(5u128);
        builder.insert_store(v0, five);

        let v1 = builder.insert_load(v0, Type::field());
        let b1 = builder.insert_block();
        builder.terminate_with_jmp(b1, vec![v1]);

        builder.switch_to_block(b1);
        let v2 = builder.add_block_parameter(b1, Type::field());
        let v3 = builder.insert_load(v0, Type::field());

        let six = builder.field_constant(6u128);
        builder.insert_store(v0, six);
        let v4 = builder.insert_load(v0, Type::field());

        builder.terminate_with_return(vec![v2, v3, v4]);

        let ssa = builder.finish();
        assert_eq!(ssa.main().reachable_blocks().len(), 2);

        // Expected result:
        // fn main {
        //   b0():
        //     v0 = allocate
        //     jmp b1(Field 5)
        //   b1(v3: Field):
        //     return v3, Field 5, Field 6 // Optimized to constants 5 and 6
        // }
        let ssa = ssa.mem2reg();

        let main = ssa.main();
        assert_eq!(main.reachable_blocks().len(), 2);

        // The loads should be removed
        assert_eq!(count_loads(main.entry_block(), &main.dfg), 0);
        assert_eq!(count_loads(b1, &main.dfg), 0);

        // All stores should be removed
        assert_eq!(count_stores(main.entry_block(), &main.dfg), 0);
        assert_eq!(count_stores(b1, &main.dfg), 0);

        // The jmp to b1 should also be a constant 5 now
        match main.dfg[main.entry_block()].terminator() {
            Some(TerminatorInstruction::Jmp { arguments, .. }) => {
                assert_eq!(arguments.len(), 1);
                let argument =
                    main.dfg.get_numeric_constant(arguments[0]).expect("Expected constant value");
                assert_eq!(argument.to_u128(), 5);
            }
            _ => unreachable!(),
        };
    }

    // Test that a load in a predecessor block has been removed if the value
    // is later stored in a successor block
    #[test]
    fn store_with_load_in_predecessor_block() {
        // fn main {
        //     b0():
        //       v0 = allocate
        //       store Field 0 at v0
        //       v2 = allocate
        //       store v0 at v2
        //       v3 = load v2
        //       v4 = load v2
        //       jmp b1()
        //     b1():
        //       store Field 1 at v3
        //       store Field 2 at v4
        //       v8 = load v3
        //       v9 = eq v8, Field 2
        //       return
        // }
        let main_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id, RuntimeType::Acir);

        let v0 = builder.insert_allocate();

        let zero = builder.field_constant(0u128);
        builder.insert_store(v0, zero);

        let v2 = builder.insert_allocate();
        builder.insert_store(v2, v0);

        let v3 = builder.insert_load(v2, Type::field());
        let v4 = builder.insert_load(v2, Type::field());
        let b1 = builder.insert_block();
        builder.terminate_with_jmp(b1, vec![]);

        builder.switch_to_block(b1);

        let one = builder.field_constant(1u128);
        builder.insert_store(v3, one);

        let two = builder.field_constant(2u128);
        builder.insert_store(v4, two);

        let v8 = builder.insert_load(v3, Type::field());
        let _ = builder.insert_binary(v8, BinaryOp::Eq, two);

        builder.terminate_with_return(vec![]);

        let ssa = builder.finish();
        assert_eq!(ssa.main().reachable_blocks().len(), 2);

        // Expected result:
        // fn main {
        //     b0():
        //       v0 = allocate
        //       v2 = allocate
        //       jmp b1()
        //     b1():
        //       v8 = eq Field 2, Field 2
        //       return
        // }
        let ssa = ssa.mem2reg();

        let main = ssa.main();
        assert_eq!(main.reachable_blocks().len(), 2);

        // All loads should be removed
        assert_eq!(count_loads(main.entry_block(), &main.dfg), 0);
        assert_eq!(count_loads(b1, &main.dfg), 0);

        // All stores should be removed
        assert_eq!(count_stores(main.entry_block(), &main.dfg), 0);
        assert_eq!(count_stores(b1, &main.dfg), 0);

        let b1_instructions = main.dfg[b1].instructions();
        // The first instruction should be a binary operation
        match &main.dfg[b1_instructions[0]] {
            Instruction::Binary(binary) => {
                let lhs =
                    main.dfg.get_numeric_constant(binary.lhs).expect("Expected constant value");
                let rhs =
                    main.dfg.get_numeric_constant(binary.rhs).expect("Expected constant value");

                assert_eq!(lhs, rhs);
                assert_eq!(lhs, FieldElement::from(2u128));
            }
            _ => unreachable!(),
        }
    }
}
