use std::collections::BTreeMap;

use acvm::acir::circuit::ErrorSelector;
use iter_extended::btree_map;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::ssa::ir::{
    function::{Function, FunctionId},
    value::Value,
};
use noirc_frontend::hir_def::types::Type as HirType;

use super::ValueId;

/// Contains the entire SSA representation of the program.
#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct Ssa {
    #[serde_as(as = "Vec<(_, _)>")]
    pub functions: BTreeMap<FunctionId, Function>,
    pub main_id: FunctionId,
    /// The id to give the next function added by [`Ssa::add_fn`].
    ///
    /// Not derivable from `functions`: `remove_unreachable_functions` and `inlining` both shrink
    /// that map, so the highest id in it can go down, while an id that has been handed out must
    /// never be handed out again.
    #[serde(skip)]
    next_id: u32,
    /// Maps SSA entry point function ID -> Final generated ACIR artifact index.
    /// There can be functions specified in SSA which do not act as ACIR entry points.
    /// This mapping is necessary to use the correct function pointer for an ACIR call,
    /// as the final program artifact will be a list of only entry point functions.
    #[serde(skip)]
    entry_point_to_generated_index: BTreeMap<FunctionId, u32>,
    /// Types of dynamic assertion messages, keyed by their error selector. Only dynamic messages
    /// are recorded here; static string messages are handled inline (see
    /// `FunctionContext::codegen_constrain_error`).
    ///
    /// We can skip serializing this field as the error selector types end up as part of the
    /// ABI not the actual SSA IR.
    #[serde(skip)]
    pub error_selector_to_type: BTreeMap<ErrorSelector, HirType>,
}

impl Ssa {
    /// Create a new Ssa object from the given SSA functions.
    /// The first function in this vector is expected to be the main function.
    pub fn new(functions: Vec<Function>, error_types: BTreeMap<ErrorSelector, HirType>) -> Self {
        let main_id = functions.first().expect("Expected at least 1 SSA function").id();
        let mut max_id = main_id;

        let functions = btree_map(functions, |f| {
            max_id = std::cmp::max(max_id, f.id());
            (f.id(), f)
        });

        Self {
            functions,
            main_id,
            next_id: max_id.to_u32() + 1,
            entry_point_to_generated_index: BTreeMap::new(),
            error_selector_to_type: error_types,
        }
    }

    /// Returns the entry-point function of the program
    pub(crate) fn main(&self) -> &Function {
        &self.functions[&self.main_id]
    }

    /// Returns the entry-point function of the program as a mutable reference
    #[cfg(test)]
    pub(crate) fn main_mut(&mut self) -> &mut Function {
        self.functions.get_mut(&self.main_id).expect("ICE: Ssa should have a main function")
    }

    /// Adds a new function to the program
    pub(crate) fn add_fn(
        &mut self,
        build_with_id: impl FnOnce(FunctionId) -> Function,
    ) -> FunctionId {
        // `next_id` is not serialized, so a deserialized `Ssa` starts it at zero. Taking the
        // highest id in use into account keeps a round-tripped program from minting an id that
        // collides with a function it already has.
        let highest_in_use = self.functions.keys().next_back().map_or(0, |id| id.to_u32() + 1);
        let new_id = FunctionId::new(self.next_id.max(highest_in_use));
        self.next_id = new_id.to_u32() + 1;
        let function = build_with_id(new_id);
        self.functions.insert(new_id, function);
        new_id
    }

    pub(crate) fn generate_entry_point_index(mut self) -> Self {
        let entry_points =
            self.functions.keys().filter(|function| self.is_entry_point(**function)).enumerate();
        self.entry_point_to_generated_index = btree_map(entry_points, |(i, id)| (*id, i as u32));
        self
    }

    pub(crate) fn get_entry_point_index(&self, func_id: &FunctionId) -> Option<u32> {
        // Ensure the map has been initialized
        assert!(
            !self.entry_point_to_generated_index.is_empty(),
            "Trying to read uninitialized entry point index"
        );
        self.entry_point_to_generated_index.get(func_id).copied()
    }

    pub(crate) fn is_entry_point(&self, function: FunctionId) -> bool {
        function == self.main_id || self.functions[&function].runtime().is_entry_point()
    }

    /// Collect all the global value IDs used per function.
    pub(crate) fn used_globals_in_functions(&self) -> HashMap<FunctionId, HashSet<ValueId>> {
        fn add_value_to_globals_if_global(
            function: &Function,
            value_id: ValueId,
            used_globals: &mut HashSet<ValueId>,
        ) {
            if !function.dfg.is_global(value_id) {
                return;
            }

            if !used_globals.insert(value_id) {
                return;
            }

            // If we found a new global, its value could be an instruction that points to other globals.
            let globals = &function.dfg.globals;
            if let Value::Instruction { instruction, .. } = globals[value_id] {
                let instruction = &globals[instruction];
                instruction.for_each_value(|value_id| {
                    add_value_to_globals_if_global(function, value_id, used_globals);
                });
            }
        }

        let mut used_globals = HashMap::default();

        for (function_id, function) in &self.functions {
            let mut used_globals_in_function = HashSet::default();

            for call_data in &function.dfg.data_bus.call_data {
                add_value_to_globals_if_global(
                    function,
                    call_data.array_id,
                    &mut used_globals_in_function,
                );
            }

            for block_id in function.reachable_blocks() {
                let block = &function.dfg[block_id];
                for instruction_id in block.instructions() {
                    let instruction = &function.dfg[*instruction_id];
                    instruction.for_each_value(|value_id| {
                        add_value_to_globals_if_global(
                            function,
                            value_id,
                            &mut used_globals_in_function,
                        );
                    });
                }

                block.unwrap_terminator().for_each_value(|value_id| {
                    add_value_to_globals_if_global(
                        function,
                        value_id,
                        &mut used_globals_in_function,
                    );
                });
            }

            used_globals.insert(*function_id, used_globals_in_function);
        }

        used_globals
    }
}

impl std::fmt::Display for Ssa {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.print_without_locations().fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use crate::ssa::ir::map::Id;

    use crate::ssa::ssa_gen::Ssa;
    use crate::ssa::{
        function_builder::FunctionBuilder,
        ir::{function::Function, instruction::BinaryOp, types::Type},
    };

    /// `next_id` is not serialized, so [`Ssa::add_fn`] has to work out where the free ids start
    /// from the program itself. Minting `f0` here would silently replace `main`.
    #[test]
    fn add_fn_after_deserialization_does_not_reuse_an_id() {
        let mut builder = FunctionBuilder::new("main".into(), Id::test_new(0));
        let v0 = builder.add_parameter(Type::field());
        builder.terminate_with_return(vec![v0]);
        let ssa = builder.finish();

        let serialized = serde_json::to_string(&ssa).unwrap();
        let mut ssa: Ssa = serde_json::from_str(&serialized).unwrap();

        let functions_before = ssa.functions.len();
        let new_id = ssa.add_fn(|id| Function::new("added".into(), id));

        assert!(!ssa.functions.keys().any(|id| *id == new_id && *id == ssa.main_id));
        assert_eq!(ssa.functions.len(), functions_before + 1);
        assert_eq!(ssa.functions[&ssa.main_id].name(), "main");
    }

    #[test]
    fn serialization_roundtrip() {
        let main_id = Id::test_new(0);

        // Compiling main
        let mut builder = FunctionBuilder::new("main".into(), main_id);
        let v0 = builder.add_parameter(Type::field());

        let one = builder.field_constant(1u128);
        let three = builder.field_constant(3u128);

        let v1 = builder.insert_binary(v0, BinaryOp::Add { unchecked: false }, one);
        let v2 = builder.insert_binary(v1, BinaryOp::Mul { unchecked: false }, three);
        builder.terminate_with_return(vec![v2]);

        let ssa = builder.finish();
        let serialized_ssa = &serde_json::to_string(&ssa).unwrap();
        let deserialized_ssa: Ssa = serde_json::from_str(serialized_ssa).unwrap();
        let actual_string = format!("{}", deserialized_ssa.print_without_locations());

        let expected_string = "acir(inline) fn main f0 {\n  \
        b0(v0: Field):\n    \
            v3 = add v0, Field 1\n    \
            v4 = mul v3, Field 3\n    \
            return v4\n\
        }\n";
        assert_eq!(actual_string, expected_string);
    }
}
