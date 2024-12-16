use iter_extended::vecmap;

use crate::ssa::ir::types::Type;

use super::{
    basic_block::BasicBlockId,
    call_stack::CallStackId,
    dfg::InsertInstructionResult,
    function::Function,
    instruction::{Instruction, InstructionId},
    value::ValueId,
};
use fxhash::FxHashMap as HashMap;

/// The FunctionInserter can be used to help modify existing Functions
/// and map old values to new values after re-inserting optimized versions
/// of old instructions.
pub(crate) struct FunctionInserter<'f> {
    pub(crate) function: &'f mut Function,

    values: HashMap<ValueId, ValueId>,

    /// Map containing repeat array constants so that we do not initialize a new
    /// array unnecessarily. An extra tuple field is included as part of the key to
    /// distinguish between array/slice types.
    ///
    /// This is optional since caching arrays relies on the inserter inserting strictly
    /// in control-flow order. Otherwise, if arrays later in the program are cached first,
    /// they may be referred to by instructions earlier in the program.
    array_cache: Option<ArrayCache>,

    /// If this pass is loop unrolling, store the block before the loop to optionally
    /// hoist any make_array instructions up to after they are retrieved from the `array_cache`.
    pre_loop: Option<BasicBlockId>,
}

pub(crate) type ArrayCache = HashMap<im::Vector<ValueId>, HashMap<Type, ValueId>>;

impl<'f> FunctionInserter<'f> {
    pub(crate) fn new(function: &'f mut Function) -> FunctionInserter<'f> {
        Self { function, values: HashMap::default(), array_cache: None, pre_loop: None }
    }

    /// Resolves a ValueId to its new, updated value.
    /// If there is no updated value for this id, this returns the same
    /// ValueId that was passed in.
    pub(crate) fn resolve(&mut self, mut value: ValueId) -> ValueId {
        value = self.function.dfg.resolve(value);
        match self.values.get(&value) {
            Some(value) => self.resolve(*value),
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

    /// Get an instruction and make sure all the values in it are freshly resolved.
    pub(crate) fn map_instruction(&mut self, id: InstructionId) -> (Instruction, CallStackId) {
        let mut instruction = self.function.dfg[id].clone();
        instruction.map_values_mut(|id| self.resolve(id));
        (instruction, self.function.dfg.get_instruction_call_stack_id(id))
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
        mut block: BasicBlockId,
        call_stack: CallStackId,
    ) -> InsertInstructionResult {
        let results = self.function.dfg.instruction_results(id);
        let results = vecmap(results, |id| self.function.dfg.resolve(*id));

        let ctrl_typevars = instruction
            .requires_ctrl_typevars()
            .then(|| vecmap(&results, |result| self.function.dfg.type_of_value(*result)));

        // Large arrays can lead to OOM panics if duplicated from being unrolled in loops.
        // To prevent this, try to reuse the same ID for identical arrays instead of inserting
        // another MakeArray instruction. Note that this assumes the function inserter is inserting
        // in control-flow order. Otherwise we could refer to ValueIds defined later in the program.
        let make_array = if let Instruction::MakeArray { elements, typ } = &instruction {
            if self.array_is_constant(elements) && self.function.runtime().is_acir() {
                if let Some(fetched_value) = self.get_cached_array(elements, typ) {
                    assert_eq!(results.len(), 1);
                    self.values.insert(results[0], fetched_value);
                    return InsertInstructionResult::SimplifiedTo(fetched_value);
                }

                // Hoist constant arrays out of the loop and cache their value
                if let Some(pre_loop) = self.pre_loop {
                    block = pre_loop;
                }
                Some((elements.clone(), typ.clone()))
            } else {
                None
            }
        } else {
            None
        };

        let new_results = self.function.dfg.insert_instruction_and_results(
            instruction,
            block,
            ctrl_typevars,
            call_stack,
        );

        // Cache an array in the fresh_array_cache if array caching is enabled.
        // The fresh cache isn't used for deduplication until an external pass confirms we
        // pass a sequence point and all blocks that may be before the current insertion point
        // are finished.
        if let Some((elements, typ)) = make_array {
            Self::cache_array(&mut self.array_cache, elements, typ, new_results.first());
        }

        Self::insert_new_instruction_results(&mut self.values, &results, &new_results);
        new_results
    }

    fn get_cached_array(&self, elements: &im::Vector<ValueId>, typ: &Type) -> Option<ValueId> {
        self.array_cache.as_ref()?.get(elements)?.get(typ).copied()
    }

    fn cache_array(
        arrays: &mut Option<ArrayCache>,
        elements: im::Vector<ValueId>,
        typ: Type,
        result_id: ValueId,
    ) {
        if let Some(arrays) = arrays {
            arrays.entry(elements).or_default().insert(typ, result_id);
        }
    }

    fn array_is_constant(&self, elements: &im::Vector<ValueId>) -> bool {
        elements.iter().all(|element| self.function.dfg.is_constant(*element))
    }

    pub(crate) fn set_array_cache(
        &mut self,
        new_cache: Option<ArrayCache>,
        pre_loop: BasicBlockId,
    ) {
        self.array_cache = new_cache;
        self.pre_loop = Some(pre_loop);
    }

    /// Finish this inserter, returning its array cache merged with the fresh array cache.
    /// Since this consumes the inserter this assumes we're at a sequence point where all
    /// predecessor blocks to the current block are finished. Since this is true, the fresh
    /// array cache can be merged with the existing array cache.
    pub(crate) fn into_array_cache(self) -> Option<ArrayCache> {
        self.array_cache
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
