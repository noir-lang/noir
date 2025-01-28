pub(crate) mod brillig_gen;
pub(crate) mod brillig_ir;

use acvm::FieldElement;
use brillig_gen::{brillig_globals::convert_ssa_globals, constant_allocation::ConstantAllocation};
use brillig_ir::{artifact::LabelType, brillig_variable::BrilligVariable, registers::GlobalSpace};

use self::{
    brillig_gen::convert_ssa_function,
    brillig_ir::{
        artifact::{BrilligArtifact, Label},
        procedures::compile_procedure,
    },
};
use crate::ssa::{
    ir::{
        dfg::DataFlowGraph,
        function::{Function, FunctionId},
        types::NumericType,
        value::ValueId,
    },
    ssa_gen::Ssa,
};
use fxhash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::{borrow::Cow, collections::BTreeSet};

pub use self::brillig_ir::procedures::ProcedureId;

/// Context structure for the brillig pass.
/// It stores brillig-related data required for brillig generation.
#[derive(Default)]
pub struct Brillig {
    /// Maps SSA function labels to their brillig artifact
    ssa_function_to_brillig: HashMap<FunctionId, BrilligArtifact<FieldElement>>,
    globals: BrilligArtifact<FieldElement>,
    globals_memory_size: usize,
}

impl Brillig {
    /// Compiles a function into brillig and store the compilation artifacts
    pub(crate) fn compile(
        &mut self,
        func: &Function,
        enable_debug_trace: bool,
        globals: &HashMap<ValueId, BrilligVariable>,
        hoisted_global_constants: &HashMap<(FieldElement, NumericType), BrilligVariable>,
    ) {
        let obj = convert_ssa_function(func, enable_debug_trace, globals, hoisted_global_constants);
        self.ssa_function_to_brillig.insert(func.id(), obj);
    }

    /// Finds a brillig artifact by its label
    pub(crate) fn find_by_label(
        &self,
        function_label: Label,
    ) -> Option<Cow<BrilligArtifact<FieldElement>>> {
        match function_label.label_type {
            LabelType::Function(function_id, _) => {
                self.ssa_function_to_brillig.get(&function_id).map(Cow::Borrowed)
            }
            // Procedures are compiled as needed
            LabelType::Procedure(procedure_id) => Some(Cow::Owned(compile_procedure(procedure_id))),
            LabelType::GlobalInit => Some(Cow::Borrowed(&self.globals)),
            _ => unreachable!("ICE: Expected a function or procedure label"),
        }
    }
}

impl std::ops::Index<FunctionId> for Brillig {
    type Output = BrilligArtifact<FieldElement>;
    fn index(&self, id: FunctionId) -> &Self::Output {
        &self.ssa_function_to_brillig[&id]
    }
}

impl Ssa {
    /// Compile Brillig functions and ACIR functions reachable from them
    #[tracing::instrument(level = "trace", skip_all)]
    pub(crate) fn to_brillig(&self, enable_debug_trace: bool) -> Brillig {
        // Collect all the function ids that are reachable from brillig
        // That means all the functions marked as brillig and ACIR functions called by them
        let brillig_reachable_function_ids = self
            .functions
            .iter()
            .filter_map(|(id, func)| func.runtime().is_brillig().then_some(*id))
            .collect::<BTreeSet<_>>();

        let mut brillig = Brillig::default();

        if brillig_reachable_function_ids.is_empty() {
            return brillig;
        }

        // We can potentially have multiple local constants with the same value and type
        let mut hoisted_global_consts: HashMap<(FieldElement, NumericType), usize> =
            HashMap::default();
        for brillig_function_id in brillig_reachable_function_ids.iter() {
            let function = &self.functions[brillig_function_id];
            let constants = ConstantAllocation::from_function(function);
            for (constant, _) in constants.constant_usage {
                let value = function.dfg.get_numeric_constant(constant);
                let value = value.unwrap();
                let typ = function.dfg.type_of_value(constant);
                if !function.dfg.is_global(constant) {
                    hoisted_global_consts
                        .entry((value, typ.unwrap_numeric()))
                        .and_modify(|counter| *counter += 1)
                        .or_insert(1);
                }
            }
        }

        // We want to hoist only if there are repeat occurrences of a constant.
        let hoisted_global_consts = hoisted_global_consts
            .into_iter()
            .filter_map(
                |(value, num_occurrences)| {
                    if num_occurrences > 1 {
                        Some(value)
                    } else {
                        None
                    }
                },
            )
            .collect::<HashSet<_>>();

        // Globals are computed once at compile time and shared across all functions,
        // thus we can just fetch globals from the main function.
        let globals = (*self.functions[&self.main_id].dfg.globals).clone();
        let (artifact, brillig_globals, globals_size, hoisted_global_constants) =
            convert_ssa_globals(
                enable_debug_trace,
                globals,
                &self.used_global_values,
                &hoisted_global_consts,
            );
        brillig.globals = artifact;
        brillig.globals_memory_size = globals_size;

        for brillig_function_id in brillig_reachable_function_ids {
            let func = &self.functions[&brillig_function_id];
            brillig.compile(func, enable_debug_trace, &brillig_globals, &hoisted_global_constants);
        }

        brillig
    }
}
