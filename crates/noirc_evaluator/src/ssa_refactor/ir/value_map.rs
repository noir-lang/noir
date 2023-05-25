use std::collections::HashMap;

use iter_extended::vecmap;

use super::{
    dfg::{DataFlowGraph, InsertInstructionResult},
    instruction::{Instruction, InstructionId},
    types::Type,
    value::ValueId,
};

#[derive(Default)]
pub(crate) struct ValueMap {
    map: HashMap<ValueId, ValueId>,
}

impl ValueMap {
    pub(crate) fn insert(&mut self, old_id: ValueId, new_id: ValueId) {
        self.map.insert(old_id, new_id);
    }

    /// Insert the given old_id -> new_id mapping if old_id isn't already in the map.
    pub(crate) fn insert_if_not_present(&mut self, old_id: ValueId, new_id: ValueId) {
        self.map.entry(old_id).or_insert(new_id);
    }

    pub(crate) fn get(&self, value: ValueId) -> Option<ValueId> {
        self.map.get(&value).copied()
    }

    pub(crate) fn get_value(&self, value: ValueId) -> ValueId {
        self.get(value).unwrap_or(value)
    }

    pub(crate) fn extend(&mut self, iterator: impl Iterator<Item = (ValueId, ValueId)>) {
        self.map.extend(iterator);
    }

    pub(crate) fn map_instruction<'dfg>(
        &mut self,
        dfg: &'dfg DataFlowGraph,
        id: InstructionId,
    ) -> (Instruction, &'dfg [ValueId], Option<Vec<Type>>) {
        let instruction = dfg[id].map_values(|id| self.get_value(id));
        let results = dfg.instruction_results(id);

        let ctrl_typevars = instruction
            .requires_ctrl_typevars()
            .then(|| vecmap(results, |result| dfg.type_of_value(*result)));

        (instruction, results, ctrl_typevars)
    }

    /// Modify the values HashMap to remember the mapping between an instruction result's previous
    /// ValueId (from the source_function) and its new ValueId in the destination function.
    pub(crate) fn insert_new_instruction_results(
        &mut self,
        old_results: &[ValueId],
        new_results: InsertInstructionResult,
    ) {
        assert_eq!(old_results.len(), new_results.len());

        match new_results {
            InsertInstructionResult::SimplifiedTo(new_result) => {
                self.map.insert(old_results[0], new_result);
            }
            InsertInstructionResult::Results(new_results) => {
                for (old_result, new_result) in old_results.iter().zip(new_results) {
                    self.map.insert(*old_result, *new_result);
                }
            }
            InsertInstructionResult::InstructionRemoved => (),
        }
    }
}
