pub(crate) mod brillig_gen;
pub(crate) mod brillig_ir;

use acvm::FieldElement;
use brillig_gen::brillig_globals::BrilligGlobals;
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
        instruction::Instruction,
        value::{Value, ValueId},
    },
    opt::inlining::called_functions_vec,
    ssa_gen::Ssa,
};
use fxhash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::{borrow::Cow, collections::BTreeSet};

pub use self::brillig_ir::procedures::ProcedureId;

/// Options that affect Brillig code generation.
#[derive(Default, Clone, Debug)]
pub struct BrilligOptions {
    pub enable_debug_trace: bool,
    pub enable_debug_assertions: bool,
}

/// Context structure for the brillig pass.
/// It stores brillig-related data required for brillig generation.
#[derive(Default)]
pub struct Brillig {
    /// Maps SSA function labels to their brillig artifact
    ssa_function_to_brillig: HashMap<FunctionId, BrilligArtifact<FieldElement>>,
    globals: HashMap<FunctionId, BrilligArtifact<FieldElement>>,
    globals_memory_size: HashMap<FunctionId, usize>,
}

impl Brillig {
    /// Compiles a function into brillig and store the compilation artifacts
    pub(crate) fn compile(
        &mut self,
        func: &Function,
        options: &BrilligOptions,
        globals: &HashMap<ValueId, BrilligVariable>,
    ) {
        let obj = convert_ssa_function(func, options, globals);
        self.ssa_function_to_brillig.insert(func.id(), obj);
    }

    /// Finds a brillig artifact by its label
    pub(crate) fn find_by_label(
        &self,
        function_label: Label,
        options: &BrilligOptions,
    ) -> Option<Cow<BrilligArtifact<FieldElement>>> {
        match function_label.label_type {
            LabelType::Function(function_id, _) => {
                self.ssa_function_to_brillig.get(&function_id).map(Cow::Borrowed)
            }
            // Procedures are compiled as needed
            LabelType::Procedure(procedure_id) => {
                Some(Cow::Owned(compile_procedure(procedure_id, options)))
            }
            LabelType::GlobalInit(function_id) => self.globals.get(&function_id).map(Cow::Borrowed),
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
    #[tracing::instrument(level = "trace", skip_all)]
    pub(crate) fn to_brillig(&self, options: &BrilligOptions) -> Brillig {
        self.to_brillig_with_globals(options, HashMap::default())
    }

    /// Compile Brillig functions and ACIR functions reachable from them
    #[tracing::instrument(level = "trace", skip_all)]
    pub(crate) fn to_brillig_with_globals(
        &self,
        options: &BrilligOptions,
        used_globals_map: HashMap<FunctionId, HashSet<ValueId>>,
    ) -> Brillig {
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

        let mut brillig_globals =
            BrilligGlobals::new(&self.functions, used_globals_map, self.main_id);

        // SSA Globals are computed once at compile time and shared across all functions,
        // thus we can just fetch globals from the main function.
        // This same globals graph will then be used to declare Brillig globals for the respective entry points.
        let globals = (*self.functions[&self.main_id].dfg.globals).clone();
        let globals_dfg = DataFlowGraph::from(globals);
        brillig_globals.declare_globals(&globals_dfg, &mut brillig, options);

        for brillig_function_id in brillig_reachable_function_ids {
            let globals_allocations = brillig_globals.get_brillig_globals(brillig_function_id);

            let func = &self.functions[&brillig_function_id];
            brillig.compile(func, options, &globals_allocations);
        }

        brillig
    }
}
