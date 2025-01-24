use acvm::FieldElement;
use fxhash::{FxHashMap as HashMap, FxHashSet as HashSet};

use super::{BrilligVariable, FunctionContext, ValueId};
use crate::{
    brillig::{brillig_ir::BrilligContext, BrilligArtifact, BrilligBlock, DataFlowGraph, Label},
    ssa::ir::dfg::GlobalsGraph,
};

impl crate::brillig::Brillig {
    pub(crate) fn convert_ssa_globals(
        &mut self,
        enable_debug_trace: bool,
        globals: GlobalsGraph,
        used_globals: &HashSet<ValueId>,
    ) -> (BrilligArtifact<FieldElement>, HashMap<ValueId, BrilligVariable>, usize) {
        let mut brillig_context = BrilligContext::new_for_global_init(enable_debug_trace);
        // The global space does not have globals itself
        let empty_globals = HashMap::default();
        // We can use any ID here as this context is only going to be used for globals which does not differentiate
        // by functions and blocks. The only Label that should be used in the globals context is `Label::globals_init()`
        let mut function_context = FunctionContext::default();
        brillig_context.enter_context(Label::globals_init());

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

        let globals_dfg = DataFlowGraph::from(globals);
        brillig_block.compile_globals(&globals_dfg, used_globals, &mut self.call_stacks);

        let globals_size = brillig_block.brillig_context.global_space_size();

        brillig_context.return_instruction();

        let artifact = brillig_context.artifact();
        (artifact, function_context.ssa_value_allocations, globals_size)
    }
}
