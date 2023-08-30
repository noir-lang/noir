use std::collections::HashMap;

use iter_extended::vecmap;

use super::{
    basic_block::BasicBlockId,
    dfg::{CallStack, InsertInstructionResult},
    function::Function,
    instruction::{Instruction, InstructionId},
    value::ValueId,
};

/// The FunctionInserter can be used to help modify existing Functions
/// and map old values to new values after re-inserting optimized versions
/// of old instructions.
pub(crate) struct FunctionInserter<'f> {
    pub(crate) function: &'f mut Function,

    values: HashMap<ValueId, ValueId>,
}

impl<'f> FunctionInserter<'f> {
    pub(crate) fn new(function: &'f mut Function) -> FunctionInserter<'f> {
        Self { function, values: HashMap::new() }
    }

    /// Resolves a ValueId to its new, updated value.
    /// If there is no updated value for this id, this returns the same
    /// ValueId that was passed in.
    pub(crate) fn resolve(&mut self, value: ValueId) -> ValueId {
        match self.values.get(&value) {
            Some(value) => self.resolve(*value),
            None => match &self.function.dfg[value] {
                super::value::Value::Array { array, typ } => {
                    let array = array.clone();
                    let typ = typ.clone();
                    let new_array = array.iter().map(|id| self.resolve(*id)).collect();
                    let new_id = self.function.dfg.make_array(new_array, typ);
                    self.values.insert(value, new_id);
                    new_id
                }
                _ => value,
            },
        }
    }

    /// Insert a key, value pair if the key isn't already present in the map
    pub(crate) fn try_map_value(&mut self, key: ValueId, value: ValueId) {
        if key == value {
            // This case is technically not needed since try_map_value isn't meant to change
            // existing entries, but we should never have a value in the map referring to itself anyway.
            self.values.remove(&key);
        } else {
            self.values.entry(key).or_insert(value);
        }
    }

    /// Insert a key, value pair in the map
    pub(crate) fn map_value(&mut self, key: ValueId, value: ValueId) {
        if key == value {
            self.values.remove(&key);
        } else {
            self.values.insert(key, value);
        }
    }

    pub(crate) fn map_instruction(&mut self, id: InstructionId) -> (Instruction, CallStack) {
        (
            self.function.dfg[id].clone().map_values(|id| self.resolve(id)),
            self.function.dfg.get_call_stack(id),
        )
    }

    /// Maps a terminator in place, replacing any ValueId in the terminator with the
    /// resolved version of that value id from this FunctionInserter's internal value mapping.
    pub(crate) fn map_terminator_in_place(&mut self, block: BasicBlockId) {
        let mut terminator = self.function.dfg[block].take_terminator();
        terminator.mutate_values(|value| self.resolve(value));
        self.function.dfg[block].set_terminator(terminator);
    }

    /// Push a new instruction to the given block and return its new InstructionId.
    /// If the instruction was simplified out of the program, None is returned.
    pub(crate) fn push_instruction(
        &mut self,
        id: InstructionId,
        block: BasicBlockId,
    ) -> Option<InstructionId> {
        let (instruction, location) = self.map_instruction(id);

        match self.push_instruction_value(instruction, id, block, location) {
            InsertInstructionResult::Results(new_id, _) => Some(new_id),
            _ => None,
        }
    }

    pub(crate) fn push_instruction_value(
        &mut self,
        instruction: Instruction,
        id: InstructionId,
        block: BasicBlockId,
        call_stack: CallStack,
    ) -> InsertInstructionResult {
        let results = self.function.dfg.instruction_results(id).to_vec();

        let ctrl_typevars = instruction
            .requires_ctrl_typevars()
            .then(|| vecmap(&results, |result| self.function.dfg.type_of_value(*result)));

        let new_results = self.function.dfg.insert_instruction_and_results(
            instruction,
            block,
            ctrl_typevars,
            call_stack,
        );

        Self::insert_new_instruction_results(&mut self.values, &results, &new_results);
        new_results
    }

    /// Modify the values HashMap to remember the mapping between an instruction result's previous
    /// ValueId (from the source_function) and its new ValueId in the destination function.
    pub(crate) fn insert_new_instruction_results(
        values: &mut HashMap<ValueId, ValueId>,
        old_results: &[ValueId],
        new_results: &InsertInstructionResult,
    ) {
        assert_eq!(old_results.len(), new_results.len());

        match new_results {
            InsertInstructionResult::SimplifiedTo(new_result) => {
                values.insert(old_results[0], *new_result);
            }
            InsertInstructionResult::SimplifiedToMultiple(new_results) => {
                for (old_result, new_result) in old_results.iter().zip(new_results) {
                    values.insert(*old_result, *new_result);
                }
            }
            InsertInstructionResult::Results(_, new_results) => {
                for (old_result, new_result) in old_results.iter().zip(*new_results) {
                    values.insert(*old_result, *new_result);
                }
            }
            InsertInstructionResult::InstructionRemoved => (),
        }
    }

    pub(crate) fn remember_block_params(&mut self, block: BasicBlockId, new_values: &[ValueId]) {
        let old_parameters = self.function.dfg.block_parameters(block);

        for (param, new_param) in old_parameters.iter().zip(new_values) {
            self.values.entry(*param).or_insert(*new_param);
        }
    }

    pub(crate) fn remember_block_params_from_block(
        &mut self,
        block: BasicBlockId,
        new_block: BasicBlockId,
    ) {
        let old_parameters = self.function.dfg.block_parameters(block);
        let new_parameters = self.function.dfg.block_parameters(new_block);

        for (param, new_param) in old_parameters.iter().zip(new_parameters) {
            // Don't overwrite any existing entries to avoid overwriting the induction variable
            self.values.entry(*param).or_insert(*new_param);
        }
    }
}
