//! mem2reg implements a pass for promoting values stored in memory to values in registers where
//! possible. This is particularly important for converting our memory-based representation of
//! mutable variables into values that are easier to manipulate.
use std::collections::{BTreeMap, BTreeSet};

use crate::ssa_refactor::{
    ir::{
        basic_block::BasicBlockId,
        dfg::DataFlowGraph,
        instruction::{Instruction, InstructionId, TerminatorInstruction},
        value::ValueId,
    },
    ssa_gen::Ssa,
};

impl Ssa {
    /// Attempts to remove any load instructions that recover values that are already available in
    /// scope. Also attempts to remove store instructions if the function contains only a single
    /// block.
    pub(crate) fn mem2reg(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            let blocks = function.reachable_blocks();
            let is_single_block = blocks.len() == 1;
            for block in function.reachable_blocks() {
                let mut context = PerBlockContext::new(block);
                context.eliminate_known_loads(&mut function.dfg);
                if is_single_block {
                    // If this function has only a single block, we know that the side effects of a
                    // store instruction only have bearing within the scope of the block.
                    context.remove_unused_stores(&mut function.dfg);
                }
            }
        }

        self
    }
}

struct PerBlockContext {
    block_id: BasicBlockId,
    last_stores: BTreeMap<ValueId, ValueId>,
    store_ids: Vec<InstructionId>,
    failed_substitutes: BTreeSet<ValueId>,
}

impl PerBlockContext {
    fn new(block_id: BasicBlockId) -> Self {
        PerBlockContext {
            block_id,
            last_stores: BTreeMap::new(),
            store_ids: Vec::new(),
            failed_substitutes: BTreeSet::new(),
        }
    }

    // Attempts to remove load instructions for which the result is already known from previous
    // store instructions to the same address in the same block.
    fn eliminate_known_loads(&mut self, dfg: &mut DataFlowGraph) {
        let mut loads_to_substitute = Vec::new();
        let block = &dfg[self.block_id];

        for instruction_id in block.instructions() {
            match &dfg[*instruction_id] {
                Instruction::Store { address, value } => {
                    self.last_stores.insert(*address, *value);
                    self.store_ids.push(*instruction_id);
                }
                Instruction::Load { address } => {
                    if let Some(last_value) = self.last_stores.get(address) {
                        loads_to_substitute.push((*instruction_id, *last_value));
                    } else {
                        self.failed_substitutes.insert(*address);
                    }
                }
                Instruction::Call { arguments, .. } => {
                    for value in arguments {
                        assert!(!self.last_stores.contains_key(value), "Mutable vars are loaded before being passed as function arguments - if this pattern changes, so do our safety assumptions.");
                    }
                }
                _ => {
                    // Nothing to do
                }
            }
        }

        // Substitute load result values
        for (instruction_id, new_value) in &loads_to_substitute {
            let result_values = dfg.instruction_results(*instruction_id);
            assert_eq!(result_values.len(), 1);
            dfg.set_value_from_id(result_values[0], *new_value);
        }

        let block = &mut dfg[self.block_id];
        for (instruction_id, _) in &loads_to_substitute {
            // Technically we could leave this removal to the DIE pass, but the debug print is
            // easier to read if we remove it now.
            block.remove_instruction(*instruction_id);
        }

        if let TerminatorInstruction::Return { return_values } = block.unwrap_terminator() {
            for value in return_values {
                assert!(!self.last_stores.contains_key(value), "Mutable vars are loaded before being returned - if this pattern changes, so do our safety assumptions.");
            }
        }
    }

    fn remove_unused_stores(self, dfg: &mut DataFlowGraph) {
        // Scan for unused stores
        let mut stores_to_remove: Vec<InstructionId> = Vec::new();
        for instruction_id in &self.store_ids {
            let address = match &dfg[*instruction_id] {
                Instruction::Store { address, .. } => *address,
                _ => unreachable!("store_ids should contain only store instructions"),
            };
            if !self.failed_substitutes.contains(&address) {
                stores_to_remove.push(*instruction_id);
            }
        }

        // Delete unused stores
        let block = &mut dfg[self.block_id];
        for instruction_id in stores_to_remove {
            println!("rm {:?}", instruction_id);
            block.remove_instruction(instruction_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use acvm::FieldElement;
    use im::vector;

    use crate::ssa_refactor::{
        ir::{
            basic_block::BasicBlockId,
            dfg::DataFlowGraph,
            function::RuntimeType,
            instruction::{Instruction, Intrinsic, TerminatorInstruction},
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
        let array = builder.array_constant(vector![one, two], element_type.clone());

        builder.insert_store(v0, array);
        let v1 = builder.insert_load(v0, Type::Array(element_type, 2));
        let v2 = builder.insert_array_get(v1, one, Type::field());
        builder.terminate_with_return(vec![v2]);

        let ssa = builder.finish().mem2reg().fold_constants();

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
    #[should_panic]
    fn test_calls_disallowed() {
        // fn func {
        //   b0():
        //     v0 = allocate
        //     store v0, Field 1
        //     v1 = load v0
        //     call f0(v0)
        //     return v1
        // }
        // Passing a memory address as function arguments is unsupported

        let func_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("func".into(), func_id, RuntimeType::Acir);
        let v0 = builder.insert_allocate();
        let one = builder.field_constant(FieldElement::one());
        builder.insert_store(v0, one);
        let v1 = builder.insert_load(v0, Type::field());
        let f0 = builder.import_intrinsic_id(Intrinsic::Println);
        builder.insert_call(f0, vec![v0], vec![]);
        builder.terminate_with_return(vec![v1]);

        builder.finish().mem2reg();
    }

    #[test]
    #[should_panic]
    fn test_return_disallowed() {
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

        builder.finish().mem2reg();
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

    // Test that loads across multiple blocks are not removed
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
        //     store v0, Field 5
        //     jmp b1(Field 5):  // Optimized to constant 5
        //   b1(v2: Field):
        //     v3 = load v0      // kept in program
        //     store v0, Field 6
        //     return v2, v3, Field 6 // Optimized to constant 6
        // }
        let ssa = ssa.mem2reg();
        let main = ssa.main();
        assert_eq!(main.reachable_blocks().len(), 2);

        // Only the load from the entry block should be removed
        assert_eq!(count_loads(main.entry_block(), &main.dfg), 0);
        assert_eq!(count_loads(b1, &main.dfg), 1);

        // All stores should be kept
        assert_eq!(count_stores(main.entry_block(), &main.dfg), 1);
        assert_eq!(count_stores(b1, &main.dfg), 1);

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
}
