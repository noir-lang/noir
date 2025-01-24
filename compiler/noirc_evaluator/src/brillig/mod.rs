pub(crate) mod brillig_gen;
pub(crate) mod brillig_ir;

use acvm::FieldElement;
use brillig_gen::brillig_globals::convert_ssa_globals;
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
    ssa_gen::Ssa,
    opt::inlining::called_functions,
};
use fxhash::FxHashMap as HashMap;
use std::{borrow::Cow, collections::BTreeSet};

pub use self::brillig_ir::procedures::ProcedureId;

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
        enable_debug_trace: bool,
        globals: &HashMap<ValueId, BrilligVariable>,
    ) {
        let obj = convert_ssa_function(func, enable_debug_trace, globals);
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
            LabelType::GlobalInit(function_id) => {
                self.globals.get(&function_id).map(Cow::Borrowed)
            }
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
    pub(crate) fn to_brillig(&mut self, enable_debug_trace: bool) -> Brillig {
        // Collect all the function ids that are reachable from brillig
        // That means all the functions marked as brillig and ACIR functions called by them
        let brillig_reachable_function_ids = self
            .functions
            .iter()
            .filter_map(|(id, func)| func.runtime().is_brillig().then_some(*id))
            .collect::<BTreeSet<_>>();


        let mut used_globals_map = std::mem::take(&mut self.used_globals);
        // TODO: combine this with the reachable brillig functions set above
        let mut brillig_entry_points = HashMap::default();
        for (_, function) in self.functions.iter() {
            if function.runtime().is_acir() {
                for block_id in function.reachable_blocks() {
                    for instruction_id in function.dfg[block_id].instructions() {
                        let instruction = &function.dfg[*instruction_id];
                        let Instruction::Call { func: func_id, arguments: _ } = instruction else {
                            continue;
                        };

                        let func_value = &function.dfg[*func_id];
                        let Value::Function(func_id) = func_value else { continue };

                        let called_function = &self.functions[func_id];
                        if called_function.runtime().is_acir() {
                            continue;
                        } 

                        let inner_calls = called_functions(called_function);
                        dbg!(&inner_calls);
                        for inner_call in inner_calls.iter() {
                            let inner_globals = used_globals_map.get(inner_call).expect("Should have a slot for each function").clone();
                            used_globals_map.get_mut(func_id).expect("ICE: should have func").extend(inner_globals);
                        }
                        brillig_entry_points.insert(*func_id, inner_calls);
                    }
                }
            }
        }
        // dbg!(&brillig_entry_points);
        // dbg!(used_globals_map.clone());

        let mut brillig = Brillig::default();

        if brillig_reachable_function_ids.is_empty() {
            return brillig;
        }

        // Globals are computed once at compile time and shared across all functions,
        // thus we can just fetch globals from the main function.
        let globals = (*self.functions[&self.main_id].dfg.globals).clone();
        let globals_dfg = DataFlowGraph::from(globals);
        let mut inner_call_to_entry_point = HashMap::default();
        let mut entry_point_globals_map = HashMap::default();
        for (entry_point, used_globals) in used_globals_map {
            let (artifact, brillig_globals, globals_size) =
                convert_ssa_globals(enable_debug_trace, &globals_dfg, &used_globals, entry_point);

            // dbg!(brillig_globals.clone());
            // dbg!(entry_point);
            let entry_point_inner_calls = brillig_entry_points.remove(&entry_point).expect("ICE: Should have entry point map");
            for inner_call in entry_point_inner_calls {
                if inner_call_to_entry_point.get(&inner_call).is_some() {
                    dbg!("got here");
                }
                inner_call_to_entry_point.insert(inner_call, entry_point);
            }
            // entry_point_inner_calls.insert(entry_point);
            // brillig_globals_map.insert(entry_point_inner_calls, brillig_globals);

            entry_point_globals_map.insert(entry_point, brillig_globals);
            dbg!(globals_size);
            brillig.globals.insert(entry_point, artifact);
            brillig.globals_memory_size.insert(entry_point, globals_size);
        }

        for brillig_function_id in brillig_reachable_function_ids {
            // If we do not have an inner call slot, we are compiling an entry point.
            let entry_point = inner_call_to_entry_point.get(&brillig_function_id).copied().unwrap_or(brillig_function_id);
            let brillig_globals = entry_point_globals_map.get(&entry_point).expect("ICE: Should have globals associated with entry point");

            let func = &self.functions[&brillig_function_id];
            brillig.compile(func, enable_debug_trace, &brillig_globals);
        }

        brillig
    }
}
