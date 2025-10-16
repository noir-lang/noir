use iter_extended::vecmap;
use noirc_errors::call_stack::CallStackId;

use super::{
    basic_block::BasicBlockId,
    dfg::InsertInstructionResult,
    function::Function,
    instruction::{Instruction, InstructionId},
    value::ValueId,
};
use rustc_hash::FxHashMap as HashMap;

/// The FunctionInserter can be used to help modify existing Functions
/// and map old values to new values after re-inserting optimized versions
/// of old instructions.
pub(crate) struct FunctionInserter<'f> {
    pub(crate) function: &'f mut Function,

    values: HashMap<ValueId, ValueId>,
}

impl<'f> FunctionInserter<'f> {
    pub(crate) fn new(function: &'f mut Function) -> FunctionInserter<'f> {
        Self { function, values: HashMap::default() }
    }

    /// Resolves a ValueId to its new, updated value.
    /// If there is no updated value for this id, this returns the same
    /// ValueId that was passed in.
    pub(crate) fn resolve(&self, value: ValueId) -> ValueId {
        match self.values.get(&value) {
            Some(new_value) => self.resolve(*new_value),
            None => value,
        }
    }

    /// Insert a key, value pair if the key isn't already present in the map
    pub(crate) fn try_map_value(&mut self, key: ValueId, value: ValueId) {
        if key == value {
            // This case is technically not needed since try_map_value isn't meant to change
            // existing entries, but we should never have a value in the map referring to itself anyway.
            self.values.remove(&key);
        } else {
            #[cfg(debug_assertions)]
            self.validate_map_value(key, value);
            self.values.entry(key).or_insert(value);
        }
    }

    /// Insert a key, value pair in the map
    pub(crate) fn map_value(&mut self, key: ValueId, value: ValueId) {
        if key == value {
            self.values.remove(&key);
        } else {
            #[cfg(debug_assertions)]
            self.validate_map_value(key, value);
            self.values.insert(key, value);
        }
    }

    /// Sanity check that we are not creating back-and-forth cycles.
    /// Doesn't catch longer cycles, but has detected mistakes with reusing instructions.
    #[cfg(debug_assertions)]
    fn validate_map_value(&self, key: ValueId, value: ValueId) {
        if let Some(value_of_value) = self.values.get(&value) {
            assert!(*value_of_value != key, "mapping short-circuit: {key} <-> {value}");
        }
    }

    /// Get an instruction and make sure all the values in it are freshly resolved.
    pub(crate) fn map_instruction(&mut self, id: InstructionId) -> (Instruction, CallStackId) {
        let mut instruction = self.function.dfg[id].clone();
        instruction.map_values_mut(|id| self.resolve(id));
        (instruction, self.function.dfg.get_instruction_call_stack_id(id))
    }

    /// Get an instruction, map all its values, and replace it with the resolved instruction.
    pub(crate) fn map_instruction_in_place(&mut self, id: InstructionId) {
        let mut instruction = self.function.dfg[id].clone();
        instruction.map_values_mut(|id| self.resolve(id));
        self.function.dfg.set_instruction(id, instruction);
    }

    /// Maps a terminator in place, replacing any ValueId in the terminator with the
    /// resolved version of that value id from this FunctionInserter's internal value mapping.
    pub(crate) fn map_terminator_in_place(&mut self, block: BasicBlockId) {
        let mut terminator = self.function.dfg[block].take_terminator();
        terminator.map_values_mut(|value| self.resolve(value));
        self.function.dfg[block].set_terminator(terminator);
    }

    /// Maps the data bus in place, replacing any ValueId in the data bus with the
    /// resolved version of that value id from this FunctionInserter's internal value mapping.
    pub(crate) fn map_data_bus_in_place(&mut self) {
        let mut data_bus = self.function.dfg.data_bus.clone();
        data_bus.map_values_mut(|value| self.resolve(value));
        self.function.dfg.data_bus = data_bus;
    }

    /// Push an instruction by ID, after re-mapping the values in it, to the given block and return its new `InstructionId`.
    /// If the instruction was simplified out of the program, `None` is returned.
    pub(crate) fn push_instruction(
        &mut self,
        id: InstructionId,
        block: BasicBlockId,
        allow_reinsert: bool,
    ) -> Option<InstructionId> {
        let (instruction, location) = self.map_instruction(id);

        match self.push_instruction_value(instruction, id, block, location, allow_reinsert) {
            InsertInstructionResult::Results(new_id, _) => Some(new_id),
            _ => None,
        }
    }

    /// Push a instruction that already exists with an ID, given as an instance with potential modification already applied to it.
    ///
    /// If `allow_insert` is set, we consider reinserting the instruction as it is, if it hasn't changed and cannot be simplified.
    /// Consider using this if we are re-inserting when we take instructions from a block and re-insert them into the same block.
    pub(crate) fn push_instruction_value(
        &mut self,
        instruction: Instruction,
        id: InstructionId,
        block: BasicBlockId,
        call_stack: CallStackId,
        allow_reinsert: bool,
    ) -> InsertInstructionResult<'_> {
        let results = self.function.dfg.instruction_results(id).to_vec();

        let ctrl_typevars = instruction
            .requires_ctrl_typevars()
            .then(|| vecmap(&results, |result| self.function.dfg.type_of_value(*result)));

        let new_results = self.function.dfg.insert_instruction_and_results_if_simplified(
            instruction,
            block,
            ctrl_typevars,
            call_stack,
            allow_reinsert.then_some(id),
        );

        Self::insert_new_instruction_results(&mut self.values, &results, &new_results);
        new_results
    }

    /// Modify the values HashMap to remember the mapping between an instruction result's previous
    /// ValueId (from the source_function) and its new ValueId in the destination function.
    fn insert_new_instruction_results(
        values: &mut HashMap<ValueId, ValueId>,
        old_results: &[ValueId],
        new_results: &InsertInstructionResult,
    ) {
        assert_eq!(old_results.len(), new_results.len());
        for i in 0..old_results.len() {
            if old_results[i] != new_results[i] {
                values.insert(old_results[i], new_results[i]);
            }
        }
    }

    /// Associates each block parameter with a value, unless the parameter already has a value,
    /// in which case it is kept as-is.
    pub(crate) fn remember_block_params(&mut self, block: BasicBlockId, new_values: &[ValueId]) {
        let old_parameters = self.function.dfg.block_parameters(block);
        assert_eq!(old_parameters.len(), new_values.len());
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
        assert_eq!(old_parameters.len(), new_parameters.len(),);
        for (param, new_param) in old_parameters.iter().zip(new_parameters) {
            // Don't overwrite any existing entries to avoid overwriting the induction variable
            self.values.entry(*param).or_insert(*new_param);
        }
    }

    /// Merge the internal mapping into the given mapping
    /// The merge is guaranteed to be coherent because ambiguous cases are prevented
    pub(crate) fn extract_mapping(&self, mapping: &mut HashMap<ValueId, ValueId>) {
        for (k, v) in &self.values {
            if mapping.contains_key(k) {
                unreachable!("cannot merge key");
            }
            if mapping.contains_key(v) {
                unreachable!("cannot merge value");
            }
            mapping.insert(*k, *v);
        }
    }

    pub(crate) fn set_mapping(&mut self, mapping: HashMap<ValueId, ValueId>) {
        self.values = mapping;
    }
}
