pub(crate) mod brillig_black_box;
pub(crate) mod brillig_block;
pub(crate) mod brillig_directive;
pub(crate) mod brillig_fn;
pub(crate) mod brillig_slice_ops;

use crate::ssa_refactor::ir::{function::Function, post_order::PostOrder};

use std::collections::HashMap;

use self::{brillig_block::BrilligBlock, brillig_fn::FunctionContext};

use super::brillig_ir::{artifact::BrilligArtifact, BrilligContext};

/// Converting an SSA function into Brillig bytecode.
pub(crate) fn convert_ssa_function(func: &Function, enable_debug_trace: bool) -> BrilligArtifact {
    let mut reverse_post_order = Vec::new();
    reverse_post_order.extend_from_slice(PostOrder::with_function(func).as_slice());
    reverse_post_order.reverse();

    let mut function_context =
        FunctionContext { function_id: func.id(), ssa_value_to_brillig_variable: HashMap::new() };

    let mut brillig_context = BrilligContext::new(enable_debug_trace);

    brillig_context.enter_context(FunctionContext::function_id_to_function_label(func.id()));
    for block in reverse_post_order {
        BrilligBlock::compile(&mut function_context, &mut brillig_context, block, &func.dfg);
    }

    brillig_context.artifact()
}
