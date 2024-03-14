//! The goal of the "remove enable side effects" optimization pass is to delay any [Instruction::EnableSideEffects]
//! instructions such that they cover the minimum number of instructions possible.
//!
//! The pass works as follows:
//! - Insert instructions until an [Instruction::EnableSideEffects] is encountered, save this [InstructionId].
//! - Continue inserting instructions until either
//!     - Another [Instruction::EnableSideEffects] is encountered, if so then drop the previous [InstructionId] in favour
//!       of this one.
//!     - An [Instruction] with side-effects is encountered, if so then insert thecurrently saved [Instruction::EnableSideEffects]
//!       before the [Instruction]. Continue inserting instructions until the next [Instruction::EnableSideEffects] is encountered.
use std::collections::HashSet;

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        function::Function,
        instruction::{Instruction, InstructionId},
    },
    ssa_gen::Ssa,
};

impl Ssa {
    /// See [`remove_enable_side_effects`][self] module for more information.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn remove_enable_side_effects(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            remove_enable_side_effects(function);
        }
        self
    }
}

fn remove_enable_side_effects(function: &mut Function) {
    let mut context = Context::default();
    context.block_queue.push(function.entry_block());

    while let Some(block) = context.block_queue.pop() {
        if context.visited_blocks.contains(&block) {
            continue;
        }

        context.visited_blocks.insert(block);
        context.remove_enable_side_effects_in_block(function, block);
    }
}

#[derive(Default)]
struct Context {
    visited_blocks: HashSet<BasicBlockId>,
    block_queue: Vec<BasicBlockId>,
}

impl Context {
    fn remove_enable_side_effects_in_block(
        &mut self,
        function: &mut Function,
        block: BasicBlockId,
    ) {
        let instructions = function.dfg[block].take_instructions();

        let mut last_side_effects_enabled_instruction: Option<InstructionId> = None;

        let mut new_instructions = Vec::with_capacity(instructions.len());
        for instruction_id in instructions {
            let instruction = &function.dfg[instruction_id];

            // If we run into another `Instruction::EnableSideEffects` before encountering any
            // instructions with side effects then we can drop the instruction we're holding and
            // continue with the new `Instruction::EnableSideEffects`.
            if let Instruction::EnableSideEffects { condition } = instruction {
                // If we're seeing an `enable_side_effects u1 1` then we want to insert it immediately.
                // This is because we want to maximize the effect it will have.
                if function
                    .dfg
                    .get_numeric_constant(*condition)
                    .map_or(false, |condition| condition.is_one())
                {
                    new_instructions.push(instruction_id);
                    last_side_effects_enabled_instruction = None;
                    continue;
                }

                last_side_effects_enabled_instruction = Some(instruction_id);
                continue;
            }

            // If we hit an instruction with side effects then we must insert the `Instruction::EnableSideEffects`
            // before we insert this new instruction.
            if instruction.has_side_effects(&function.dfg)
                || matches!(
                    instruction,
                    Instruction::ArrayGet { .. } | Instruction::ArraySet { .. }
                )
            {
                if let Some(enable_side_effects_instruction_id) =
                    last_side_effects_enabled_instruction.take()
                {
                    new_instructions.push(enable_side_effects_instruction_id);
                }
            }
            new_instructions.push(instruction_id);
        }

        *function.dfg[block].instructions_mut() = new_instructions;

        self.block_queue.extend(function.dfg[block].successors());
    }
}
