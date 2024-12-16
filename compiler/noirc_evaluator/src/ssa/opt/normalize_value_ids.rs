use std::collections::BTreeMap;

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        function::{Function, FunctionId},
        map::SparseMap,
        post_order::PostOrder,
        value::{Value, ValueId},
    },
    ssa_gen::Ssa,
};
use fxhash::FxHashMap as HashMap;
use iter_extended::vecmap;

impl Ssa {
    /// This is a debugging pass which re-inserts each instruction
    /// and block in a fresh DFG context for each function so that ValueIds,
    /// BasicBlockIds, and FunctionIds are always identical for the same SSA code.
    ///
    /// During normal compilation this is often not the case since prior passes
    /// may increase the ID counter so that later passes start at different offsets,
    /// even if they contain the same SSA code.
    pub(crate) fn normalize_ids(&mut self) {
        let mut context = Context::default();
        context.populate_functions(&self.functions);
        for function in self.functions.values_mut() {
            context.normalize_ids(function);
        }
        self.functions = context.functions.into_btree();
    }
}

#[derive(Default)]
struct Context {
    functions: SparseMap<Function>,

    new_ids: IdMaps,
}

/// Maps from old ids to new ones.
/// Separate from the rest of Context so we can call mutable methods on it
/// while Context gives out mutable references to functions within.
#[derive(Default)]
struct IdMaps {
    // Maps old function id -> new function id
    function_ids: HashMap<FunctionId, FunctionId>,

    // Maps old block id -> new block id
    // Cleared in between each function.
    blocks: HashMap<BasicBlockId, BasicBlockId>,

    // Maps old value id -> new value id
    // Cleared in between each function.
    values: HashMap<ValueId, ValueId>,
}

impl Context {
    fn populate_functions(&mut self, functions: &BTreeMap<FunctionId, Function>) {
        for (id, function) in functions {
            self.functions.insert_with_id(|new_id| {
                self.new_ids.function_ids.insert(*id, new_id);
                Function::clone_signature(new_id, function)
            });
        }
    }

    fn normalize_ids(&mut self, old_function: &mut Function) {
        self.new_ids.blocks.clear();
        self.new_ids.values.clear();

        let new_function_id = self.new_ids.function_ids[&old_function.id()];
        let new_function = &mut self.functions[new_function_id];

        let mut reachable_blocks = PostOrder::with_function(old_function).into_vec();
        reachable_blocks.reverse();

        self.new_ids.populate_blocks(&reachable_blocks, old_function, new_function);

        // Map each parameter, instruction, and terminator
        for old_block_id in reachable_blocks {
            let new_block_id = self.new_ids.blocks[&old_block_id];

            let old_block = &mut old_function.dfg[old_block_id];
            for old_instruction_id in old_block.take_instructions() {
                let instruction = old_function.dfg[old_instruction_id]
                    .map_values(|value| self.new_ids.map_value(new_function, old_function, value));

                let call_stack = old_function.dfg.get_instruction_call_stack_id(old_instruction_id);
                let locations = old_function.dfg.get_call_stack(call_stack);
                let new_call_stack =
                    new_function.dfg.call_stack_data.get_or_insert_locations(locations);
                let old_results = old_function.dfg.instruction_results(old_instruction_id);

                let ctrl_typevars = instruction
                    .requires_ctrl_typevars()
                    .then(|| vecmap(old_results, |result| old_function.dfg.type_of_value(*result)));

                let new_results = new_function.dfg.insert_instruction_and_results(
                    instruction,
                    new_block_id,
                    ctrl_typevars,
                    new_call_stack,
                );

                assert_eq!(old_results.len(), new_results.len());
                for (old_result, new_result) in old_results.iter().zip(new_results.results().iter())
                {
                    let old_result = old_function.dfg.resolve(*old_result);
                    self.new_ids.values.insert(old_result, *new_result);
                }
            }

            let old_block = &mut old_function.dfg[old_block_id];
            let mut terminator = old_block.take_terminator();
            terminator
                .map_values_mut(|value| self.new_ids.map_value(new_function, old_function, value));

            terminator.mutate_blocks(|old_block| self.new_ids.blocks[&old_block]);
            let locations = old_function.dfg.get_call_stack(terminator.call_stack());
            let new_call_stack =
                new_function.dfg.call_stack_data.get_or_insert_locations(locations);
            terminator.set_call_stack(new_call_stack);
            new_function.dfg.set_block_terminator(new_block_id, terminator);
        }

        // Also map the values in the databus
        let old_databus = &old_function.dfg.data_bus;
        new_function.dfg.data_bus = old_databus
            .map_values(|old_value| self.new_ids.map_value(new_function, old_function, old_value));
    }
}

impl IdMaps {
    fn populate_blocks(
        &mut self,
        reachable_blocks: &[BasicBlockId],
        old_function: &mut Function,
        new_function: &mut Function,
    ) {
        let old_entry = old_function.entry_block();
        self.blocks.insert(old_entry, new_function.entry_block());

        for old_id in reachable_blocks {
            if *old_id != old_entry {
                let new_id = new_function.dfg.make_block();
                self.blocks.insert(*old_id, new_id);
            }

            let new_id = self.blocks[old_id];
            let old_block = &mut old_function.dfg[*old_id];
            for old_parameter in old_block.take_parameters() {
                let old_parameter = old_function.dfg.resolve(old_parameter);
                let typ = old_function.dfg.type_of_value(old_parameter);
                let new_parameter = new_function.dfg.add_block_parameter(new_id, typ);
                self.values.insert(old_parameter, new_parameter);
            }
        }
    }

    fn map_value(
        &mut self,
        new_function: &mut Function,
        old_function: &Function,
        old_value: ValueId,
    ) -> ValueId {
        let old_value = old_function.dfg.resolve(old_value);
        match &old_function.dfg[old_value] {
            value @ Value::Instruction { instruction, .. } => {
                *self.values.get(&old_value).unwrap_or_else(|| {
                    let instruction = &old_function.dfg[*instruction];
                    unreachable!("Unmapped value with id {old_value}: {value:?}\n  from instruction: {instruction:?}, SSA: {old_function}")
                })
            }

            value @ Value::Param { .. } => {
                *self.values.get(&old_value).unwrap_or_else(|| {
                    unreachable!("Unmapped value with id {old_value}: {value:?}")
                })
            }

            Value::Function(id) => {
                let new_id = self.function_ids[id];
                new_function.dfg.import_function(new_id)
            }

            Value::NumericConstant { constant, typ } => {
                new_function.dfg.make_constant(*constant, *typ)
            }
            Value::Intrinsic(intrinsic) => new_function.dfg.import_intrinsic(*intrinsic),
            Value::ForeignFunction(name) => new_function.dfg.import_foreign_function(name),
        }
    }
}
