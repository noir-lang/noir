pub(crate) mod brillig_black_box;
pub(crate) mod brillig_block;
pub(crate) mod brillig_directive;
pub(crate) mod brillig_fn;
pub(crate) mod brillig_slice_ops;
mod variable_liveness;

use self::{brillig_block::BrilligBlock, brillig_fn::FunctionContext};
use super::brillig_ir::{artifact::BrilligArtifact, BrilligContext};
use crate::ssa::ir::function::Function;

/// Converting an SSA function into Brillig bytecode.
pub(crate) fn convert_ssa_function(func: &Function, enable_debug_trace: bool) -> BrilligArtifact {
    let mut function_context = FunctionContext::new(func);
    let mut brillig_context = BrilligContext::new(enable_debug_trace);

    brillig_context.enter_context(FunctionContext::function_id_to_function_label(func.id()));
    for block in function_context.blocks.clone() {
        BrilligBlock::compile(&mut function_context, &mut brillig_context, block, &func.dfg);
    }

    brillig_context.artifact()
}
