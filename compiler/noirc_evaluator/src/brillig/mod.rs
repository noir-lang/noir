pub(crate) mod brillig_gen;
pub mod brillig_ir;

use acvm::FieldElement;
use brillig_gen::brillig_block::BrilligBlock;
use brillig_gen::constant_allocation::ConstantAllocation;
use brillig_gen::{brillig_fn::FunctionContext, brillig_globals::BrilligGlobals};
use brillig_ir::BrilligContext;
use brillig_ir::{artifact::LabelType, brillig_variable::BrilligVariable, registers::GlobalSpace};
use noirc_errors::call_stack::CallStackHelper;

use self::brillig_ir::{
    artifact::{BrilligArtifact, Label},
    procedures::compile_procedure,
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
use fxhash::FxHashMap as HashMap;
use std::{borrow::Cow, collections::BTreeSet};

pub use self::brillig_ir::procedures::ProcedureId;

/// Options that affect Brillig code generation.
#[derive(Default, Clone, Debug)]
pub struct BrilligOptions {
    pub enable_debug_trace: bool,
    pub enable_debug_assertions: bool,
    pub enable_array_copy_counter: bool,
}

/// Context structure for the brillig pass.
/// It stores brillig-related data required for brillig generation.
#[derive(Default, Clone)]
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
        options: &BrilligOptions,
        globals: &HashMap<ValueId, BrilligVariable>,
        hoisted_global_constants: &HashMap<(FieldElement, NumericType), BrilligVariable>,
        is_entry_point: bool,
    ) {
        let obj = self.convert_ssa_function(
            func,
            options,
            globals,
            hoisted_global_constants,
            is_entry_point,
        );
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

    /// Converting an SSA function into Brillig bytecode.
    pub(crate) fn convert_ssa_function(
        &mut self,
        func: &Function,
        options: &BrilligOptions,
        globals: &HashMap<ValueId, BrilligVariable>,
        hoisted_global_constants: &HashMap<(FieldElement, NumericType), BrilligVariable>,
        is_entry_point: bool,
    ) -> BrilligArtifact<FieldElement> {
        let mut brillig_context = BrilligContext::new(options);

        let mut function_context = FunctionContext::new(func, is_entry_point);

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
                hoisted_global_constants,
            );
        }

        let mut artifact = brillig_context.artifact();
        artifact.name = func.name().to_string();
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
    /// Compile Brillig functions and ACIR functions reachable from them
    #[tracing::instrument(level = "trace", skip_all)]
    pub fn to_brillig(&self, options: &BrilligOptions) -> Brillig {
        let used_globals_map = self.used_globals_in_brillig_functions();

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
            let empty_allocations = HashMap::default();
            let empty_const_allocations = HashMap::default();
            let (globals_allocations, hoisted_constant_allocations) = brillig_globals
                .get_brillig_globals(brillig_function_id)
                .unwrap_or((&empty_allocations, &empty_const_allocations));

            let func = &self.functions[&brillig_function_id];
            let is_entry_point = brillig_globals.entry_points().contains_key(&brillig_function_id);

            brillig.compile(
                func,
                options,
                globals_allocations,
                hoisted_constant_allocations,
                is_entry_point,
            );
        }

        brillig
    }
}
