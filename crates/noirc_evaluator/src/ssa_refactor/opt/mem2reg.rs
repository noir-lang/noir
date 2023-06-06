//! mem2reg implements a pass for promoting values stored in memory to values in registers where
//! possible. This is particularly important for converting our memory-based representation of
//! mutable variables into values that are easier to manipulate.
use std::collections::{BTreeMap, BTreeSet};

use acvm::FieldElement;

use crate::ssa_refactor::{
    ir::{
        basic_block::BasicBlockId,
        dfg::DataFlowGraph,
        instruction::{BinaryOp, Instruction, InstructionId, TerminatorInstruction},
        types::Type,
        value::{Value, ValueId},
    },
    ssa_gen::Ssa,
};

impl Ssa {
    /// Attempts to remove any load instructions that recover values that already available in
    /// scope, and attempts to remove store that are subsequently redundant, as long as they are
    /// not stores on memory that will be passed into a function call.
    ///
    /// This pass also assumes that constant folding has been run, such that all addresses given
    /// as input to store/load instructions are represented as one of:
    /// - a value that directly resolves to an allocate instruction
    /// - a value that directly resolves to a binary add instruction which has a allocate
    /// instruction and a numeric constant as its operands
    pub(crate) fn mem2reg(mut self) -> Ssa {
        let mut first_context = None;

        for function in self.functions.values_mut() {
            for block in function.reachable_blocks() {
                let mut context = PerBlockContext::new(block);
                context.eliminate_known_loads(&mut function.dfg);
                first_context = Some(context);
            }
        }

        // If there is only one block in total, remove any unused stores as well since we
        // know there is no other block they can impact.
        if self.functions.len() == 1 && self.main().dfg.basic_blocks_iter().len() == 1 {
            first_context.unwrap().remove_unused_stores(&mut self.main_mut().dfg);
        }

        self
    }
}

#[derive(PartialEq, PartialOrd, Eq, Ord)]
enum Address {
    Zeroth(InstructionId),
    Offset(InstructionId, FieldElement),
}

impl Address {
    fn alloc_id(&self) -> InstructionId {
        match self {
            Address::Zeroth(alloc_id) => *alloc_id,
            Address::Offset(alloc_id, _) => *alloc_id,
        }
    }
}

struct PerBlockContext {
    block_id: BasicBlockId,
    last_stores: BTreeMap<Address, ValueId>,
    store_ids: Vec<InstructionId>,
    failed_substitutes: BTreeSet<Address>,
    alloc_ids_used_externally: BTreeSet<InstructionId>,
}

impl PerBlockContext {
    fn new(block_id: BasicBlockId) -> Self {
        PerBlockContext {
            block_id,
            last_stores: BTreeMap::new(),
            store_ids: Vec::new(),
            failed_substitutes: BTreeSet::new(),
            alloc_ids_used_externally: BTreeSet::new(),
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
                    if let Some(address) = self.try_const_address(*address, dfg) {
                        // We can only track the address if it is a constant offset from an
                        // allocation. A previous constant folding pass should make such addresses
                        // possible to identify.
                        self.last_stores.insert(address, *value);
                    }
                    // TODO: Consider if it's worth falling back to storing addresses by their
                    // value id such we can shallowly check for dynamic address reuse.
                    self.store_ids.push(*instruction_id);
                }
                Instruction::Load { address } => {
                    if let Some(address) = self.try_const_address(*address, dfg) {
                        if let Some(last_value) = self.last_stores.get(&address) {
                            let last_value = dfg[*last_value].clone();
                            loads_to_substitute.push((*instruction_id, last_value));
                        } else {
                            self.failed_substitutes.insert(address);
                        }
                    }
                }
                Instruction::Call { arguments, .. } => {
                    for arg in arguments {
                        if let Some(address) = self.try_const_address(*arg, dfg) {
                            self.alloc_ids_used_externally.insert(address.alloc_id());
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
                if let Some(address) = self.try_const_address(*value, dfg) {
                    self.alloc_ids_used_externally.insert(address.alloc_id());
                }
            }
        }

        // Substitute load result values
        for (instruction_id, new_value) in &loads_to_substitute {
            let result_value = *dfg
                .instruction_results(*instruction_id)
                .first()
                .expect("ICE: Load instructions should have single result");
            dfg.set_value(result_value, new_value.clone());
        }

        // Delete load instructions
        // TODO: should we let DCE do this instead?
        let block = &mut dfg[self.block_id];
        for (instruction_id, _) in loads_to_substitute {
            block.remove_instruction(instruction_id);
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

            if let Some(address) = self.try_const_address(address, dfg) {
                if !self.failed_substitutes.contains(&address)
                    && !self.alloc_ids_used_externally.contains(&address.alloc_id())
                {
                    stores_to_remove.push(*instruction_id);
                }
            }
        }

        // Delete unused stores
        let block = &mut dfg[self.block_id];
        for instruction_id in stores_to_remove {
            block.remove_instruction(instruction_id);
        }
    }

    // Attempts to normalize the given value into a const address
    fn try_const_address(&self, value_id: ValueId, dfg: &DataFlowGraph) -> Option<Address> {
        if dfg.type_of_value(value_id) != Type::Reference {
            return None;
        }
        let value = &dfg[value_id];
        let instruction_id = match value {
            Value::Instruction { instruction, .. } => *instruction,
            _ => return None,
        };
        let instruction = &dfg[instruction_id];
        match instruction {
            // Arrays can be returned by allocations and function calls
            Instruction::Allocate { .. } | Instruction::Call { .. } => {
                Some(Address::Zeroth(instruction_id))
            }
            Instruction::Binary(binary) => {
                if binary.operator != BinaryOp::Add {
                    return None;
                }
                let lhs = &dfg[binary.lhs];
                let rhs = &dfg[binary.rhs];
                self.try_const_address_offset(lhs, rhs, dfg)
                    .or_else(|| self.try_const_address_offset(rhs, lhs, dfg))
            }
            _ => None,
        }
    }

    // Tries val1 as an allocation instruction id and val2 as a constant offset
    fn try_const_address_offset(
        &self,
        val1: &Value,
        val2: &Value,
        dfg: &DataFlowGraph,
    ) -> Option<Address> {
        let alloc_id = match val1 {
            Value::Instruction { instruction, .. } => match &dfg[*instruction] {
                Instruction::Allocate { .. } => *instruction,
                _ => return None,
            },
            _ => return None,
        };
        if let Value::NumericConstant { constant, .. } = val2 {
            Some(Address::Offset(alloc_id, *constant))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
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
        let array = builder.array_constant(vector![one, two]);

        builder.insert_store(v0, array);
        let v1 = builder.insert_load(v0, Type::Array);
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
        //     v2 = call f0(v0)
        //     return v1
        // }

        let func_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("func".into(), func_id, RuntimeType::Acir);
        let v0 = builder.insert_allocate();
        let one = builder.field_constant(FieldElement::one());
        builder.insert_store(v0, one);
        let v1 = builder.insert_load(v0, Type::field());
        let f0 = builder.import_intrinsic_id(Intrinsic::Println);
        builder.insert_call(f0, vec![v0], vec![Type::Unit]);
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
