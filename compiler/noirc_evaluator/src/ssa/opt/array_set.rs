use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        dfg::DataFlowGraph,
        instruction::{Instruction, InstructionId},
        types::Type::{Array, Slice},
    },
    ssa_gen::Ssa,
};
use fxhash::{FxHashMap as HashMap, FxHashSet};

impl Ssa {
    /// Map arrays with the last instruction that uses it
    /// For this we simply process all the instructions in execution order
    /// and update the map whenever there is a match
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn array_set_optimization(mut self) -> Self {
        for func in self.functions.values_mut() {
            if !func.runtime().is_entry_point() {
                let mut reachable_blocks = func.reachable_blocks();
                assert_eq!(reachable_blocks.len(), 1, "Expected there to be 1 block remaining in Acir function for array_set optimization");

                let block = reachable_blocks.pop_first().unwrap();
                let instructions_to_update = analyze_last_uses(&func.dfg, block);
                make_mutable(&mut func.dfg, block, instructions_to_update);
            }
        }
        self
    }
}

/// Returns the set of ArraySet instructions that can be made mutable
/// because their input value is unused elsewhere afterward.
fn analyze_last_uses(dfg: &DataFlowGraph, block_id: BasicBlockId) -> FxHashSet<InstructionId> {
    let block = &dfg[block_id];
    let mut array_to_last_use = HashMap::default();
    let mut instructions_that_can_be_made_mutable = FxHashSet::default();

    for instruction_id in block.instructions() {
        match &dfg[*instruction_id] {
            Instruction::ArrayGet { array, .. } => {
                let array = dfg.resolve(*array);

                if let Some(existing) = array_to_last_use.insert(array, *instruction_id) {
                    instructions_that_can_be_made_mutable.remove(&existing);
                }
            }
            Instruction::ArraySet { array, .. } => {
                let array = dfg.resolve(*array);

                if let Some(existing) = array_to_last_use.insert(array, *instruction_id) {
                    instructions_that_can_be_made_mutable.remove(&existing);
                }
                instructions_that_can_be_made_mutable.insert(*instruction_id);
            }
            Instruction::Call { arguments, .. } => {
                for argument in arguments {
                    if matches!(dfg.type_of_value(*argument), Array { .. } | Slice { .. }) {
                        let argument = dfg.resolve(*argument);

                        if let Some(existing) = array_to_last_use.insert(argument, *instruction_id)
                        {
                            instructions_that_can_be_made_mutable.remove(&existing);
                        }
                    }
                }
            }
            _ => (),
        }
    }

    instructions_that_can_be_made_mutable
}

/// Make each ArraySet instruction in `instructions_to_update` mutable.
fn make_mutable(
    dfg: &mut DataFlowGraph,
    block_id: BasicBlockId,
    instructions_to_update: FxHashSet<InstructionId>,
) {
    if instructions_to_update.is_empty() {
        return;
    }

    // Take the instructions temporarily so we can mutate the DFG while we iterate through them
    let block = &mut dfg[block_id];
    let instructions = block.take_instructions();

    for instruction in &instructions {
        if instructions_to_update.contains(instruction) {
            let instruction = &mut dfg[*instruction];

            if let Instruction::ArraySet { mutable, .. } = instruction {
                *mutable = true;
            } else {
                unreachable!(
                    "Non-ArraySet instruction in instructions_to_update!\n{instruction:?}"
                );
            }
        }
    }

    *dfg[block_id].instructions_mut() = instructions;
}
