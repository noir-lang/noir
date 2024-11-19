pub(crate) mod brillig_black_box;
pub(crate) mod brillig_block;
pub(crate) mod brillig_block_variables;
pub(crate) mod brillig_fn;
pub(crate) mod brillig_slice_ops;
mod constant_allocation;
mod variable_liveness;

use acvm::FieldElement;

use self::{brillig_block::BrilligBlock, brillig_fn::FunctionContext};
use super::brillig_ir::{
    artifact::{BrilligArtifact, Label},
    BrilligContext,
};
use crate::ssa::ir::function::Function;

/// Converting an SSA function into Brillig bytecode.
pub(crate) fn convert_ssa_function(
    func: &Function,
    enable_debug_trace: bool,
) -> BrilligArtifact<FieldElement> {
    let mut brillig_context = BrilligContext::new(enable_debug_trace);

    let mut function_context = FunctionContext::new(func);

    brillig_context.enter_context(Label::function(func.id()));

    brillig_context.call_check_max_stack_depth_procedure();

    for block in function_context.blocks.clone() {
        BrilligBlock::compile(&mut function_context, &mut brillig_context, block, &func.dfg);
    }

    let mut artifact = brillig_context.artifact();
    artifact.name = func.name().to_string();
    artifact
}
