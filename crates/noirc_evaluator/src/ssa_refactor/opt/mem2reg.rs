//! mem2reg implements a pass for promoting values stored in memory to values in registers where
//! possible. This is particularly important for converting our memory-based representation of
//! mutable variables into values that are easier to manipulate.
use std::collections::{BTreeMap, BTreeSet};

use crate::ssa_refactor::{
    ir::{
        basic_block::BasicBlockId,
        constant::NumericConstantId,
        dfg::DataFlowGraph,
        instruction::{BinaryOp, Instruction, InstructionId},
        value::{Value, ValueId},
    },
    ssa_gen::Ssa,
};

impl Ssa {
    /// Attempts to remove any load instructions that recover values that already available in
    /// scope, and attempts to remove store that are subsequently redundant, as long as they are
    /// not stores on memory that will be passed into a function call.
    ///
    /// This pass assumes that the whole program has been inlined into a single block, such that
    /// we can be sure that store instructions cannot have side effects outside of this block
    /// (apart from intrinsic function calls).
    ///
    /// This pass also assumes that constant folding has been run, such that all addresses given
    /// as input to store/load instructions are represented as one of:
    /// - a value that directly resolves to an allocate instruction
    /// - a value that directly resolves to a binary add instruction which has a allocate
    /// instruction and a numeric constant as its operands
    pub(crate) fn mem2reg_final(mut self) -> Ssa {
        let func = self.main_mut();
        assert_eq!(func.dfg.basic_blocks_iter().count(), 1);
        let block_id = func.entry_block();
        PerBlockContext::new(&mut func.dfg, block_id).eliminate_store_load();
        self
    }
}

#[derive(PartialEq, PartialOrd, Eq, Ord)]
enum Address {
    Zeroth(InstructionId),
    Offset(InstructionId, NumericConstantId),
}

impl Address {
    fn alloc_id(&self) -> InstructionId {
        match self {
            Address::Zeroth(alloc_id) => *alloc_id,
            Address::Offset(alloc_id, _) => *alloc_id,
        }
    }
}

struct PerBlockContext<'dfg> {
    dfg: &'dfg mut DataFlowGraph,
    block_id: BasicBlockId,
}

impl<'dfg> PerBlockContext<'dfg> {
    fn new(dfg: &'dfg mut DataFlowGraph, block_id: BasicBlockId) -> Self {
        PerBlockContext { dfg, block_id }
    }

    // Attempts to remove redundant load & store instructions for constant addresses. Returns the
    // count of remaining store instructions.
    //
    // This method assumes the entire program is now represented in a single block (minus any
    // intrinsic function calls). Therefore we needn't be concerned with store instructions having
    // an effect beyond the scope of this block.
    fn eliminate_store_load(&mut self) -> u32 {
        let mut store_count: u32 = 0;
        let mut last_stores: BTreeMap<Address, ValueId> = BTreeMap::new();
        let mut loads_to_substitute: Vec<(InstructionId, Value)> = Vec::new();
        let mut store_ids: Vec<InstructionId> = Vec::new();
        let mut failed_substitutes: BTreeSet<Address> = BTreeSet::new();
        let mut alloc_ids_in_calls: BTreeSet<InstructionId> = BTreeSet::new();

        let block = &self.dfg[self.block_id];
        for instruction_id in block.instructions() {
            match &self.dfg[*instruction_id] {
                Instruction::Store { address, value } => {
                    store_count += 1;
                    if let Some(address) = self.try_const_address(*address) {
                        // We can only track the address if it is a constant offset from an
                        // allocation. A previous constant folding pass should make such addresses
                        // possible to identify.
                        last_stores.insert(address, *value);
                    }
                    // TODO: Consider if it's worth falling back to storing addresses by their
                    // value id such we can shallowly check for dynamic address reuse.
                    store_ids.push(*instruction_id);
                }
                Instruction::Load { address } => {
                    if let Some(address) = self.try_const_address(*address) {
                        if let Some(last_value) = last_stores.get(&address) {
                            let last_value = self.dfg[*last_value];
                            loads_to_substitute.push((*instruction_id, last_value));
                        } else {
                            failed_substitutes.insert(address);
                        }
                    }
                }
                Instruction::Call { arguments, .. } => {
                    for arg in arguments {
                        if let Some(address) = self.try_const_address(*arg) {
                            alloc_ids_in_calls.insert(address.alloc_id());
                        }
                    }
                }
                _ => {
                    // Nothing to do
                }
            }
        }

        // Substitute load result values
        for (instruction_id, new_value) in &loads_to_substitute {
            let result_value = *self
                .dfg
                .instruction_results(*instruction_id)
                .first()
                .expect("ICE: Load instructions should have single result");
            self.dfg.set_value(result_value, *new_value);
        }

        // Delete load instructions
        // TODO: should we let DCE do this instead?
        let block = &mut self.dfg[self.block_id];
        for (instruction_id, _) in loads_to_substitute {
            block.remove_instruction(instruction_id);
        }

        // Scan for unused stores
        let mut stores_to_remove: Vec<InstructionId> = Vec::new();
        for instruction_id in store_ids {
            let address = match &self.dfg[instruction_id] {
                Instruction::Store { address, .. } => *address,
                _ => unreachable!("store_ids should contain only store instructions"),
            };
            if let Some(address) = self.try_const_address(address) {
                if !failed_substitutes.contains(&address)
                    && !alloc_ids_in_calls.contains(&address.alloc_id())
                {
                    stores_to_remove.push(instruction_id);
                }
            }
        }

        // Delete unused stores
        let block = &mut self.dfg[self.block_id];
        for instruction_id in stores_to_remove {
            store_count -= 1;
            block.remove_instruction(instruction_id);
        }

        store_count
    }

    // Attempts to normalize the given value into a const address
    fn try_const_address(&self, value_id: ValueId) -> Option<Address> {
        let value = &self.dfg[value_id];
        let instruction_id = match value {
            Value::Instruction { instruction, .. } => *instruction,
            _ => return None,
        };
        let instruction = &self.dfg[instruction_id];
        match instruction {
            Instruction::Allocate { .. } => Some(Address::Zeroth(instruction_id)),
            Instruction::Binary(binary) => {
                if binary.operator != BinaryOp::Add {
                    return None;
                }
                let lhs = &self.dfg[binary.lhs];
                let rhs = &self.dfg[binary.rhs];
                self.try_const_address_offset(lhs, rhs)
                    .or_else(|| self.try_const_address_offset(rhs, lhs))
            }
            _ => None,
        }
    }

    // Tries val1 as an allocation instruction id and val2 as a constant offset
    fn try_const_address_offset(&self, val1: &Value, val2: &Value) -> Option<Address> {
        let alloc_id = match val1 {
            Value::Instruction { instruction, .. } => match &self.dfg[*instruction] {
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

    use crate::ssa_refactor::{
        ir::{
            instruction::{BinaryOp, Instruction, Intrinsic, TerminatorInstruction},
            map::Id,
            types::Type,
        },
        ssa_builder::FunctionBuilder,
    };

    use super::PerBlockContext;

    #[test]
    fn test_simple() {
        // func() {
        //   block0():
        //     v0 = alloc 2
        //     v1 = add v0, Field 1
        //     store v1, Field 1
        //     v2 = add v0, Field 1
        //     v3 = load v1
        //     return v3

        let func_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("func".into(), func_id);
        let v0 = builder.insert_allocate(2);
        let const_one = builder.field_constant(FieldElement::one());
        let v1 = builder.insert_binary(v0, BinaryOp::Add, const_one);
        builder.insert_store(v1, const_one);
        // v2 is created internally by builder.insert_load
        let v3 = builder.insert_load(v0, const_one, Type::field());
        builder.terminate_with_return(vec![v3]);

        let mut ssa = builder.finish();

        let mut func = ssa.functions.remove(&func_id).unwrap();
        let block_id = func.entry_block();

        let mut mem2reg_context = PerBlockContext::new(&mut func.dfg, block_id);
        let remaining_stores = mem2reg_context.eliminate_store_load();

        assert_eq!(remaining_stores, 0);

        let block = &func.dfg[block_id];
        let load_count = block
            .instructions()
            .iter()
            .filter(|instruction_id| matches!(func.dfg[**instruction_id], Instruction::Load { .. }))
            .count();
        assert_eq!(load_count, 0);
        let store_count = block
            .instructions()
            .iter()
            .filter(|instruction_id| {
                matches!(func.dfg[**instruction_id], Instruction::Store { .. })
            })
            .count();
        assert_eq!(store_count, 0);
        let ret_val_id = match block.terminator().unwrap() {
            TerminatorInstruction::Return { return_values } => return_values.first().unwrap(),
            _ => unreachable!(),
        };
        assert_eq!(func.dfg[*ret_val_id], func.dfg[const_one]);
    }

    #[test]
    fn test_simple_with_call() {
        // func() {
        //   block0():
        //     v0 = alloc 2
        //     v1 = add v0, Field 1
        //     store v1, Field 1
        //     v2 = add v0, Field 1
        //     v3 = load v1
        //     v4 = call f0, v0
        //     return v3

        let func_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("func".into(), func_id);
        let v0 = builder.insert_allocate(2);
        let const_one = builder.field_constant(FieldElement::one());
        let v1 = builder.insert_binary(v0, BinaryOp::Add, const_one);
        builder.insert_store(v1, const_one);
        // v2 is created internally by builder.insert_load
        let v3 = builder.insert_load(v0, const_one, Type::field());
        let f0 = builder.import_intrinsic_id(Intrinsic::Println);
        builder.insert_call(f0, vec![v0], vec![Type::Unit]);
        builder.terminate_with_return(vec![v3]);

        let mut ssa = builder.finish();

        let mut func = ssa.functions.remove(&func_id).unwrap();
        let block_id = func.entry_block();

        let mut mem2reg_context = PerBlockContext::new(&mut func.dfg, block_id);
        let remaining_stores = mem2reg_context.eliminate_store_load();

        assert_eq!(
            remaining_stores, 1,
            "Store cannot be removed as it affects intrinsic function call"
        );

        let block = &func.dfg[block_id];
        let load_count = block
            .instructions()
            .iter()
            .filter(|instruction_id| matches!(func.dfg[**instruction_id], Instruction::Load { .. }))
            .count();
        assert_eq!(load_count, 0);
        let store_count = block
            .instructions()
            .iter()
            .filter(|instruction_id| {
                matches!(func.dfg[**instruction_id], Instruction::Store { .. })
            })
            .count();
        assert_eq!(store_count, 1);
        let ret_val_id = match block.terminator().unwrap() {
            TerminatorInstruction::Return { return_values } => return_values.first().unwrap(),
            _ => unreachable!(),
        };
        assert_eq!(func.dfg[*ret_val_id], func.dfg[const_one]);
    }
}
