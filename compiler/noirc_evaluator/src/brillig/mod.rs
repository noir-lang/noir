pub(crate) mod brillig_gen;
pub(crate) mod brillig_ir;

use acvm::FieldElement;
use brillig_gen::{
    brillig_block::BrilligBlock, brillig_fn::FunctionContext, brillig_globals::BrilligGlobals,
};
use brillig_ir::{
    artifact::LabelType, brillig_variable::BrilligVariable, registers::GlobalSpace, BrilligContext,
};
use noirc_errors::call_stack::CallStackHelper;

use self::brillig_ir::{
    artifact::{BrilligArtifact, Label},
    procedures::compile_procedure,
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

/// Context structure for the brillig pass.
/// It stores brillig-related data required for brillig generation.
#[derive(Default)]
pub struct Brillig {
    /// Maps SSA function labels to their brillig artifact
    ssa_function_to_brillig: HashMap<FunctionId, BrilligArtifact<FieldElement>>,
    pub call_stacks: CallStackHelper,
    globals: HashMap<FunctionId, BrilligArtifact<FieldElement>>,
    globals_memory_size: HashMap<FunctionId, usize>,
}

impl Brillig {
    /// Compiles a function into brillig and store the compilation artifacts
    pub(crate) fn compile(
        &mut self,
        func: &Function,
        enable_debug_trace: bool,
        globals: &HashMap<ValueId, BrilligVariable>,
    ) {
        let obj = self.convert_ssa_function(func, enable_debug_trace, globals);
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
            LabelType::GlobalInit(function_id) => self.globals.get(&function_id).map(Cow::Borrowed),
            _ => unreachable!("ICE: Expected a function or procedure label"),
        }
    }

    /// Converting an SSA function into Brillig bytecode.
    pub(crate) fn convert_ssa_function(
        &mut self,
        func: &Function,
        enable_debug_trace: bool,
        globals: &HashMap<ValueId, BrilligVariable>,
    ) -> BrilligArtifact<FieldElement> {
        let mut brillig_context = BrilligContext::new(enable_debug_trace);

        let mut function_context = FunctionContext::new(func);

        brillig_context.enter_context(Label::function(func.id()));

        brillig_context.call_check_max_stack_depth_procedure();

        for block in function_context.blocks.clone() {
            BrilligBlock::compile(
                &mut function_context,
                &mut brillig_context,
                block,
                &func.dfg,
                &mut self.call_stacks,
                globals,
            );
        }
        let mut artifact = brillig_context.artifact();
        artifact.name = func.name().to_string();
        artifact.location_tree = func.dfg.call_stack_data.to_location_tree();

        artifact
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
    pub(crate) fn to_brillig(&self, enable_debug_trace: bool) -> Brillig {
        self.to_brillig_with_globals(enable_debug_trace, HashMap::default())
    }

    /// Compile Brillig functions and ACIR functions reachable from them
    #[tracing::instrument(level = "trace", skip_all)]
    pub(crate) fn to_brillig_with_globals(
        &self,
        enable_debug_trace: bool,
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
        brillig_globals.declare_globals(&globals_dfg, &mut brillig, enable_debug_trace);

        for brillig_function_id in brillig_reachable_function_ids {
            let globals_allocations = brillig_globals.get_brillig_globals(brillig_function_id);

            let func = &self.functions[&brillig_function_id];
            brillig.compile(func, enable_debug_trace, &globals_allocations);
        }

        brillig
    }
}
