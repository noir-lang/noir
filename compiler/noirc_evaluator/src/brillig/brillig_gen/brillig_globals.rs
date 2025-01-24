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
    inner_call_to_entry_point: HashMap<FunctionId, FunctionId>,
    entry_point_globals_map: HashMap<FunctionId, SsaToBrilligGlobals>,
}

/// Mapping of SSA value ids to their Brillig allocations
pub(crate) type SsaToBrilligGlobals = HashMap<ValueId, BrilligVariable>;

impl BrilligGlobals {
    pub(crate) fn new(
        functions: &BTreeMap<FunctionId, Function>,
        mut used_globals: HashMap<FunctionId, HashSet<ValueId>>,
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

                        let inner_calls = called_functions_vec(called_function)
                            .into_iter()
                            .collect::<HashSet<_>>();
                        for inner_call in inner_calls.iter() {
                            let inner_globals = used_globals
                                .get(inner_call)
                                .expect("Should have a slot for each function")
                                .clone();
                            used_globals
                                .get_mut(func_id)
                                .expect("ICE: should have func")
                                .extend(inner_globals);
                        }
                        brillig_entry_points.insert(*func_id, inner_calls);
                    }
                }
            }
        }

        Self { used_globals, brillig_entry_points, ..Default::default() }
    }

    pub(crate) fn declare_globals(
        &mut self,
        globals_dfg: &DataFlowGraph,
        brillig: &mut Brillig,
        enable_debug_trace: bool,
    ) {
        let mut inner_call_to_entry_point = HashMap::default();
        let mut entry_point_globals_map = HashMap::default();
        for (entry_point, used_globals) in self.used_globals.iter() {
            let entry_point = *entry_point;
            let (artifact, brillig_globals, globals_size) =
                convert_ssa_globals(enable_debug_trace, globals_dfg, used_globals, entry_point);

            let entry_point_inner_calls = self
                .brillig_entry_points
                .remove(&entry_point)
                .expect("ICE: Should have entry point map");
            for inner_call in entry_point_inner_calls {
                // TODO: this might be a bug if we have an inner call with a different entry point
                // just panic for now until we handle it appropriately
                if inner_call_to_entry_point.get(&inner_call).is_some() {
                    panic!("ah we have a repeat inner call to entry point");
                }
                inner_call_to_entry_point.insert(inner_call, entry_point);
            }

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
    ) -> &SsaToBrilligGlobals {
        // If we do not have an inner call slot, we are compiling an entry point.
        let entry_point = self
            .inner_call_to_entry_point
            .get(&brillig_function_id)
            .copied()
            .unwrap_or(brillig_function_id);
        self.entry_point_globals_map
            .get(&entry_point)
            .expect("ICE: Should have globals associated with entry point")
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

    // let globals_dfg = DataFlowGraph::from(globals);
    brillig_block.compile_globals(globals_dfg, used_globals);

    let globals_size = brillig_block.brillig_context.global_space_size();

    brillig_context.return_instruction();

    let artifact = brillig_context.artifact();
    (artifact, function_context.ssa_value_allocations, globals_size)
}
