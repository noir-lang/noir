pub(crate) mod brillig_block;
pub(crate) mod brillig_fn;

use crate::ssa_refactor::ir::{
    function::Function, instruction::TerminatorInstruction, post_order::PostOrder,
};

use std::collections::HashMap;

use self::{brillig_block::BrilligBlock, brillig_fn::FunctionContext};

use super::{
    brillig_ir::{artifact::BrilligArtifact, BrilligContext},
    FuncIdEntryBlockId,
};

/// Converting an SSA function into Brillig bytecode.
///
/// TODO: Change this to use `dfg.basic_blocks_iter` which will return an
/// TODO iterator of all of the basic blocks.
/// TODO(Jake): what order is this ^
pub(crate) fn convert_ssa_function(
    func: &Function,
    ssa_function_id_to_block_id: &FuncIdEntryBlockId,
) -> BrilligArtifact {
    let mut reverse_post_order = Vec::new();
    reverse_post_order.extend_from_slice(PostOrder::with_function(func).as_slice());
    reverse_post_order.reverse();

    let mut function_context =
        FunctionContext { function_id: func.id(), ssa_value_to_register: HashMap::new() };

    fn func_num_return_values(func: &Function) -> usize {
        let dfg = &func.dfg;
        let term = dfg[func.entry_block()]
            .terminator()
            .expect("expected a terminator instruction, as block is finished construction ");
        match term {
            TerminatorInstruction::Return { return_values } => return_values.len(),
            _ => panic!("expected a return instruction, as block is finished construction "),
        }
    }
    let num_parameters = func.parameters().len();
    let num_return_values = func_num_return_values(func);
    let mut brillig_context = BrilligContext::new(num_parameters, num_return_values);

    for block in reverse_post_order {
        BrilligBlock::compile(
            &mut function_context,
            &mut brillig_context,
            ssa_function_id_to_block_id,
            block,
            &func.dfg,
        );
    }

    brillig_context.artifact()
}
