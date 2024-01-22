use std::collections::HashMap;

use crate::ssa::{
    ir::instruction::{Instruction, InstructionId},
    ssa_gen::Ssa,
};

impl Ssa {
    /// A simple SSA pass to go through each instruction and move every `Instruction::Constrain` to immediately
    /// after when all of its inputs are available.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn bubble_up_constrains(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            for block in function.reachable_blocks() {
                let instructions = function.dfg[block].take_instructions();
                let mut filtered_instructions = Vec::with_capacity(instructions.len());

                // Some insertions will be done at the same index, so we need to keep track of how many
                // Some assertions don't operate on instruction results, so we use Option so we also track the None case
                let mut inserted_at_instruction: HashMap<Option<InstructionId>, usize> =
                    HashMap::with_capacity(instructions.len());

                let dfg = &function.dfg;
                for instruction in instructions {
                    let (lhs, rhs) = match dfg[instruction] {
                        Instruction::Constrain(lhs, rhs, ..) => (lhs, rhs),
                        _ => {
                            filtered_instructions.push(instruction);
                            continue;
                        }
                    };

                    let last_instruction_that_creates_inputs = filtered_instructions
                        .iter()
                        .rev()
                        .position(|&instruction_id| {
                            let results = dfg.instruction_results(instruction_id).to_vec();
                            results.contains(&lhs) || results.contains(&rhs)
                        })
                        // We iterate through the previous instructions in reverse order so the index is from the
                        // back of the vector
                        .map(|reversed_index| filtered_instructions.len() - reversed_index - 1);

                    let insertion_index = last_instruction_that_creates_inputs
                        .map(|index| {
                            // We want to insert just after the last instruction that creates the inputs
                            index + 1
                        })
                        // If it doesn't depend from the previous instructions, then we insert at the start
                        .unwrap_or_default();

                    let already_inserted_for_this_instruction = inserted_at_instruction
                        .entry(
                            last_instruction_that_creates_inputs
                                .map(|index| filtered_instructions[index]),
                        )
                        .or_default();

                    filtered_instructions.insert(
                        insertion_index + *already_inserted_for_this_instruction,
                        instruction,
                    );

                    *already_inserted_for_this_instruction += 1;
                }

                *function.dfg[block].instructions_mut() = filtered_instructions;
            }
        }
        self
    }
}
