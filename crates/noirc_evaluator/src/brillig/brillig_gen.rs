pub(crate) mod brillig_block;
pub(crate) mod brillig_fn;

use crate::ssa_refactor::ir::{function::Function, post_order::PostOrder};

use std::collections::HashMap;

use self::{brillig_block::BrilligBlock, brillig_fn::FunctionContext};

use super::brillig_ir::{artifact::BrilligArtifact, BrilligContext};

/// Converting an SSA function into Brillig bytecode.
///
/// TODO: Change this to use `dfg.basic_blocks_iter` which will return an
/// TODO iterator of all of the basic blocks.
/// TODO(Jake): what order is this ^
pub(crate) fn convert_ssa_function(func: &Function) -> BrilligArtifact {
    let mut reverse_post_order = Vec::new();
    reverse_post_order.extend_from_slice(PostOrder::with_function(func).as_slice());
    reverse_post_order.reverse();

    let mut function_context =
        FunctionContext { function_id: func.id(), ssa_value_to_register: HashMap::new() };

    let mut brillig_context = BrilligContext::default();

    for block in reverse_post_order {
        BrilligBlock::compile(&mut function_context, &mut brillig_context, block, &func.dfg);
    }

    brillig_context.artifact()
}

/// Creates an entry point artifact, that will be linked with the brillig functions being called
pub(crate) fn create_entry_point_function(num_arguments: usize) -> BrilligArtifact {
    let mut brillig_context = BrilligContext::default();
    brillig_context.entry_point_instruction(num_arguments);
    brillig_context.artifact()
}
