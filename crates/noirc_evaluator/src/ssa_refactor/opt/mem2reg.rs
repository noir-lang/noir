use std::collections::{BTreeMap, BTreeSet};

use crate::ssa_refactor::ir::{
    basic_block::BasicBlockId,
    constant::NumericConstantId,
    dfg::DataFlowGraph,
    instruction::{BinaryOp, Instruction, InstructionId},
    value::{Value, ValueId},
};

#[derive(PartialEq, PartialOrd, Eq, Ord)]
enum Address {
    Zeroth(InstructionId),
    Offset(InstructionId, NumericConstantId),
}

struct PerBlockContext<'dfg> {
    dfg: &'dfg mut DataFlowGraph,
    block_id: BasicBlockId,
    allocations: BTreeSet<ValueId>,
    last_stores: BTreeMap<Address, ValueId>,
}

impl<'dfg> PerBlockContext<'dfg> {
    fn new(dfg: &'dfg mut DataFlowGraph, block_id: BasicBlockId) -> Self {
        PerBlockContext {
            dfg,
            block_id,
            allocations: BTreeSet::new(),
            last_stores: BTreeMap::new(),
        }
    }

    // Attempts to remove redundant load & store instructions for constant addresses. Returns the
    // count of remaining store instructions.
    //
    // This method assumes the entire program is now represented in a single block (minus any
    // intrinsic function calls). Therefore we needn't be concerned with store instructions having
    // an effect beyond the scope of this block.
    fn eliminate_store_load(&mut self) -> u32 {
        let mut store_count: u32 = 0;
        let instructions = self.dfg[self.block_id].instructions();
        let mut loads_to_substitute: Vec<(InstructionId, Value)> = Vec::new();
        let mut store_ids: Vec<InstructionId> = Vec::new();
        let mut failed_substitutes: BTreeSet<Address> = BTreeSet::new();

        for instruction_id in instructions {
            match &self.dfg[*instruction_id] {
                Instruction::Store { address, value } => {
                    store_count += 1;
                    if let Some(address) = self.try_const_address(*address) {
                        self.last_stores.insert(address, *value);
                    }
                    store_ids.push(*instruction_id);
                }
                Instruction::Load { address } => {
                    if let Some(address) = self.try_const_address(*address) {
                        if let Some(last_value) = self.last_stores.get(&address) {
                            let last_value = self.dfg[*last_value];
                            loads_to_substitute.push((*instruction_id, last_value));
                        } else {
                            failed_substitutes.insert(address);
                        }
                    }
                }
                _ => {
                    // Nothing to do
                }
            }
        }

        // TODO: identify address that make their make into intrinsic function calls

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
            if let Some(alloc_offset_pair) = self.try_const_address(address) {
                if !failed_substitutes.contains(&alloc_offset_pair) {
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
            Value::Instruction { instruction, .. } => match self.dfg[*instruction] {
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
