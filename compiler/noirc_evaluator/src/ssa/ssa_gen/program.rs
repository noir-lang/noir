use std::{collections::BTreeMap, fmt::Display};

use acvm::acir::circuit::ErrorSelector;
use iter_extended::btree_map;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::ssa::ir::{
    function::{Function, FunctionId, RuntimeType},
    map::AtomicCounter,
};
use noirc_frontend::hir_def::types::Type as HirType;

/// Contains the entire SSA representation of the program.
#[serde_as]
#[derive(Serialize, Deserialize)]
pub(crate) struct Ssa {
    #[serde_as(as = "Vec<(_, _)>")]
    pub(crate) functions: BTreeMap<FunctionId, Function>,
    pub(crate) main_id: FunctionId,
    #[serde(skip)]
    pub(crate) next_id: AtomicCounter<Function>,
    /// Maps SSA entry point function ID -> Final generated ACIR artifact index.
    /// There can be functions specified in SSA which do not act as ACIR entry points.
    /// This mapping is necessary to use the correct function pointer for an ACIR call,
    /// as the final program artifact will be a list of only entry point functions.
    #[serde(skip)]
    pub(crate) entry_point_to_generated_index: BTreeMap<FunctionId, u32>,
    // We can skip serializing this field as the error selector types end up as part of the
    // ABI not the actual SSA IR.
    #[serde(skip)]
    pub(crate) error_selector_to_type: BTreeMap<ErrorSelector, HirType>,
}

impl Ssa {
    /// Create a new Ssa object from the given SSA functions.
    /// The first function in this vector is expected to be the main function.
    pub(crate) fn new(
        functions: Vec<Function>,
        error_types: BTreeMap<ErrorSelector, HirType>,
    ) -> Self {
        let main_id = functions.first().expect("Expected at least 1 SSA function").id();
        let mut max_id = main_id;

        let functions = btree_map(functions, |f| {
            max_id = std::cmp::max(max_id, f.id());
            (f.id(), f)
        });

        let entry_point_to_generated_index = btree_map(
            functions
                .iter()
                .filter(|(_, func)| {
                    let runtime = func.runtime();
                    match func.runtime() {
                        RuntimeType::Acir(_) => runtime.is_entry_point() || func.id() == main_id,
                        RuntimeType::Brillig => false,
                    }
                })
                .enumerate(),
            |(i, (id, _))| (*id, i as u32),
        );

        Self {
            functions,
            main_id,
            next_id: AtomicCounter::starting_after(max_id),
            entry_point_to_generated_index,
            error_selector_to_type: error_types,
        }
    }

    /// Returns the entry-point function of the program
    pub(crate) fn main(&self) -> &Function {
        &self.functions[&self.main_id]
    }

    /// Returns the entry-point function of the program as a mutable reference
    pub(crate) fn main_mut(&mut self) -> &mut Function {
        self.functions.get_mut(&self.main_id).expect("ICE: Ssa should have a main function")
    }

    /// Adds a new function to the program
    pub(crate) fn add_fn(
        &mut self,
        build_with_id: impl FnOnce(FunctionId) -> Function,
    ) -> FunctionId {
        let new_id = self.next_id.next();
        let function = build_with_id(new_id);
        self.functions.insert(new_id, function);
        new_id
    }

    /// Clones an already existing function with a fresh id
    pub(crate) fn clone_fn(&mut self, existing_function_id: FunctionId) -> FunctionId {
        let new_id = self.next_id.next();
        let function = Function::clone_with_id(new_id, &self.functions[&existing_function_id]);
        self.functions.insert(new_id, function);
        new_id
    }
}

impl Display for Ssa {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for function in self.functions.values() {
            writeln!(f, "{function}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::ssa::ir::map::Id;

    use crate::ssa::ssa_gen::Ssa;
    use crate::ssa::{
        function_builder::FunctionBuilder,
        ir::{instruction::BinaryOp, types::Type},
    };

    #[test]
    fn serialization_roundtrip() {
        let main_id = Id::test_new(0);

        // Compiling main
        let mut builder = FunctionBuilder::new("main".into(), main_id);
        let v0 = builder.add_parameter(Type::field());

        let one = builder.field_constant(1u128);
        let three = builder.field_constant(3u128);

        let v1 = builder.insert_binary(v0, BinaryOp::Add, one);
        let v2 = builder.insert_binary(v1, BinaryOp::Mul, three);
        builder.terminate_with_return(vec![v2]);

        let ssa = builder.finish();
        let serialized_ssa = &serde_json::to_string(&ssa).unwrap();
        let deserialized_ssa: Ssa = serde_json::from_str(serialized_ssa).unwrap();
        let actual_string = format!("{}", deserialized_ssa);

        let expected_string = "acir(inline) fn main f0 {\n  \
        b0(v0: Field):\n    \
            v3 = add v0, Field 1\n    \
            v4 = mul v3, Field 3\n    \
            return v4\n\
        }\n";
        assert_eq!(actual_string, expected_string);
    }
}
