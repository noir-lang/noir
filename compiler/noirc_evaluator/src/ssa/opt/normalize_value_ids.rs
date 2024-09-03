use std::collections::BTreeMap;

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        function::{Function, FunctionId},
        map::SparseMap,
        value::{Value, ValueId}, post_order::PostOrder,
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

    // Maps old function id -> new function id
    function_ids: HashMap<FunctionId, FunctionId>,

    per_function_context: PerFunctionContext,
}

#[derive(Default)]
struct PerFunctionContext {
    // Maps old block id -> new block id
    blocks: HashMap<BasicBlockId, BasicBlockId>,

    // Maps old value id -> new value id
    values: HashMap<ValueId, ValueId>,
}

impl Context {
    fn populate_functions(&mut self, functions: &BTreeMap<FunctionId, Function>) {
        for (id, function) in functions {
            self.functions.insert_with_id(|new_id| {
                self.function_ids.insert(*id, new_id);
                Function::new(function.name().to_string(), new_id)
            });
        }
    }

    fn normalize_ids(&mut self, old_function: &mut Function) {
        self.per_function_context.clear();

        eprintln!("Working on function:\n{old_function}");

        let new_function_id = self.function_ids[&old_function.id()];
        let new_function = &mut self.functions[new_function_id];

        let mut reachable_blocks = PostOrder::with_function(old_function).into_vec();
        reachable_blocks.reverse();

        let old_entry = old_function.entry_block();
        self.per_function_context.populate_blocks(&reachable_blocks, old_entry, old_function, new_function);

        // Map each parameter, instruction, and terminator
        for old_block_id in reachable_blocks {
            let new_block_id = self.per_function_context.blocks[&old_block_id];
            eprintln!("On block {old_block_id}");

            let old_block = &mut old_function.dfg[old_block_id];
            for old_instruction_id in old_block.take_instructions() {
                let instruction = old_function.dfg[old_instruction_id].map_values(|value| {
                    self.per_function_context.map_value(new_function, old_function, value)
                });

                let call_stack = old_function.dfg.get_call_stack(old_instruction_id);
                let old_results = old_function.dfg.instruction_results(old_instruction_id);

                let ctrl_typevars = instruction
                    .requires_ctrl_typevars()
                    .then(|| vecmap(old_results, |result| old_function.dfg.type_of_value(*result)));

                let new_results = new_function.dfg.insert_instruction_and_results(
                    instruction,
                    new_block_id,
                    ctrl_typevars,
                    call_stack,
                );

                assert_eq!(old_results.len(), new_results.len());
                for (old_result, new_result) in old_results.iter().zip(new_results.results().iter())
                {
                    let old_result = old_function.dfg.resolve(*old_result);
                    self.per_function_context.values.insert(old_result, *new_result);
                }
            }

            let old_block = &mut old_function.dfg[old_block_id];
            let terminator = old_block.take_terminator().map_values(|value| {
                self.per_function_context.map_value(new_function, old_function, value)
            });
            new_function.dfg.set_block_terminator(new_block_id, terminator);
        }
    }
}

impl PerFunctionContext {
    fn clear(&mut self) {
        self.blocks.clear();
        self.values.clear();
    }

    fn populate_blocks(
        &mut self,
        reachable_blocks: &[BasicBlockId],
        entry: BasicBlockId,
        old_function: &mut Function,
        new_function: &mut Function,
    ) {
        self.blocks.insert(entry, new_function.entry_block());

        for old_id in reachable_blocks {
            if *old_id != entry {
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

            Value::Function(id) => new_function.dfg.import_function(*id),

            Value::NumericConstant { constant, typ } => {
                new_function.dfg.make_constant(*constant, typ.clone())
            }
            Value::Array { array, typ } => {
                if let Some(value) = self.values.get(&old_value) {
                    return *value;
                }

                let array = array
                    .iter()
                    .map(|value| self.map_value(new_function, old_function, *value))
                    .collect();
                let new_value = new_function.dfg.make_array(array, typ.clone());
                self.values.insert(old_value, new_value);
                new_value
            }
            Value::Intrinsic(intrinsic) => new_function.dfg.import_intrinsic(*intrinsic),
            Value::ForeignFunction(name) => new_function.dfg.import_foreign_function(name),
        }
    }
}
