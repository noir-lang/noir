//! This is a debugging pass which re-inserts each instruction
//! and block in a fresh DFG context for each function so that ValueIds,
//! BasicBlockIds, and FunctionIds are always identical for the same SSA code.
//!
//! During normal compilation this is often not the case since prior passes
//! may increase the ID counter so that later passes start at different offsets,
//! even if they contain the same SSA code.

use std::{collections::BTreeMap, sync::Arc};

use crate::ssa::{
    function_builder::data_bus::DatabusVisibility,
    ir::{
        basic_block::BasicBlockId,
        function::{Function, FunctionId},
        map::SparseMap,
        post_order::PostOrder,
        value::{Value, ValueId},
    },
    ssa_gen::Ssa,
};
use iter_extended::vecmap;
use itertools::Itertools;
use rustc_hash::FxHashMap as HashMap;

impl Ssa {
    /// Re-inserts each instruction and block in a fresh DFG context for each function so that
    /// ValueIds, BasicBlockIds, and FunctionIds are always identical for the same SSA code.
    pub fn normalize_ids(&mut self) {
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
        let Some(old_purities) = &functions.iter().next().map(|f| &f.1.dfg.function_purities)
        else {
            return;
        };
        let mut new_purities = HashMap::default();

        for (id, function) in functions {
            self.functions.insert_with_id(|new_id| {
                self.new_ids.function_ids.insert(*id, new_id);

                if let Some(purity) = old_purities.get(id) {
                    new_purities.insert(new_id, *purity);
                }

                Function::clone_signature(new_id, function)
            });
        }

        let new_purities = Arc::new(new_purities);
        for new_id in self.new_ids.function_ids.values() {
            self.functions[*new_id].dfg.set_function_purities(new_purities.clone());
        }
    }

    fn normalize_ids(&mut self, old_function: &mut Function) {
        self.new_ids.blocks.clear();
        self.new_ids.values.clear();

        let new_function_id = self.new_ids.function_ids[&old_function.id()];
        let new_function = &mut self.functions[new_function_id];

        for (_, value) in old_function.dfg.globals.values_iter() {
            new_function.dfg.make_global(value.get_type().into_owned());
        }

        let reachable_blocks = old_function.reachable_blocks();
        self.new_ids.populate_blocks(reachable_blocks, old_function, new_function);

        let reverse_post_order = PostOrder::with_function(old_function).into_vec_reverse();

        // Map each parameter, instruction, and terminator
        for old_block_id in reverse_post_order {
            let new_block_id = self.new_ids.blocks[&old_block_id];

            let old_block = &mut old_function.dfg[old_block_id];
            for old_instruction_id in old_block.take_instructions() {
                let instruction = old_function.dfg[old_instruction_id]
                    .map_values(|value| self.new_ids.map_value(new_function, old_function, value));

                let call_stack = old_function.dfg.get_instruction_call_stack_id(old_instruction_id);
                let locations = old_function.dfg.get_call_stack(call_stack);
                let new_call_stack =
                    new_function.dfg.call_stack_data.get_or_insert_locations(&locations);
                let old_results = old_function.dfg.instruction_results(old_instruction_id);

                let ctrl_typevars = instruction.requires_ctrl_typevars().then(|| {
                    vecmap(old_results, |result| {
                        old_function.dfg.type_of_value(*result).into_owned()
                    })
                });

                let new_results =
                    new_function.dfg.insert_instruction_and_results_without_simplification(
                        instruction,
                        new_block_id,
                        ctrl_typevars,
                        new_call_stack,
                    );

                for (old_result, new_result) in
                    old_results.iter().zip_eq(new_results.results().iter())
                {
                    let old_result = *old_result;
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
                new_function.dfg.call_stack_data.get_or_insert_locations(&locations);
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
        reachable_blocks: impl IntoIterator<Item = BasicBlockId>,
        old_function: &mut Function,
        new_function: &mut Function,
    ) {
        let old_entry = old_function.entry_block();
        self.blocks.insert(old_entry, new_function.entry_block());

        for old_id in reachable_blocks {
            if old_id != old_entry {
                let new_id = new_function.dfg.make_block();
                self.blocks.insert(old_id, new_id);
            }

            let new_id = self.blocks[&old_id];
            let old_block = &mut old_function.dfg[old_id];
            for old_parameter in old_block.take_parameters() {
                let typ = old_function.dfg.type_of_value(old_parameter).into_owned();
                let visibility = match &old_function.dfg[old_parameter] {
                    Value::Param { visibility, .. } => *visibility,
                    _ => DatabusVisibility::None,
                };
                let new_parameter =
                    new_function.dfg.add_block_parameter_with_visibility(new_id, typ, visibility);
                self.values.insert(old_parameter, new_parameter);
            }
        }
    }

    fn map_value(
        &self,
        new_function: &mut Function,
        old_function: &Function,
        old_value: ValueId,
    ) -> ValueId {
        if old_function.dfg.is_global(old_value) {
            // Globals are computed at compile-time and thus are expected to be remain normalized
            // between SSA passes
            return old_value;
        }
        match &old_function.dfg[old_value] {
            value @ Value::Instruction { instruction, .. } => {
                *self.values.get(&old_value).unwrap_or_else(|| {
                    let instruction = &old_function.dfg[*instruction];
                    unreachable!("Unmapped value with id {old_value}: {value:?}\n  from instruction: {instruction:?}, from function: {}", old_function.id())
                })
            }

            value @ Value::Param { .. } => {
                *self.values.get(&old_value).unwrap_or_else(|| {
                    unreachable!("Unmapped value with id {old_value}: {value:?}")
                })
            }

            Value::Function(id) => {
                let new_id = *self.function_ids.get(id).unwrap_or_else(|| {
                    unreachable!("Unmapped function with id {id}")
                });
                new_function.dfg.import_function(new_id)
            }

            Value::NumericConstant { constant, typ } => {
                new_function.dfg.make_constant(*constant, *typ)
            }
            Value::Intrinsic(intrinsic) => new_function.dfg.import_intrinsic(*intrinsic),
            Value::ForeignFunction { name, pure } => {
                new_function.dfg.import_foreign_function(name, *pure)
            }
            Value::Global(_) => {
                unreachable!("Should have handled the global case already");
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use acvm::FieldElement;

    use crate::ssa::{
        function_builder::{FunctionBuilder, data_bus::DatabusVisibility},
        ir::{function::FunctionId, types::Type, value::Value},
    };

    /// Check`call_data` visibility is preserved when
    /// rebuilding a function in `normalize_ids`.
    #[test]
    fn normalize_ids_preserves_databus_visibility_on_main_entry_params() {
        let main_id = FunctionId::test_new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);

        // Entry-block param is a call_data(0)
        let param =
            builder.add_parameter_with_visibility(Type::field(), DatabusVisibility::CallData(0));
        let other = builder.add_parameter(Type::field());

        // make_array [call_data_param, other] : [Field; 2]
        // (mirrors how the databus aggregator builds the call_data)
        let arr_typ = Type::Array(
            std::sync::Arc::new(vec![Type::field()]),
            acvm::acir::brillig::lengths::SemanticLength(2),
        );
        let arr = builder.insert_make_array(im::vector![param, other], arr_typ);
        // array_get arr, 0 -> Field:
        let zero = builder.length_constant(FieldElement::from(0_u128));
        let read = builder.insert_array_get(arr, zero, Type::field());
        builder.terminate_with_return(vec![read]);

        let mut ssa = builder.finish();
        ssa.normalize_ids();

        let main = ssa.main();
        let entry_params = main.dfg.block_parameters(main.entry_block());
        assert_eq!(entry_params.len(), 2, "expected two entry-block params");

        match &main.dfg[entry_params[0]] {
            Value::Param { visibility, .. } => {
                assert_eq!(
                    *visibility,
                    DatabusVisibility::CallData(0),
                    "normalize_ids dropped the call_data visibility on the entry-block param",
                );
            }
            other => panic!("entry-block param was not a Value::Param: {other:?}"),
        }
        match &main.dfg[entry_params[1]] {
            Value::Param { visibility, .. } => {
                assert_eq!(
                    *visibility,
                    DatabusVisibility::None,
                    "non-databus param visibility should remain None after normalize_ids",
                );
            }
            other => panic!("entry-block param was not a Value::Param: {other:?}"),
        }
    }
}
