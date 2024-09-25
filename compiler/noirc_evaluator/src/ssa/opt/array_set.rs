use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        dfg::DataFlowGraph,
        instruction::{Instruction, InstructionId},
        types::Type::{Array, Slice},
        value::ValueId,
    },
    ssa_gen::Ssa,
};
use fxhash::{FxHashMap as HashMap, FxHashSet as HashSet};

impl Ssa {
    /// Map arrays with the last instruction that uses it
    /// For this we simply process all the instructions in execution order
    /// and update the map whenever there is a match
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn array_set_optimization(mut self) -> Self {
        for func in self.functions.values_mut() {
            let reachable_blocks = func.reachable_blocks();

            if !func.runtime().is_entry_point() {
                assert_eq!(reachable_blocks.len(), 1, "Expected there to be 1 block remaining in Acir function for array_set optimization");
            }
            let mut array_to_last_use = HashMap::default();
            let mut instructions_to_update = HashSet::default();
            let mut arrays_from_load = HashSet::default();
            for block in reachable_blocks.iter() {
                analyze_last_uses_new(
                    &func.dfg,
                    *block,
                    &mut array_to_last_use,
                    &mut instructions_to_update,
                    &mut arrays_from_load,
                );
            }
            for block in reachable_blocks {
                make_mutable(&mut func.dfg, block, &instructions_to_update);
            }
        }
        self
    }
}

/// Builds the set of ArraySet instructions that can be made mutable
/// because their input value is unused elsewhere afterward.
fn analyze_last_uses_new(
    dfg: &DataFlowGraph,
    block_id: BasicBlockId,
    array_to_last_use: &mut HashMap<ValueId, InstructionId>,
    instructions_that_can_be_made_mutable: &mut HashSet<InstructionId>,
    arrays_from_load: &mut HashSet<ValueId>,
) {
    let block = &dfg[block_id];

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
                // If the array we are setting does not come from a load we can safely mark it mutable.
                // If the array comes from a load we may potentially being mutating an array at a reference
                // that is loaded from by other values.
                if !arrays_from_load.contains(&array) {
                    instructions_that_can_be_made_mutable.insert(*instruction_id);
                }
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
            Instruction::Load { .. } => {
                let result = dfg.instruction_results(*instruction_id)[0];
                if matches!(dfg.type_of_value(result), Array { .. } | Slice { .. }) {
                    arrays_from_load.insert(result);
                }
            }
            _ => (),
        }
    }
}

/// Make each ArraySet instruction in `instructions_to_update` mutable.
fn make_mutable(
    dfg: &mut DataFlowGraph,
    block_id: BasicBlockId,
    instructions_to_update: &HashSet<InstructionId>,
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
