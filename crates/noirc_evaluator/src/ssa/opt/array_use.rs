use std::collections::HashMap;

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        dfg::DataFlowGraph,
        instruction::{Instruction, InstructionId},
        post_order::PostOrder,
        value::{Value, ValueId},
    },
    ssa_gen::Ssa,
};

impl Ssa {
    /// Map arrays with the last instruction that uses it
    /// For this we simply process all the instructions in execution order
    /// and update the map whenever there is a match
    pub(crate) fn find_last_array_uses(&self) -> HashMap<ValueId, InstructionId> {
        let mut array_use = HashMap::new();
        for func in self.functions.values() {
            let mut reverse_post_order = PostOrder::with_function(func).into_vec();
            reverse_post_order.reverse();
            for block in reverse_post_order {
                last_use(block, &func.dfg, &mut array_use);
            }
        }
        array_use
    }
}

/// Updates the array_def map when an instructions is using an array
fn last_use(
    block_id: BasicBlockId,
    dfg: &DataFlowGraph,
    array_def: &mut HashMap<ValueId, InstructionId>,
) {
    let block = &dfg[block_id];
    for instruction_id in block.instructions() {
        match &dfg[*instruction_id] {
            Instruction::ArrayGet { array, .. } | Instruction::ArraySet { array, .. } => {
                let array = dfg.resolve(*array);
                array_def.insert(array, *instruction_id);
            }
            Instruction::Call { arguments, .. } => {
                for argument in arguments {
                    let resolved_arg = dfg.resolve(*argument);
                    if matches!(dfg[resolved_arg], Value::Array { .. }) {
                        array_def.insert(resolved_arg, *instruction_id);
                    }
                }
            }
            _ => {
                // Nothing to do
            }
        }
    }
}
