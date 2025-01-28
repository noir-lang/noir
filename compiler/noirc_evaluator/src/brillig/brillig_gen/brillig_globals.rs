use std::collections::BTreeMap;

use acvm::FieldElement;
use fxhash::{FxHashMap as HashMap, FxHashSet as HashSet};

use super::{
    BrilligArtifact, BrilligBlock, BrilligVariable, Function, FunctionContext, Label, ValueId,
};
use crate::brillig::{
    brillig_ir::BrilligContext, called_functions_vec, Brillig, DataFlowGraph, FunctionId,
    Instruction, Value,
};

/// Context structure for generating Brillig globals
/// it stores globals related data required for code generation of regular Brillig functions.
#[derive(Default)]
pub(crate) struct BrilligGlobals {
    /// Maps a Brillig function to the globals used in that function,
    used_globals: HashMap<FunctionId, HashSet<ValueId>>,
    brillig_entry_points: HashMap<FunctionId, HashSet<FunctionId>>,

    /// Maps an inner call to its Brillig entry point,
    inner_call_to_entry_point: HashMap<FunctionId, Vec<FunctionId>>,
    entry_point_globals_map: HashMap<FunctionId, SsaToBrilligGlobals>,
}

/// Mapping of SSA value ids to their Brillig allocations
pub(crate) type SsaToBrilligGlobals = HashMap<ValueId, BrilligVariable>;

impl BrilligGlobals {
    pub(crate) fn new(
        functions: &BTreeMap<FunctionId, Function>,
        mut used_globals: HashMap<FunctionId, HashSet<ValueId>>,
        main_id: FunctionId,
    ) -> Self {
        let mut brillig_entry_points = HashMap::default();
        for (_, function) in functions.iter() {
            if function.runtime().is_acir() {
                for block_id in function.reachable_blocks() {
                    for instruction_id in function.dfg[block_id].instructions() {
                        let instruction = &function.dfg[*instruction_id];
                        let Instruction::Call { func: func_id, arguments: _ } = instruction else {
                            continue;
                        };

                        let func_value = &function.dfg[*func_id];
                        let Value::Function(func_id) = func_value else { continue };

                        let called_function = &functions[func_id];
                        if called_function.runtime().is_acir() {
                            continue;
                        }

                        brillig_entry_points.insert(*func_id, HashSet::default());
                        Self::mark_entry_points_calls_recursive(
                            functions,
                            *func_id,
                            called_function,
                            &mut used_globals,
                            &mut brillig_entry_points,
                            im::HashSet::new(),
                        );
                    }
                }
            }
        }

        let main_func = &functions[&main_id];
        if main_func.runtime().is_brillig() {
            brillig_entry_points.insert(main_id, HashSet::default());
            Self::mark_entry_points_calls_recursive(
                functions,
                main_id,
                main_func,
                &mut used_globals,
                &mut brillig_entry_points,
                im::HashSet::new(),
            );
        }

        Self { used_globals, brillig_entry_points, ..Default::default() }
    }

    fn mark_entry_points_calls_recursive(
        functions: &BTreeMap<FunctionId, Function>,
        entry_point: FunctionId,
        called_function: &Function,
        used_globals: &mut HashMap<FunctionId, HashSet<ValueId>>,
        brillig_entry_points: &mut HashMap<FunctionId, HashSet<FunctionId>>,
        mut explored_functions: im::HashSet<FunctionId>,
    ) {
        if explored_functions.contains(&called_function.id()) {
            return;
        }
        explored_functions.insert(called_function.id());

        let inner_calls = called_functions_vec(called_function).into_iter().collect::<HashSet<_>>();

        for inner_call in inner_calls.iter() {
            let inner_globals =
                used_globals.get(inner_call).expect("Should have a slot for each function").clone();
            used_globals
                .get_mut(&entry_point)
                .expect("ICE: should have func")
                .extend(inner_globals);

            if let Some(inner_calls) = brillig_entry_points.get_mut(&entry_point) {
                inner_calls.insert(*inner_call);
            }

            Self::mark_entry_points_calls_recursive(
                functions,
                entry_point,
                &functions[inner_call],
                used_globals,
                brillig_entry_points,
                explored_functions.clone(),
            );
        }
    }

    pub(crate) fn declare_globals(
        &mut self,
        globals_dfg: &DataFlowGraph,
        brillig: &mut Brillig,
        enable_debug_trace: bool,
    ) {
        // Map for fetching the correct entry point globals when compiling any function
        let mut inner_call_to_entry_point: HashMap<FunctionId, Vec<FunctionId>> =
            HashMap::default();
        let mut entry_point_globals_map = HashMap::default();
        for (entry_point, used_globals) in self.used_globals.iter() {
            let entry_point = *entry_point;

            // TODO: should just loop these rather than `used_globals`
            let entry_point_inner_calls = self.brillig_entry_points.remove(&entry_point);
            if entry_point.to_u32() == 6 {
                dbg!(entry_point_inner_calls.clone());
            }
            let Some(entry_point_inner_calls) = entry_point_inner_calls else {
                // Otherwise we do not have an entry point and do not need to generate globals
                continue;
            };

            for inner_call in entry_point_inner_calls {
                inner_call_to_entry_point.entry(inner_call).or_default().push(entry_point);
            }

            let (artifact, brillig_globals, globals_size) =
                convert_ssa_globals(enable_debug_trace, globals_dfg, used_globals, entry_point);

            entry_point_globals_map.insert(entry_point, brillig_globals);

            brillig.globals.insert(entry_point, artifact);
            brillig.globals_memory_size.insert(entry_point, globals_size);
        }

        self.inner_call_to_entry_point = inner_call_to_entry_point;
        self.entry_point_globals_map = entry_point_globals_map;
    }

    pub(crate) fn get_brillig_globals(
        &self,
        brillig_function_id: FunctionId,
    ) -> Vec<&SsaToBrilligGlobals> {
        let entry_points = self.inner_call_to_entry_point.get(&brillig_function_id);

        if let Some(entry_points) = entry_points {
            entry_points
                .iter()
                .flat_map(|entry_point| self.entry_point_globals_map.get(entry_point))
                .collect()
        } else {
            vec![self
                .entry_point_globals_map
                .get(&brillig_function_id)
                .expect("ICE: Must have allocated globals for entry point")]
        }
    }
}

pub(crate) fn convert_ssa_globals(
    enable_debug_trace: bool,
    globals_dfg: &DataFlowGraph,
    used_globals: &HashSet<ValueId>,
    entry_point: FunctionId,
) -> (BrilligArtifact<FieldElement>, HashMap<ValueId, BrilligVariable>, usize) {
    let mut brillig_context = BrilligContext::new_for_global_init(enable_debug_trace, entry_point);
    // The global space does not have globals itself
    let empty_globals = HashMap::default();
    // We can use any ID here as this context is only going to be used for globals which does not differentiate
    // by functions and blocks. The only Label that should be used in the globals context is `Label::globals_init()`
    let mut function_context = FunctionContext::default();
    brillig_context.enter_context(Label::globals_init(entry_point));

    let block_id = DataFlowGraph::default().make_block();
    let mut brillig_block = BrilligBlock {
        function_context: &mut function_context,
        block_id,
        brillig_context: &mut brillig_context,
        variables: Default::default(),
        last_uses: HashMap::default(),
        globals: &empty_globals,
        building_globals: true,
    };

    brillig_block.compile_globals(globals_dfg, used_globals);

    let globals_size = brillig_context.global_space_size();

    brillig_context.return_instruction();

    let artifact = brillig_context.artifact();
    (artifact, function_context.ssa_value_allocations, globals_size)
}

#[cfg(test)]
mod tests {}
