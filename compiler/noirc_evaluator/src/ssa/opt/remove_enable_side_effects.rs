//! The goal of the "remove enable side effects" optimization pass is to delay any [Instruction::EnableSideEffects]
//! instructions such that they cover the minimum number of instructions possible.
//!
//! The pass works as follows:
//! - Insert instructions until an [Instruction::EnableSideEffects] is encountered, save this [InstructionId].
//! - Continue inserting instructions until either
//!     - Another [Instruction::EnableSideEffects] is encountered, if so then drop the previous [InstructionId] in favour
//!       of this one.
//!     - An [Instruction] with side-effects is encountered, if so then insert the currently saved [Instruction::EnableSideEffects]
//!       before the [Instruction]. Continue inserting instructions until the next [Instruction::EnableSideEffects] is encountered.
use std::collections::HashSet;

use acvm::{acir::AcirField, FieldElement};

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        dfg::DataFlowGraph,
        function::Function,
        instruction::{BinaryOp, Instruction, Intrinsic},
        value::Value,
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

        let mut last_side_effects_enabled_instruction = None;

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

            // If we hit an instruction which is affected by the side effects var then we must insert the
            // `Instruction::EnableSideEffects` before we insert this new instruction.
            if Self::responds_to_side_effects_var(&function.dfg, instruction) {
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

    fn responds_to_side_effects_var(dfg: &DataFlowGraph, instruction: &Instruction) -> bool {
        use Instruction::*;
        match instruction {
            Binary(binary) => match binary.operator {
                BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul => {
                    dfg.type_of_value(binary.lhs).is_unsigned()
                }
                BinaryOp::Div | BinaryOp::Mod => {
                    if let Some(rhs) = dfg.get_numeric_constant(binary.rhs) {
                        rhs == FieldElement::zero()
                    } else {
                        true
                    }
                }
                _ => false,
            },

            Cast(_, _)
            | Not(_)
            | Truncate { .. }
            | Constrain(..)
            | RangeCheck { .. }
            | IfElse { .. }
            | IncrementRc { .. }
            | DecrementRc { .. } => false,

            EnableSideEffects { .. }
            | ArrayGet { .. }
            | ArraySet { .. }
            | Allocate
            | Store { .. }
            | Load { .. } => true,

            // Some `Intrinsic`s have side effects so we must check what kind of `Call` this is.
            Call { func, .. } => match dfg[*func] {
                Value::Intrinsic(intrinsic) => match intrinsic {
                    Intrinsic::SlicePushBack
                    | Intrinsic::SlicePushFront
                    | Intrinsic::SlicePopBack
                    | Intrinsic::SlicePopFront
                    | Intrinsic::SliceInsert
                    | Intrinsic::SliceRemove => true,

                    Intrinsic::ArrayLen
                    | Intrinsic::AssertConstant
                    | Intrinsic::ApplyRangeConstraint
                    | Intrinsic::StrAsBytes
                    | Intrinsic::ToBits(_)
                    | Intrinsic::ToRadix(_)
                    | Intrinsic::BlackBox(_)
                    | Intrinsic::FromField
                    | Intrinsic::AsField
                    | Intrinsic::AsSlice
                    | Intrinsic::AsWitness
                    | Intrinsic::IsUnconstrained => false,
                },

                // We must assume that functions contain a side effect as we cannot inspect more deeply.
                Value::Function(_) => true,

                _ => false,
            },
        }
    }
}
