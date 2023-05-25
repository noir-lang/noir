use std::collections::HashMap;

use iter_extended::vecmap;

use super::{
    basic_block::BasicBlockId,
    dfg::{DataFlowGraph, InsertInstructionResult},
    instruction::{Instruction, InstructionId},
    types::Type,
    value::ValueId,
};

/// Maps old value ids to a new set of value ids, not necessarily in the same function.
/// This is useful for optimization passes which rewrite instructions and values.
/// Notably `map_instruction` and `push_instruction` are useful operations for such
/// passes so that they need to worry less about remembering to update each ValueId
/// within each Instruction.
#[derive(Default)]
pub(crate) struct ValueMap {
    map: HashMap<ValueId, ValueId>,
}

impl ValueMap {
    /// Insert a new mapping into this value map, overwriting any previous entry if present
    pub(crate) fn insert(&mut self, old_id: ValueId, new_id: ValueId) {
        self.map.insert(old_id, new_id);
    }

    /// Insert the given old_id -> new_id mapping if old_id isn't already in the map
    pub(crate) fn insert_if_not_present(&mut self, old_id: ValueId, new_id: ValueId) {
        self.map.entry(old_id).or_insert(new_id);
    }

    /// Retrieve the mapped value or return None otherwise
    pub(crate) fn get(&self, value: ValueId) -> Option<ValueId> {
        self.map.get(&value).copied()
    }

    /// Retrieve the mapped value, or if there is none, return the given value.
    pub(crate) fn get_value(&self, value: ValueId) -> ValueId {
        self.get(value).unwrap_or(value)
    }

    /// Extend this value map with each new mapping given from the given iterator
    pub(crate) fn extend(&mut self, iterator: impl Iterator<Item = (ValueId, ValueId)>) {
        self.map.extend(iterator);
    }

    /// Map the instruction's ValueIds to their mapped versions from this ValueMap.
    /// This will also return the (non-mapped) results of the instruction as well as the control
    /// type variables if needed.
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

    /// Push an instruction to the given block within the CFG.
    /// This will handle mapping the instruction's argument ValueIds and
    /// remembering the new result ValueIds.
    pub(crate) fn push_instruction(
        &mut self,
        id: InstructionId,
        dfg: &mut DataFlowGraph,
        block: BasicBlockId,
    ) {
        let (instruction, results, ctrl_typevars) = self.map_instruction(dfg, id);

        // We must collect the results into a Vec, otherwise we cannot mutate the dfg afterwards
        let results = results.to_vec();

        let new_results = dfg.insert_instruction_and_results(instruction, block, ctrl_typevars);
        self.insert_new_instruction_results(&results, new_results);
    }
}
