use std::collections::HashMap;

use crate::ssa::{ir::instruction::Instruction, ssa_gen::Ssa};

impl Ssa {
    /// A simple SSA pass to go through each instruction and move every `Instruction::Constrain` to immediately
    /// after when all of its inputs are available.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn bubble_up_constrains(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            for block in function.reachable_blocks() {
                let instructions = function.dfg[block].take_instructions();
                let mut filtered_instructions = Vec::with_capacity(instructions.len());
                let mut inserted_at_index: HashMap<usize, usize> =
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

                    let index = filtered_instructions
                        .iter()
                        .rev()
                        .position(|instruction_id| {
                            let results = dfg.instruction_results(*instruction_id).to_vec();
                            results.contains(&lhs) || results.contains(&rhs)
                        })
                        // We iterate through the previous instructions in reverse order so the index is from the
                        // back of the vector. Subtract from vector length to get correct index.
                        .map(|reversed_index| filtered_instructions.len() - reversed_index)
                        .unwrap_or(0);

                    let already_inserted_at_index = inserted_at_index.entry(index).or_default();

                    filtered_instructions.insert(index + *already_inserted_at_index, instruction);
                    *already_inserted_at_index += 1;
                }

                *function.dfg[block].instructions_mut() = filtered_instructions;
            }
        }
        self
    }
}
